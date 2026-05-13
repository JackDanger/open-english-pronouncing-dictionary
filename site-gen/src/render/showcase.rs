//! Top-N most-frequent words, click-to-analyse. Drives word lookup
//! via `data-word` + the document-level delegated listener.

use maud::{html, Markup};

use crate::analysis::Stats;

pub fn render(stats: &Stats) -> Markup {
    html! {
        section id="top-words" class="showcase-section" {
            div class="showcase-section-inner" {
                h2 class="section-title" { "100 Most Common Words" }
                p class="section-sub" { "Click any row to analyse that word." }
                div class="word-grid" {
                    @for (word, ipa, rank) in &stats.showcase {
                        button class="wrow"
                               data-word=(word)
                               type="button"
                               aria-label=(format!("Analyse {}", word)) {
                            span class="wrow-rank" { "#" (rank) }
                            span class="wrow-word" { (word) }
                            span class="wrow-ipa ipa-font" { (ipa) }
                        }
                    }
                }
            }
        }
    }
}
