//! The chart-centric workspace — the single interactive panel that
//! replaces the previous click-card / panel model. Three coordinated
//! views share one piece of state ("which word is being explored,
//! which phoneme is focused"):
//!
//!   ┌────────────────────────────────────────────────────────────┐
//!   │  word + IPA + frequency (big serif)                        │
//!   │  spelling band — letters above, phonemes below, links      │
//!   ├──────────────────────────────────────────────┬─────────────┤
//!   │                                              │ schematic   │
//!   │            IPA chart                         │ vocal tract │
//!   │    (phonemes positioned anatomically,        │ (tongue,    │
//!   │     SVG path overlay for selected word)      │  lips,      │
//!   │                                              │  cords,     │
//!   │                                              │  airflow)   │
//!   └──────────────────────────────────────────────┴─────────────┘
//!   │ word picker (search + sample words)                        │
//!
//! All three sub-views are driven from the same JS state. Picking a
//! word triggers the path-trace animation; hovering a chart phoneme
//! updates the sagittal inset and the heat overlay.

use maud::{html, Markup};

use crate::analysis::Stats;
use crate::render::chart;

pub fn render(stats: &Stats) -> Markup {
    html! {
        section id="workspace" class="workspace-section" {
            div class="workspace-inner" {
                // Three-stage flow, top to bottom, no card chrome:
                //   1. picker       (what word)
                //   2. spelling band (letters → sounds)
                //   3. chart + tract (where those sounds live + how to make them)
                //
                // A single workspace, not a stack of separate panels.
                (word_pane())
                (spelling_band_shell())
                div class="chart-row" {
                    (chart::render(stats))
                    (sagittal_shell())
                }
                (footer_strip())
            }
        }
    }
}

/// Search input on top, current word + IPA + hint underneath. One
/// block, so the eye sees the input feeding into the result rather
/// than two unrelated panels.
fn word_pane() -> Markup {
    html! {
        header class="word-pane" {
            div class="word-picker" {
                div class="search-box" {
                    span class="search-icon" aria-hidden="true" { "▶" }
                    input id="q"
                          type="search"
                          autocomplete="off"
                          spellcheck="false"
                          placeholder="type any English word"
                          aria-label="Word to explore";
                    button id="search-clear"
                           class="search-clear"
                           aria-label="Clear" { "✕" }
                }
                div id="suggestions" class="suggestions" role="listbox" {}
            }
            div class="word-display" {
                span id="word-glyph" class="word-glyph serif" {}
                span id="word-ipa" class="word-ipa ipa-font" {}
                span id="word-rank" class="word-rank" {}
                // Morph breadcrumb: when the user has dragged or
                // tapped a path stop to morph the word, "↺ from X"
                // shows the original (clickable to return).
                span id="morph-breadcrumb" class="morph-breadcrumb" {
                    span class="mb-arrow" { "↺" }
                    " from "
                    button id="morph-reset" class="morph-reset" type="button" {}
                }
                // Defensive shell — the lit-moves interaction can no
                // longer produce a non-word, but the indicator stays
                // in the DOM in case some future path produces one.
                span id="morph-noword" class="morph-noword" {
                    "no English word with these sounds"
                }
            }
            p id="word-hint" class="word-hint" {}
        }
    }
}

/// Footer strip: sample chips on top, reverse-search disclosure
/// below. Sits as one row of supporting controls under the chart.
fn footer_strip() -> Markup {
    html! {
        div class="footer-strip" {
            (sample_strip())
            (reverse_strip())
        }
    }
}

/// Empty SVG shell for the spelling-to-sound alignment. JS fills it
/// per word.
fn spelling_band_shell() -> Markup {
    html! {
        div class="spelling-band-wrap" {
            svg id="spelling-band"
                viewBox="0 0 100 28"
                preserveAspectRatio="xMidYMid meet"
                aria-label="Letters of the word linked to the phonemes they spell" {}
            div class="spelling-legend" {
                span class="legend-key letters" { "letters" }
                span class="legend-key arrow" { "→" }
                span class="legend-key phonemes" { "sounds" }
                span class="legend-note" { "silent letters dangle, digraphs converge" }
            }
        }
    }
}

