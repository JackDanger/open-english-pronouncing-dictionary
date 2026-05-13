//! Source-provenance cards. Each preferred-source bucket gets a card
//! showing its label, entry count, and a plain-English description.

use maud::{html, Markup};

use crate::analysis::Stats;
use crate::util::fmt_num;

pub fn render(stats: &Stats) -> Markup {
    html! {
        section id="sources" class="sources-section" {
            div style="max-width:1300px;margin:0 auto;padding:0" {
                h2 class="section-title" { "Source Lexicons" }
                p class="section-sub" {
                    "Each word carries provenance — which source contributed its transcription(s)."
                }
                div class="sources-grid" {
                    @for (tag, count) in &stats.sources {
                        div class="src-card" {
                            div class="src-tag mono" { (tag) }
                            div class="src-num serif" { (fmt_num(*count)) }
                            p class="src-desc" { (description(tag)) }
                        }
                    }
                }
            }
        }
    }
}

fn description(tag: &str) -> &'static str {
    match tag {
        "cmu" => "Carnegie Mellon Pronouncing Dictionary — the gold standard for American \
                  English broad transcription. ARPABET-derived IPA, preferred for function \
                  words in their weak contextual forms.",
        "misaki_gold" => "Vetted near-IPA from the Kokoro TTS project (gold tier). Narrow \
                          vowel distinctions; tends to use citation/strong forms for function words.",
        "misaki_silver" => "Misaki silver tier — broader coverage than gold, slightly noisier. \
                            Excellent gap-fill for words absent from CMU.",
        "wikipron" => "WikiPron broad scrape from Wiktionary. Widest open coverage; \
                       captures regional variants.",
        "phonemicchart" => "Phonemic Chart corpus — legacy source from the pre-OpenEPD lexicon.",
        "wiktionary" => "Wiktionary IPA annotations — legacy source superseded by the \
                         WikiPron scrape.",
        _ => "Open-source IPA transcription source.",
    }
}
