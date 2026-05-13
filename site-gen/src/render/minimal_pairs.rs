//! Minimal-pair examples: pairs of common words that differ in
//! exactly one phoneme. Curated at build time for maximum
//! pedagogical value (one example per unique contrast).

use maud::{html, Markup};

use crate::minimal_pairs::MinimalPair;

pub fn render(pairs: &[MinimalPair]) -> Markup {
    html! {
        section id="minimal-pairs" class="pairs-section" {
            div class="pairs-inner" {
                h2 class="section-title" { "Minimal Pairs" }
                p class="section-sub" {
                    "Word pairs that differ by exactly one sound. "
                    "Each pair shows you that the contrasting phonemes really are "
                    "separate sounds in English — change one, change the meaning. "
                    "Curated automatically from the corpus."
                }
                div class="pairs-grid" {
                    @for p in pairs {
                        (card(p))
                    }
                }
            }
        }
    }
}

fn card(p: &MinimalPair) -> Markup {
    let (a_word, a_ipa) = (&p.a.0, &p.a.1);
    let (b_word, b_ipa) = (&p.b.0, &p.b.1);
    html! {
        div class="pair-card" {
            div class="pair-contrast ipa-font" {
                span class="contrast-glyph" { (p.contrast.0) }
                span { "↔" }
                span class="contrast-glyph" { (p.contrast.1) }
            }
            div class="pair-row" data-word=(a_word) {
                span class="pair-word" { (a_word) }
                span class="pair-ipa" { "/" (a_ipa) "/" }
            }
            div class="pair-row" data-word=(b_word) {
                span class="pair-word" { (b_word) }
                span class="pair-ipa" { "/" (b_ipa) "/" }
            }
            div class="pair-distance" {
                "phonetic distance "
                (format!("{:.2}", p.distance))
            }
        }
    }
}
