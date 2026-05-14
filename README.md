# Open English Pronouncing Dictionary

*OpenEPD*  is a synthesis of multiple open phonetics corpi. You can use this freely under CC-BY-SA 4.0

This has ~280,000 US English words with canonical IPA, optional pronunciation variants, frequency-derived rarity rank, and per-entry source provenance.

## Interactive explorer

We render `data/openepd.json` into [an educational site](https://jackdanger.github.io/open-english-pronouncing-dictionary/) that lets a non-linguist type any word and see its phonemes annotated.

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

## Why it exists

The widely-used open English pronunciation dictionaries each have a major hole:

- **[CMUdict](https://github.com/cmusphinx/cmudict)** uses ARPABET, which collapses /i/–/ɪ/ and /u/–/ʊ/. Best long-tail coverage (~135k entries), but vowel quality is lossy.
- **[Misaki](https://github.com/hexgrad/misaki)** ships ~90k vetted entries with proper narrow IPA — what [Kokoro TTS](https://huggingface.co/hexgrad/Kokoro-82M) uses — but it's small.
- **[WikiPron](https://github.com/CUNY-CL/wikipron)** scrapes Wiktionary and captures pronunciation variation, but the data is raw and CC-BY-SA.
- **Neural G2P** (LatPhon, ByT5) currently sits at ~13% English PER — worse than dictionaries.

Fusing them gives narrow vowel quality, long-tail coverage, *and* variant capture, at the cost of CC-BY-SA on the result. This repo is the fusion pipeline plus the artifact it produces.

## Sources

| Source | Entries | License | Role |
|---|---|---|---|
| [Misaki `us_gold`](https://github.com/hexgrad/misaki) | 90,201 | Apache 2.0 | Vetted narrow IPA. Highest-quality core. |
| [CMUdict 0.7b](https://github.com/cmusphinx/cmudict) | 135,166 | BSD-style | Broad ARPABET, converted to IPA at build time. |
| Misaki `us_silver` | 93,361 | Apache 2.0 | Less-vetted near-IPA; gap fill. |
| [WikiPron `eng_latn_us_broad`](https://github.com/CUNY-CL/wikipron) | 101,371 | CC-BY-SA / Apache 2.0 | Wiktionary scrape, pronunciation variants. |
| [`wordfreq`](https://pypi.org/project/wordfreq/) | n/a | MIT | Per-word rarity rank from Zipf frequency. |

The pipeline in `corpus/` is reproducible — fetch the four source files, run `corpus/build.py`, get the exact `data/openepd.json` we ship. Details (ARPABET → IPA mapping, Misaki near-IPA expansion, WikiPron narrow-form stripping) live in [`corpus/README.md`](corpus/README.md).

Run it locally:

```bash
cargo run --release -p site-gen -- data/openepd.json _site
open _site/index.html
```

## Using it

### From Rust

```toml
[dependencies]
open-english-pronouncing-dictionary = "0.1"
phonetics = { package = "phonetics-rs", version = "0.3", features = ["transcriptions"] }
```

```rust
use open_english_pronouncing_dictionary as openepd;

let corpus = openepd::load()?;
corpus.preferred_ipa("stupid");        // → Some("stˈupəd")
corpus.transcribe("cat dog");          // → Some("kˈætdˈɔɡ")
```

`load()` returns a `phonetics::transcriptions::Corpus`, which carries both a forward word→IPA map and a reverse IPA-prefix trie. The raw JSON is also exposed as `openepd::CORPUS_JSON` (a `&'static str`) for callers that want to parse it themselves.

### From any other language

`data/openepd.json` is plain JSON. Schema is documented in [`corpus/README.md`](corpus/README.md#output-schema).

## Why CC-BY-SA 4.0

WikiPron inherits Wiktionary's CC-BY-SA. Including its entries — which is what gives us the variant-capture advantage — binds the merged corpus to CC-BY-SA too. To produce a permissive (Apache 2.0) build, omit WikiPron in `corpus/build.py`; you'll lose ~25k variant transcriptions and the long-tail Wiktionary entries.

The Rust loader code in `src/` is also CC-BY-SA, for simplicity — it's ten lines of `include_str!`, and the data is the only thing worth licensing here.

## Compared to

- **CMUdict**: this is CMUdict-as-IPA *plus* three other layers. ~2× the unique words and proper narrow vowels.
- **Misaki gold**: this is a strict superset (Misaki gold is included verbatim as the preferred layer for the 90k words it covers).
- **WikiPron US broad scrape**: same source data, but cleaned to broad IPA, merged with higher-quality dictionaries, and indexed by frequency.

## Status

v0.1 — first published release. The schema is stable; refresh cadence will be driven by upstream source updates (Misaki, CMUdict, WikiPron all see periodic revisions).

## License

CC-BY-SA 4.0. See `LICENSE`.

Upstream attributions for the merged data are listed in `corpus/README.md` and preserved per-entry as the `ipa` source labels (`misaki_gold`, `cmu`, `misaki_silver`, `wikipron`).
