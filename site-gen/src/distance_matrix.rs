//! Build-time precompute of the per-pair phoneme distance matrix
//! using `phonetics-rs`. Two metrics are bundled:
//!
//! * **`distance`** — strict per-phoneme acoustic distance (Bark-space
//!   vowel distance, 2D consonant place embedding). Answers: how
//!   different do these two sounds *sound*?
//! * **`confusion`** — listener-confusion distance, calibrated against
//!   Mad Gab puzzle data and English speech-perception literature.
//!   Answers: how often will a listener mistake one for the other?
//!
//! The matrix is computed at build time and embedded into the page as
//! a JSON literal so the visualizer can drive its heatmap without
//! shipping the phonetic-distance math to the browser.

use crate::analysis::Stats;

/// One row of the distance matrix: every phoneme paired with `a`,
/// with both metrics.
pub struct DistanceRow {
    pub a: char,
    /// Indexed alongside [`Matrix::phonemes`] — `cells[i]` is the
    /// distance from `a` to `phonemes[i]`.
    pub cells: Vec<DistanceCell>,
}

pub struct DistanceCell {
    pub distance: f64,
    pub confusion: f64,
}

pub struct Matrix {
    /// Phoneme axis (rows = columns).
    pub phonemes: Vec<char>,
    pub rows: Vec<DistanceRow>,
}

/// Build the full matrix. Only "phonemic" symbols participate (vowels,
/// consonants, affricates) — stress and length marks are excluded
/// because they aren't sounds in their own right and phonetics-rs
/// returns 0 for both metrics on them.
pub fn compute(stats: &Stats) -> Matrix {
    let phonemes: Vec<char> = stats
        .phonemes
        .iter()
        .filter(|p| p.category.slug != "supra" && p.category.slug != "other")
        .map(|p| p.ch)
        .collect();

    let rows: Vec<DistanceRow> = phonemes
        .iter()
        .map(|&a| {
            let a_s = a.to_string();
            let cells = phonemes
                .iter()
                .map(|&b| {
                    let b_s = b.to_string();
                    DistanceCell {
                        distance: phonetics::distance(&a_s, &b_s),
                        confusion: phonetics::confusion(&a_s, &b_s),
                    }
                })
                .collect();
            DistanceRow { a, cells }
        })
        .collect();

    Matrix { phonemes, rows }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_stats() -> Stats {
        // Construct just enough of a Stats to exercise compute(). We
        // only need three phonemes total.
        use crate::analysis::PhonemeInfo;
        use crate::phoneme_meta::category;
        let mk = |ch: char| PhonemeInfo {
            ch,
            count: 1,
            category: category(ch),
            size_ratio: 1.0,
            examples: vec![],
        };
        Stats {
            total_words: 0,
            sources: vec![],
            phonemes: vec![mk('s'), mk('z'), mk('ʌ')],
            search_words: vec![],
            showcase: vec![],
            avg_ipa_len: 0.0,
        }
    }

    #[test]
    fn diagonal_is_zero() {
        let m = compute(&tiny_stats());
        for (i, row) in m.rows.iter().enumerate() {
            let c = &row.cells[i];
            assert_eq!(c.distance, 0.0, "distance({},{}) should be 0", row.a, row.a);
            assert_eq!(c.confusion, 0.0, "confusion({},{}) should be 0", row.a, row.a);
        }
    }

    #[test]
    fn voicing_pair_is_close_but_nonzero() {
        // /s/ and /z/ differ only in voicing — should be a small
        // distance, definitely nonzero.
        let m = compute(&tiny_stats());
        let s_idx = m.phonemes.iter().position(|&c| c == 's').unwrap();
        let z_idx = m.phonemes.iter().position(|&c| c == 'z').unwrap();
        let d = m.rows[s_idx].cells[z_idx].distance;
        assert!(d > 0.0, "s↔z distance should be positive");
        assert!(d < 0.5, "s↔z distance should be small; got {}", d);
    }
}
