//! Compose the full page from per-section render functions.
//!
//! `html_doc` is the only public entry point. Each section lives in
//! its own module so the rendering surface area can be reasoned about
//! piece by piece — and so the structural HTML tests can target one
//! section at a time.

use maud::{html, Markup, DOCTYPE};

use crate::analysis::Stats;
use crate::distance_matrix::Matrix;
use crate::minimal_pairs::MinimalPair;
use crate::util::fmt_num;

mod css;
mod hero;
mod stats_band;
mod search;
mod phonemes;
mod showcase;
mod distance_viz;
mod minimal_pairs;
mod reverse_search;
mod sources;
mod footer;
mod scripts;

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
                     content="An interactive phonetics explorer: type any English word to see its IPA breakdown, then click each sound to learn how it is made.";
                meta property="og:title" content="The English IPA Corpus — OpenEPD";
                meta property="og:description"
                     content=(format!("{} words · explore English phonetics interactively", total));
                link rel="preconnect" href="https://fonts.googleapis.com";
                link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
                link rel="stylesheet"
                     href="https://fonts.googleapis.com/css2?family=Playfair+Display:ital,wght@0,400;0,700;0,900;1,400&family=Inter:wght@400;500;600&family=JetBrains+Mono:wght@400;500&family=Noto+Serif:wght@400;600&display=swap";
                style { (css::CSS) }
            }
            body {
                (hero::render(stats))
                (search::render())
                (stats_band::render(stats))
                (phonemes::render(stats))
                (distance_viz::render(matrix))
                (minimal_pairs::render(pairs))
                (reverse_search::render())
                (showcase::render(stats))
                (sources::render(stats))
                (footer::render())
                (scripts::render(stats, matrix, pairs))
            }
        }
    }
}
