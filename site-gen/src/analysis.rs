//! Walk the corpus JSON and pull out everything the renderer needs:
//! per-phoneme statistics, source provenance, a frequency-sorted
//! word list, and a curated showcase.
//!
//! This module is pure data extraction — no rendering. The output
//! `Stats` struct is the single contract between the corpus and
//! every render module downstream.

use std::collections::{BTreeMap, HashMap};

use serde_json::Map;

use crate::phoneme_meta::{self, Category};

/// Source-resolution order. Mirrors phonetics-rs's `Corpus::preferred_ipa`
/// but reimplemented here so the build tool doesn't have to embed a
/// runtime corpus.
const SOURCE_PREFERENCE: &[&str] = &[
    "cmu",
    "misaki_gold",
    "misaki_silver",
    "phonemicchart",
    "wiktionary",
    "wikipron",
];

/// Pick the best transcription for one corpus entry's `ipa` map: try
/// each preference as an exact label match, then as a prefix match,
/// before falling back to whatever's first.
fn preferred_ipa(ipa_map: &Map<String, serde_json::Value>) -> Option<String> {
    for pref in SOURCE_PREFERENCE {
        if let Some(v) = ipa_map.get(*pref).and_then(|v| v.as_str()) {
            if !v.is_empty() {
                return Some(v.to_owned());
            }
        }
        for (k, v) in ipa_map {
            if k.starts_with(pref) {
                if let Some(s) = v.as_str() {
                    if !s.is_empty() {
                        return Some(s.to_owned());
                    }
                }
            }
        }
    }
    ipa_map
        .values()
        .find_map(|v| v.as_str().filter(|s| !s.is_empty()))
        .map(|s| s.to_owned())
}

/// One IPA phoneme's statistics across the whole corpus.
pub struct PhonemeInfo {
    pub ch: char,
    /// Total occurrences across all preferred-IPA transcriptions.
    pub count: usize,
    /// Categorical bucket for visual grouping.
    pub category: Category,
    /// Logarithmic size ratio (0..1) relative to the most common
    /// phoneme; drives the tile sizing in the Phoneme Universe.
    pub size_ratio: f64,
    /// Up to 30 example words containing this phoneme, sorted by
    /// frequency (most common first).
    pub examples: Vec<(String, String)>,
}

/// Aggregated corpus statistics consumed by every render module.
pub struct Stats {
    /// Total unique words with at least one transcription.
    pub total_words: usize,
    /// Per-source preferred-source bucket counts (cmu / misaki_gold /
    /// misaki_silver / wikipron / etc.).
    pub sources: Vec<(String, usize)>,
    /// Per-phoneme stats, sorted by count descending.
    pub phonemes: Vec<PhonemeInfo>,
    /// (word, ipa) for the top 50k words by frequency — drives both
    /// the search-as-you-type suggestions and the reverse phoneme
    /// search.
    pub search_words: Vec<(String, String)>,
    /// (word, ipa, rank) for the top 120 words by frequency — drives
    /// the showcase grid.
    pub showcase: Vec<(String, String, usize)>,
    /// Mean IPA length per word.
    pub avg_ipa_len: f64,
}

/// Walk the corpus JSON and produce a [`Stats`].
pub fn analyze(raw: &Map<String, serde_json::Value>) -> Stats {
    type PhonemeEntry = (usize, Vec<(String, String, f64)>);
    let mut phoneme_data: BTreeMap<char, PhonemeEntry> = BTreeMap::new();
    let mut source_counts: HashMap<String, usize> = HashMap::new();
    let mut all_words: Vec<(String, String, f64)> = Vec::new();
    let mut total_ipa_chars: usize = 0;

    for (word, entry) in raw {
        let obj = match entry.as_object() {
            Some(o) => o,
            None => continue,
        };
        let rarity = obj.get("rarity").and_then(|v| v.as_f64()).unwrap_or(f64::MAX);
        let ipa_map = match obj.get("ipa").and_then(|v| v.as_object()) {
            Some(m) => m,
            None => continue,
        };

        // Bucket sources by SOURCE_PREFERENCE prefix so cmu, cmu2,
        // cmu3 all collapse to "cmu" for the stats panel.
        for k in ipa_map.keys() {
            let bucket = SOURCE_PREFERENCE
                .iter()
                .find(|p| k.starts_with(*p))
                .copied()
                .unwrap_or(k.as_str());
            *source_counts.entry(bucket.to_owned()).or_default() += 1;
        }

        let ipa = match preferred_ipa(ipa_map) {
            Some(s) => s,
            None => continue,
        };

        total_ipa_chars += ipa.chars().count();
        all_words.push((word.clone(), ipa.clone(), rarity));

        for ch in ipa.chars() {
            let e = phoneme_data.entry(ch).or_default();
            e.0 += 1;
            if e.1.len() < 40 {
                e.1.push((word.clone(), ipa.clone(), rarity));
            }
        }
    }

    // Per-phoneme examples: sorted by frequency, deduped, capped at 30.
    for (_, examples) in phoneme_data.values_mut() {
        examples.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
        examples.dedup_by(|a, b| a.0 == b.0);
        examples.truncate(30);
    }

    all_words.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

    // Top 50k: large enough for reverse phoneme search to feel substantive
    // without bloating the embedded JS payload too aggressively.
    let search_words: Vec<(String, String)> = all_words
        .iter()
        .take(50_000)
        .map(|(w, ipa, _)| (w.clone(), ipa.clone()))
        .collect();

    let showcase: Vec<(String, String, usize)> = all_words
        .iter()
        .take(120)
        .enumerate()
        .map(|(i, (w, ipa, _))| (w.clone(), ipa.clone(), i + 1))
        .collect();

    let max_count = phoneme_data.values().map(|(c, _)| *c).max().unwrap_or(1);
    let max_log = (max_count as f64).ln();

    let mut phonemes: Vec<PhonemeInfo> = phoneme_data
        .into_iter()
        .map(|(ch, (count, examples))| {
            let size_ratio = if max_log > 0.0 {
                (count as f64).ln() / max_log
            } else {
                0.0
            };
            PhonemeInfo {
                ch,
                count,
                category: phoneme_meta::category(ch),
                size_ratio,
                examples: examples.into_iter().map(|(w, ipa, _)| (w, ipa)).collect(),
            }
        })
        .collect();

    phonemes.sort_by_key(|p| std::cmp::Reverse(p.count));

    let avg_ipa_len = if all_words.is_empty() {
        0.0
    } else {
        total_ipa_chars as f64 / all_words.len() as f64
    };

    let mut sources: Vec<(String, usize)> = source_counts.into_iter().collect();
    sources.sort_by_key(|s| std::cmp::Reverse(s.1));

    Stats {
        total_words: all_words.len(),
        sources,
        phonemes,
        search_words,
        showcase,
        avg_ipa_len,
    }
}
