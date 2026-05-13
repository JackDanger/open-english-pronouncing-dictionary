//! Phoneme distance visualizer.
//!
//! Renders a square heatmap of every phoneme pair, coloured by
//! `phonetics::distance`. Clicking a cell pops a side panel showing
//! both the strict per-phoneme distance and the listener-confusion
//! distance, together with a short interpretation.
//!
//! All heatmap data is precomputed at build time and dropped into a
//! JS literal by `scripts.rs`. This module just builds the empty
//! shell; the client wires it up.

use maud::{html, Markup};

use crate::distance_matrix::Matrix;

pub fn render(matrix: &Matrix) -> Markup {
    html! {
        section id="distance-viz" class="distance-section" {
            div class="distance-inner" {
                h2 class="section-title" { "How close do these sounds sound?" }
                p class="distance-intro" {
                    "Every cell is a phoneme pair, coloured by their "
                    em { "acoustic" }
                    " distance — short distances mean a listener is more likely to confuse them. "
                    "Click any cell to see both the strict per-phoneme distance (Bark-space "
                    "vowel + 2D consonant place embedding) and the listener-confusion distance "
                    "calibrated against Mad Gab puzzle data."
                }

                div class="distance-layout" {
                    div class="heatmap-wrap" {
                        // The grid columns + cell DOM are emitted at
                        // render time so we get static HTML the browser
                        // doesn't need to build; the colour fills come
                        // from a JS pass on first paint (the cell
                        // index → distance map is embedded in scripts.rs).
                        div class="heatmap"
                            id="heatmap"
                            style=(format!(
                                "grid-template-columns: repeat({}, 18px);",
                                matrix.phonemes.len() + 1
                            )) {
                            // Top-left corner spacer.
                            div class="heatmap-cell axis" {}
                            // Column header row.
                            @for ch in &matrix.phonemes {
                                div class="heatmap-cell axis axis-col" { (ch) }
                            }
                            // Body: one row per `a`, prefixed by row label.
                            @for (row_idx, row) in matrix.rows.iter().enumerate() {
                                div class="heatmap-cell axis axis-row" { (row.a) }
                                @for (col_idx, _cell) in row.cells.iter().enumerate() {
                                    div class="heatmap-cell"
                                        data-row=(row_idx.to_string())
                                        data-col=(col_idx.to_string())
                                        data-pair=(format!("{}{}", matrix.phonemes[row_idx], matrix.phonemes[col_idx])) {}
                                }
                            }
                        }
                        div class="dist-legend" {
                            span { "close" }
                            div class="dist-legend-grad" {}
                            span { "far" }
                        }
                    }

                    aside class="distance-panel" id="distance-panel" {
                        div class="dp-title" { "Pick a pair" }
                        p class="dp-empty" {
                            "Click any cell in the heatmap to compare two phonemes. "
                            "The diagonal is identity (distance 0); cells near the diagonal "
                            "are typically near-mergers — pairs that learners and listeners "
                            "routinely confuse."
                        }
                    }
                }
            }
        }
    }
}
