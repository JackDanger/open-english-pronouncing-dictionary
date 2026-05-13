//! Per-phoneme articulator spec for the schematic vocal-tract inset.
//!
//! Not a realistic sagittal view of a human head — that's a *huge*
//! drawing job and the lesson doesn't need it. Instead, a schematic
//! horizontal tract:
//!
//!   ┌────────────────────────────────────────┐
//!   │ cords │     tongue body              │ │  ← lips at right
//!   │ ::::  │           ●                   │◯│
//!   │       │                              │ │
//!   └────────────────────────────────────────┘
//!     voicing  vocal tract                  lips
//!
//! Each phoneme is a `Spec` of where the tongue blob sits, what the
//! lips do, whether the cords vibrate, and what the airflow looks
//! like (free, narrowed at the tongue, fully blocked, nasal, lateral).
//! The JS reads the spec and updates the SVG inset whenever the word
//! path moves to a new phoneme.
//!
//! The tongue position uses the **same coordinate system** as
//! `chart_layout::position` so the inset and the IPA chart stay
//! synchronised: when a word path visits a vowel, the tongue blob
//! sits at the same (x, y) the vowel occupies on the chart.

use crate::chart_layout::{position, voiced, Plane};

#[derive(Debug, Clone, Copy)]
pub enum Lips {
    Spread,    // i, ɪ, e
    Neutral,   // most consonants, mid vowels
    Rounded,   // u, ʊ, o, ɔ, w, ʃ, ʒ, ʧ, ʤ
    Closed,    // p, b, m  — full closure at the lips
    LowerLip,  // f, v     — lower lip on upper teeth
}

#[derive(Debug, Clone, Copy)]
pub enum Airflow {
    Free,      // vowels, approximants
    Narrow,    // fricatives — air squeezing through a gap
    Blocked,   // stops — full closure
    Nasal,     // m, n, ŋ — through the nose
    Lateral,   // l — over the sides of the tongue
}

#[derive(Debug, Clone, Copy)]
pub struct Spec {
    /// 0 = front of mouth, 100 = back.
    pub tongue_x: f32,
    /// 0 = high (close to palate), 100 = low.
    pub tongue_y: f32,
    pub lips: Lips,
    pub airflow: Airflow,
    pub voiced: bool,
}

impl Spec {
    fn from_chart_position(ch: char) -> Option<Spec> {
        let pos = position(ch);
        let (tx, ty) = match pos.plane {
            Plane::Vowel => (pos.x, pos.y),
            Plane::Consonant => {
                // Map place (0..100) into the tract; manner doesn't
                // map to tongue *position* directly, so we hold the
                // tongue at a neutral height for consonants.
                (pos.x, 35.0)
            }
            _ => return None,
        };
        let lips = lips_for(ch);
        let airflow = airflow_for(ch);
        Some(Spec {
            tongue_x: tx,
            tongue_y: ty,
            lips,
            airflow,
            voiced: voiced(ch),
        })
    }
}

fn lips_for(ch: char) -> Lips {
    match ch {
        // Closure at the lips.
        'p' | 'b' | 'm' => Lips::Closed,
        // Labiodental contact.
        'f' | 'v' | 'ɱ' | 'ʋ' => Lips::LowerLip,
        // Rounded.
        'u' | 'ʊ' | 'o' | 'ɔ' | 'w' | 'ʍ' | 'ʃ' | 'ʒ' | 'ʧ' | 'ʤ' | 'y' | 'ø' | 'œ' => Lips::Rounded,
        // Spread (front high vowels).
        'i' | 'ɪ' | 'e' | 'ɛ' => Lips::Spread,
        _ => Lips::Neutral,
    }
}

fn airflow_for(ch: char) -> Airflow {
    match ch {
        // Stops — full closure somewhere.
        'p' | 'b' | 't' | 'd' | 'k' | 'ɡ' | 'g' | 'ʔ' => Airflow::Blocked,
        // Nasals — closure but air through the nose.
        'm' | 'ɱ' | 'n' | 'ɲ' | 'ŋ' => Airflow::Nasal,
        // Fricatives — narrow passage.
        'f' | 'v' | 'θ' | 'ð' | 's' | 'z' | 'ʃ' | 'ʒ' | 'x' | 'ɣ' | 'h' | 'ħ' => Airflow::Narrow,
        // Affricates — start blocked, release narrow. Show narrow
        // for the schematic.
        'ʧ' | 'ʤ' => Airflow::Narrow,
        // Lateral — air over the sides of the tongue.
        'l' | 'ɫ' => Airflow::Lateral,
        _ => Airflow::Free,
    }
}

/// JSON-serialisable form for embedding in the page script.
pub fn spec_json(ch: char) -> Option<serde_json::Value> {
    let s = Spec::from_chart_position(ch)?;
    let lips = match s.lips {
        Lips::Spread => "spread",
        Lips::Neutral => "neutral",
        Lips::Rounded => "rounded",
        Lips::Closed => "closed",
        Lips::LowerLip => "lowerlip",
    };
    let airflow = match s.airflow {
        Airflow::Free => "free",
        Airflow::Narrow => "narrow",
        Airflow::Blocked => "blocked",
        Airflow::Nasal => "nasal",
        Airflow::Lateral => "lateral",
    };
    Some(serde_json::json!({
        "tx": (s.tongue_x * 10.0_f32).round() / 10.0,
        "ty": (s.tongue_y * 10.0_f32).round() / 10.0,
        "lips": lips,
        "air": airflow,
        "voiced": s.voiced,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lips_close_for_bilabials() {
        for ch in ['p', 'b', 'm'] {
            let s = Spec::from_chart_position(ch).unwrap();
            assert!(matches!(s.lips, Lips::Closed), "lips for {ch} should be closed");
        }
    }

    #[test]
    fn rounded_back_vowels() {
        for ch in ['u', 'ʊ', 'o', 'ɔ'] {
            let s = Spec::from_chart_position(ch).unwrap();
            assert!(matches!(s.lips, Lips::Rounded), "{ch} should be rounded");
        }
    }

    #[test]
    fn voiced_consonants() {
        let s = Spec::from_chart_position('z').unwrap();
        assert!(s.voiced);
        let s = Spec::from_chart_position('s').unwrap();
        assert!(!s.voiced);
    }
}
