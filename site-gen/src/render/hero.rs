//! Top-of-page hero with floating IPA characters and CTAs.

use maud::{html, Markup};

use crate::analysis::Stats;

pub fn render(_stats: &Stats) -> Markup {
    // Slim header strip — title + GitHub link. The workspace's word
    // pane carries the actual orientation ("type any word"). No
    // stats band; no eyebrow; no CTAs that duplicate the search.
    html! {
        header id="hero" {
            div class="hero-content" {
                span class="hero-title serif" { "The English IPA Corpus" }
                a href="https://github.com/JackDanger/open-english-pronouncing-dictionary"
                  class="hero-link" target="_blank" rel="noopener" { "github ↗" }
            }
        }
    }
}