/// Empty schematic vocal-tract SVG. Per-phoneme positions update via
/// CSS custom properties set by JS.
fn sagittal_shell() -> Markup {
    html! {
        aside class="sagittal-wrap" aria-label="Schematic vocal tract" {
            div class="sagittal-header" {
                span id="sagittal-label" { "Tongue, lips, voice" }
                span id="sagittal-glyph" class="ipa-font" {}
            }
            svg id="sagittal"
                viewBox="0 0 120 80"
                preserveAspectRatio="xMidYMid meet" {
                // Tract outline (a rounded rectangle from cords to lips).
                rect class="tract" x="6" y="20" width="98" height="40" rx="8" ry="8" {}
                // Vocal-cord buzz indicator on the left.
                g id="cords" class="cords" transform="translate(10 40)" {
                    line x1="0" y1="-9" x2="0" y2="9" {}
                    line x1="3" y1="-9" x2="3" y2="9" {}
                    line x1="6" y1="-9" x2="6" y2="9" {}
                }
                // Tongue blob — position set via CSS custom props
                // --tx and --ty driven by JS (0..1 each).
                ellipse id="tongue" class="tongue"
                        cx="50" cy="40" rx="14" ry="9" {}
                // Lips on the right — a path whose `d` attribute
                // changes per lip mode (spread / neutral / rounded /
                // closed / lowerlip).
                path id="lips" class="lips" d="M104 24 Q108 40 104 56" {}
                // Airflow arrow over the tongue.
                path id="airflow" class="airflow" d="M16 22 Q60 14 100 22" {}
                // Nasal escape channel — visible only for nasals.
                path id="nasal-path" class="nasal-path"
                     d="M70 20 Q70 8 90 8 Q104 8 104 14" {}
            }
            p id="sagittal-desc" class="sagittal-desc" {}
        }
    }
}

/// Quick-pick chips for sample words. Picking one fires `data-word`
/// → the delegated listener treats it as a search.
fn sample_strip() -> Markup {
    // Every sample is well inside the top-50k payload AND illustrates
    // some particular pedagogical feature: silent letters, digraphs,
    // diphthongs, vowel-pair minimal contrast, rhotic vowel, etc.
    let samples: &[&str] = &[
        "pneumonia",  // silent p (silent prefix)
        "knee",       // silent k (silent prefix)
        "thought",    // gh silent (in standard American), ɔ vowel
        "though",     // gh silent and OW diphthong
        "rhythm",     // no overt vowel letter — schwa hides in 'y'
        "queue",      // most letters silent; just /kju/
        "colonel",    // famous "weirdest English spelling"
        "subtle",     // silent b mid-word
        "ship",       // ɪ vs… (pairs with sheep)
        "sheep",      // …i (minimal-pair partner of ship)
        "bird",       // ɝ rhotic vowel
        "would",      // silent l, ʊ
    ];
    html! {
        div class="sample-strip" {
            span class="sample-label" { "Try:" }
            @for w in samples {
                button class="sample-chip" data-word=(w) { (w) }
            }
        }
    }
}

/// Reverse phoneme search lives inside the workspace as a sibling
/// strip below the chart, not a separate section. Same input model
/// as before — type IPA, find words.
fn reverse_strip() -> Markup {
    html! {
        details class="reverse-details" {
            summary class="reverse-summary" {
                "Find words by sound pattern"
                small { "  (type IPA — use _ as a wildcard, like θ_n)" }
            }
            div class="reverse-body" {
                input id="reverse-q"
                      type="search"
                      placeholder="e.g. eɪʃ, θɪŋk, _ɪŋ"
                      aria-label="IPA pattern";
                div id="reverse-count" class="reverse-count" {}
                div id="reverse-results" class="reverse-results" {}
            }
        }
    }
}
