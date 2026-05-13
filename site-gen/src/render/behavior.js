/* OpenEPD chart-centric workspace behaviour.
 *
 * Two coordinated views share one piece of state ("the current word
 * and its current focused phoneme"):
 *
 *   - The IPA chart: phonemes laid out anatomically, a path drawn
 *     across it to show the word's articulatory trajectory.
 *   - The schematic sagittal/vocal-tract inset: tongue, lips, cords
 *     and airflow update per phoneme.
 *
 * Picking a word triggers an animation that traces both at once.
 * Hovering a chart phoneme jumps the inset to that phoneme.
 *
 * All click handling is delegated; no inline onclick / onmouseover
 * anywhere in the rendered HTML. Inputs are read from data-* attrs.
 *
 * Data declarations (WORDS, PHONEME_INFO, PHONEME_WORDS, CHART_POS,
 * SAGITTAL_SPECS, DISTANCE_MATRIX, PHONEME_AXIS) are emitted as
 * top-level `const`s immediately above this block by scripts.rs.
 */

(() => {
  /* ── DOM cache ──────────────────────────────────────────────── */
  const qEl       = document.getElementById('q');
  const clearBtn  = document.getElementById('search-clear');
  const suggestEl = document.getElementById('suggestions');
  const wGlyph    = document.getElementById('word-glyph');
  const wIpa      = document.getElementById('word-ipa');
  const wRank     = document.getElementById('word-rank');
  const wHint     = document.getElementById('word-hint');
  const spellEl   = document.getElementById('spelling-band');
  const chartEl   = document.getElementById('ipa-chart');
  const pathLayer = document.getElementById('path-layer');
  const sagGlyph  = document.getElementById('sagittal-glyph');
  const sagLabel  = document.getElementById('sagittal-label');
  const sagDesc   = document.getElementById('sagittal-desc');
  const sagEl     = document.getElementById('sagittal');
  const tongue    = document.getElementById('tongue');
  const lips      = document.getElementById('lips');
  const airflow   = document.getElementById('airflow');
  const cords     = document.getElementById('cords');
  const nasalPath = document.getElementById('nasal-path');
  const reverseQ  = document.getElementById('reverse-q');
  const reverseR  = document.getElementById('reverse-results');
  const reverseC  = document.getElementById('reverse-count');

  const SVG_NS = 'http://www.w3.org/2000/svg';

  let inputDebounce = null;
  let reverseDebounce = null;
  let animationToken = 0;   // monotonic id; cancels old animations

  /* ── Workspace state ────────────────────────────────────────────
   *
   *   selectedCh — phoneme the user has *clicked* on the chart.
   *                Sticky: persists across hover and mouseleave.
   *                When set, the sagittal stays on it and the chart
   *                phoneme shows the `.selected` visual state.
   *
   *   lastStep   — the phoneme the most-recent word-path animation
   *                ended on. When nothing is selected, this is what
   *                the sagittal falls back to on mouseleave so the
   *                inset never "snaps to nothing".
   *
   *   The interaction model: hover *previews*, click *commits*.
   *   Click again on the same phoneme to release.
   */
  let selectedCh = null;
  let lastStep   = null;

  /* ── Helpers ────────────────────────────────────────────────── */
  function escHTML(s) {
    return String(s)
      .replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
  }
  function ns(tag, attrs = {}, text = null) {
    const el = document.createElementNS(SVG_NS, tag);
    for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, v);
    if (text != null) el.textContent = text;
    return el;
  }
  function sleep(ms) { return new Promise(r => setTimeout(r, ms)); }

  /* ── Search-as-you-type ─────────────────────────────────────── */
  function handleSearchInput() {
    const v = qEl.value;
    clearBtn.classList.toggle('visible', v.length > 0);
    clearTimeout(inputDebounce);
    const q = v.trim().toLowerCase();
    if (!q) { hideSuggestions(); return; }
    const exact = WORDS.findIndex(w => w[0] === q);
    if (exact >= 0) { hideSuggestions(); selectWord(WORDS[exact][0], WORDS[exact][1], exact); return; }
    inputDebounce = setTimeout(() => showSuggestions(q), 100);
  }

  function showSuggestions(q) {
    const hits = [];
    for (let i = 0; i < WORDS.length && hits.length < 8; i++) {
      if (WORDS[i][0].startsWith(q)) hits.push(i);
    }
    if (!hits.length) {
      suggestEl.innerHTML = '<div class="suggestion-none">No match in top 50,000 words</div>';
      suggestEl.classList.add('active');
      return;
    }
    suggestEl.innerHTML = hits.map(i =>
      `<button class="suggestion-item" type="button" data-word="${escHTML(WORDS[i][0])}">` +
        `<span class="sug-word">${escHTML(WORDS[i][0])}</span>` +
        `<span class="sug-ipa ipa-font">${escHTML(WORDS[i][1])}</span>` +
        `<span class="sug-rank">#${i + 1}</span>` +
      `</button>`
    ).join('');
    suggestEl.classList.add('active');
  }
  function hideSuggestions() { suggestEl.innerHTML = ''; suggestEl.classList.remove('active'); }

  /* ── Spelling → phoneme alignment (JS port of spelling_align.rs)
   *
   * Heuristic: digraph table, silent-prefix table, silent-suffix
   * table, letter-to-phoneme compatibility table; greedy left-to-
   * right pass. Returns an array of pieces:
   *   {kind: 'direct',  letter, phoneme}
   *   {kind: 'digraph', letters, phoneme}
   *   {kind: 'silent',  letter}
   *   {kind: 'insert',  phoneme}
   * The visual band uses the kinds to draw connection lines, Y-shapes,
   * dangling letters and unattached phonemes respectively.
   */
  const DIGRAPHS = [
    ['ph','f'], ['th','θ'], ['th','ð'], ['ch','ʧ'], ['ch','k'],
    ['sh','ʃ'], ['zh','ʒ'], ['ng','ŋ'], ['ck','k'], ['dge','ʤ'],
    ['dg','ʤ'], ['gh','f'], ['gh','ɡ'],
  ];
  const SILENT_PREFIXES = [
    ['kn','n'], ['gn','n'], ['pn','n'], ['ps','s'],
    ['wr','ɹ'], ['mn','n'], ['rh','ɹ'],
  ];
  const SILENT_SUFFIXES = [
    ['mb','m'], ['mn','m'], ['gn','n'],
  ];
  const LETTER_OK = {
    a: 'æəɑɔaeɛ',  e: 'ɛiəeɪ', i: 'ɪiəa', o: 'ɑɔəʌouʊ',
    u: 'ʌuʊəja',   y: 'ɪij',    c: 'ksʧʃ', g: 'ɡgʤʒ',
    s: 'szʃʒ',     z: 'zsʒ',    x: 'kɡz',  q: 'k',
    j: 'ʤj',       r: 'ɹr',     l: 'lɫ',   n: 'nŋ',
  };
  function letterOk(L, P) {
    if (L === P) return true;
    const ok = LETTER_OK[L];
    return ok ? ok.includes(P) : false;
  }
  function alignSpelling(word, ipa) {
    const letters = Array.from(word).filter(c => /[a-zA-Z]/.test(c)).map(c => c.toLowerCase());
    const phonemes = Array.from(ipa).filter(c => !'ˈˌːˑ'.includes(c));
    const pieces = [];
    let i = 0, j = 0;
    // word-initial silent prefix
    if (letters.length >= 2) {
      const pref = letters[0] + letters[1];
      for (const [pat, ph] of SILENT_PREFIXES) {
        if (pref === pat && phonemes[0] === ph) {
          pieces.push({kind: 'silent', letter: letters[0]});
          pieces.push({kind: 'direct', letter: letters[1], phoneme: phonemes[0]});
          i = 2; j = 1;
          break;
        }
      }
    }
    while (i < letters.length) {
      let matched = false;
      // digraphs
      if (i + 1 < letters.length && j < phonemes.length) {
        const pair = letters[i] + letters[i + 1];
        for (const [pat, ph] of DIGRAPHS) {
          if (pair === pat && phonemes[j] === ph) {
            pieces.push({kind: 'digraph', letters: pair, phoneme: ph});
            i += 2; j += 1; matched = true; break;
          }
        }
      }
      if (matched) continue;
      // word-final silent suffix
      if (i + 2 === letters.length) {
        const pair = letters[i] + letters[i + 1];
        for (const [pat, surv] of SILENT_SUFFIXES) {
          if (pair === pat && phonemes[j] === surv) {
            pieces.push({kind: 'direct', letter: letters[i], phoneme: surv});
            pieces.push({kind: 'silent', letter: letters[i + 1]});
            i += 2; j += 1; matched = true; break;
          }
        }
      }
      if (matched) continue;
      // single-letter step
      if (j < phonemes.length && letterOk(letters[i], phonemes[j])) {
        pieces.push({kind: 'direct', letter: letters[i], phoneme: phonemes[j]});
        i += 1; j += 1;
      } else if (j + 1 < phonemes.length && letterOk(letters[i], phonemes[j + 1])) {
        pieces.push({kind: 'insert', phoneme: phonemes[j]});
        j += 1;
      } else {
        const lLeft = letters.length - i;
        const pLeft = phonemes.length - j;
        if (pLeft < lLeft) {
          pieces.push({kind: 'silent', letter: letters[i]});
          i += 1;
        } else if (j < phonemes.length) {
          pieces.push({kind: 'direct', letter: letters[i], phoneme: phonemes[j]});
          i += 1; j += 1;
        } else {
          pieces.push({kind: 'silent', letter: letters[i]});
          i += 1;
        }
      }
    }
    while (j < phonemes.length) {
      pieces.push({kind: 'insert', phoneme: phonemes[j]});
      j += 1;
    }
    return pieces;
  }

  /* ── Draw the spelling band ─────────────────────────────────── */
  function renderSpellingBand(word, ipa) {
    while (spellEl.firstChild) spellEl.removeChild(spellEl.firstChild);
    const pieces = alignSpelling(word, ipa);
    if (!pieces.length) return;
    const COLS = pieces.length;
    const colW = 100 / COLS;
    pieces.forEach((p, i) => {
      const cx = (i + 0.5) * colW;
      // Letters row at y=6.
      const lettersText = p.kind === 'digraph' ? p.letters
                       : (p.kind === 'silent' || p.kind === 'direct') ? p.letter
                       : '·';      // inserted phoneme: no letter source
      const lcls = p.kind === 'silent' ? 'sb-letter silent'
                 : p.kind === 'digraph' ? 'sb-letter digraph'
                 : p.kind === 'insert' ? 'sb-letter insert'
                 : 'sb-letter';
      spellEl.appendChild(ns('text',
        { x: cx, y: 7, 'text-anchor': 'middle', class: lcls }, lettersText));

      // Phonemes row at y=22.
      const pText = p.kind === 'silent' ? ''
                  : (p.kind === 'direct' || p.kind === 'insert') ? p.phoneme
                  : p.phoneme;
      if (pText) {
        spellEl.appendChild(ns('text',
          { x: cx, y: 23, 'text-anchor': 'middle', class: 'sb-phoneme ipa-font' }, pText));
      }

      // Connection line.
      if (p.kind === 'direct' || p.kind === 'digraph') {
        spellEl.appendChild(ns('line',
          { x1: cx, y1: 10, x2: cx, y2: 19, class: 'sb-link' }));
      } else if (p.kind === 'silent') {
        // dangling — a short stub to indicate "no sound"
        spellEl.appendChild(ns('line',
          { x1: cx, y1: 10, x2: cx, y2: 13, class: 'sb-link silent' }));
      } else if (p.kind === 'insert') {
        // inserted phoneme: dotted line up to a stand-in dot
        spellEl.appendChild(ns('line',
          { x1: cx, y1: 16, x2: cx, y2: 19, class: 'sb-link insert' }));
      }
    });
  }

  /* ── Build phoneme path on the chart ────────────────────────── */
  function chartPathFor(ipa) {
    const path = [];
    let order = 0;
    for (const ch of ipa) {
      if ('ˈˌːˑ'.includes(ch)) continue;
      const p = CHART_POS[ch];
      if (!p) continue;
      // For chart drawing, transform consonant coords from chart_layout
      // coordinate space (which uses 0..100 per axis split into vowel
      // top / consonant bottom bands) to SVG y. Mirrors what
      // render/chart.rs does in Rust.
      const [px, py, plane] = p;
      const VOWEL_TOP = 4, VOWEL_BOTTOM = 42, CONS_TOP = 56, CONS_BOTTOM = 96;
      const y = plane === 0
        ? VOWEL_TOP + (py / 100) * (VOWEL_BOTTOM - VOWEL_TOP)
        : CONS_TOP  + (py / 100) * (CONS_BOTTOM - CONS_TOP);
      path.push({ ch, x: px, y, order: ++order });
    }
    return path;
  }

  /* ── Animate the path across the chart + sagittal inset ─────── */
  async function animateWord(path) {
    const token = ++animationToken;
    // Clear the prior word's drawing — both the path strokes and the
    // `.on-path` marker on every chart phoneme that was in it.
    while (pathLayer.firstChild) pathLayer.removeChild(pathLayer.firstChild);
    chartEl.querySelectorAll('.ph.on-path').forEach(el => el.classList.remove('on-path'));

    if (path.length === 0) { lastStep = null; return; }

    let d = `M ${path[0].x.toFixed(2)} ${path[0].y.toFixed(2)}`;
    for (let i = 1; i < path.length; i++) {
      d += ` L ${path[i].x.toFixed(2)} ${path[i].y.toFixed(2)}`;
    }
    const pathEl = ns('path', { d, class: 'word-path' });
    pathLayer.appendChild(pathEl);
    const totalLen = pathEl.getTotalLength();
    pathEl.style.strokeDasharray = `${totalLen}`;
    pathEl.style.strokeDashoffset = `${totalLen}`;
    void pathEl.getBoundingClientRect();
    const totalMs = Math.max(700, path.length * 220);
    pathEl.style.transition = `stroke-dashoffset ${totalMs}ms cubic-bezier(.45,.05,.55,.95)`;
    pathEl.style.strokeDashoffset = '0';

    const perStep = Math.floor(totalMs / Math.max(path.length, 1));
    for (let i = 0; i < path.length; i++) {
      if (token !== animationToken) return;
      const stop = path[i];
      // Numbered, clickable badge at this stop. Carries data-ch +
      // data-step so the delegated click handler can route to it.
      const g = ns('g', {
        class: 'path-stop',
        'data-ch': stop.ch,
        'data-step': String(stop.order),
        transform: `translate(${stop.x.toFixed(2)} ${stop.y.toFixed(2)})`,
      });
      g.appendChild(ns('circle', { r: 2.6, class: 'stop-bg' }));
      g.appendChild(ns('text',
        { 'text-anchor': 'middle', 'dominant-baseline': 'central', y: 0.4, class: 'stop-n' },
        String(stop.order)));
      pathLayer.appendChild(g);
      const glyph = chartEl.querySelector(`.ph[data-ch="${CSS.escape(stop.ch)}"]`);
      if (glyph) glyph.classList.add('on-path');
      paintSagittal(stop.ch);
      lastStep = stop.ch;
      await sleep(perStep);
    }
  }

  /* ── Sagittal inset driver ──────────────────────────────────── */
  function paintSagittal(ch) {
    const spec = SAGITTAL_SPECS[ch];
    if (!spec) return;
    // Tongue: map tract x (0..100) into svg x (16..96), tract y
    // (0..100) into svg y (28..52). The schematic doesn't need to
    // be exact — proximity to the lips / palate is what teaches.
    const tx = 16 + (spec.tx / 100) * 80;
    const ty = 28 + (spec.ty / 100) * 24;
    tongue.setAttribute('cx', tx.toFixed(1));
    tongue.setAttribute('cy', ty.toFixed(1));
    tongue.setAttribute('rx', spec.air === 'blocked' ? '16' : '14');
    tongue.setAttribute('ry', spec.air === 'blocked' ? '11' : '9');
    // Lips.
    const lipPaths = {
      spread:   'M104 22 Q102 40 104 58',
      neutral:  'M104 24 Q108 40 104 56',
      rounded:  'M104 28 Q116 40 104 52',
      closed:   'M104 30 Q98 40 104 50',
      lowerlip: 'M104 24 Q102 40 100 50',
    };
    lips.setAttribute('d', lipPaths[spec.lips] || lipPaths.neutral);
    lips.setAttribute('class', `lips lips-${spec.lips}`);
    // Voicing — class on the cords group; CSS animates the buzz.
    cords.setAttribute('class', spec.voiced ? 'cords voiced' : 'cords');
    // Airflow path & class — CSS varies stroke style by mode.
    airflow.setAttribute('class', `airflow air-${spec.air}`);
    // Nasal channel — only visible for nasals.
    nasalPath.setAttribute('class', spec.air === 'nasal' ? 'nasal-path visible' : 'nasal-path');

    // Label + description.
    sagGlyph.textContent = ch;
    const info = PHONEME_INFO[ch];
    if (info) {
      sagLabel.textContent = info.name;
      sagDesc.textContent = info.desc;
    } else {
      sagLabel.textContent = '/' + ch + '/';
      sagDesc.textContent = '';
    }
  }

  /* ── Word selection entry point ─────────────────────────────── */
  function selectWord(word, ipa, rankZeroIdx) {
    qEl.value = word;
    clearBtn.classList.add('visible');
    hideSuggestions();
    wGlyph.textContent = word;
    wIpa.textContent = '/' + ipa + '/';
    wRank.textContent = '#' + (rankZeroIdx + 1) + ' by frequency';
    wHint.textContent = makeHint(word, ipa);
    renderSpellingBand(word, ipa);
    // A new word resets the per-phoneme commitment — the user is
    // now studying the word, not a specific phoneme.
    releaseSelection();
    const path = chartPathFor(ipa);
    animateWord(path);
    // Surface the first phoneme on the sagittal immediately so a user
    // who doesn't wait through the animation still sees something.
    if (path.length > 0) paintSagittal(path[0].ch);
  }

  function searchAndShow(word) {
    const q = word.toLowerCase();
    const i = WORDS.findIndex(w => w[0] === q);
    if (i >= 0) selectWord(WORDS[i][0], WORDS[i][1], i);
    else {
      qEl.value = word;
      clearBtn.classList.add('visible');
      hideSuggestions();
      wGlyph.textContent = word;
      wIpa.textContent = '';
      wRank.textContent = 'not in the top 50,000 — try a more common spelling';
      wHint.textContent = '';
      while (pathLayer.firstChild) pathLayer.removeChild(pathLayer.firstChild);
      while (spellEl.firstChild) spellEl.removeChild(spellEl.firstChild);
    }
  }

  function makeHint(word, ipa) {
    const letters = Array.from(word).filter(c => /[a-zA-Z]/.test(c));
    const phonemes = Array.from(ipa).filter(c => !'ˈˌːˑ'.includes(c));
    const diff = letters.length - phonemes.length;
    if (diff > 0) return `${word}: ${letters.length} letters → ${phonemes.length} sounds (${diff} letter${diff !== 1 ? 's' : ''} ${diff === 1 ? 'produces' : 'produce'} no extra sound)`;
    if (diff < 0) return `${word}: ${letters.length} letters → ${phonemes.length} sounds (${-diff} extra sound${-diff !== 1 ? 's' : ''} from a single letter)`;
    return `${word}: ${letters.length} letters, ${phonemes.length} sounds — a clean 1-to-1 spelling`;
  }

  function clearAll() {
    qEl.value = ''; clearBtn.classList.remove('visible');
    hideSuggestions();
    wGlyph.textContent = ''; wIpa.textContent = ''; wRank.textContent = ''; wHint.textContent = '';
    while (pathLayer.firstChild) pathLayer.removeChild(pathLayer.firstChild);
    while (spellEl.firstChild) spellEl.removeChild(spellEl.firstChild);
    chartEl.querySelectorAll('.ph.on-path').forEach(el => el.classList.remove('on-path'));
    releaseSelection();
    lastStep = null;
  }

  /* ── Chart phoneme focus model ──────────────────────────────────
   *
   * `previewPhoneme` runs on hover. It paints the sagittal but only
   * if the user hasn't *committed* to a phoneme via click. With a
   * commitment in place, hovering is purely visual on the chart side
   * (the underlying tile shows :hover styling) but the sagittal
   * stays put.
   *
   * `togglePhonemeSelection` runs on click. Clicking a fresh phoneme
   * commits to it. Clicking the same phoneme again releases the
   * commitment.
   *
   * `revertToBaseline` runs on mouseleave from the chart. With a
   * selection, sagittal stays on it. Without one, it falls back to
   * the final phoneme of the most-recent word path so the inset is
   * never "snapped to nothing".
   */
  function previewPhoneme(ch) {
    if (selectedCh) return;
    paintSagittal(ch);
  }
  function togglePhonemeSelection(ch) {
    if (selectedCh === ch) {
      releaseSelection();
      return;
    }
    if (selectedCh) {
      const prev = chartEl.querySelector(`.ph[data-ch="${CSS.escape(selectedCh)}"]`);
      if (prev) prev.classList.remove('selected');
    }
    selectedCh = ch;
    const next = chartEl.querySelector(`.ph[data-ch="${CSS.escape(ch)}"]`);
    if (next) next.classList.add('selected');
    paintSagittal(ch);
  }
  function releaseSelection() {
    if (!selectedCh) return;
    const prev = chartEl.querySelector(`.ph[data-ch="${CSS.escape(selectedCh)}"]`);
    if (prev) prev.classList.remove('selected');
    selectedCh = null;
    revertToBaseline();
  }
  function revertToBaseline() {
    const target = selectedCh || lastStep;
    if (target) paintSagittal(target);
  }
  /** Click a path-stop badge to jump the sagittal to that step. */
  function focusPathStep(ch) {
    paintSagittal(ch);
    lastStep = ch;
  }

  /* ── Reverse phoneme search ─────────────────────────────────── */
  function runReverseSearch() {
    if (!reverseQ) return;
    clearTimeout(reverseDebounce);
    reverseDebounce = setTimeout(() => {
      const raw = reverseQ.value.trim();
      if (!raw) { reverseR.innerHTML = ''; reverseC.textContent = ''; return; }
      const escaped = raw.replace(/[.*+?^${}()|[\]\\]/g, '\\$&').replace(/_/g, '.');
      let rx;
      try { rx = new RegExp(escaped); }
      catch (_) { reverseR.innerHTML = '<div class="reverse-empty">Invalid pattern.</div>'; return; }
      const hits = [];
      for (let i = 0; i < WORDS.length && hits.length < 200; i++) {
        const m = WORDS[i][1].match(rx);
        if (m) hits.push({ word: WORDS[i][0], ipa: WORDS[i][1], start: m.index, len: m[0].length });
      }
      if (!hits.length) { reverseR.innerHTML = '<div class="reverse-empty">No matches.</div>'; return; }
      reverseC.textContent = hits.length + (hits.length === 200 ? '+' : '') + ' matches';
      reverseR.innerHTML = hits.map(h => {
        const ipa = h.ipa;
        const before = escHTML(ipa.slice(0, h.start));
        const hit    = '<span class="match">' + escHTML(ipa.slice(h.start, h.start + h.len)) + '</span>';
        const after  = escHTML(ipa.slice(h.start + h.len));
        return `<button class="rrow" type="button" data-word="${escHTML(h.word)}">` +
                 `<span class="rrow-word">${escHTML(h.word)}</span>` +
                 `<span class="rrow-ipa ipa-font">${before}${hit}${after}</span>` +
               `</button>`;
      }).join('');
    }, 80);
  }

  /* ── Delegated listeners ────────────────────────────────────── */
  document.addEventListener('click', (e) => {
    if (e.target.closest('#search-clear')) { clearAll(); return; }
    const wordEl = e.target.closest('[data-word]');
    if (wordEl) { e.preventDefault(); searchAndShow(wordEl.dataset.word); return; }
    const stopEl = e.target.closest('.path-stop[data-ch]');
    if (stopEl) { e.preventDefault(); focusPathStep(stopEl.dataset.ch); return; }
    const phEl = e.target.closest('.ph[data-ch]');
    if (phEl) { e.preventDefault(); togglePhonemeSelection(phEl.dataset.ch); return; }
    // Click on chart background (anywhere in the chart svg that
    // isn't a phoneme tile or a path stop) clears any committed
    // selection — same gesture as clicking on the underlying canvas
    // in a typical map / diagram interaction.
    if (e.target.closest('#ipa-chart')) {
      releaseSelection();
      return;
    }
    if (!e.target.closest('.word-picker')) hideSuggestions();
  });

  // Hover on chart phonemes previews the sagittal — but only while
  // nothing is *committed* via click. With a selection, hovering
  // still shows the chart's :hover state on the tile but the
  // sagittal stays locked.
  chartEl?.addEventListener('mouseover', (e) => {
    const phEl = e.target.closest('.ph[data-ch]');
    if (phEl) previewPhoneme(phEl.dataset.ch);
  });
  chartEl?.addEventListener('mouseleave', () => revertToBaseline());

  document.addEventListener('input', (e) => {
    if (e.target === qEl) handleSearchInput();
    else if (e.target === reverseQ) runReverseSearch();
  });

  document.addEventListener('keydown', (e) => {
    if (e.target === qEl) {
      if (e.key === 'Escape') clearAll();
      if (e.key === 'Enter') {
        const q = qEl.value.trim().toLowerCase();
        const i = WORDS.findIndex(w => w[0] === q);
        if (i >= 0) selectWord(WORDS[i][0], WORDS[i][1], i);
      }
    }
  });

  /* ── First-load: pick a featured word ───────────────────────── */
  const featured = ['pneumonia','rhythm','through','queue','phonetics','colonel','thought'];
  const w = featured[Math.floor(Math.random() * featured.length)];
  const idx = WORDS.findIndex(x => x[0] === w);
  if (idx >= 0) selectWord(WORDS[idx][0], WORDS[idx][1], idx);
})();
