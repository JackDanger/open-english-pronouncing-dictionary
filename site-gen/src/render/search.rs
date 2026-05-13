//! The search hero: input box, suggestions container, result card
//! and phoneme detail panel. All interactivity is wired up by the
//! client JS (see `scripts.rs`) via delegated event listeners that
//! read `data-word` / `data-ch` attributes; no inline onclicks.

use maud::{html, Markup};

const TRY_WORDS: &[&str] = &[
    "rhythm", "through", "thought", "colonel", "queue", "pneumonia",
    "bouquet", "receipt",
];

pub fn render() -> Markup {
    html! {
        section id="search" class="search-hero" {
            div class="search-hero-inner" {
                h2 class="search-headline serif" { "Type any English word." }
                p class="search-tagline" {
                    "See exactly what sounds make it up — then click each sound to understand it."
                }

                div class="search-wrap" {
                    div class="search-box" {
                        span class="search-icon" aria-hidden="true" { "▶" }
                        input
                            id="q"
                            type="search"
                            autocomplete="off"
                            spellcheck="false"
                            placeholder="e.g. rhythm, colonel, through…"
                            aria-label="Search for a word";
                        button
                            class="search-clear"
                            id="search-clear"
                            aria-label="Clear search" { "✕" }
                    }
                    div id="suggestions" class="suggestions" role="listbox"
                        aria-label="Word suggestions" {}
                }

                div class="try-words" aria-label="Example words to try" {
                    span class="try-label" { "Try:" }
                    @for w in TRY_WORDS {
                        button class="try-word" data-word=(w) { (w) }
                    }
                }

                div class="result-grid" id="result-grid" {
                    div id="word-result"
                        class="word-result"
                        role="region"
                        aria-label="Word analysis"
                        aria-live="polite" {}
                    div id="phoneme-panel"
                        class="phoneme-panel"
                        role="complementary"
                        aria-label="Phoneme detail"
                        aria-live="polite" {}
                }
            }
        }
    }
}
