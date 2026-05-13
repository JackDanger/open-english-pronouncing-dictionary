//! Four-stat band: words, phonemes, sources, avg IPA length.

use maud::{html, Markup};

use crate::analysis::Stats;
use crate::util::fmt_num;

pub fn render(stats: &Stats) -> Markup {
    html! {
        section id="stats" class="stats-band" {
            div class="stats-inner" {
                (stat(&fmt_num(stats.total_words), "Words indexed"))
                (stat(&stats.phonemes.len().to_string(), "IPA characters"))
                (stat(&stats.sources.len().to_string(), "Source lexicons"))
                (stat(&format!("{:.1}", stats.avg_ipa_len), "Avg phonemes/word"))
            }
        }
    }
}

fn stat(num: &str, label: &str) -> Markup {
    html! {
        div class="stat-card" {
            span class="stat-num serif" { (num) }
            span class="stat-label" { (label) }
        }
    }
}
