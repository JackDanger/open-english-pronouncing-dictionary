//! Anatomical positions for every IPA phoneme in our English corpus.
//!
//! Two coordinate systems share an `(x, y)` plane normalised to
//! `[0, 100]` for each axis:
//!
//! * **Vowels** sit in the classic IPA quadrilateral. `x` is
//!   front↔back: 0 = front of the mouth (lips), 100 = back of the
//!   mouth (pharynx). `y` is close↔open: 0 = tongue high (close),
//!   100 = jaw open (open).
//!
//! * **Consonants** sit in a place × manner grid. `x` is place of
//!   articulation: 0 = bilabial (lips), 100 = glottal (vocal folds),
//!   marching back through the mouth in the order
//!   *labiodental, dental, alveolar, post-alveolar, palatal, velar*.
//!   `y` is manner: 0 = stop, then nasal, fricative, affricate,
//!   approximant, with stress/length marks pushed off-grid.
//!
//! The chart in `render/chart.rs` uses these coordinates verbatim
//! when laying out the SVG. The sagittal-tract module in
//! `render/sagittal.rs` reads tongue position from these same numbers
//! so the chart and the anatomy stay synchronised.

/// Which sub-chart a phoneme belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plane {
    Vowel,
    Consonant,
    /// Stress / length marks; rendered as supra-segmental markers
    /// off to one side, not on the chart proper.
    Supra,
    /// Catch-all for phonemes we don't have a position for. The
    /// chart renders them in a "miscellaneous" lane.
    Other,
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub plane: Plane,
    /// Front↔back for vowels, place of articulation for consonants.
    pub x: f32,
    /// Close↔open for vowels, manner for consonants.
    pub y: f32,
}

impl Position {
    const fn vowel(x: f32, y: f32) -> Self {
        Self { plane: Plane::Vowel, x, y }
    }
    const fn consonant(x: f32, y: f32) -> Self {
        Self { plane: Plane::Consonant, x, y }
    }
    const fn supra() -> Self {
        Self { plane: Plane::Supra, x: 0.0, y: 0.0 }
    }
    const fn other() -> Self {
        Self { plane: Plane::Other, x: 0.0, y: 0.0 }
    }
}

/// Look up the anatomical position for a single IPA character.
///
/// Diphthongs aren't represented as single characters in our broad
/// transcriptions — they're already decomposed into vowel pairs
/// (`eɪ`, `aɪ`, etc.) when the corpus was built. So this function
/// only needs to map *atomic* IPA symbols.
pub fn position(ch: char) -> Position {
    match ch {
        // ── Vowels ──
        // Front, close → front, open along the left side of the chart.
        'i' => Position::vowel( 8.0,  6.0),    // beet
        'ɪ' => Position::vowel(15.0, 22.0),    // bit
        'e' => Position::vowel(10.0, 36.0),    // (start of eɪ)
        'ɛ' => Position::vowel(16.0, 52.0),    // bet
        'æ' => Position::vowel(20.0, 78.0),    // bat
        'a' => Position::vowel(28.0, 92.0),    // (start of aɪ, aʊ)
        // Central column.
        'ɨ' => Position::vowel(50.0,  8.0),    // roses
        'ə' => Position::vowel(52.0, 50.0),    // about
        // ɚ is the unstressed central rhotic vowel — "letter", "butter".
        // We place it next to the schwa to mark the kinship; CSS gives it
        // a small rhotic decoration in the chart.
        'ɚ' => Position::vowel(60.0, 50.0),
        'ʌ' => Position::vowel(58.0, 70.0),    // but
        'ɜ' => Position::vowel(56.0, 56.0),    // bird (non-rhotic)
        // ɝ is the stressed rhotic counterpart of ɜ — "nurse", "bird"
        // in General American.
        'ɝ' => Position::vowel(63.0, 56.0),
        'ɐ' => Position::vowel(54.0, 80.0),    // comma
        // Back column.
        'u' => Position::vowel(92.0,  6.0),    // boot
        'ʊ' => Position::vowel(85.0, 22.0),    // book
        'o' => Position::vowel(90.0, 36.0),    // (start of oʊ)
        'ɔ' => Position::vowel(86.0, 56.0),    // bought
        'ɑ' => Position::vowel(82.0, 88.0),    // father

        // Front rounded — present in some loanwords.
        'y' => Position::vowel(25.0,  8.0),
        'ø' => Position::vowel(25.0, 38.0),
        'œ' => Position::vowel(28.0, 70.0),

        // ── Consonants ──
        // Place: bilabial (5), labiodental (15), dental (28),
        //        alveolar (42), post-alveolar (55), palatal (68),
        //        velar (82), glottal (95).
        // Manner: stop (12), nasal (28), fricative (48),
        //         affricate (62), approximant (78), trill/flap (90).
        // Stops
        'p' => Position::consonant( 5.0, 12.0),
        'b' => Position::consonant( 5.0, 12.0),  // overlap on chart;
        // CSS handles voiced/voiceless within the same cell via small
        // x-offset in the renderer.
        't' => Position::consonant(42.0, 12.0),
        'd' => Position::consonant(42.0, 12.0),
        'k' => Position::consonant(82.0, 12.0),
        'ɡ' | 'g' => Position::consonant(82.0, 12.0),
        'ʔ' => Position::consonant(95.0, 12.0),

        // Nasals
        'm' => Position::consonant( 5.0, 28.0),
        'ɱ' => Position::consonant(15.0, 28.0),
        'n' => Position::consonant(42.0, 28.0),
        'ɲ' => Position::consonant(68.0, 28.0),
        'ŋ' => Position::consonant(82.0, 28.0),

        // Fricatives
        'f' => Position::consonant(15.0, 48.0),
        'v' => Position::consonant(15.0, 48.0),
        'θ' => Position::consonant(28.0, 48.0),
        'ð' => Position::consonant(28.0, 48.0),
        's' => Position::consonant(42.0, 48.0),
        'z' => Position::consonant(42.0, 48.0),
        'ʃ' => Position::consonant(55.0, 48.0),
        'ʒ' => Position::consonant(55.0, 48.0),
        'x' => Position::consonant(82.0, 48.0),
        'ɣ' => Position::consonant(82.0, 48.0),
        'ħ' => Position::consonant(90.0, 48.0),
        'h' => Position::consonant(95.0, 48.0),

        // Affricates (rendered as single tiles even though they're
        // technically a stop+fricative).
        'ʧ' => Position::consonant(55.0, 62.0),
        'ʤ' => Position::consonant(55.0, 62.0),

        // Approximants
        'l' => Position::consonant(42.0, 78.0),
        'ɫ' => Position::consonant(42.0, 80.0),
        'ɹ' => Position::consonant(48.0, 78.0),
        'ɻ' => Position::consonant(50.0, 78.0),
        'j' => Position::consonant(68.0, 78.0),
        'w' => Position::consonant( 8.0, 78.0),
        'ʍ' => Position::consonant( 8.0, 78.0),
        'ʋ' => Position::consonant(15.0, 78.0),

        // Trills / flaps / taps
        'r' => Position::consonant(42.0, 90.0),
        'ɾ' => Position::consonant(42.0, 90.0),

        // Stress + length marks.
        'ˈ' | 'ˌ' | 'ː' | 'ˑ' => Position::supra(),

        _ => Position::other(),
    }
}

