//! OpenEPD — open English Pronouncing Dictionary.
//!
//! A fused ~280k-word US English IPA dictionary, built from Misaki
//! gold/silver + CMUdict 0.7b + WikiPron en_us broad. The corpus
//! ships embedded in the crate as a single JSON literal so consumers
//! don't need any runtime file lookup or build-time fetch.
//!
//! Two usage shapes:
//!
//! ```
//! # use phonetics::transcriptions::Corpus;
//! // Most callers want a ready-built phonetics-rs Corpus:
//! let corpus = open_english_pronouncing_dictionary::load().unwrap();
//! assert_eq!(corpus.preferred_ipa("stupid"), Some("stˈupəd"));
//! ```
//!
//! ```
//! // Callers that need the raw JSON (e.g. to send it to a JS frontend)
//! // can read the embedded literal directly:
//! let json = open_english_pronouncing_dictionary::CORPUS_JSON;
//! assert!(json.starts_with('{'));
//! ```
//!
//! The corpus is CC-BY-SA 4.0. See the project README for license
//! reasoning and per-source attribution.

use phonetics::transcriptions::{Corpus, Error};

/// The embedded corpus, ~28 MB of UTF-8 JSON. See the project README
/// and `corpus/README.md` for the schema and source provenance.
///
/// `include_str!` materialises the JSON into the binary at compile
/// time; there's no on-disk lookup at runtime.
pub const CORPUS_JSON: &str = include_str!("../data/openepd.json");

/// Parse the embedded corpus into a [`phonetics::transcriptions::Corpus`].
///
/// Equivalent to `Corpus::from_json(CORPUS_JSON, None)`. Returns the
/// full dictionary without a rarity cap; pass an explicit cap with
/// [`load_with_max_rarity`] if you want to drop the long tail.
///
/// Parsing the full corpus walks ~280k entries and builds a trie;
/// expect ~300 ms on a modern laptop. Most callers should hold the
/// returned `Corpus` for the process lifetime.
pub fn load() -> Result<Corpus, Error> {
    Corpus::from_json(CORPUS_JSON, None)
}

/// Like [`load`], but drops entries whose rarity rank exceeds `cap`.
///
/// `cap = 50_000` is a sensible default for downstream tools that
/// want common vocabulary without the obscure long tail (the rebuilt
/// corpus ranks ~280k words; the top 50k covers roughly the most
/// frequent ~18%).
pub fn load_with_max_rarity(cap: f64) -> Result<Corpus, Error> {
    Corpus::from_json(CORPUS_JSON, Some(cap))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_json_parses() {
        let corpus = load().expect("corpus should parse");
        // ~280k entries; just assert a sane lower bound rather than
        // freezing the exact count across rebuilds.
        assert!(corpus.word_count() > 200_000,
                "expected >200k words, got {}", corpus.word_count());
    }

    #[test]
    fn canonical_words_resolve() {
        let corpus = load().unwrap();
        // Narrow-vowel sanity checks: these were the words the old
        // ARPABET-derived corpora couldn't distinguish properly.
        assert!(corpus.preferred_ipa("stupid").unwrap().contains("up"));
        assert!(corpus.preferred_ipa("hid").unwrap().contains("ɪ"));
        assert!(corpus.preferred_ipa("just").unwrap().contains("dʒ"));
        // Function-word weak form:
        assert_eq!(corpus.preferred_ipa("a"), Some("ə"));
    }

    #[test]
    fn rarity_cap_reduces_word_count() {
        let full = load().unwrap();
        let capped = load_with_max_rarity(20_000.0).unwrap();
        assert!(capped.word_count() < full.word_count());
        // The cap should still leave common vocabulary in place.
        assert!(capped.preferred_ipa("cat").is_some());
        assert!(capped.preferred_ipa("the").is_some());
    }
}
