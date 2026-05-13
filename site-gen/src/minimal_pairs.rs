//! Extract minimal pairs from the corpus at build time.
//!
//! A *minimal pair* is two words whose IPA transcriptions differ by
//! exactly one phoneme position — `ship` /ʃɪp/ vs `sheep` /ʃip/,
//! `cot` /kɑt/ vs `caught` /kɔt/, `thin` /θɪn/ vs `then` /ðɛn`. The
//! contrast a minimal pair captures is the *only* difference in
//! meaning — that's what makes them prized pedagogically and the
//! standard tool for arguing that two sounds are distinct phonemes.
//!
//! Algorithm: for every word's IPA, produce a "blanked variant" for
//! each phoneme position (i.e. the IPA with one character replaced
//! by `\0`). Bucket by blanked variant. Any two words sharing a
//! bucket differ by exactly that one position.
//!
//! Pairs we emit are filtered for "interestingness":
//!
//! * both words appear in the top-20k by frequency, so they're real
//!   English a learner has heard
//! * the contrasting phoneme pair is unique on this run (we don't
//!   emit five examples of the same ʃ↔s contrast)
//! * the pair's phonetic distance is small but nonzero (so it's
//!   genuinely a near-merger contrast, not a /p/↔/ɹ/ accident)

use std::collections::HashMap;

use crate::analysis::Stats;

pub struct MinimalPair {
    /// `(word1, ipa1)` — the more frequent of the two by rank.
    pub a: (String, String),
    pub b: (String, String),
    /// Position in the IPA stream where they differ. Kept for tests
    /// and potential future highlighting; not consumed by the
    /// renderer today.
    #[allow(dead_code)]
    pub diff_index: usize,
    /// The phoneme contrast — `(in a, in b)`.
    pub contrast: (char, char),
    /// `phonetics::distance` between the two contrasting phonemes.
    pub distance: f64,
    /// Sum of the two words' frequency ranks — lower means a more
    /// common pair, used to sort the curated output.
    pub combined_rank: usize,
}

/// Extract a curated batch of minimal pairs from the search-words
/// list. Returns up to `max_pairs` examples, one per unique contrast.
pub fn extract(stats: &Stats, max_pairs: usize) -> Vec<MinimalPair> {
    // Bucket by blanked variant. Top-20k words only — minimal pairs
    // of obscure vocabulary aren't useful examples.
    let mut buckets: HashMap<(String, usize), Vec<(String, String, usize)>> = HashMap::new();
    for (rank, (word, ipa)) in stats.search_words.iter().take(20_000).enumerate() {
        for (i, _ch) in ipa.char_indices() {
            // Build the IPA with position `i` removed. Using
            // char_indices ensures multi-byte characters are handled
            // correctly.
            let mut blank: String = ipa[..i].to_string();
            let after = ipa[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(0);
            blank.push_str(&ipa[i + after..]);
            // Keep the position so words differing at the same
            // position group together but those at different
            // positions don't.
            buckets
                .entry((blank, i))
                .or_default()
                .push((word.clone(), ipa.clone(), rank));
        }
    }

    // Walk every bucket with ≥2 entries and emit one candidate per
    // unique pair.
    let mut candidates: Vec<MinimalPair> = Vec::new();
    for ((_blank, diff_index), entries) in &buckets {
        if entries.len() < 2 {
            continue;
        }
        for i in 0..entries.len() {
            for j in i + 1..entries.len() {
                let (w1, ipa1, _r1) = &entries[i];
                let (w2, ipa2, _r2) = &entries[j];
                // Same lexical headword with case variation, or
                // hyphenated compounds where the alphabetic match is
                // trivial — skip.
                if w1 == w2 || w1.contains('-') || w2.contains('-') {
                    continue;
                }
                // Filter out trivial / non-pedagogical pairs:
                //   - single/double letters (alphabet entries:
                //     b/bˈi/ vs p/pˈi/)
                //   - 3-letter pairs that overwhelmingly turn out to
                //     be proper-noun nicknames (abe/ape, ada/ava)
                //   - apostrophe-bearing forms (clitic noise)
                if w1.len() < 4 || w2.len() < 4 {
                    continue;
                }
                if w1.contains('\'') || w2.contains('\'') {
                    continue;
                }
                // Identify the two contrasting phonemes.
                let ch_a = ipa1[*diff_index..].chars().next();
                let ch_b = ipa2[*diff_index..].chars().next();
                let (Some(ch_a), Some(ch_b)) = (ch_a, ch_b) else {
                    continue;
                };
                if ch_a == ch_b {
                    continue;
                }
                // Skip stress-mark contrasts; they don't make
                // phonetic minimal pairs.
                if matches!(ch_a, 'ˈ' | 'ˌ' | 'ː') || matches!(ch_b, 'ˈ' | 'ˌ' | 'ː') {
                    continue;
                }
                let distance = phonetics::distance(&ch_a.to_string(), &ch_b.to_string());
                // Filter for near-mergers: phonemes that are
                // genuinely confusable, not p/ɹ outliers.
                if !(0.05..=0.65).contains(&distance) {
                    continue;
                }
                let (_, _, r1) = &entries[i];
                let (_, _, r2) = &entries[j];
                candidates.push(MinimalPair {
                    a: (w1.clone(), ipa1.clone()),
                    b: (w2.clone(), ipa2.clone()),
                    diff_index: *diff_index,
                    contrast: (ch_a, ch_b),
                    distance,
                    combined_rank: r1 + r2,
                });
            }
        }
    }

    // Dedupe by contrast pair (unordered) — we only want one example
    // per contrast. Prefer pairs where both words are common
    // (combined rank low) so the showcase reads like words a learner
    // would actually encounter.
    candidates.sort_by(|x, y| {
        x.combined_rank
            .cmp(&y.combined_rank)
            .then_with(|| x.a.0.cmp(&y.a.0))
    });

    let mut seen: std::collections::HashSet<(char, char)> = std::collections::HashSet::new();
    let mut out = Vec::with_capacity(max_pairs);
    for c in candidates {
        let key = if c.contrast.0 < c.contrast.1 {
            (c.contrast.0, c.contrast.1)
        } else {
            (c.contrast.1, c.contrast.0)
        };
        if seen.insert(key) {
            out.push(c);
            if out.len() >= max_pairs {
                break;
            }
        }
    }
    out
}
