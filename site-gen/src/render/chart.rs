//! The anatomical IPA chart — a single SVG that places every English
//! phoneme at its articulatory position.
//!
//! Two sub-grids share one 100×100 viewBox:
//!
//! * The **vowel quadrilateral** sits in the top half (y ∈ 0..45).
//!   x maps front↔back. y maps close↔open. The classic IPA shape is
//!   trapezoidal (back of the mouth is more constrained than the
//!   front); we hint at it with a subtle background trapezoid but
//!   coordinates are plain rectangular.
//!
//! * The **consonant grid** sits in the bottom half (y ∈ 55..100).
//!   x maps place of articulation: bilabial → glottal, left to right.
//!   y maps manner: stop on top, then nasal, fricative, affricate,
//!   approximant, trill/flap.
//!
//! Each phoneme is rendered as an SVG `<g class="ph" data-ch=…>`.
//! All interaction is wired by the delegated click listener in
//! `behavior.js`; the chart carries no inline event handlers.

use std::collections::BTreeSet;

use maud::{html, Markup, PreEscaped};

use crate::analysis::Stats;
use crate::chart_layout::{position, voicing_offset, Plane};

/// The vowel sub-chart spans this y range of the 100-unit viewBox.
const VOWEL_TOP: f32 = 4.0;
const VOWEL_BOTTOM: f32 = 42.0;
/// The consonant grid spans this y range.
const CONS_TOP: f32 = 56.0;
const CONS_BOTTOM: f32 = 96.0;

pub fn render(stats: &Stats) -> Markup {
    // Only emit phonemes that actually appear in our corpus, so we
    // don't crowd the chart with symbols nobody on this page will
    // see. Plus the few extras that are always useful to learners
    // (h, ʔ) even if they're rare.
    let mut chars: BTreeSet<char> = stats
        .phonemes
        .iter()
        .map(|p| p.ch)
        .filter(|c| !matches!(c, 'ˈ' | 'ˌ' | 'ː'))
        .collect();
    for c in ['ʔ', 'h'] {
        chars.insert(c);
    }

    html! {
        section id="chart" class="chart-section" {
            div class="chart-inner" {
                h2 class="chart-title serif" { "Where each sound lives in your mouth" }
                p class="chart-sub" {
                    "Vowels in the top quadrilateral — front-to-back is your tongue's "
                    "position in your mouth, top-to-bottom is how open your jaw is. "
                    "Consonants below — left-to-right marches from your lips backwards "
                    "to your throat; top-to-bottom is how you make the sound."
                }
                div class="chart-viewport" {
                    (chart_svg(&chars))
                }
            }
        }
    }
}

fn chart_svg(chars: &BTreeSet<char>) -> Markup {
    // Two SVG layers: background labels + axis decoration first,
    // then the phoneme glyphs on top. We emit both inside one
    // <svg> so they share the same coordinate space.
    let vowels: Vec<char> = chars
        .iter()
        .copied()
        .filter(|c| matches!(position(*c).plane, Plane::Vowel))
        .collect();
    let consonants: Vec<char> = chars
        .iter()
        .copied()
        .filter(|c| matches!(position(*c).plane, Plane::Consonant))
        .collect();

    html! {
        svg id="ipa-chart"
            xmlns="http://www.w3.org/2000/svg"
            viewBox="-12 -2 120 104"
            preserveAspectRatio="xMidYMid meet"
            role="img"
            aria-label="IPA chart with phonemes positioned by articulation" {

            (background_decoration())

            // Word-path overlay layer. Populated by behavior.js when
            // a word is selected; we just emit the empty placeholder
            // with stable id so the JS doesn't have to mint nodes.
            g id="path-layer" class="path-layer" {}

            // Phoneme glyphs.
            g class="phonemes" {
                @for ch in &vowels {
                    (phoneme_glyph(*ch))
                }
                @for ch in &consonants {
                    (phoneme_glyph(*ch))
                }
            }
        }
    }
}

fn background_decoration() -> Markup {
    // The vowel quadrilateral as a faint trapezoid + axis labels.
    let v_t = VOWEL_TOP;
    let v_b = VOWEL_BOTTOM;
    let c_t = CONS_TOP;
    let _c_b = CONS_BOTTOM;
    let trap = format!(
        "M 4 {v_t} L 96 {v_t} L 84 {v_b} L 16 {v_b} Z"
    );
    html! {
        g class="chart-bg" {
            // Vowel quadrilateral outline.
            path class="vowel-trap" d=(trap) {}
            text class="axis-label" x="50" y=(v_t - 1.0) text-anchor="middle" { "VOWELS  ←front · back→" }
            text class="axis-label rot" x="-3" y=(((v_t + v_b) / 2.0)) text-anchor="middle"
                 transform=(format!("rotate(-90 -3 {})", (v_t + v_b) / 2.0))
                 { "close · open" }

            // Consonant grid background — light row stripes.
            @for (i, (label, y)) in [
                ("stop",        c_t + 4.0),
                ("nasal",       c_t + 12.0),
                ("fricative",   c_t + 22.0),
                ("affricate",   c_t + 30.0),
                ("approximant", c_t + 36.0),
            ].iter().enumerate() {
                rect class=(format!("cons-row r{}", i % 2))
                     x="0" y=(y - 4.0) width="100" height="8" {}
                text class="row-label" x="-2" y=(y) text-anchor="end" { (label) }
            }
            text class="axis-label" x="50" y=(c_t - 1.0) text-anchor="middle"
                { "CONSONANTS  ←lips · throat→" }
            // Place markers across the top of the consonant grid.
            @for (label, x) in [
                ("bilab", 5.0_f32),
                ("lab/dent", 15.0),
                ("dental", 28.0),
                ("alveolar", 42.0),
                ("post-alv", 55.0),
                ("palatal", 68.0),
                ("velar", 82.0),
                ("glot", 95.0),
            ].iter() {
                text class="col-label" x=(x) y=(c_t + 0.6) text-anchor="middle" { (label) }
            }
        }
    }
}

fn phoneme_glyph(ch: char) -> Markup {
    let p = position(ch);
    let (x, y) = match p.plane {
        Plane::Vowel => {
            // Map 0..100 input coords into the vowel band.
            let yy = VOWEL_TOP + (p.y / 100.0) * (VOWEL_BOTTOM - VOWEL_TOP);
            (p.x, yy)
        }
        Plane::Consonant => {
            let xx = p.x + voicing_offset(ch);
            // Map manner y from 0..100 into the consonant band.
            let yy = CONS_TOP + (p.y / 100.0) * (CONS_BOTTOM - CONS_TOP);
            (xx, yy)
        }
        _ => return html! {},
    };

    let ch_str = ch.to_string();
    html! {
        g class="ph"
          data-ch=(ch_str)
          transform=(format!("translate({:.2} {:.2})", x, y)) {
            circle class="ph-bg" r="3.4" {}
            text class="ph-glyph" text-anchor="middle" dominant-baseline="central" y="0.4" { (ch) }
            // Path-overlay dot, hidden until a word path makes it visible.
            circle class="ph-dot" r="0" {}
        }
    }
}

#[allow(dead_code)]
pub fn _silence_warnings() -> PreEscaped<String> {
    PreEscaped(String::new())
}
