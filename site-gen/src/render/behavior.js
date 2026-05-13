/* OpenEPD client behaviour.
 *
 * Data values (WORDS, PHONEME_INFO, PHONEME_WORDS, PHONEME_AXIS,
 * DISTANCE_MATRIX, MINIMAL_PAIR_INDEX) are injected by site-gen as
 * `const X = …;` declarations *immediately above* this block. We
 * never reach into the DOM to read data; we never embed JS inside
 * HTML attributes. Everything is wired through delegated event
 * listeners reading `data-word` / `data-ch` / `data-action` / etc.
 *
 * This avoids the entire bug class where escaping for an HTML
 * attribute and escaping for JS source conflict.
 */

(() => {
  /* ── DOM cache ──────────────────────────────────────────────── */
  const qEl       = document.getElementById('q');
  const clearBtn  = document.getElementById('search-clear');
  const suggestEl = document.getElementById('suggestions');
  const resultEl  = document.getElementById('word-result');
  const panelEl   = document.getElementById('phoneme-panel');
  const reverseQ  = document.getElementById('reverse-q');
  const reverseR  = document.getElementById('reverse-results');
  const reverseC  = document.getElementById('reverse-count');
  const distPanel = document.getElementById('distance-panel');
  const heatmap   = document.getElementById('heatmap');

  let debounceTimer = null;
  let reverseTimer  = null;

  /* ── Helpers ────────────────────────────────────────────────── */
  function fmtK(n) {
    if (n >= 1e6) return (n/1e6).toFixed(1) + 'M';
    if (n >= 1e3) return Math.round(n/1e3) + 'k';
    return String(n);
  }

  function escHTML(s) {
    return String(s)
      .replace(/&/g,'&amp;')
      .replace(/</g,'&lt;')
      .replace(/>/g,'&gt;')
      .replace(/"/g,'&quot;');
  }

  function catShort(cat) {
    return ({vowel:'vowel',stop:'stop',fricative:'fric.',
             nasal:'nasal',approx:'approx.',affricate:'aff.',
             supra:'',other:''}[cat]) || cat;
  }

  /* Category-level plain-English copy for the phoneme panel. */
  const CAT_DESC = {
    vowel: 'Vowels are made with an open vocal tract. The height and position of your tongue, and the shape of your lips, are the only things that change the sound.',
    stop: 'Stops (plosives) block all airflow completely — then release it in a tiny burst. Like a valve snapping open.',
    fricative: 'Fricatives squeeze air through a narrow gap, creating turbulence: hissing, buzzing, or hushing.',
    nasal: 'Nasals route air through the nose while the mouth is blocked. Humming produces a nasal sound.',
    approx: 'Approximants narrow the vocal tract without creating enough friction to produce turbulence. They glide between vowel-like and consonant-like.',
    affricate: 'Affricates start as a complete stop, then release as a fricative. Two sounds fused into one.',
    supra: 'Suprasegmentals mark properties of syllables rather than sounds: stress, length, and tone.',
    other: 'A transcription symbol used in phonetic notation.'
  };

  /* ── Top-level word search ──────────────────────────────────── */
  function handleInput() {
    const v = qEl.value;
    clearBtn.classList.toggle('visible', v.length > 0);
    clearTimeout(debounceTimer);
    const q = v.trim().toLowerCase();
    if (!q) { hideSuggestions(); return; }
    const ex = WORDS.findIndex(w => w[0] === q);
    if (ex >= 0) { hideSuggestions(); showWordResult(q, WORDS[ex][1], ex); return; }
    debounceTimer = setTimeout(() => showSuggestions(q), 100);
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
      '<button class="suggestion-item" type="button" data-word="' + escHTML(WORDS[i][0]) + '">' +
        '<span class="sug-word">' + escHTML(WORDS[i][0]) + '</span>' +
        '<span class="sug-ipa ipa-font">' + escHTML(WORDS[i][1]) + '</span>' +
        '<span class="sug-rank">#' + (i + 1) + '</span>' +
      '</button>'
    ).join('');
    suggestEl.classList.add('active');
  }

  function hideSuggestions() {
    suggestEl.innerHTML = '';
    suggestEl.classList.remove('active');
  }

  function clearAll() {
    qEl.value = '';
    clearBtn.classList.remove('visible');
    hideSuggestions();
    resultEl.innerHTML = '';
    resultEl.classList.remove('has-result');
    panelEl.innerHTML = '';
    panelEl.classList.remove('active');
    document.querySelectorAll('.phoneme-token.active').forEach(el => el.classList.remove('active'));
    document.querySelectorAll('.ptile.universe-highlight').forEach(el => el.classList.remove('universe-highlight'));
  }

  function searchAndShow(word) {
    qEl.value = word;
    clearBtn.classList.add('visible');
    hideSuggestions();
    const q = word.toLowerCase();
    const i = WORDS.findIndex(w => w[0] === q);
    if (i >= 0) {
      showWordResult(word, WORDS[i][1], i);
    } else {
      resultEl.innerHTML =
        '<div style="padding:1.5rem;color:#999;font-size:.9rem">"' +
        escHTML(word) + '" was not found in the top 50,000 most common words.</div>';
      resultEl.classList.add('has-result');
    }
  }

  /* ── Word result card ───────────────────────────────────────── */
  function showWordResult(word, ipa, rank) {
    panelEl.innerHTML = '';
    panelEl.classList.remove('active');
    document.querySelectorAll('.phoneme-token.active').forEach(el => el.classList.remove('active'));

    const chars = Array.from(ipa);
    const phonemeChars = chars.filter(c => c !== 'ˈ' && c !== 'ˌ' && c !== 'ː');
    const lc = word.length;
    const pc = phonemeChars.length;

    const breakdown = chars.map(ch => {
      const info = PHONEME_INFO[ch] || { name: '/' + ch + '/', category: 'other', color: '#6b7280' };
      const label = catShort(info.category);
      return '<button class="phoneme-token" type="button"' +
        ' style="--tok-color:' + escHTML(info.color) + ';"' +
        ' aria-label="' + escHTML(info.name) + '"' +
        ' title="' + escHTML(info.name) + '"' +
        ' data-ch="' + escHTML(ch) + '">' +
        '<span class="tok-sym">' + escHTML(ch) + '</span>' +
        (label ? '<span class="tok-label">' + escHTML(label) + '</span>' : '') +
        '</button>';
    }).join('');

    let insight = '';
    if (lc !== pc) {
      const diff = lc - pc;
      const msg = diff > 0
        ? lc + ' letter' + (lc !== 1 ? 's' : '') + ' → ' + pc + ' sound' + (pc !== 1 ? 's' : '') +
          ' &nbsp;·&nbsp; ' + Math.abs(diff) + ' letter' + (Math.abs(diff) !== 1 ? 's' : '') + ' produce no extra sounds'
        : lc + ' letters → ' + pc + ' sounds';
      insight = '<div class="spelling-insight"><span class="si-word">' + escHTML(word) + '</span>: ' + msg + '</div>';
    }

    const fact = getFact(chars, word, ipa);

    resultEl.innerHTML =
      '<div class="result-top">' +
        '<span class="result-word serif">' + escHTML(word) + '</span>' +
        '<span class="result-ipa ipa-font">' + escHTML(ipa) + '</span>' +
        '<span class="result-rank">#' + (rank + 1) + ' most common</span>' +
      '</div>' +
      insight +
      '<div class="phoneme-breakdown" role="group" aria-label="Click each sound to explore">' + breakdown + '</div>' +
      '<p class="breakdown-hint">Click any sound above to learn how to make it</p>' +
      (fact ? '<div class="fun-fact">' + fact + '</div>' : '');

    resultEl.classList.add('has-result');

    if (window.innerWidth < 768) {
      setTimeout(() => resultEl.scrollIntoView({ behavior: 'smooth', block: 'nearest' }), 50);
    }
  }

  /* ── Phoneme detail panel ───────────────────────────────────── */
  function selectPhoneme(btnOrNull, ch) {
    document.querySelectorAll('.phoneme-token.active').forEach(el => el.classList.remove('active'));
    if (btnOrNull) btnOrNull.classList.add('active');
    renderPhonemePanel(ch);
  }

  function selectPhonemeFromUniverse(ch) {
    document.getElementById('search').scrollIntoView({ behavior: 'smooth', block: 'start' });
    setTimeout(() => renderPhonemePanel(ch), 350);
  }

  function renderPhonemePanel(ch) {
    const info = PHONEME_INFO[ch] || {
      name: '/' + ch + '/', desc: 'An IPA symbol found in this corpus.',
      category: 'other', color: '#6b7280', wordCount: 0
    };
    const words = PHONEME_WORDS[ch] || [];
    const catDesc = CAT_DESC[info.category] || '';

    panelEl.innerHTML =
      '<div class="panel-glyph ipa-font" style="background:' + escHTML(info.color) + '14;color:' + escHTML(info.color) + ';">' + escHTML(ch) + '</div>' +
      '<div class="panel-body">' +
        '<div class="panel-header-row">' +
          '<span class="panel-name">' + escHTML(info.name) + '</span>' +
          '<span class="panel-cat-badge" style="background:' + escHTML(info.color) + '1a;color:' + escHTML(info.color) + ';">' + escHTML(info.category) + '</span>' +
        '</div>' +
        '<p class="panel-desc">' + escHTML(info.desc) + '</p>' +
        (catDesc ? '<p class="panel-cat-desc">' + escHTML(catDesc) + '</p>' : '') +
        '<div class="panel-stat">' + fmtK(info.wordCount) + ' words in this corpus use /' + escHTML(ch) + '/</div>' +
        (words.length
          ? '<div class="panel-words-label">Words with /' + escHTML(ch) + '/</div>' +
            '<div class="panel-words">' +
              words.slice(0, 20).map(w =>
                '<button class="panel-word" type="button" data-word="' + escHTML(w[0]) + '">' +
                  escHTML(w[0]) +
                  '<span class="panel-word-ipa ipa-font">' + escHTML(w[1]) + '</span>' +
                '</button>'
              ).join('') +
            '</div>'
          : '') +
      '</div>';

    panelEl.classList.add('active');

    document.querySelectorAll('.ptile.universe-highlight').forEach(el => el.classList.remove('universe-highlight'));
    const tile = document.querySelector('.ptile[data-ch="' + CSS.escape(ch) + '"]');
    if (tile) tile.classList.add('universe-highlight');
  }

  /* ── Fun facts (heuristic) ──────────────────────────────────── */
  function getFact(chars, word, ipa) {
    const plain = chars.filter(c => c !== 'ˈ' && c !== 'ˌ' && c !== 'ː');
    const schwaCount = chars.filter(c => c === 'ə').length;
    if (schwaCount > 1) {
      return 'The schwa /ə/ — the most common vowel in English — appears ' + schwaCount +
             ' times in this word. It\'s the sound of unstressed syllables in hundreds of common words.';
    }
    if (schwaCount === 1 && word.length > 4) {
      return 'The schwa /ə/ hides in this word. It\'s the most frequent vowel in English, ' +
             'appearing in every unstressed syllable — yet it has no unique letter, borrowed instead from whatever vowel the spelling uses.';
    }
    if (chars.includes('θ') && chars.includes('ð')) {
      return 'This word contains both English "th" sounds: /θ/ (voiceless, as in "thin") and /ð/ (voiced, as in "this"). Most of the world\'s languages have neither.';
    }
    if (chars.includes('ʔ')) {
      return 'This word contains a glottal stop /ʔ/ — the catch in the throat you make in "uh-oh". Many speakers insert it unconsciously before stressed vowels.';
    }
    if (chars.includes('ŋ') && !word.startsWith('ng')) {
      return 'The /ŋ/ sound ("ng") cannot begin a word in English — only appear mid-word or at the end. It\'s a positional constraint called a phonotactic rule.';
    }
    const vowelSet = 'iɪeɛæaɑɔoʊuəʌɜ';
    const vowels = plain.filter(c => vowelSet.includes(c));
    if (vowels.length === 0) {
      return 'Remarkably, "' + word + '" has no conventional vowel letters — yet English speakers produce what linguists call "syllabic consonants" that carry the syllable the way a vowel normally would.';
    }
    if (chars.includes('ˈ')) {
      return 'The stress mark ˈ shows where emphasis falls. English uses stress to distinguish words: "REcord" (noun) vs "reCORD" (verb) have identical sounds — only the stress differs.';
    }
    if (chars.includes('ː')) {
      return 'The length mark ː means the preceding sound is held longer. English vowel length is contrastive: "ship" /ʃɪp/ vs "sheep" /ʃiːp/ — same sounds, different duration.';
    }
    return null;
  }

  /* ── Distance heatmap colouring + interaction ───────────────── */
  function paintHeatmap() {
    // `DISTANCE_MATRIX` is a top-level `const` in the page script —
    // block-scoped, not on `window`. Reference it by name.
    if (!heatmap || typeof DISTANCE_MATRIX === 'undefined') return;
    // The DISTANCE_MATRIX is a flat array of length n*n with one
    // {d,c} object per pair (row-major). PHONEME_AXIS gives the axis.
    const cells = heatmap.querySelectorAll('.heatmap-cell:not(.axis)');
    cells.forEach(cell => {
      const r = +cell.dataset.row;
      const c = +cell.dataset.col;
      const entry = DISTANCE_MATRIX[r * PHONEME_AXIS.length + c];
      if (!entry) return;
      const t = Math.min(1, entry.d / 1.5);
      // Interpolate cream → b500 by t (close = light, far = dark)
      // (rough approximation; CSS color-mix would be ideal but
      //  inline-style support is uneven)
      const r0=253,g0=248,b0=242,r1=30,g1=8,b1=16;
      const rgb = function (a,b,t){ return Math.round(a + (b-a)*t); };
      cell.style.setProperty('--cell-color',
        'rgb(' + rgb(r0,r1,t) + ',' + rgb(g0,g1,t) + ',' + rgb(b0,b1,t) + ')');
      cell.title = '/' + PHONEME_AXIS[r] + '/ ↔ /' + PHONEME_AXIS[c] + '/  d=' +
                   entry.d.toFixed(2) + '  conf=' + entry.c.toFixed(2);
    });
  }

  function showDistancePair(r, c) {
    if (!distPanel || typeof DISTANCE_MATRIX === 'undefined') return;
    const a = PHONEME_AXIS[r];
    const b = PHONEME_AXIS[c];
    const entry = DISTANCE_MATRIX[r * PHONEME_AXIS.length + c];
    if (!entry) return;
    const aInfo = PHONEME_INFO[a] || { name: '/' + a + '/' };
    const bInfo = PHONEME_INFO[b] || { name: '/' + b + '/' };
    const dPct = Math.min(100, entry.d / 1.5 * 100);
    const cPct = Math.min(100, entry.c / 1.5 * 100);

    let tip;
    if (a === b) {
      tip = 'Identity — the same phoneme. Distance is zero by definition.';
    } else if (entry.d < 0.2) {
      tip = 'These two sounds are very close acoustically. Listeners often confuse them, especially in noise or fast speech.';
    } else if (entry.d < 0.5) {
      tip = 'Moderately close. The contrast is real but not robust — exactly the kind of pair you find in minimal pairs that confuse learners.';
    } else if (entry.d < 1.0) {
      tip = 'Audibly distinct. A native listener separates these reliably; an L2 learner may still need practice.';
    } else {
      tip = 'Very different sounds — produced in different parts of the mouth, with different airflow. No native speaker confuses these.';
    }

    distPanel.innerHTML =
      '<div class="dp-title">' + escHTML(aInfo.name) + ' &nbsp;↔&nbsp; ' + escHTML(bInfo.name) + '</div>' +
      '<div class="dp-pair">' +
        '<span class="dp-glyph">' + escHTML(a) + '</span>' +
        '<span class="dp-arrow">↔</span>' +
        '<span class="dp-glyph">' + escHTML(b) + '</span>' +
      '</div>' +
      '<div class="dp-rows">' +
        '<span class="dp-label">Acoustic distance</span>' +
        '<span class="dp-value">' + entry.d.toFixed(3) + '</span>' +
        '<div class="dp-bar"><div class="dp-bar-fill" style="--pct:' + dPct.toFixed(1) + '%"></div></div>' +
        '<span class="dp-label">Listener confusion</span>' +
        '<span class="dp-value">' + entry.c.toFixed(3) + '</span>' +
        '<div class="dp-bar"><div class="dp-bar-fill" style="--pct:' + cPct.toFixed(1) + '%"></div></div>' +
      '</div>' +
      '<p class="dp-tip">' + escHTML(tip) + '</p>';

    document.querySelectorAll('.heatmap-cell.selected, .heatmap-cell.peer').forEach(el => {
      el.classList.remove('selected'); el.classList.remove('peer');
    });
    const sel = heatmap.querySelector('.heatmap-cell[data-row="' + r + '"][data-col="' + c + '"]');
    if (sel) sel.classList.add('selected');
    if (r !== c) {
      const peer = heatmap.querySelector('.heatmap-cell[data-row="' + c + '"][data-col="' + r + '"]');
      if (peer) peer.classList.add('peer');
    }
  }

  /* ── Reverse phoneme search ─────────────────────────────────── */
  function runReverseSearch() {
    if (!reverseQ) return;
    clearTimeout(reverseTimer);
    reverseTimer = setTimeout(() => {
      const raw = reverseQ.value.trim();
      if (!raw) {
        reverseR.innerHTML = '';
        reverseC.textContent = '';
        return;
      }
      // Build a regex from the pattern: `_` → `.`; everything else
      // is literal. We escape regex metachars in case a user types
      // a `.` or `+`.
      const escaped = raw.replace(/[.*+?^${}()|[\]\\]/g, '\\$&').replace(/_/g, '.');
      let rx;
      try { rx = new RegExp(escaped); }
      catch (_) {
        reverseR.innerHTML = '<div class="reverse-empty">Invalid pattern.</div>';
        reverseC.textContent = '';
        return;
      }
      const hits = [];
      for (let i = 0; i < WORDS.length && hits.length < 200; i++) {
        const m = WORDS[i][1].match(rx);
        if (m) hits.push({ word: WORDS[i][0], ipa: WORDS[i][1], start: m.index, len: m[0].length });
      }
      if (!hits.length) {
        reverseR.innerHTML = '<div class="reverse-empty">No matches in the top 50,000 words.</div>';
        reverseC.textContent = '';
        return;
      }
      reverseC.textContent = hits.length + (hits.length === 200 ? '+' : '') + ' matches';
      reverseR.innerHTML = hits.map(h => {
        const ipa = h.ipa;
        const before = escHTML(ipa.slice(0, h.start));
        const hit    = '<span class="match">' + escHTML(ipa.slice(h.start, h.start + h.len)) + '</span>';
        const after  = escHTML(ipa.slice(h.start + h.len));
        return '<button class="rrow" type="button" data-word="' + escHTML(h.word) + '">' +
                 '<span class="rrow-word">' + escHTML(h.word) + '</span>' +
                 '<span class="rrow-ipa ipa-font">' + before + hit + after + '</span>' +
               '</button>';
      }).join('');
    }, 80);
  }

  /* ── Delegated event listeners ──────────────────────────────── */
  document.addEventListener('click', (e) => {
    // The clear (✕) button next to the search field.
    if (e.target.closest('#search-clear')) {
      clearAll();
      return;
    }
    // Any element with data-word — top-100, try, suggestion, pair-row,
    // panel-word, reverse-search result.
    const wordEl = e.target.closest('[data-word]');
    if (wordEl) {
      e.preventDefault();
      searchAndShow(wordEl.dataset.word);
      return;
    }
    // Phoneme-token clicks inside the result card.
    const tokenEl = e.target.closest('.phoneme-token[data-ch]');
    if (tokenEl) {
      e.preventDefault();
      selectPhoneme(tokenEl, tokenEl.dataset.ch);
      return;
    }
    // Phoneme-Universe tile clicks.
    const tileEl = e.target.closest('.ptile[data-ch]');
    if (tileEl) {
      e.preventDefault();
      selectPhonemeFromUniverse(tileEl.dataset.ch);
      return;
    }
    // Distance-heatmap cell clicks.
    const cellEl = e.target.closest('.heatmap-cell[data-row]');
    if (cellEl) {
      e.preventDefault();
      showDistancePair(+cellEl.dataset.row, +cellEl.dataset.col);
      return;
    }
    // Outside the search wrap → close the suggestion dropdown.
    if (!e.target.closest('.search-wrap')) {
      hideSuggestions();
    }
  });

  document.addEventListener('input', (e) => {
    if (e.target === qEl) handleInput();
    else if (e.target === reverseQ) runReverseSearch();
  });

  document.addEventListener('keydown', (e) => {
    if (e.target === qEl) {
      if (e.key === 'Escape') clearAll();
      if (e.key === 'Enter') {
        const q = qEl.value.trim().toLowerCase();
        const i = WORDS.findIndex(w => w[0] === q);
        if (i >= 0) { hideSuggestions(); showWordResult(q, WORDS[i][1], i); }
      }
    }
  });

  /* ── First-load setup ───────────────────────────────────────── */
  paintHeatmap();

  // Pick an interesting featured word.
  const featured = ['rhythm','through','colonel','pneumonia','bouquet','queue'];
  const w = featured[Math.floor(Math.random() * featured.length)];
  const idx = WORDS.findIndex(x => x[0] === w);
  if (idx >= 0) {
    qEl.value = w;
    clearBtn.classList.add('visible');
    showWordResult(w, WORDS[idx][1], idx);
  }
})();
