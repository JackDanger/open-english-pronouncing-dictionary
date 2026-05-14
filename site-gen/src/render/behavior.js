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

  /** Bump the animation token so any in-flight walk-the-sagittal
   * loop bails on its next iteration. Called whenever the user does
   * something that should take precedence over the running word
   * animation (selecting a phoneme, dragging, loading another word). */
  function cancelAnimation() { animationToken++; }

  /* ── Workspace state ────────────────────────────────────────────
   *
   *   selectedCh    — phoneme the user *clicked* on the chart.
   *                   Sticky: persists across hover and mouseleave.
   *
   *   lastStep      — the phoneme the most-recent word-path step
   *                   landed on; used as fallback when nothing is
   *                   selected so the sagittal never snaps to nothing.
   *
   *   currentPath   — the mutable phoneme sequence currently drawn
   *                   on the chart. Drag-to-morph rewrites entries
   *                   in place; redrawPath() rebuilds the SVG layer
   *                   from this array. Each entry is {ch, x, y}.
   *
   *   originalWord  — the word the user started from. Stays put for
   *                   the morph breadcrumb ("↺ from PNEUMONIA") so
   *                   they can return at any time.
   */
  let selectedCh   = null;
  let lastStep     = null;
  let currentPath  = [];
  let originalWord = null;

  /* ── Reverse IPA → words index ──────────────────────────────────
   * Built once at script init. Drag-to-morph reads it on every snap
   * to decide whether the current phoneme sequence spells a real
   * English word. Stripped of stress / length marks since the chart
   * doesn't represent those positions. */
  const IPA_INDEX = new Map();
  const STRIP_STRESS_RE = /[ˈˌːˑ]/g;
  for (const [word, ipa] of WORDS) {
    const key = ipa.replace(STRIP_STRESS_RE, '');
    if (!IPA_INDEX.has(key)) IPA_INDEX.set(key, []);
    IPA_INDEX.get(key).push({ word, ipa });
  }

  /* Phonemes that are *only* the first half of a diphthong pair in
   * English. The path renderer thickens / curves the segment between
   * such a phoneme and the next path stop, since they're a single
   * articulatory glide rather than two independent moves. */
  const DIPHTHONG_PAIRS = new Set([
    'eɪ', 'aɪ', 'ɔɪ', 'oʊ', 'aʊ', 'ju',
    'iə', 'ɛə', 'ʊə',
  ]);
  function isDiphthongPair(a, b) {
    return DIPHTHONG_PAIRS.has(a + b);
  }

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

  /* ── Coordinate math: chart_layout space ↔ SVG space ─────────── */
  const VOWEL_TOP = 4, VOWEL_BOTTOM = 42, CONS_TOP = 56, CONS_BOTTOM = 96;
  function chartCoordsFor(ch) {
    const p = CHART_POS[ch];
    if (!p) return null;
    const [px, py, plane] = p;
    const y = plane === 0
      ? VOWEL_TOP + (py / 100) * (VOWEL_BOTTOM - VOWEL_TOP)
      : CONS_TOP  + (py / 100) * (CONS_BOTTOM - CONS_TOP);
    return { x: px, y, plane };
  }
  function planeForSvgY(y) {
    // Anything above the gap between the two bands is "vowel zone",
    // anything below is "consonant zone".
    return y < (VOWEL_BOTTOM + CONS_TOP) / 2 ? 0 : 1;
  }
  function findNearestPhoneme(svgX, svgY, plane) {
    let best = null;
    let bestDist = Infinity;
    for (const [ch, [px, py, p]] of Object.entries(CHART_POS)) {
      if (p !== plane) continue;
      const c = chartCoordsFor(ch);
      const dx = c.x - svgX;
      const dy = c.y - svgY;
      const d2 = dx * dx + dy * dy;
      if (d2 < bestDist) { bestDist = d2; best = ch; }
    }
    return best;
  }
  function screenToSvg(clientX, clientY) {
    const pt = chartEl.createSVGPoint();
    pt.x = clientX;
    pt.y = clientY;
    const ctm = chartEl.getScreenCTM();
    if (!ctm) return { x: 0, y: 0 };
    return pt.matrixTransform(ctm.inverse());
  }

  /* ── Build the phoneme path for a word's IPA ────────────────── */
  function chartPathFor(ipa) {
    const path = [];
    let order = 0;
    for (const ch of ipa) {
      if ('ˈˌːˑ'.includes(ch)) continue;
      const c = chartCoordsFor(ch);
      if (!c) continue;
      path.push({ ch, x: c.x, y: c.y, plane: c.plane, order: ++order });
    }
    return path;
  }

  /* ── Path rendering: build SVG path + numbered draggable stops ─
   *
   * Two render paths:
   *   - `redrawPath()` rebuilds the chart's path layer from
   *     `currentPath` in one shot. Called after every mutation
   *     (drag-snap, full word load post-animation).
   *   - `animateWord(path)` is the initial draw — `redrawPath`
   *     followed by the stroke-dashoffset reveal + sagittal walk.
   *
   * `currentPath` is the source of truth. Drag handlers mutate
   * entries in place and call `redrawPath`. */
  function clearPathLayer() {
    while (pathLayer.firstChild) pathLayer.removeChild(pathLayer.firstChild);
    chartEl.querySelectorAll('.ph.on-path').forEach(el => el.classList.remove('on-path'));
  }

  /** Render the current path as a static (no animation) layer. */
  function redrawPath() {
    clearPathLayer();
    if (currentPath.length === 0) return;

    // Build the underlying path with per-segment <line>s rather than
    // a single <path d="M…L…L…">. Separate segments let us:
    //   - apply marker-end (arrow) to each one independently
    //   - tag diphthong glides with a different class
    //   - keep stops sitting on top of the lines, not under them.
    for (let i = 0; i < currentPath.length - 1; i++) {
      const a = currentPath[i];
      const b = currentPath[i + 1];
      const cls = isDiphthongPair(a.ch, b.ch) ? 'word-seg diphthong' : 'word-seg';
      const seg = ns('line', {
        x1: a.x.toFixed(2), y1: a.y.toFixed(2),
        x2: b.x.toFixed(2), y2: b.y.toFixed(2),
        class: cls,
        'marker-end': 'url(#path-arrow)',
      });
      pathLayer.appendChild(seg);
    }

    // Draggable, numbered stop at each phoneme.
    currentPath.forEach((stop, idx) => {
      const g = ns('g', {
        class: 'path-stop',
        'data-ch': stop.ch,
        'data-step': String(stop.order),
        'data-idx': String(idx),
        transform: `translate(${stop.x.toFixed(2)} ${stop.y.toFixed(2)})`,
      });
      g.appendChild(ns('circle', { r: 2.8, class: 'stop-bg' }));
      g.appendChild(ns('text',
        { 'text-anchor': 'middle', 'dominant-baseline': 'central', y: 0.4, class: 'stop-n' },
        String(stop.order)));
      pathLayer.appendChild(g);

      const glyph = chartEl.querySelector(`.ph[data-ch="${CSS.escape(stop.ch)}"]`);
      if (glyph) glyph.classList.add('on-path');
    });
  }

  /** Initial draw for a freshly-selected word: redraw then animate. */
  async function animateWord(path) {
    const token = ++animationToken;
    currentPath = path;
    redrawPath();
    if (path.length === 0) { lastStep = null; return; }

    // Animate the line stroke-in. We don't have one single <path>
    // anymore — each segment is its own <line>. Sum total length and
    // animate each in turn.
    const segs = pathLayer.querySelectorAll('.word-seg');
    let totalLen = 0;
    const segLens = [];
    segs.forEach(s => {
      const x1 = +s.getAttribute('x1'), y1 = +s.getAttribute('y1');
      const x2 = +s.getAttribute('x2'), y2 = +s.getAttribute('y2');
      const len = Math.hypot(x2 - x1, y2 - y1);
      segLens.push(len);
      totalLen += len;
      s.style.strokeDasharray = `${len}`;
      s.style.strokeDashoffset = `${len}`;
    });
    void pathLayer.getBoundingClientRect();
    const totalMs = Math.max(700, path.length * 220);
    let elapsed = 0;
    segs.forEach((s, i) => {
      const segMs = (segLens[i] / Math.max(totalLen, 1)) * totalMs;
      s.style.transition = `stroke-dashoffset ${segMs.toFixed(0)}ms cubic-bezier(.45,.05,.55,.95) ${elapsed.toFixed(0)}ms`;
      s.style.strokeDashoffset = '0';
      elapsed += segMs;
    });

    // Walk the sagittal in sync.
    const perStep = Math.floor(totalMs / Math.max(path.length, 1));
    for (let i = 0; i < path.length; i++) {
      if (token !== animationToken) return;
      paintSagittal(path[i].ch);
      lastStep = path[i].ch;
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
    // A new word resets the per-phoneme commitment AND the morph
    // breadcrumb — the user is now studying this word, not riffing
    // on something else.
    releaseSelection();
    originalWord = word;
    document.getElementById('morph-breadcrumb')?.classList.remove('visible');
    document.getElementById('morph-noword')?.classList.remove('visible');
    const path = chartPathFor(ipa);
    animateWord(path);
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
    // Any in-flight word animation should defer to the user's
    // explicit click — otherwise the animation's next paintSagittal
    // call would clobber what the user just selected.
    cancelAnimation();
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

  /* ── Drag-to-morph ──────────────────────────────────────────────
   *
   * Each path stop is a draggable handle. While dragging, the stop
   * snaps to the nearest phoneme *in its plane* (vowels stay in the
   * vowel quadrilateral, consonants in the consonant grid). The
   * current path's IPA is looked up after every snap; if the result
   * is a real English word, the word header swaps to it. If not,
   * the "no word" indicator surfaces.
   *
   * When the morph lands on a real word, that word becomes the new
   * basis for further morphing — the user can keep dragging from
   * there. The original word stays as a breadcrumb ("↺ from SHIP")
   * so they can always return.
   */
  let dragState = null;
  function onStopPointerDown(e, stopGroup) {
    const idx = +stopGroup.dataset.idx;
    if (!Number.isFinite(idx) || idx < 0 || idx >= currentPath.length) return;
    e.preventDefault();
    // Cancel any in-flight word animation so the sagittal walk
    // doesn't fight the user's drag for control.
    cancelAnimation();
    dragState = { idx, dragging: false };
    stopGroup.setPointerCapture?.(e.pointerId);
    stopGroup.classList.add('dragging');
  }
  function onStopPointerMove(e) {
    if (!dragState) return;
    const pt = screenToSvg(e.clientX, e.clientY);
    const plane = currentPath[dragState.idx].plane;
    const nearest = findNearestPhoneme(pt.x, pt.y, plane);
    if (!nearest) return;
    if (nearest === currentPath[dragState.idx].ch) {
      dragState.dragging = true;
      return;
    }
    dragState.dragging = true;
    const c = chartCoordsFor(nearest);
    currentPath[dragState.idx] = {
      ...currentPath[dragState.idx],
      ch: nearest,
      x: c.x,
      y: c.y,
    };
    redrawPath();
    paintSagittal(nearest);
    lastStep = nearest;
    refreshWordFromPath();
  }
  function onStopPointerUp() {
    if (!dragState) return;
    const wasDragging = dragState.dragging;
    dragState = null;
    chartEl.querySelectorAll('.path-stop.dragging').forEach(el => el.classList.remove('dragging'));
    // If the pointer never actually moved across a phoneme boundary,
    // treat the gesture as a click-on-stop (existing focusPathStep
    // behaviour). The standard `click` event still fires after
    // pointerup, so we don't need to invoke it ourselves.
    void wasDragging;
  }

  /** Recompute IPA from `currentPath`, look it up, and update the
   *  word header / breadcrumb / "no word" indicator accordingly. */
  function refreshWordFromPath() {
    const ipa = currentPath.map(s => s.ch).join('');
    const hits = IPA_INDEX.get(ipa);
    const mbEl = document.getElementById('morph-breadcrumb');
    const nwEl = document.getElementById('morph-noword');
    const mbResetBtn = document.getElementById('morph-reset');

    if (hits && hits.length) {
      // We've landed on a real word. Show it as the current word.
      const { word, ipa: fullIpa } = hits[0];
      wGlyph.textContent = word;
      wIpa.textContent = '/' + fullIpa + '/';
      const i = WORDS.findIndex(w => w[0] === word);
      wRank.textContent = i >= 0 ? '#' + (i + 1) + ' by frequency' : '';
      wHint.textContent = makeHint(word, fullIpa);
      renderSpellingBand(word, fullIpa);
      nwEl.classList.remove('visible');
      if (originalWord && originalWord !== word) {
        mbResetBtn.textContent = originalWord;
        mbEl.classList.add('visible');
      } else {
        mbEl.classList.remove('visible');
      }
    } else {
      // Not in the corpus — show the phoneme sequence as-is, with a
      // clear "this isn't a word" indicator. Spelling band clears.
      wGlyph.textContent = '⟨—⟩';
      wIpa.textContent = '/' + ipa + '/';
      wRank.textContent = '';
      wHint.textContent = '';
      while (spellEl.firstChild) spellEl.removeChild(spellEl.firstChild);
      nwEl.classList.add('visible');
      if (originalWord) {
        mbResetBtn.textContent = originalWord;
        mbEl.classList.add('visible');
      } else {
        mbEl.classList.remove('visible');
      }
    }
  }

  function resetToOriginal() {
    if (!originalWord) return;
    const i = WORDS.findIndex(w => w[0] === originalWord);
    if (i >= 0) selectWord(WORDS[i][0], WORDS[i][1], i);
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
    if (e.target.closest('#morph-reset')) { e.preventDefault(); resetToOriginal(); return; }
    const wordEl = e.target.closest('[data-word]');
    if (wordEl) { e.preventDefault(); searchAndShow(wordEl.dataset.word); return; }
    const stopEl = e.target.closest('.path-stop[data-ch]');
    if (stopEl) { e.preventDefault(); focusPathStep(stopEl.dataset.ch); return; }
    const phEl = e.target.closest('.ph[data-ch]');
    if (phEl) { e.preventDefault(); togglePhonemeSelection(phEl.dataset.ch); return; }
    // Click on chart background (anywhere in the chart svg that
    // isn't a phoneme tile or a path stop) clears any committed
    // selection.
    if (e.target.closest('#ipa-chart')) {
      releaseSelection();
      return;
    }
    if (!e.target.closest('.word-picker')) hideSuggestions();
  });

  // Drag-to-morph on path stops. Pointer events because SVG mouse
  // drag is unreliable across browsers, and `setPointerCapture`
  // handles pointer-leave-during-drag for free.
  chartEl?.addEventListener('pointerdown', (e) => {
    const stop = e.target.closest('.path-stop[data-idx]');
    if (stop) onStopPointerDown(e, stop);
  });
  document.addEventListener('pointermove', (e) => {
    if (dragState) onStopPointerMove(e);
  });
  document.addEventListener('pointerup',     () => onStopPointerUp());
  document.addEventListener('pointercancel', () => onStopPointerUp());

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
