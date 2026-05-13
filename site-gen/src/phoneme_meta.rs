//! Human-readable metadata for each IPA phoneme: a name, an
//! anatomical / instructional description, an example word, and a
//! categorical bucket with a colour for visual grouping.
//!
//! Audience is non-linguists — the descriptions deliberately avoid
//! technical jargon and use body-position analogies ("tongue tip at
//! the gum ridge", "lips snap apart"). When a phoneme has multiple
//! descriptions across linguistic conventions, we pick the one
//! easiest for a curious layperson to act on.

/// Verbal description of how to produce a single IPA phoneme.
pub struct Phoneme {
    /// Human-readable name (e.g. "Schwa", "American R sound").
    pub name: &'static str,
    /// 2–4 sentence description, written for a non-linguist.
    pub desc: &'static str,
    /// A common English word that contains this sound.
    pub example: &'static str,
}

/// Look up the description triple for an IPA character.
pub fn phoneme(ch: char) -> Phoneme {
    match ch {
        // ── Vowels ──
        'ə' => Phoneme {
            name: "Schwa",
            desc: "The most common sound in English. Your mouth is barely open, \
                   your tongue rests in the middle of your mouth, and you make no \
                   special effort. It's the 'a' in 'a-bout', the 'e' in 'the', \
                   and the 'o' in 'to-day'.",
            example: "about",
        },
        'ɪ' => Phoneme {
            name: "Short I vowel",
            desc: "Like the 'i' in 'bit'. Your tongue is high and forward in your \
                   mouth, but more relaxed than for the long 'ee' sound. Your lips \
                   spread slightly.",
            example: "bit",
        },
        'i' => Phoneme {
            name: "Long EE vowel",
            desc: "Like the 'ee' in 'see'. Tongue as high and forward as it goes, \
                   lips spread wide. English often uses this at the end of words: \
                   'happy', 'city'.",
            example: "see",
        },
        'ɛ' => Phoneme {
            name: "Short E vowel",
            desc: "Like the 'e' in 'bed'. Your jaw drops further than for 'ɪ' — \
                   tongue lowers and moves slightly back. Think of a relaxed, \
                   open 'eh' sound.",
            example: "bed",
        },
        'æ' => Phoneme {
            name: "Short A vowel",
            desc: "Like the 'a' in 'cat'. This is as far forward and open as a vowel \
                   gets — jaw drops, tongue pushes all the way to the front. \
                   Many non-English speakers find this one hard to produce.",
            example: "cat",
        },
        'ʌ' => Phoneme {
            name: "Short U vowel",
            desc: "Like the 'u' in 'cup'. Short and central — neither front nor back. \
                   Your jaw drops moderately. Think of saying a short, sharp 'uh'.",
            example: "cup",
        },
        'ɑ' => Phoneme {
            name: "Open back vowel",
            desc: "Like the 'a' in 'father'. Your jaw drops as wide as it can, \
                   tongue pulls back and lies flat. This is what doctors ask you to \
                   say when they check your throat.",
            example: "father",
        },
        'ɔ' => Phoneme {
            name: "Open O vowel",
            desc: "Like the 'aw' in 'thought'. Your lips round into an 'O' shape \
                   while the jaw stays fairly open. British and American English \
                   differ greatly on this one.",
            example: "thought",
        },
        'ʊ' => Phoneme {
            name: "Short OO vowel",
            desc: "Like the 'oo' in 'foot'. Lips are slightly rounded, tongue \
                   pulls back — but more relaxed than the tight 'oo' in 'food'. \
                   Compare 'look' vs 'Luke'.",
            example: "foot",
        },
        'u' => Phoneme {
            name: "Long OO vowel",
            desc: "Like the 'oo' in 'food'. Lips are tightly pursed and rounded, \
                   tongue pulled all the way back and up. The most extreme back \
                   vowel in English.",
            example: "food",
        },
        'ɜ' => Phoneme {
            name: "Nurse vowel",
            desc: "Like the 'ur' in 'bird'. Tongue sits in the middle of your \
                   mouth — neither front nor back, neither high nor low. In \
                   American English it's often colored by an 'r'-like quality.",
            example: "bird",
        },
        'e' => Phoneme {
            name: "Pure E vowel",
            desc: "A tense, pure 'e' — like in Spanish 'mesa'. English usually \
                   turns this into a diphthong (gliding toward 'ɪ'), but some \
                   accents and all IPA transcriptions use the pure form.",
            example: "day",
        },
        'o' => Phoneme {
            name: "Pure O vowel",
            desc: "A pure rounded 'o' without any glide — like in Spanish 'no'. \
                   English 'go' starts here but glides toward 'ʊ'. Some accents \
                   keep it pure.",
            example: "go",
        },
        'a' => Phoneme {
            name: "Bright A vowel",
            desc: "A bright, open 'ah' — like in Italian 'pasta'. In English it \
                   mainly appears as the starting point of diphthongs: the 'a' \
                   sound at the start of 'my' (/maɪ/) or 'how' (/haʊ/).",
            example: "my",
        },
        'ɐ' => Phoneme {
            name: "Near-open central vowel",
            desc: "A vowel between the schwa (ə) and the open 'a' — common in \
                   some accents at the ends of words like 'comma'. Your mouth \
                   is slightly more open than for a schwa.",
            example: "comma",
        },
        'y' => Phoneme {
            name: "Front rounded vowel",
            desc: "Say the 'ee' in 'see', then round your lips as if saying 'oo'. \
                   You get this sound — common in French ('tu') and German ('über') \
                   but rare in English.",
            example: "über",
        },
        'ø' => Phoneme {
            name: "Front rounded mid vowel",
            desc: "Say 'e' as in 'bed', then round your lips. This appears in \
                   French ('feu') and German ('schön') but not standard English.",
            example: "feu",
        },
        'œ' => Phoneme {
            name: "Open front rounded vowel",
            desc: "Like 'æ' in 'cat' but with rounded lips. Found in French 'peur' \
                   and some non-standard English pronunciations.",
            example: "peur",
        },
        'ɨ' => Phoneme {
            name: "Central close vowel",
            desc: "A high vowel with your tongue in the center of your mouth — \
                   not as far forward as 'i' nor as far back as 'u'. Common in \
                   Russian and some American English pronunciations.",
            example: "roses",
        },

        // ── Plosives ──
        'p' => Phoneme {
            name: "P sound",
            desc: "Both lips press together firmly, blocking all air, then snap \
                   apart in a tiny burst. In English, a 'p' at the start of a \
                   word comes with a puff of air (aspiration) — hold your hand \
                   in front of your mouth and feel it on 'pin' vs 'spin'.",
            example: "pin",
        },
        'b' => Phoneme {
            name: "B sound",
            desc: "Exactly like 'p' — lips seal and burst — but your vocal cords \
                   vibrate. Put your fingers on your throat: you'll feel buzzing \
                   for 'b' but not for 'p'.",
            example: "bin",
        },
        't' => Phoneme {
            name: "T sound",
            desc: "Your tongue tip presses against the ridge just behind your \
                   upper front teeth, blocking air, then snaps away. In American \
                   English, a 't' between vowels often becomes a quick 'd'-like \
                   tap: 'butter', 'city'.",
            example: "top",
        },
        'd' => Phoneme {
            name: "D sound",
            desc: "Just like 't' — same tongue position and same burst — but with \
                   your vocal cords vibrating. Say 'ten' then 'den' and feel your \
                   throat buzz on the 'd'.",
            example: "dog",
        },
        'k' => Phoneme {
            name: "K sound",
            desc: "The back of your tongue presses against your soft palate (the \
                   soft part at the back of the roof of your mouth), blocks all \
                   air, then releases. Feel it happen by saying 'king' slowly.",
            example: "king",
        },
        'ɡ' | 'g' => Phoneme {
            name: "G sound",
            desc: "Same as 'k' — back of tongue against soft palate — but with \
                   vocal cords vibrating. The 'g' in 'go'. Compare 'cold' vs 'gold' \
                   and feel the difference in your throat.",
            example: "go",
        },
        'ʔ' => Phoneme {
            name: "Glottal stop",
            desc: "Your vocal cords snap completely shut, briefly cutting off all \
                   sound, then release. You make this sound between the syllables \
                   of 'uh-oh'. In many British accents it replaces the 't' in \
                   'butter' — 'bu-uh'.",
            example: "uh-oh",
        },

        // ── Fricatives ──
        'f' => Phoneme {
            name: "F sound",
            desc: "Your upper front teeth lightly rest on your lower lip. Air \
                   squeezes through the narrow gap, creating a hissing sound. \
                   No vocal cord vibration. Feel it by prolonging the 'f' in 'fan'.",
            example: "fan",
        },
        'v' => Phoneme {
            name: "V sound",
            desc: "Exactly like 'f' — same teeth-on-lip position — but with your \
                   vocal cords buzzing. Hold your lower lip to your teeth and hum: \
                   that's a 'v'. The 'f'/'v' pair is the same sound, voiced vs voiceless.",
            example: "van",
        },
        'θ' => Phoneme {
            name: "Voiceless TH sound",
            desc: "Tongue tip slides between your teeth, or just behind them. Air \
                   flows over the tongue — no buzzing. This is 'th' in 'thin', 'think', \
                   'bath'. Many languages don't have this sound at all, making it \
                   tricky for learners.",
            example: "thin",
        },
        'ð' => Phoneme {
            name: "Voiced TH sound",
            desc: "Same tongue position as 'θ' — tip between or behind teeth — but \
                   your vocal cords vibrate. This is 'th' in 'this', 'the', 'smooth'. \
                   English has two 'th' sounds and most speakers never notice.",
            example: "this",
        },
        's' => Phoneme {
            name: "S sound",
            desc: "Tongue tip near the gum ridge, air squeezed through a tiny \
                   channel and directed at the back of your teeth. The high-pitched \
                   hiss of 's'. No vocal buzz. Try prolonging it in 'sun'.",
            example: "sun",
        },
        'z' => Phoneme {
            name: "Z sound",
            desc: "Same as 's' — same tongue, same narrow channel — but your vocal \
                   cords buzz. 'z' in 'zoo'. Put your fingertips on your throat: \
                   you'll feel it vibrate for 'z' but go silent for 's'.",
            example: "zoo",
        },
        'ʃ' => Phoneme {
            name: "SH sound",
            desc: "Your tongue moves slightly further back than for 's', and your \
                   lips round a little. The result is the hushing 'sh' in 'ship', \
                   'she', 'fashion'. Lower-pitched than 's'.",
            example: "ship",
        },
        'ʒ' => Phoneme {
            name: "Voiced SH sound",
            desc: "Same tongue position as 'ʃ', but with vocal cord vibration. \
                   This is the 's' in 'measure', 'treasure', 'vision'. It's also \
                   the 'j' in French 'bonjour'.",
            example: "measure",
        },
        'h' => Phoneme {
            name: "H sound",
            desc: "Your vocal cords are open and relaxed — air just rushes out \
                   from your lungs without any shaping. The 'h' in 'hat', 'hello'. \
                   It's sometimes called the 'voiceless breath' — the lightest \
                   sound in English.",
            example: "hat",
        },
        'x' => Phoneme {
            name: "Voiceless velar fricative",
            desc: "Like 'k' but instead of blocking completely, you let air \
                   squeeze through — a raspy sound. This is the 'ch' in Scottish \
                   'loch', German 'Bach', or Hebrew 'Chanukah'.",
            example: "loch",
        },
        'ɣ' => Phoneme {
            name: "Voiced velar fricative",
            desc: "Like 'x' but with vocal cord vibration. Appears in Spanish \
                   'lago' (between vowels) and Greek. Rare in English.",
            example: "lago",
        },
        'ħ' => Phoneme {
            name: "Pharyngeal fricative",
            desc: "A deep, tightly constricted 'h'-like sound from the back of \
                   your throat — pharynx narrowed. Common in Arabic. In English \
                   some loanwords carry this.",
            example: "חָ",
        },

        // ── Nasals ──
        'm' => Phoneme {
            name: "M sound",
            desc: "Your lips press together (just like 'p' and 'b'), but instead \
                   of blocking the air, you let it flow out through your nose. \
                   Hum with your lips shut: that's 'm'. Feel the vibration in \
                   your nose.",
            example: "man",
        },
        'n' => Phoneme {
            name: "N sound",
            desc: "Your tongue tip presses against the gum ridge (like 't' and 'd'), \
                   but air flows through your nose. Hum while touching the ridge \
                   with your tongue tip. Feel your nose vibrate.",
            example: "no",
        },
        'ŋ' => Phoneme {
            name: "NG sound",
            desc: "Back of tongue against the soft palate (like 'k' and 'g'), \
                   but air flows through the nose. The 'ng' in 'sing', 'ring'. \
                   This sound can never start a word in English — always in the \
                   middle or at the end.",
            example: "sing",
        },
        'ɲ' => Phoneme {
            name: "NY sound",
            desc: "Like 'n' but your tongue presses against the hard palate \
                   further back. The 'ny' in 'canyon', Spanish 'ñ' in 'mañana', \
                   Italian 'gn' in 'gnocchi'.",
            example: "canyon",
        },
        'ɱ' => Phoneme {
            name: "Labiodental nasal",
            desc: "Like 'm' but your upper teeth touch your lower lip (like 'f' \
                   and 'v') while air flows through the nose. Occurs naturally \
                   before 'f' in words like 'symphony' — try saying it slowly.",
            example: "symphony",
        },

        // ── Approximants ──
        'l' => Phoneme {
            name: "L sound",
            desc: "Tongue tip presses the gum ridge, but instead of blocking all \
                   air, you let it flow around the sides of your tongue. English \
                   'l' varies: 'light l' at the start of 'lip', 'dark l' at the \
                   end of 'feel' (tongue pulls back).",
            example: "lip",
        },
        'ɫ' => Phoneme {
            name: "Dark L sound",
            desc: "A 'dark' or 'velarized' l — tongue tip at the gum ridge, but \
                   the body of your tongue simultaneously pulls backward. The 'l' \
                   in 'milk', 'feel', 'ball'. Many accents turn this into a 'w'- \
                   or 'o'-like sound.",
            example: "milk",
        },
        'ɹ' => Phoneme {
            name: "American R sound",
            desc: "The iconic American English 'r'. Your tongue tip either curls \
                   back without touching anything, or bunches up in the middle of \
                   your mouth. No part of your tongue makes contact. This is why \
                   English 'r' is so hard for learners.",
            example: "red",
        },
        'r' => Phoneme {
            name: "Trilled R sound",
            desc: "The tongue tip vibrates rapidly against the gum ridge — a \
                   'rolled r'. Standard in Spanish ('perro'), Italian, Russian, \
                   and Scottish English. The American English 'r' (ɹ) is a \
                   different, non-trilled sound.",
            example: "perro",
        },
        'ɻ' => Phoneme {
            name: "Retroflex R sound",
            desc: "Like the American R but with the tongue tip curled further \
                   back. Common in South Asian languages and some American \
                   dialects.",
            example: "red",
        },
        'j' => Phoneme {
            name: "Y glide",
            desc: "Like saying 'ee' but then immediately sliding into another \
                   vowel. The 'y' in 'yes', 'you'. Your tongue starts high and \
                   forward, then moves to the position of the next vowel. \
                   Not a hard consonant — a smooth glide.",
            example: "yes",
        },
        'w' => Phoneme {
            name: "W glide",
            desc: "Your lips round tightly and your tongue pulls back, then you \
                   immediately slide into the next vowel. The 'w' in 'wet', 'way'. \
                   Like a very quick 'oo' before the main vowel.",
            example: "wet",
        },
        'ʍ' => Phoneme {
            name: "Voiceless W sound",
            desc: "Like 'w' but without vocal cord vibration — a breathy 'hw' \
                   sound. Traditional pronunciation of 'wh' in 'which', 'where', \
                   'whale'. Most modern American and British speakers merge \
                   this with regular 'w'.",
            example: "which",
        },
        'ʋ' => Phoneme {
            name: "Labiodental approximant",
            desc: "Between 'v' and 'w' — lower lip approaches upper teeth but \
                   doesn't quite make contact enough for friction. Appears in \
                   Dutch and some languages. English speakers hear it as \
                   either 'v' or 'w'.",
            example: "vlag",
        },

        // ── Affricates ──
        'ʧ' => Phoneme {
            name: "CH sound",
            desc: "Starts like 't' (tongue tip at gum ridge, complete closure) \
                   then releases as 'ʃ' (the 'sh' sound). 'ch' in 'church'. \
                   Two sounds in one: a stop that turns into a fricative.",
            example: "church",
        },
        'ʤ' => Phoneme {
            name: "J sound",
            desc: "Like 'ʧ' (the 'ch' sound) but with vocal cord vibration. \
                   The 'j' in 'judge'. Starts as 'd', releases as 'ʒ'. \
                   Also the 'g' in 'gem', 'giant'.",
            example: "judge",
        },

        // ── Stress & length ──
        'ˈ' => Phoneme {
            name: "Primary stress mark",
            desc: "This symbol doesn't represent a sound — it marks the syllable \
                   that gets the strongest emphasis. In English, stress changes \
                   meaning: 'CON-tent' (noun) vs 'con-TENT' (adjective). \
                   Always placed just before the stressed syllable.",
            example: "a-BOUT",
        },
        'ˌ' => Phoneme {
            name: "Secondary stress mark",
            desc: "Marks a syllable with lighter emphasis — weaker than primary \
                   stress but stronger than unstressed syllables. Long words \
                   often have both: 'pho-ne-TI-cian' has primary stress on \
                   'TI' and secondary on 'pho'.",
            example: "pho-ne-TI-cian",
        },
        'ː' => Phoneme {
            name: "Length mark",
            desc: "The preceding sound is held for longer than usual. Compare \
                   the 'i' in 'sit' /sɪt/ (short) with the 'ee' in 'seat' \
                   /siːt/ (long). Length can be the only difference between \
                   two words in some languages.",
            example: "seat vs sit",
        },

        _ => Phoneme {
            name: "IPA symbol",
            desc: "An IPA transcription symbol found in this corpus.",
            example: "",
        },
    }
}

