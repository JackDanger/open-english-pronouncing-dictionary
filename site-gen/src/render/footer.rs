//! Page footer with project links and license badge.

use maud::{html, Markup};

pub fn render() -> Markup {
    // One-line footer: license + source attribution + repo. The
    // CC-BY-SA license obligates source acknowledgement for the data,
    // but it doesn't need its own card. Folded inline.
    html! {
        footer {
            span class="license-badge" { "CC BY-SA 4.0" }
            span { "Sources: Misaki · CMU · WikiPron" }
            a href="https://github.com/JackDanger/open-english-pronouncing-dictionary"
              target="_blank" rel="noopener" { "GitHub ↗" }
        }
    }
}
