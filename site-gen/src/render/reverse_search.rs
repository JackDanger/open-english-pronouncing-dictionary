//! Reverse phoneme search: type a phoneme pattern (with `_` wildcards),
//! get every word matching it. Pure-client JS scans the embedded
//! WORDS array.

use maud::{html, Markup};

pub fn render() -> Markup {
    html! {
        section id="reverse-search" class="reverse-section" {
            div class="reverse-inner" {
                h2 class="section-title" { "Reverse phoneme search" }
                p class="section-sub" {
                    "Have a sound or sound-sequence in mind? Type IPA below and "
                    "find every common English word containing it."
                }
                div class="reverse-box" {
                    span aria-hidden="true" { "/" }
                    input
                        id="reverse-q"
                        type="search"
                        autocomplete="off"
                        spellcheck="false"
                        aria-label="Phoneme pattern"
                        placeholder="e.g. θ, eɪ, _ɪŋ";
                    span aria-hidden="true" { "/" }
                }
                p class="reverse-hint" {
                    "Use plain IPA (paste from the Phoneme Universe). "
                    code { "_" }
                    " matches any one phoneme. So "
                    code { "θ_n" }
                    " finds "
                    em { "thin" }
                    ", "
                    em { "then" }
                    ", "
                    em { "than" }
                    "."
                }
                div id="reverse-count" class="reverse-count" {}
                div id="reverse-results" class="reverse-results" {}
            }
        }
    }
}
