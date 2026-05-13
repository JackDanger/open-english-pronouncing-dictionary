//! Heuristic alignment between a word's spelling and its IPA.
//!
//! English orthography is famously messy — "pneumonia" has a silent
//! "p", "ph" is one sound, "ough" can be /ʌf/ or /oʊ/ or /u/ or /ɔ/
//! depending on the word. A perfect alignment requires a trained
//! grapheme-to-phoneme model.
//!
//! For pedagogical purposes we don't need perfection: we need
//! enough alignment quality that the **visual** band above the IPA
//! chart correctly shows the *kinds* of asymmetry that make English
//! spelling hard:
//!
//! * silent letters (a letter with no phoneme below it)
//! * digraphs (two letters fan into one phoneme)
//! * straight 1-to-1 for the easy cases
//!
//! When the heuristic gets a specific letter wrong, the band still
//! teaches the right lesson — it just labels the wrong column.
//! Worth it for the simplicity.

/// One piece of an alignment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Piece {
    /// One letter → one phoneme.
    Direct { letter: char, phoneme: char },
    /// Two letters → one phoneme (`ph` → /f/, `th` → /θ ð/, etc.).
    Digraph { letters: (char, char), phoneme: char },
    /// Letter present in the spelling but not produced. Renders as
    /// a "dangling" letter with no line.
    Silent { letter: char },
    /// Phoneme produced but not directly represented by any letter
    /// (rare: glide insertions in "use" /jus/, "cute" /kjut/).
    Inserted { phoneme: char },
}

#[derive(Debug, Clone)]
pub struct Alignment {
    pub pieces: Vec<Piece>,
}

/// Digraphs (or trigraphs) where two/three letters map to one
/// phoneme. The table is consulted only when the next phoneme in
/// the IPA stream actually matches the right-hand side — we don't
/// blindly collapse every `ph` into /f/ just because.
const DIGRAPHS: &[(&str, char)] = &[
    ("ph", 'f'),
    ("th", 'θ'),
    ("th", 'ð'),
    ("ch", 'ʧ'),
    ("ch", 'k'),
    ("sh", 'ʃ'),
    ("zh", 'ʒ'),
    ("ng", 'ŋ'),
    ("ck", 'k'),
    ("dge", 'ʤ'),
    ("dg", 'ʤ'),
    ("gh", 'f'),
    ("gh", 'ɡ'),
];

/// Word-initial digraphs where one letter is silent.
/// (letters, surviving phoneme).
const SILENT_PREFIXES: &[(&str, char)] = &[
    ("kn", 'n'),
    ("gn", 'n'),
    ("pn", 'n'),
    ("ps", 's'),
    ("wr", 'ɹ'),
    ("mn", 'n'),
    ("rh", 'ɹ'),
];

/// Word-final letter pairs where one is silent.
const SILENT_SUFFIXES: &[(&str, char)] = &[
    ("mb", 'm'),   // lamb, comb
    ("mn", 'm'),   // autumn
    ("gn", 'n'),   // sign, design
    ("gh", '\0'),  // through, though (gh entirely silent here)
];

/// How a letter spells a single-character phoneme. Diphthongs are
/// already two characters in our broad transcriptions (`eɪ`, `aɪ`,
/// `oʊ`), so we let the diphthong's *first* phoneme be the
/// alignment target and the alignment loop's "trailing inserted"
/// path picks up the second.
fn letter_compatible(letter: char, phoneme: char) -> bool {
    if letter == phoneme {
        return true;
    }
    // Diphthongs in our broad transcriptions are already two
    // characters (eɪ, aɪ, oʊ, aʊ), so the alignment loop will land on
    // their first phoneme here and the trailing-inserted clause picks
    // up the second. That's why the vowel-onset chars 'e', 'a', 'o'
    // show up alongside the more-canonical monophthong targets.
    let table: &[(char, &[char])] = &[
        ('a', &['æ', 'ə', 'ɑ', 'ɔ', 'a', 'e', 'ɛ']),
        ('e', &['ɛ', 'i', 'ə', 'e', 'ɪ']),
        ('i', &['ɪ', 'i', 'ə', 'a']),
        ('o', &['ɑ', 'ɔ', 'ə', 'ʌ', 'o', 'u', 'ʊ']),
        ('u', &['ʌ', 'u', 'ʊ', 'ə', 'j', 'a']),
        ('y', &['ɪ', 'i', 'j', 'a']),
        ('c', &['k', 's', 'ʧ', 'ʃ']),
        ('g', &['ɡ', 'g', 'ʤ', 'ʒ']),
        ('s', &['s', 'z', 'ʃ', 'ʒ']),
        ('z', &['z', 's', 'ʒ']),
        ('x', &['k', 'ɡ', 'z']),
        ('q', &['k']),
        ('j', &['ʤ', 'j']),
        ('r', &['ɹ', 'r']),
        ('l', &['l', 'ɫ']),
        ('n', &['n', 'ŋ']),
    ];
    for (l, opts) in table {
        if *l == letter {
            return opts.contains(&phoneme);
        }
    }
    false
}

