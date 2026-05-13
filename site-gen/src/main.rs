//! site-gen — produces the OpenEPD GitHub Pages site from
//! `data/openepd.json`.
//!
//! Architecture:
//!
//! ```text
//!   corpus JSON
//!       │
//!       ▼
//!   analysis::analyze        → Stats (per-phoneme + per-source aggregates,
//!                                     50k-word search list, 120-word showcase)
//!       │
//!       ├──▶ distance_matrix::compute  → Matrix (77×77 phonetic distances)
//!       └──▶ minimal_pairs::extract    → Vec<MinimalPair>
//!                          │
//!                          ▼
//!                      render::html_doc → Markup
//!                          │
//!                          ▼
//!                      _site/index.html
//! ```
//!
//! All rendering goes through maud, which means every interpolated
//! value is context-escaped automatically (attribute vs element body
//! vs URL). All interactive elements use `data-*` attributes
//! consumed by a single delegated click listener in `render/behavior.js`
//! — there is no inline JavaScript anywhere in the rendered HTML.

mod analysis;
mod distance_matrix;
mod minimal_pairs;
mod phoneme_meta;
mod render;
mod util;

use std::{env, fs, process};

use crate::util::fmt_num;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: site-gen <corpus.json> <output-dir>");
        eprintln!();
        eprintln!("  site-gen ./data/openepd.json ./_site");
        process::exit(1);
    }
    let corpus_path = &args[1];
    let output_dir = &args[2];

    eprint!("Reading {corpus_path} … ");
    let json = fs::read_to_string(corpus_path)
        .unwrap_or_else(|e| panic!("Cannot read {corpus_path}: {e}"));
    eprintln!("{:.1} MB", json.len() as f64 / 1_048_576.0);

    eprint!("Parsing JSON … ");
    let root: serde_json::Value =
        serde_json::from_str(&json).expect("corpus is not valid JSON");
    let raw = root.as_object().expect("corpus root must be a JSON object");
    eprintln!("{} entries", fmt_num(raw.len()));

    eprint!("Analysing corpus … ");
    let stats = analysis::analyze(raw);
    eprintln!(
        "{} words · {} phonemes · {:.1} avg IPA len",
        fmt_num(stats.total_words),
        stats.phonemes.len(),
        stats.avg_ipa_len
    );

    eprint!("Computing phoneme distance matrix … ");
    let matrix = distance_matrix::compute(&stats);
    eprintln!("{}×{} = {} cells", matrix.phonemes.len(), matrix.phonemes.len(),
              matrix.phonemes.len() * matrix.phonemes.len());

    eprint!("Extracting minimal pairs … ");
    let pairs = minimal_pairs::extract(&stats, 60);
    eprintln!("{} pairs", pairs.len());

    eprint!("Rendering HTML … ");
    let html = render::html_doc(&stats, &matrix, &pairs).into_string();
    eprintln!("{:.0} KB", html.len() as f64 / 1_024.0);

    fs::create_dir_all(output_dir).unwrap_or_else(|e| panic!("Cannot create {output_dir}: {e}"));
    let index_path = format!("{output_dir}/index.html");
    fs::write(&index_path, &html).unwrap_or_else(|e| panic!("Cannot write {index_path}: {e}"));
    // GitHub Pages otherwise tries to run Jekyll on the upload.
    fs::write(format!("{output_dir}/.nojekyll"), b"" as &[u8])
        .unwrap_or_else(|e| panic!("Cannot write .nojekyll: {e}"));
    eprintln!("Done → {index_path}");
}
