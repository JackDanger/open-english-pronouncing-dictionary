//! Top-of-page hero with floating IPA characters and CTAs.

use maud::{html, Markup};

use crate::analysis::Stats;
use crate::util::fmt_num;

pub fn render(stats: &Stats) -> Markup {
    // Pick up to 28 of the most common phonemic characters for the
    // animated background. Exclude stress/length marks and any
    // "other" residue.
    let bg_chars: Vec<char> = stats
        .phonemes
        .iter()
        .filter(|p| p.category.slug != "supra" && p.category.slug != "other")
        .take(28)
        .map(|p| p.ch)
        .collect();

    html! {
        header id="hero" {
            div class="hero-bg" aria-hidden="true" {
                @for (i, ch) in bg_chars.iter().enumerate() {
                    @let x = (i * 13 + 7) % 92;
                    @let y = (i * 17 + 5) % 82;
                    @let delay = (i * 3) % 22;
                    @let dur = 16 + (i % 7) * 4;
                    @let size = 7 + (i % 5) * 3;
                    span style=(format!(
                        "left:{x}%;top:{y}%;font-size:{size}rem;animation-duration:{dur}s;animation-delay:-{delay}s"
                    )) { (ch) }
                }
            }
            div class="hero-content" {
                p class="hero-eyebrow" { "openepd · open english pronouncing dictionary" }
                h1 class="hero-title serif" {
                    "The English IPA Corpus"
                }
                p class="hero-sub" {
                    (fmt_num(stats.total_words))
                    " words fused from four open lexicons — now an interactive phonetics lesson. "
                    "Type any word below to hear its phonemes."
                }
                div class="hero-cta" {
                    a href="#search" class="btn btn-primary" { "Explore a word" }
                    a href="#phonemes" class="btn btn-ghost" { "Browse phonemes" }
                    a href="https://github.com/JackDanger/open-english-pronouncing-dictionary"
                      class="btn btn-ghost" target="_blank" rel="noopener" { "GitHub ↗" }
                }
            }
        }
    }
}
