# Corpus build

This directory is the reproducible recipe for `data/openepd.json` — the per-word English IPA dictionary that ships in this repo.

## Sources

Four open dictionaries fuse into one. License of the result is **CC-BY-SA 4.0** because WikiPron inherits Wiktionary's copyleft; omit WikiPron from the build to produce a permissively-licensed (Apache 2.0) variant.

| Source | Entries | License | Role |
|---|---|---|---|
| [Misaki `us_gold.json`](https://github.com/hexgrad/misaki/blob/main/misaki/data/us_gold.json) | 90,201 | Apache 2.0 | Vetted narrow IPA. Highest-quality core. Used by Kokoro TTS. |
| [CMUdict 0.7b](https://github.com/cmusphinx/cmudict) | 135,166 | BSD-style | Broad ARPABET. Best coverage of proper nouns; converted to IPA here. |
| Misaki `us_silver.json` | 93,361 | Apache 2.0 | Less-vetted near-IPA; fills CMU gaps. |
| [WikiPron `eng_latn_us_broad`](https://github.com/CUNY-CL/wikipron) | 101,371 | CC-BY-SA / Apache 2.0 | Wiktionary scrape. Captures pronunciation variants. |
| [`wordfreq`](https://pypi.org/project/wordfreq/) | n/a | MIT | Per-word frequency → rarity rank. |

## How the merge works

For each word, the build script attaches transcriptions from every source that has it. The preference order embedded in [phonetics-rs's `Corpus::preferred_ipa`](https://github.com/JackDanger/phonetics/blob/main/rust/phonetics/src/transcriptions.rs) decides which one a caller gets as canonical:

```
cmu  >  misaki_gold  >  misaki_silver  >  wikipron
```

Reasoning:

- **CMU first** because its broad transcription gives function words in their weak (contextual) form — `a` → `/ə/`, not `/eɪ/`. That's what English speakers actually produce.
- **Misaki gold second** for words CMU lacks. Misaki distinguishes narrow vowel pairs that ARPABET collapses (`i` vs `ɪ`, `u` vs `ʊ`), but it tends to give citation/strong forms for function words.
- **Misaki silver, then WikiPron** for the long tail.

Every variant from every source survives in the per-word `ipa` map; preference order only resolves the single canonical answer. Callers that want to enumerate alternates (Mad Gab generators, ASR pronunciation models, etc.) walk the whole map.

## ARPABET → IPA conversion

CMUdict uses ARPABET with stress digits (`HH AH0 L OW1`). The conversion is the standard one with two reductions:

- `AH0` (unstressed) → `/ə/` (not `/ʌ/`)
- `ER0` (unstressed) → `/ɚ/` (not `/ɝ/`)

Stress digits map: `0` no marker, `1` → `ˈ`, `2` → `ˌ` (placed before the syllable containing the marked vowel).

See `ARPABET_VOWELS` and `arpabet_to_ipa` in `build.py`.

## Misaki near-IPA → canonical IPA

Misaki ships a near-IPA convention with custom single-character diphthong tokens (`A`, `I`, `W`, `Y`, `O`) and a few invented symbols (`ᵊ`, `ᵻ`). The build expands these to canonical IPA — see `MISAKI_TO_IPA` and `misaki_to_ipa` in `build.py`. The expansion is loss-free in the direction we use; Misaki's own `to_espeak` function is the canonical reference.

## WikiPron normalisation

The "broad" WikiPron scrape is actually narrow-ish — it preserves combining diacritics (`̃` nasalisation, `̩` syllabic, `̚` no-audible-release), tied-bar affricate markers (`d͡ʒ`), and length marks (`ː`). The build strips these so each character of the resulting IPA is a real broad phoneme: stress marks and IPA letters only.

## Building

The build runs in a venv that's gitignored. From the repo root:

```bash
python3 -m venv corpus/venv
corpus/venv/bin/pip install wordfreq
mkdir -p corpus/sources
# Fetch the four source files into corpus/sources/:
curl -L https://raw.githubusercontent.com/hexgrad/misaki/main/misaki/data/us_gold.json   -o corpus/sources/us_gold.json
curl -L https://raw.githubusercontent.com/hexgrad/misaki/main/misaki/data/us_silver.json -o corpus/sources/us_silver.json
curl -L https://raw.githubusercontent.com/cmusphinx/cmudict/master/cmudict.dict          -o corpus/sources/cmudict.dict
curl -L https://raw.githubusercontent.com/CUNY-CL/wikipron/master/data/scrape/tsv/eng_latn_us_broad.tsv \
                                                                                         -o corpus/sources/wikipron_us_broad.tsv
# Build:
corpus/venv/bin/python corpus/build.py
```

Writes `data/openepd.json` (~28 MB, ~280k unique words).

## Output schema

```json
{
  "stupid": {
    "rarity": 1907.0,
    "ipa": {
      "misaki_gold": "stˈupəd",
      "cmu":         "stˈupəd",
      "cmu2":        "stˈupɪd",
      "wikipron":    "stjupɪd",
      "wikipron2":   "stupɪd"
    },
    "alt_display": "STUPID"
  }
}
```

`rarity` is the 0-indexed rank by descending Zipf frequency from the `wordfreq` package. Rank 0 is `the`; words `wordfreq` doesn't know sort to the end of the rank scale. Callers commonly use it to drop the long tail (e.g. `--max-rarity 50000` keeps ~the top 18% by frequency).

`alt_display` is the surface form to show in clue output (currently the uppercased word; reserved for future use, e.g. preserved punctuation).

`ipa` is a per-source dict. Source labels:

- `cmu`, `cmu2`, `cmu3`, …  — CMUdict variants in original order
- `misaki_gold` — Misaki US gold
- `misaki_silver` — Misaki US silver
- `wikipron`, `wikipron2`, `wikipron3` — WikiPron variants

## Why this beats the alternatives

| Alternative | What's missing |
|---|---|
| Plain CMUdict | ARPABET vowel collapse (`i` vs `ɪ` indistinguishable); no narrow IPA |
| Plain Misaki gold | 90k entries — misses proper nouns, modern vocabulary |
| Plain WikiPron | Raw scrape quality issues; tied-bar/length-mark noise; CC-BY-SA constraint without compensating coverage |
| Neural G2P (LatPhon, ByT5) | English PER ~12.7% in the SOTA — worse than dictionaries; weights not yet released |

Fusing all of them gives ~280k entries with the narrow-vowel quality of Misaki, the long-tail coverage of CMU, and pronunciation-variant diversity from WikiPron — at the cost of CC-BY-SA on the result.
