//! The client-side payload: data declarations followed by behaviour.
//!
//! Data is emitted via `serde_json::to_string` so the JSON is always
//! syntactically valid JS — no hand-escaping, no quote-collision
//! risk. Behaviour lives in `behavior.js` next to this file and is
//! `include_str!`'d so it benefits from real JS syntax checking when
//! authored.

use maud::{html, Markup, PreEscaped};
use serde_json::json;

use crate::analysis::Stats;
use crate::distance_matrix::Matrix;
use crate::minimal_pairs::MinimalPair;
use crate::phoneme_meta;

const BEHAVIOR_JS: &str = include_str!("behavior.js");

pub fn render(stats: &Stats, matrix: &Matrix, _pairs: &[MinimalPair]) -> Markup {
    let words_json = encode_words(stats);
    let info_json = encode_phoneme_info(stats);
    let words_map_json = encode_phoneme_words(stats);
    let axis_json = encode_axis(matrix);
    let matrix_json = encode_matrix(matrix);

    let script_body = format!(
        "const WORDS={words_json};\n\
         const PHONEME_INFO={info_json};\n\
         const PHONEME_WORDS={words_map_json};\n\
         const PHONEME_AXIS={axis_json};\n\
         const DISTANCE_MATRIX={matrix_json};\n\
         {BEHAVIOR_JS}"
    );

    html! {
        script { (PreEscaped(script_body)) }
    }
}

/// `WORDS` is a tight `[[word, ipa], ...]` array — top 50k.
fn encode_words(stats: &Stats) -> String {
    let arr: Vec<[&str; 2]> = stats
        .search_words
        .iter()
        .map(|(w, ipa)| [w.as_str(), ipa.as_str()])
        .collect();
    serde_json::to_string(&arr).expect("WORDS json")
}

/// `PHONEME_INFO[ch] = { name, desc, example, category, color, wordCount }`.
fn encode_phoneme_info(stats: &Stats) -> String {
    let mut map = serde_json::Map::new();
    for p in &stats.phonemes {
        let meta = phoneme_meta::phoneme(p.ch);
        let entry = json!({
            "name":      meta.name,
            "desc":      meta.desc,
            "example":   meta.example,
            "category":  p.category.slug,
            "color":     p.category.color,
            "wordCount": p.count,
        });
        map.insert(p.ch.to_string(), entry);
    }
    serde_json::to_string(&map).expect("PHONEME_INFO json")
}

/// `PHONEME_WORDS[ch] = [[word, ipa], ...]` — up to 30 examples per phoneme.
fn encode_phoneme_words(stats: &Stats) -> String {
    let mut map = serde_json::Map::new();
    for p in &stats.phonemes {
        let arr: Vec<[&str; 2]> = p
            .examples
            .iter()
            .map(|(w, ipa)| [w.as_str(), ipa.as_str()])
            .collect();
        map.insert(p.ch.to_string(), json!(arr));
    }
    serde_json::to_string(&map).expect("PHONEME_WORDS json")
}

/// `PHONEME_AXIS = [ch, ch, ch, ...]` — the heatmap axis order.
fn encode_axis(matrix: &Matrix) -> String {
    let axis: Vec<String> = matrix.phonemes.iter().map(|c| c.to_string()).collect();
    serde_json::to_string(&axis).expect("PHONEME_AXIS json")
}

/// `DISTANCE_MATRIX = [{d, c}, ...]` — row-major flat array of `n*n` cells.
fn encode_matrix(matrix: &Matrix) -> String {
    // Pre-flatten to row-major so client-side indexing is `r * n + c`.
    let n = matrix.phonemes.len();
    let mut flat = Vec::with_capacity(n * n);
    for row in &matrix.rows {
        for cell in &row.cells {
            flat.push(json!({
                // Two-letter keys keep the payload smaller than the
                // semantic `distance` / `confusion` names would; the
                // dataset is large (n=77 → 5929 entries).
                "d": round3(cell.distance),
                "c": round3(cell.confusion),
            }));
        }
    }
    serde_json::to_string(&flat).expect("DISTANCE_MATRIX json")
}

fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}