/// Phoneme category bucket: a slug, a colour for visual grouping, and
/// a human-readable label.
#[derive(Debug, Clone, Copy)]
pub struct Category {
    pub slug: &'static str,
    pub color: &'static str,
    pub label: &'static str,
}

pub fn category(ch: char) -> Category {
    match ch {
        'i' | 'ɪ' | 'e' | 'ɛ' | 'æ' | 'a' | 'ɑ' | 'ɔ' | 'o' | 'u' | 'ʊ'
        | 'ə' | 'ʌ' | 'ɜ' | 'ɐ' | 'ɵ' | 'ɤ' | 'ɯ' | 'y' | 'ʏ'
        | 'ø' | 'œ' | 'ɨ' | 'ʉ' | 'ɞ' | 'ɶ' => Category {
            slug: "vowel",
            color: "#d97706",
            label: "Vowels",
        },
        'p' | 'b' | 't' | 'd' | 'k' | 'ɡ' | 'g' | 'ʔ' | 'ɓ' | 'ɗ' | 'ɠ' => Category {
            slug: "stop",
            color: "#16a34a",
            label: "Plosives",
        },
        'f' | 'v' | 'θ' | 'ð' | 's' | 'z' | 'ʃ' | 'ʒ' | 'h' | 'ɦ' | 'x'
        | 'χ' | 'ɣ' | 'ʁ' | 'ħ' | 'ʕ' | 'ɸ' | 'β' | 'ɬ' | 'ɮ' | 'ʂ'
        | 'ʐ' | 'ɕ' | 'ʑ' => Category {
            slug: "fricative",
            color: "#2563eb",
            label: "Fricatives",
        },
        'm' | 'n' | 'ŋ' | 'ɲ' | 'ɱ' | 'ɴ' => Category {
            slug: "nasal",
            color: "#9333ea",
            label: "Nasals",
        },
        'l' | 'r' | 'ɹ' | 'ɻ' | 'j' | 'w' | 'ʍ' | 'ʋ' | 'ɫ' | 'ʎ' | 'ɰ'
        | 'ʟ' => Category {
            slug: "approx",
            color: "#0891b2",
            label: "Approximants",
        },
        'ʧ' | 'ʤ' | 'ʦ' | 'ʣ' => Category {
            slug: "affricate",
            color: "#ea580c",
            label: "Affricates",
        },
        'ˈ' | 'ˌ' | 'ː' | 'ˑ' => Category {
            slug: "supra",
            color: "#a16207",
            label: "Stress & Length",
        },
        _ => Category {
            slug: "other",
            color: "#6b7280",
            label: "Other",
        },
    }
}

/// Display order of categories — controls section order in the
/// Phoneme Universe.
pub const CATEGORY_ORDER: &[&str] = &[
    "vowel",
    "stop",
    "fricative",
    "nasal",
    "approx",
    "affricate",
    "supra",
    "other",
];