/// Walk the letters and the IPA in parallel, emitting alignment
/// pieces. Stress / length marks in the IPA stream are skipped (they
/// don't correspond to a letter — they decorate phonemes).
pub fn align(word: &str, ipa: &str) -> Alignment {
    let letters: Vec<char> = word.chars().filter(|c| c.is_alphabetic()).collect();
    let phonemes: Vec<char> = ipa
        .chars()
        .filter(|c| !matches!(c, 'ˈ' | 'ˌ' | 'ː' | 'ˑ'))
        .collect();

    let mut pieces = Vec::with_capacity(letters.len().max(phonemes.len()));
    let mut i = 0usize;
    let mut j = 0usize;

    // Word-initial silent-letter check (only fires once, at i==0).
    if i == 0 && letters.len() >= 2 {
        let prefix: String = letters.iter().take(2).collect();
        for (pat, phon) in SILENT_PREFIXES {
            if prefix == *pat && phonemes.first() == Some(phon) {
                pieces.push(Piece::Silent { letter: letters[0] });
                pieces.push(Piece::Direct { letter: letters[1], phoneme: phonemes[0] });
                i = 2;
                j = 1;
                break;
            }
        }
    }

    while i < letters.len() {
        // Try digraphs first.
        let mut matched = false;
        if i + 1 < letters.len() && j < phonemes.len() {
            let pair: String = letters[i..i + 2].iter().collect();
            for (pat, phon) in DIGRAPHS {
                if pair == *pat && phonemes[j] == *phon {
                    pieces.push(Piece::Digraph {
                        letters: (letters[i], letters[i + 1]),
                        phoneme: phonemes[j],
                    });
                    i += 2;
                    j += 1;
                    matched = true;
                    break;
                }
            }
        }
        if matched {
            continue;
        }

        // Word-final silent suffix.
        if i + 2 == letters.len() {
            let pair: String = letters[i..i + 2].iter().collect();
            for (pat, surviving) in SILENT_SUFFIXES {
                if pair == *pat {
                    if *surviving == '\0' {
                        // Both letters silent.
                        pieces.push(Piece::Silent { letter: letters[i] });
                        pieces.push(Piece::Silent { letter: letters[i + 1] });
                        i += 2;
                        matched = true;
                        break;
                    } else if j < phonemes.len() && phonemes[j] == *surviving {
                        pieces.push(Piece::Direct { letter: letters[i], phoneme: *surviving });
                        pieces.push(Piece::Silent { letter: letters[i + 1] });
                        i += 2;
                        j += 1;
                        matched = true;
                        break;
                    }
                }
            }
        }
        if matched {
            continue;
        }

        // Otherwise, single-letter step.
        if j < phonemes.len() && letter_compatible(letters[i], phonemes[j]) {
            pieces.push(Piece::Direct { letter: letters[i], phoneme: phonemes[j] });
            i += 1;
            j += 1;
        } else if j < phonemes.len()
            && j + 1 < phonemes.len()
            && !letter_compatible(letters[i], phonemes[j])
            && letter_compatible(letters[i], phonemes[j + 1])
        {
            // The current phoneme has no letter source — likely a
            // glide insertion (use /jus/ where /j/ has no letter).
            pieces.push(Piece::Inserted { phoneme: phonemes[j] });
            j += 1;
        } else {
            // Letter is silent or our heuristic gave up. If the IPA
            // has clearly fewer phonemes left than we have letters,
            // assume silent; otherwise take the next phoneme as a
            // best-effort fallback.
            let letters_left = letters.len() - i;
            let phonemes_left = phonemes.len() - j;
            if phonemes_left < letters_left {
                pieces.push(Piece::Silent { letter: letters[i] });
                i += 1;
            } else if j < phonemes.len() {
                pieces.push(Piece::Direct { letter: letters[i], phoneme: phonemes[j] });
                i += 1;
                j += 1;
            } else {
                pieces.push(Piece::Silent { letter: letters[i] });
                i += 1;
            }
        }
    }

    // Trailing phonemes with no letters left → inserted.
    while j < phonemes.len() {
        pieces.push(Piece::Inserted { phoneme: phonemes[j] });
        j += 1;
    }

    Alignment { pieces }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_silent(a: &Alignment) -> usize {
        a.pieces.iter().filter(|p| matches!(p, Piece::Silent { .. })).count()
    }
    fn count_digraph(a: &Alignment) -> usize {
        a.pieces.iter().filter(|p| matches!(p, Piece::Digraph { .. })).count()
    }

    #[test]
    fn cat_is_three_directs() {
        let a = align("cat", "kæt");
        assert_eq!(a.pieces.len(), 3);
        assert!(matches!(a.pieces[0], Piece::Direct { letter: 'c', phoneme: 'k' }));
        assert!(matches!(a.pieces[2], Piece::Direct { letter: 't', phoneme: 't' }));
    }

    #[test]
    fn pneumonia_has_silent_p() {
        let a = align("pneumonia", "numoʊnjə");
        assert!(count_silent(&a) >= 1, "expected ≥1 silent letter in pneumonia, got {a:?}");
        // The first letter (p) should be silent.
        assert!(matches!(a.pieces[0], Piece::Silent { letter: 'p' }));
    }

    #[test]
    fn phone_has_ph_digraph() {
        let a = align("phone", "foʊn");
        assert_eq!(count_digraph(&a), 1, "expected one digraph in phone, got {a:?}");
        assert!(matches!(a.pieces[0], Piece::Digraph { letters: ('p','h'), phoneme: 'f' }));
    }

    #[test]
    fn thin_uses_th_digraph() {
        let a = align("thin", "θɪn");
        assert!(matches!(a.pieces[0], Piece::Digraph { letters: ('t','h'), phoneme: 'θ' }));
    }

    #[test]
    fn lamb_has_silent_b() {
        let a = align("lamb", "læm");
        // Final 'b' should be silent.
        assert!(a.pieces.iter().any(|p| matches!(p, Piece::Silent { letter: 'b' })),
                "expected silent b in lamb, got {a:?}");
    }
}