/// True if the phoneme is voiced — drives the vocal-cord vibration
/// indicator on the sagittal inset.
pub fn voiced(ch: char) -> bool {
    matches!(
        ch,
        // Voiced consonants.
        'b' | 'd' | 'ɡ' | 'g' | 'v' | 'ð' | 'z' | 'ʒ' | 'ʤ' | 'ɣ' |
        'm' | 'ɱ' | 'n' | 'ɲ' | 'ŋ' |
        'l' | 'ɫ' | 'ɹ' | 'ɻ' | 'j' | 'w' | 'ʋ' |
        'r' | 'ɾ' |
        // All vowels are voiced in English.
        'i' | 'ɪ' | 'e' | 'ɛ' | 'æ' | 'a' | 'ɑ' | 'ɔ' | 'o' | 'u' | 'ʊ' |
        'ə' | 'ʌ' | 'ɜ' | 'ɐ' | 'ɨ' | 'y' | 'ø' | 'œ' | 'ɚ' | 'ɝ'
    )
}

/// Voiced / voiceless pair offset so chart cells with both members
/// (e.g. /p/ and /b/) render side by side instead of on top of each
/// other. Returns an x-axis offset in chart units.
pub fn voicing_offset(ch: char) -> f32 {
    if voiced(ch) && matches!(ch, 'b' | 'd' | 'ɡ' | 'g' | 'v' | 'ð' | 'z' | 'ʒ' | 'ʤ' | 'ɣ') {
        2.5
    } else if matches!(ch, 'p' | 't' | 'k' | 'f' | 'θ' | 's' | 'ʃ' | 'ʧ' | 'x') {
        -2.5
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vowels_in_quadrilateral() {
        // /i/ at top-front; /u/ at top-back; /a/-or-/ɑ/ at bottom.
        let i = position('i');
        let u = position('u');
        let a = position('ɑ');
        assert!(matches!(i.plane, Plane::Vowel));
        assert!(i.x < u.x, "i should be front of u");
        assert!(i.y < a.y, "i should be high, ɑ low");
    }

    #[test]
    fn consonants_progress_front_to_back() {
        let p = position('p');  // bilabial
        let t = position('t');  // alveolar
        let k = position('k');  // velar
        assert!(p.x < t.x);
        assert!(t.x < k.x);
    }

    #[test]
    fn supra_segmental_markers_are_off_chart() {
        assert!(matches!(position('ˈ').plane, Plane::Supra));
        assert!(matches!(position('ˌ').plane, Plane::Supra));
    }
}
