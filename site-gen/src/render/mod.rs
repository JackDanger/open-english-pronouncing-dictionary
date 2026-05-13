//! Compose the full page from per-section render functions.
//!
//! `html_doc` is the only public entry point. Each section lives in
//! its own module so the rendering surface area can be reasoned
//! about piece by piece — and so the structural HTML tests can
//! target one section at a time.

use maud::{html, Markup, DOCTYPE};

use crate::analysis::Stats;
use crate::distance_matrix::Matrix;
use crate::minimal_pairs::MinimalPair;
use crate::util::fmt_num;

mod chart;
mod css;
mod footer;
mod hero;
mod scripts;
mod showcase;
mod sources;
mod stats_band;
mod workspace;

// Retired but kept around to ease bisecting; they're no longer
// composed into the page. Future cleanup commit will delete them.
#[allow(dead_code)] mod distance_viz;
#[allow(dead_code)] mod minimal_pairs;
#[allow(dead_code)] mod phonemes;
#[allow(dead_code)] mod reverse_search;
#[allow(dead_code)] mod search;

/// Build the entire static page as one HTML document.
pub fn html_doc(
    stats: &Stats,
    matrix: &Matrix,
    pairs: &[MinimalPair],
) -> Markup {
    let total = fmt_num(stats.total_words);
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "The English IPA Corpus — OpenEPD" }
                meta name="description"
                     content="An interactive phonetics workspace: trace any English word's path through the IPA chart and watch a schematic vocal tract move with it.";
                meta property="og:title" content="The English IPA Corpus — OpenEPD";
                meta property="og:description"
                     content=(format!("{} words · trace any word through the anatomical IPA chart", total));
                link rel="preconnect" href="https://fonts.googleapis.com";
                link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
                link rel="stylesheet"
                     href="https://fonts.googleapis.com/css2?family=Playfair+Display:ital,wght@0,400;0,700;0,900;1,400&family=Inter:wght@400;500;600&family=JetBrains+Mono:wght@400;500&family=Noto+Serif:wght@400;600&display=swap";
                style { (css::CSS) }
            }
            body {
                (hero::render(stats))
                (stats_band::render(stats))
                (workspace::render(stats))
                (showcase::render(stats))
                (sources::render(stats))
                (footer::render())
                (scripts::render(stats, matrix, pairs))
            }
        }
    }
}
