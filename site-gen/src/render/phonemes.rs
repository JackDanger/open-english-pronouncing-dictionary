//! Phoneme Universe — every IPA character in the corpus, grouped by
//! manner of articulation, sized by frequency. Hover for example
//! words; click to drive the phoneme detail panel in the search
//! section (the client JS does the scroll + dispatch).

use std::collections::BTreeMap;

use maud::{html, Markup};

use crate::analysis::{PhonemeInfo, Stats};
use crate::phoneme_meta::CATEGORY_ORDER;
use crate::util::fmt_k;

pub fn render(stats: &Stats) -> Markup {
    // Re-bucket phonemes by category slug.
    let mut groups: BTreeMap<&str, Vec<&PhonemeInfo>> = BTreeMap::new();
    for p in &stats.phonemes {
        groups.entry(p.category.slug).or_default().push(p);
    }

    html! {
        section id="phonemes" class="phoneme-section" {
            div class="phoneme-section-inner" {
                h2 class="section-title" { "Phoneme Universe" }
                p class="section-sub" {
                    "Every IPA sound in the corpus, sized by word-coverage. \
                     Hover for examples, click to explore in the search above."
                }
                @for slug in CATEGORY_ORDER {
                    @if let Some(items) = groups.get(slug) {
                        @if !items.is_empty() {
                            (group(slug, items))
                        }
                    }
                }
            }
        }
    }
}

fn group(slug: &str, items: &[&PhonemeInfo]) -> Markup {
    let color = items[0].category.color;
    let label = items[0].category.label;
    html! {
        div class="phoneme-group" {
            div class="phoneme-group-title" style=(format!("color:{color}")) { (label) }
            div class="phoneme-row" {
                @for p in items {
                    (tile(p))
                }
            }
        }
        // slug pulled in for testability (assert each emitted group
        // carries the right marker)
        template data-phoneme-group=(slug) {}
    }
}

fn tile(p: &PhonemeInfo) -> Markup {
    let ch_str = p.ch.to_string();
    let count_fmt = fmt_k(p.count);
    let style = format!("--tile-color:{};--sz:{:.3};", p.category.color, p.size_ratio);
    html! {
        div class="ptile"
            data-ch=(ch_str)
            data-action="select-phoneme-from-universe"
            style=(style)
            aria-label=(format!("{}: {}", ch_str, count_fmt)) {
            span class="ptile-ch ipa-font" { (p.ch) }
            span class="ptile-count" { (count_fmt) }
            div class="ptile-popup" {
                div class="popup-title" { "/" (p.ch) "/" }
                @for (word, ipa) in &p.examples {
                    div class="popup-ex" {
                        span class="popup-word" { (word) }
                        span class="popup-ipa" { (ipa) }
                    }
                }
                div class="popup-tap" { "Click to explore" }
            }
        }
    }
}
