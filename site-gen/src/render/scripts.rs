//! Page script: data declarations followed by behaviour.
//!
//! Data is `serde_json`-serialised so the JSON is syntactically
//! valid JS by construction — no hand-escaping. Behaviour lives in
//! `behavior.js` next to this file and is `include_str!`'d so it
//! benefits from real JS tooling when edited.

use maud::{html, Markup, PreEscaped};
use serde_json::json;

use crate::analysis::Stats;
use crate::chart_layout::{position, Plane};
use crate::distance_matrix::Matrix;
use crate::minimal_pairs::MinimalPair;
use crate::phoneme_meta;
use crate::sagittal;

const BEHAVIOR_JS: &str = include_str!("behavior.js");

pub fn render(stats: &Stats, matrix: &Matrix, _pairs: &[MinimalPair]) -> Markup {
    let words = encode_words(stats);
    let info = encode_phoneme_info(stats);
    let words_map = encode_phoneme_words(stats);
    let axis = encode_axis(matrix);
    let mtx = encode_matrix(matrix);
    let chart_pos = encode_chart_positions(stats);
    let sagittal = encode_sagittal_specs(stats);

    let script_body = format!(
        "const WORDS={words};\n\
         const PHONEME_INFO={info};\n\
         const PHONEME_WORDS={words_map};\n\
         const PHONEME_AXIS={axis};\n\
         const DISTANCE_MATRIX={mtx};\n\
         const CHART_POS={chart_pos};\n\
         const SAGITTAL_SPECS={sagittal};\n\
         {BEHAVIOR_JS}"
    );
    html! {
        script { (PreEscaped(script_body)) }
    }
}

fn encode_words(stats: &Stats) -> String {
    let arr: Vec<[&str; 2]> = stats
        .search_words
        .iter()
        .map(|(w, ipa)| [w.as_str(), ipa.as_str()])
        .collect();
    serde_json::to_string(&arr).expect("WORDS json")
}

fn encode_phoneme_info(stats: &Stats) -> String {
    let mut map = serde_json::Map::new();
    for p in &stats.phonemes {
        let meta = phoneme_meta::phoneme(p.ch);
        map.insert(
            p.ch.to_string(),
            json!({
                "name":      meta.name,
                "desc":      meta.desc,
                "example":   meta.example,
                "category":  p.category.slug,
                "color":     p.category.color,
                "wordCount": p.count,
            }),
        );
    }
    serde_json::to_string(&map).expect("PHONEME_INFO json")
}

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

fn encode_axis(matrix: &Matrix) -> String {
    let axis: Vec<String> = matrix.phonemes.iter().map(|c| c.to_string()).collect();
    serde_json::to_string(&axis).expect("PHONEME_AXIS json")
}

fn encode_matrix(matrix: &Matrix) -> String {
    let n = matrix.phonemes.len();
    let mut flat = Vec::with_capacity(n * n);
    for row in &matrix.rows {
        for cell in &row.cells {
            flat.push(json!({
                "d": round3(cell.distance),
                "c": round3(cell.confusion),
            }));
        }
    }
    serde_json::to_string(&flat).expect("DISTANCE_MATRIX json")
}

/// Chart positions per phoneme: { ch: [x, y, plane] }. Plane is 0
/// for vowel, 1 for consonant. The path-tracer reads x and y; the
/// sagittal driver reads tongue position from the same numbers.
fn encode_chart_positions(stats: &Stats) -> String {
    let mut map = serde_json::Map::new();
    for p in &stats.phonemes {
        let pos = position(p.ch);
        let plane_n = match pos.plane {
            Plane::Vowel => 0,
            Plane::Consonant => 1,
            _ => continue,
        };
        map.insert(
            p.ch.to_string(),
            json!([round1(pos.x), round1(pos.y), plane_n]),
        );
    }
    serde_json::to_string(&map).expect("CHART_POS json")
}

/// Per-phoneme sagittal spec: tongue position, lip shape, airflow,
/// voicing. Drives the schematic vocal-tract inset.
fn encode_sagittal_specs(stats: &Stats) -> String {
    let mut map = serde_json::Map::new();
    for p in &stats.phonemes {
        if let Some(spec) = sagittal::spec_json(p.ch) {
            map.insert(p.ch.to_string(), spec);
        }
    }
    serde_json::to_string(&map).expect("SAGITTAL_SPECS json")
}

fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

fn round1(x: f32) -> f32 {
    (x * 10.0).round() / 10.0
}
