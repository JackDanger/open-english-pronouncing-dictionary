// site-gen: reads OpenEPD (data/openepd.json) and generates a static
// GitHub Pages site designed to teach non-linguists about phonetics.
//
// Usage: site-gen <corpus.json> <output-dir>
//
// Lives inside the open-english-pronouncing-dictionary workspace as a
// private (`publish = false`) build tool; not part of the crate
// published to crates.io.

use std::collections::{BTreeMap, HashMap};
use std::{env, fs};

// ── Phoneme descriptions ───────────────────────────────────────────────────────
// Plain-English descriptions for a non-linguist audience.  Every entry has:
//   (technical_name, body_analogy_description, example_word)

fn phoneme_description(ch: char) -> (&'static str, &'static str, &'static str) {
    match ch {
        // ── Vowels ──
        'ə' => (
            "Schwa",
            "The most common sound in English. Your mouth is barely open, \
             your tongue rests in the middle of your mouth, and you make no \
             special effort. It's the 'a' in 'a-bout', the 'e' in 'the', \
             and the 'o' in 'to-day'.",
            "about",
        ),
        'ɪ' => (
            "Short I vowel",
            "Like the 'i' in 'bit'. Your tongue is high and forward in your \
             mouth, but more relaxed than for the long 'ee' sound. Your lips \
             spread slightly.",
            "bit",
        ),
        'i' => (
            "Long EE vowel",
            "Like the 'ee' in 'see'. Tongue as high and forward as it goes, \
             lips spread wide. English often uses this at the end of words: \
             'happy', 'city'.",
            "see",
        ),
        'ɛ' => (
            "Short E vowel",
            "Like the 'e' in 'bed'. Your jaw drops further than for 'ɪ' — \
             tongue lowers and moves slightly back. Think of a relaxed, \
             open 'eh' sound.",
            "bed",
        ),
        'æ' => (
            "Short A vowel",
            "Like the 'a' in 'cat'. This is as far forward and open as a vowel \
             gets — jaw drops, tongue pushes all the way to the front. \
             Many non-English speakers find this one hard to produce.",
            "cat",
        ),
        'ʌ' => (
            "Short U vowel",
            "Like the 'u' in 'cup'. Short and central — neither front nor back. \
             Your jaw drops moderately. Think of saying a short, sharp 'uh'.",
            "cup",
        ),
        'ɑ' => (
            "Open back vowel",
            "Like the 'a' in 'father'. Your jaw drops as wide as it can, \
             tongue pulls back and lies flat. This is what doctors ask you to \
             say when they check your throat.",
            "father",
        ),
        'ɔ' => (
            "Open O vowel",
            "Like the 'aw' in 'thought'. Your lips round into an 'O' shape \
             while the jaw stays fairly open. British and American English \
             differ greatly on this one.",
            "thought",
        ),
        'ʊ' => (
            "Short OO vowel",
            "Like the 'oo' in 'foot'. Lips are slightly rounded, tongue \
             pulls back — but more relaxed than the tight 'oo' in 'food'. \
             Compare 'look' vs 'Luke'.",
            "foot",
        ),
        'u' => (
            "Long OO vowel",
            "Like the 'oo' in 'food'. Lips are tightly pursed and rounded, \
             tongue pulled all the way back and up. The most extreme back \
             vowel in English.",
            "food",
        ),
        'ɜ' => (
            "Nurse vowel",
            "Like the 'ur' in 'bird'. Tongue sits in the middle of your \
             mouth — neither front nor back, neither high nor low. In \
             American English it's often colored by an 'r'-like quality.",
            "bird",
        ),
        'e' => (
            "Pure E vowel",
            "A tense, pure 'e' — like in Spanish 'mesa'. English usually \
             turns this into a diphthong (gliding toward 'ɪ'), but some \
             accents and all IPA transcriptions use the pure form.",
            "day",
        ),
        'o' => (
            "Pure O vowel",
            "A pure rounded 'o' without any glide — like in Spanish 'no'. \
             English 'go' starts here but glides toward 'ʊ'. Some accents \
             keep it pure.",
            "go",
        ),
        'a' => (
            "Bright A vowel",
            "A bright, open 'ah' — like in Italian 'pasta'. In English it \
             mainly appears as the starting point of diphthongs: the 'a' \
             sound at the start of 'my' (/maɪ/) or 'how' (/haʊ/).",
            "my",
        ),
        'ɐ' => (
            "Near-open central vowel",
            "A vowel between the schwa (ə) and the open 'a' — common in \
             some accents at the ends of words like 'comma'. Your mouth \
             is slightly more open than for a schwa.",
            "comma",
        ),
        'y' => (
            "Front rounded vowel",
            "Say the 'ee' in 'see', then round your lips as if saying 'oo'. \
             You get this sound — common in French ('tu') and German ('über') \
             but rare in English.",
            "über",
        ),
        'ø' => (
            "Front rounded mid vowel",
            "Say 'e' as in 'bed', then round your lips. This appears in \
             French ('feu') and German ('schön') but not standard English.",
            "feu",
        ),
        'œ' => (
            "Open front rounded vowel",
            "Like 'æ' in 'cat' but with rounded lips. Found in French 'peur' \
             and some non-standard English pronunciations.",
            "peur",
        ),
        'ɨ' => (
            "Central close vowel",
            "A high vowel with your tongue in the center of your mouth — \
             not as far forward as 'i' nor as far back as 'u'. Common in \
             Russian and some American English pronunciations.",
            "roses",
        ),

        // ── Plosives (stops) ──
        'p' => (
            "P sound",
            "Both lips press together firmly, blocking all air, then snap \
             apart in a tiny burst. In English, a 'p' at the start of a \
             word comes with a puff of air (aspiration) — hold your hand \
             in front of your mouth and feel it on 'pin' vs 'spin'.",
            "pin",
        ),
        'b' => (
            "B sound",
            "Exactly like 'p' — lips seal and burst — but your vocal cords \
             vibrate. Put your fingers on your throat: you'll feel buzzing \
             for 'b' but not for 'p'.",
            "bin",
        ),
        't' => (
            "T sound",
            "Your tongue tip presses against the ridge just behind your \
             upper front teeth, blocking air, then snaps away. In American \
             English, a 't' between vowels often becomes a quick 'd'-like \
             tap: 'butter', 'city'.",
            "top",
        ),
        'd' => (
            "D sound",
            "Just like 't' — same tongue position and same burst — but with \
             your vocal cords vibrating. Say 'ten' then 'den' and feel your \
             throat buzz on the 'd'.",
            "dog",
        ),
        'k' => (
            "K sound",
            "The back of your tongue presses against your soft palate (the \
             soft part at the back of the roof of your mouth), blocks all \
             air, then releases. Feel it happen by saying 'king' slowly.",
            "king",
        ),
        'ɡ' | 'g' => (
            "G sound",
            "Same as 'k' — back of tongue against soft palate — but with \
             vocal cords vibrating. The 'g' in 'go'. Compare 'cold' vs 'gold' \
             and feel the difference in your throat.",
            "go",
        ),
        'ʔ' => (
            "Glottal stop",
            "Your vocal cords snap completely shut, briefly cutting off all \
             sound, then release. You make this sound between the syllables \
             of 'uh-oh'. In many British accents it replaces the 't' in \
             'butter' — 'bu-uh'.",
            "uh-oh",
        ),

        // ── Fricatives ──
        'f' => (
            "F sound",
            "Your upper front teeth lightly rest on your lower lip. Air \
             squeezes through the narrow gap, creating a hissing sound. \
             No vocal cord vibration. Feel it by prolonging the 'f' in 'fan'.",
            "fan",
        ),
        'v' => (
            "V sound",
            "Exactly like 'f' — same teeth-on-lip position — but with your \
             vocal cords buzzing. Hold your lower lip to your teeth and hum: \
             that's a 'v'. The 'f'/'v' pair is the same sound, voiced vs voiceless.",
            "van",
        ),
        'θ' => (
            "Voiceless TH sound",
            "Tongue tip slides between your teeth, or just behind them. Air \
             flows over the tongue — no buzzing. This is 'th' in 'thin', 'think', \
             'bath'. Many languages don't have this sound at all, making it \
             tricky for learners.",
            "thin",
        ),
        'ð' => (
            "Voiced TH sound",
            "Same tongue position as 'θ' — tip between or behind teeth — but \
             your vocal cords vibrate. This is 'th' in 'this', 'the', 'smooth'. \
             English has two 'th' sounds and most speakers never notice.",
            "this",
        ),
        's' => (
            "S sound",
            "Tongue tip near the gum ridge, air squeezed through a tiny \
             channel and directed at the back of your teeth. The high-pitched \
             hiss of 's'. No vocal buzz. Try prolonging it in 'sun'.",
            "sun",
        ),
        'z' => (
            "Z sound",
            "Same as 's' — same tongue, same narrow channel — but your vocal \
             cords buzz. 'z' in 'zoo'. Put your fingertips on your throat: \
             you'll feel it vibrate for 'z' but go silent for 's'.",
            "zoo",
        ),
        'ʃ' => (
            "SH sound",
            "Your tongue moves slightly further back than for 's', and your \
             lips round a little. The result is the hushing 'sh' in 'ship', \
             'she', 'fashion'. Lower-pitched than 's'.",
            "ship",
        ),
        'ʒ' => (
            "Voiced SH sound",
            "Same tongue position as 'ʃ', but with vocal cord vibration. \
             This is the 's' in 'measure', 'treasure', 'vision'. It's also \
             the 'j' in French 'bonjour'.",
            "measure",
        ),
        'h' => (
            "H sound",
            "Your vocal cords are open and relaxed — air just rushes out \
             from your lungs without any shaping. The 'h' in 'hat', 'hello'. \
             It's sometimes called the 'voiceless breath' — the lightest \
             sound in English.",
            "hat",
        ),
        'x' => (
            "Voiceless velar fricative",
            "Like 'k' but instead of blocking completely, you let air \
             squeeze through — a raspy sound. This is the 'ch' in Scottish \
             'loch', German 'Bach', or Hebrew 'Chanukah'.",
            "loch",
        ),
        'ɣ' => (
            "Voiced velar fricative",
            "Like 'x' but with vocal cord vibration. Appears in Spanish \
             'lago' (between vowels) and Greek. Rare in English.",
            "lago",
        ),
        'ħ' => (
            "Pharyngeal fricative",
            "A deep, tightly constricted 'h'-like sound from the back of \
             your throat — pharynx narrowed. Common in Arabic. In English \
             some loanwords carry this.",
            "חָ",
        ),

        // ── Nasals ──
        'm' => (
            "M sound",
            "Your lips press together (just like 'p' and 'b'), but instead \
             of blocking the air, you let it flow out through your nose. \
             Hum with your lips shut: that's 'm'. Feel the vibration in \
             your nose.",
            "man",
        ),
        'n' => (
            "N sound",
            "Your tongue tip presses against the gum ridge (like 't' and 'd'), \
             but air flows through your nose. Hum while touching the ridge \
             with your tongue tip. Feel your nose vibrate.",
            "no",
        ),
        'ŋ' => (
            "NG sound",
            "Back of tongue against the soft palate (like 'k' and 'g'), \
             but air flows through the nose. The 'ng' in 'sing', 'ring'. \
             This sound can never start a word in English — always in the \
             middle or at the end.",
            "sing",
        ),
        'ɲ' => (
            "NY sound",
            "Like 'n' but your tongue presses against the hard palate \
             further back. The 'ny' in 'canyon', Spanish 'ñ' in 'mañana', \
             Italian 'gn' in 'gnocchi'.",
            "canyon",
        ),
        'ɱ' => (
            "Labiodental nasal",
            "Like 'm' but your upper teeth touch your lower lip (like 'f' \
             and 'v') while air flows through the nose. Occurs naturally \
             before 'f' in words like 'symphony' — try saying it slowly.",
            "symphony",
        ),

        // ── Approximants ──
        'l' => (
            "L sound",
            "Tongue tip presses the gum ridge, but instead of blocking all \
             air, you let it flow around the sides of your tongue. English \
             'l' varies: 'light l' at the start of 'lip', 'dark l' at the \
             end of 'feel' (tongue pulls back).",
            "lip",
        ),
        'ɫ' => (
            "Dark L sound",
            "A 'dark' or 'velarized' l — tongue tip at the gum ridge, but \
             the body of your tongue simultaneously pulls backward. The 'l' \
             in 'milk', 'feel', 'ball'. Many accents turn this into a 'w'- \
             or 'o'-like sound.",
            "milk",
        ),
        'ɹ' => (
            "American R sound",
            "The iconic American English 'r'. Your tongue tip either curls \
             back without touching anything, or bunches up in the middle of \
             your mouth. No part of your tongue makes contact. This is why \
             English 'r' is so hard for learners.",
            "red",
        ),
        'r' => (
            "Trilled R sound",
            "The tongue tip vibrates rapidly against the gum ridge — a \
             'rolled r'. Standard in Spanish ('perro'), Italian, Russian, \
             and Scottish English. The American English 'r' (ɹ) is a \
             different, non-trilled sound.",
            "perro",
        ),
        'ɻ' => (
            "Retroflex R sound",
            "Like the American R but with the tongue tip curled further \
             back. Common in South Asian languages and some American \
             dialects.",
            "red",
        ),
        'j' => (
            "Y glide",
            "Like saying 'ee' but then immediately sliding into another \
             vowel. The 'y' in 'yes', 'you'. Your tongue starts high and \
             forward, then moves to the position of the next vowel. \
             Not a hard consonant — a smooth glide.",
            "yes",
        ),
        'w' => (
            "W glide",
            "Your lips round tightly and your tongue pulls back, then you \
             immediately slide into the next vowel. The 'w' in 'wet', 'way'. \
             Like a very quick 'oo' before the main vowel.",
            "wet",
        ),
        'ʍ' => (
            "Voiceless W sound",
            "Like 'w' but without vocal cord vibration — a breathy 'hw' \
             sound. Traditional pronunciation of 'wh' in 'which', 'where', \
             'whale'. Most modern American and British speakers merge \
             this with regular 'w'.",
            "which",
        ),
        'ʋ' => (
            "Labiodental approximant",
            "Between 'v' and 'w' — lower lip approaches upper teeth but \
             doesn't quite make contact enough for friction. Appears in \
             Dutch and some languages. English speakers hear it as \
             either 'v' or 'w'.",
            "vlag",
        ),

        // ── Affricates (single-char versions) ──
        'ʧ' => (
            "CH sound",
            "Starts like 't' (tongue tip at gum ridge, complete closure) \
             then releases as 'ʃ' (the 'sh' sound). 'ch' in 'church'. \
             Two sounds in one: a stop that turns into a fricative.",
            "church",
        ),
        'ʤ' => (
            "J sound",
            "Like 'ʧ' (the 'ch' sound) but with vocal cord vibration. \
             The 'j' in 'judge'. Starts as 'd', releases as 'ʒ'. \
             Also the 'g' in 'gem', 'giant'.",
            "judge",
        ),

        // ── Stress & length ──
        'ˈ' => (
            "Primary stress mark",
            "This symbol doesn't represent a sound — it marks the syllable \
             that gets the strongest emphasis. In English, stress changes \
             meaning: 'CON-tent' (noun) vs 'con-TENT' (adjective). \
             Always placed just before the stressed syllable.",
            "a-BOUT",
        ),
        'ˌ' => (
            "Secondary stress mark",
            "Marks a syllable with lighter emphasis — weaker than primary \
             stress but stronger than unstressed syllables. Long words \
             often have both: 'pho-ne-TI-cian' has primary stress on \
             'TI' and secondary on 'pho'.",
            "pho-ne-TI-cian",
        ),
        'ː' => (
            "Length mark",
            "The preceding sound is held for longer than usual. Compare \
             the 'i' in 'sit' /sɪt/ (short) with the 'ee' in 'seat' \
             /siːt/ (long). Length can be the only difference between \
             two words in some languages.",
            "seat vs sit",
        ),

        _ => ("IPA symbol", "An IPA transcription symbol found in this corpus.", ""),
    }
}

// ── IPA categorisation ────────────────────────────────────────────────────────

fn phoneme_category(ch: char) -> (&'static str, &'static str, &'static str) {
    match ch {
        'i' | 'ɪ' | 'e' | 'ɛ' | 'æ' | 'a' | 'ɑ' | 'ɔ' | 'o' | 'u' | 'ʊ'
        | 'ə' | 'ʌ' | 'ɜ' | 'ɐ' | 'ɵ' | 'ɤ' | 'ɯ' | 'y' | 'ʏ'
        | 'ø' | 'œ' | 'ɨ' | 'ʉ' | 'ɞ' | 'ɶ' => ("vowel", "#d97706", "Vowels"),

        'p' | 'b' | 't' | 'd' | 'k' | 'ɡ' | 'g' | 'ʔ' | 'ɓ' | 'ɗ' | 'ɠ' => {
            ("stop", "#16a34a", "Plosives")
        }

        'f' | 'v' | 'θ' | 'ð' | 's' | 'z' | 'ʃ' | 'ʒ' | 'h' | 'ɦ' | 'x'
        | 'χ' | 'ɣ' | 'ʁ' | 'ħ' | 'ʕ' | 'ɸ' | 'β' | 'ɬ' | 'ɮ' | 'ʂ'
        | 'ʐ' | 'ɕ' | 'ʑ' => ("fricative", "#2563eb", "Fricatives"),

        'm' | 'n' | 'ŋ' | 'ɲ' | 'ɱ' | 'ɴ' => ("nasal", "#9333ea", "Nasals"),

        'l' | 'r' | 'ɹ' | 'ɻ' | 'j' | 'w' | 'ʍ' | 'ʋ' | 'ɫ' | 'ʎ' | 'ɰ'
        | 'ʟ' => ("approx", "#0891b2", "Approximants"),

        'ʧ' | 'ʤ' | 'ʦ' | 'ʣ' => ("affricate", "#ea580c", "Affricates"),

        'ˈ' | 'ˌ' | 'ː' | 'ˑ' => ("supra", "#a16207", "Stress & Length"),

        _ => ("other", "#6b7280", "Other"),
    }
}

// ── Corpus analysis ───────────────────────────────────────────────────────────

const SOURCE_PREF: &[&str] = &[
    "cmu",
    "misaki_gold",
    "misaki_silver",
    "phonemicchart",
    "wiktionary",
    "wikipron",
];

fn preferred_ipa(ipa_map: &serde_json::Map<String, serde_json::Value>) -> Option<String> {
    for pref in SOURCE_PREF {
        if let Some(v) = ipa_map.get(*pref) {
            if let Some(s) = v.as_str() {
                if !s.is_empty() {
                    return Some(s.to_owned());
                }
            }
        }
        for (k, v) in ipa_map {
            if k.starts_with(pref) {
                if let Some(s) = v.as_str() {
                    if !s.is_empty() {
                        return Some(s.to_owned());
                    }
                }
            }
        }
    }
    ipa_map
        .values()
        .find_map(|v| v.as_str().filter(|s| !s.is_empty()))
        .map(|s| s.to_owned())
}

struct PhonemeInfo {
    ch: char,
    count: usize,
    category: &'static str,
    color: &'static str,
    #[allow(dead_code)]
    cat_label: &'static str,
    size_ratio: f64,
    examples: Vec<(String, String)>, // top 30 (word, ipa)
    name: &'static str,
    desc: &'static str,
    example_word: &'static str,
}

struct Stats {
    total_words: usize,
    sources: Vec<(String, usize)>,
    phonemes: Vec<PhonemeInfo>,
    search_words: Vec<(String, String)>,     // top 10k (word, ipa)
    showcase: Vec<(String, String, usize)>,  // top 120 (word, ipa, rank)
    unique_phoneme_count: usize,
    avg_ipa_len: f64,
}

fn fmt_num(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

fn fmt_k(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.0}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn analyze(raw: &serde_json::Map<String, serde_json::Value>) -> Stats {
    type PhonemeEntry = (usize, Vec<(String, String, f64)>);
    let mut phoneme_data: BTreeMap<char, PhonemeEntry> = BTreeMap::new();
    let mut source_counts: HashMap<String, usize> = HashMap::new();
    let mut all_words: Vec<(String, String, f64)> = Vec::new();
    let mut total_ipa_chars: usize = 0;

    for (word, entry) in raw {
        let obj = match entry.as_object() {
            Some(o) => o,
            None => continue,
        };
        let rarity = obj
            .get("rarity")
            .and_then(|v| v.as_f64())
            .unwrap_or(f64::MAX);
        let ipa_map = match obj.get("ipa").and_then(|v| v.as_object()) {
            Some(m) => m,
            None => continue,
        };

        for k in ipa_map.keys() {
            let bucket = SOURCE_PREF
                .iter()
                .find(|p| k.starts_with(*p))
                .copied()
                .unwrap_or(k.as_str());
            *source_counts.entry(bucket.to_owned()).or_default() += 1;
        }

        let ipa = match preferred_ipa(ipa_map) {
            Some(s) => s,
            None => continue,
        };

        total_ipa_chars += ipa.chars().count();
        all_words.push((word.clone(), ipa.clone(), rarity));

        for ch in ipa.chars() {
            let e = phoneme_data.entry(ch).or_default();
            e.0 += 1;
            if e.1.len() < 40 {
                e.1.push((word.clone(), ipa.clone(), rarity));
            }
        }
    }

    for (_, (_, examples)) in phoneme_data.iter_mut() {
        examples.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
        examples.dedup_by(|a, b| a.0 == b.0);
        examples.truncate(30);
    }

    all_words.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

    let search_words: Vec<(String, String)> = all_words
        .iter()
        .take(10_000)
        .map(|(w, ipa, _)| (w.clone(), ipa.clone()))
        .collect();

    let showcase: Vec<(String, String, usize)> = all_words
        .iter()
        .take(120)
        .enumerate()
        .map(|(i, (w, ipa, _))| (w.clone(), ipa.clone(), i + 1))
        .collect();

    let max_count = phoneme_data.values().map(|(c, _)| *c).max().unwrap_or(1);
    let max_log = (max_count as f64).ln();

    let mut phonemes: Vec<PhonemeInfo> = phoneme_data
        .into_iter()
        .map(|(ch, (count, examples))| {
            let (category, color, cat_label) = phoneme_category(ch);
            let (name, desc, example_word) = phoneme_description(ch);
            let size_ratio = if max_log > 0.0 {
                (count as f64).ln() / max_log
            } else {
                0.0
            };
            PhonemeInfo {
                ch,
                count,
                category,
                color,
                cat_label,
                size_ratio,
                examples: examples.into_iter().map(|(w, ipa, _)| (w, ipa)).collect(),
                name,
                desc,
                example_word,
            }
        })
        .collect();

    phonemes.sort_by_key(|p| std::cmp::Reverse(p.count));

    let unique_phoneme_count = phonemes.len();
    let avg_ipa_len = if !all_words.is_empty() {
        total_ipa_chars as f64 / all_words.len() as f64
    } else {
        0.0
    };

    let mut sources: Vec<(String, usize)> = source_counts.into_iter().collect();
    sources.sort_by_key(|s| std::cmp::Reverse(s.1));

    Stats {
        total_words: all_words.len(),
        sources,
        phonemes,
        search_words,
        showcase,
        unique_phoneme_count,
        avg_ipa_len,
    }
}

// ── Utilities ─────────────────────────────────────────────────────────────────

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn js_str(s: &str) -> String {
    let mut out = String::from('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// ── Static CSS ────────────────────────────────────────────────────────────────

const CSS: &str = r#"
/* ── Reset & tokens ─────────────────────────────────────────────────── */
*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

:root {
  --b950: #1e0810;
  --b900: #330f1c;
  --b800: #4e1828;
  --b700: #6d2237;
  --b600: #8b3049;
  --b500: #a8405b;
  --b400: #c4718a;
  --b300: #dba4b8;
  --b200: #eeccd8;
  --b100: #f7e8ee;
  --cream: #fdf8f2;
  --cream-d: #f0e5d5;

  --vowel:      #d97706;
  --stop:       #16a34a;
  --fricative:  #2563eb;
  --nasal:      #9333ea;
  --approx:     #0891b2;
  --affricate:  #ea580c;
  --supra:      #a16207;
  --other:      #6b7280;

  --radius:    0.75rem;
  --shadow:    0 2px 8px rgba(0,0,0,.09);
  --shadow-lg: 0 8px 28px rgba(0,0,0,.14);
}

html { scroll-behavior: smooth; }

body {
  font-family: 'Inter', system-ui, -apple-system, sans-serif;
  background: var(--cream);
  color: #1c0d12;
  line-height: 1.65;
  font-size: 1rem;
}

/* ── Type helpers ────────────────────────────────────────────────────── */
.serif    { font-family: 'Playfair Display', Georgia, serif; }
.mono     { font-family: 'JetBrains Mono', 'Courier New', monospace; }
.ipa-font { font-family: 'Noto Serif', 'Palatino Linotype', Georgia, serif; }

/* ── Hero ────────────────────────────────────────────────────────────── */
#hero {
  min-height: 60svh;
  background: linear-gradient(160deg, var(--b950) 0%, var(--b900) 45%, var(--b800) 100%);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
  text-align: center;
  padding: 3rem 2rem 4rem;
}

.hero-bg {
  position: absolute; inset: 0;
  pointer-events: none; overflow: hidden; user-select: none;
}
.hero-bg span {
  position: absolute;
  font-family: 'Noto Serif', Georgia, serif;
  color: rgba(255,255,255,.04);
  line-height: 1;
  animation: heroFloat linear infinite;
}
@keyframes heroFloat {
  0%   { transform: translateY(0)   rotate(0deg);  opacity: .03; }
  30%  { transform: translateY(-18px) rotate(4deg); opacity: .07; }
  60%  { transform: translateY(10px)  rotate(-3deg);opacity: .04; }
  100% { transform: translateY(0)   rotate(0deg);  opacity: .03; }
}

.hero-content { position: relative; z-index: 1; max-width: 700px; }

.hero-eyebrow {
  font-family: 'JetBrains Mono', monospace;
  font-size: .78rem;
  letter-spacing: .25em;
  text-transform: uppercase;
  color: var(--b300);
  margin-bottom: 1.25rem;
}
.hero-title {
  font-family: 'Playfair Display', Georgia, serif;
  font-size: clamp(2.8rem, 9vw, 6rem);
  font-weight: 900;
  color: var(--cream);
  line-height: 1.04;
  letter-spacing: -.02em;
  margin-bottom: 1.25rem;
}
.hero-sub {
  color: var(--b200);
  font-size: 1.05rem;
  max-width: 520px;
  margin: 0 auto 1.75rem;
  line-height: 1.7;
}
.hero-cta {
  display: flex; gap: .75rem; flex-wrap: wrap; justify-content: center;
}
.btn {
  display: inline-block;
  padding: .6rem 1.35rem;
  border-radius: 2rem;
  font-size: .875rem;
  font-weight: 600;
  text-decoration: none;
  transition: all .2s;
  cursor: pointer;
  border: none;
}
.btn-primary { background: var(--cream); color: var(--b800); }
.btn-primary:hover { background: #fff; box-shadow: 0 4px 16px rgba(0,0,0,.22); }
.btn-ghost { border: 1px solid var(--b400); color: var(--b200); background: transparent; }
.btn-ghost:hover { background: rgba(255,255,255,.08); }

/* ── Stats band ──────────────────────────────────────────────────────── */
.stats-band { background: var(--b800); padding: 2.5rem 2rem; }
.stats-inner {
  max-width: 960px; margin: 0 auto;
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(170px, 1fr));
  gap: 1.25rem;
}
.stat-card {
  background: rgba(255,255,255,.06);
  border: 1px solid rgba(255,255,255,.10);
  border-radius: var(--radius);
  padding: 1.25rem;
  text-align: center;
}
.stat-num   { display: block; font-family: 'Playfair Display', serif; font-size: 2.2rem; font-weight: 700; color: #fff; line-height: 1; }
.stat-label { display: block; font-size: .7rem; color: var(--b300); text-transform: uppercase; letter-spacing: .12em; margin-top: .35rem; }

/* ── Search hero section ─────────────────────────────────────────────── */
.search-hero {
  background: #fff;
  padding: 4rem 2rem 5rem;
  position: relative;
}
.search-hero-inner { max-width: 1100px; margin: 0 auto; }

.search-headline {
  font-family: 'Playfair Display', Georgia, serif;
  font-size: clamp(2rem, 5vw, 3rem);
  font-weight: 700;
  color: var(--b800);
  margin-bottom: .4rem;
  line-height: 1.15;
}
.search-tagline { color: #6b5a5c; font-size: 1rem; margin-bottom: 2rem; }

.search-wrap { position: relative; max-width: 680px; }

.search-box {
  display: flex;
  align-items: center;
  background: var(--cream);
  border: 2px solid var(--b200);
  border-radius: 3rem;
  padding: .7rem 1.5rem;
  gap: .75rem;
  transition: border-color .2s, box-shadow .2s;
}
.search-box:focus-within {
  border-color: var(--b600);
  box-shadow: 0 0 0 4px color-mix(in srgb, var(--b600) 12%, transparent);
}
.search-icon { font-size: 1.1rem; color: var(--b400); flex-shrink: 0; }

#q {
  flex: 1;
  border: none; background: transparent;
  font-family: inherit; font-size: 1.15rem;
  color: #1c0d12; outline: none; min-width: 0;
}
#q::placeholder { color: var(--b300); }

.search-clear {
  background: none; border: none;
  color: var(--b400); font-size: 1rem;
  cursor: pointer; padding: 0 .25rem;
  line-height: 1; flex-shrink: 0;
  display: none;
}
.search-clear.visible { display: block; }

/* Suggestions dropdown */
.suggestions {
  display: none;
  position: absolute; top: calc(100% + .4rem); left: 0; right: 0;
  background: #fff;
  border: 1.5px solid var(--b200);
  border-radius: var(--radius);
  box-shadow: var(--shadow-lg);
  z-index: 50;
  overflow: hidden;
}
.suggestions.active { display: block; }

.suggestion-item {
  display: grid;
  grid-template-columns: 1fr auto auto;
  align-items: center;
  gap: .75rem;
  padding: .7rem 1.25rem;
  background: none; border: none;
  width: 100%; text-align: left;
  cursor: pointer; transition: background .12s;
}
.suggestion-item:hover { background: var(--cream); }
.sug-word { font-weight: 600; color: var(--b800); }
.sug-ipa  { font-family: 'Noto Serif', serif; color: #555; font-size: .95rem; }
.sug-rank { font-size: .7rem; color: #aaa; }
.suggestion-none { padding: .7rem 1.25rem; color: #999; font-size: .875rem; }

/* Try-these words */
.try-words {
  display: flex; align-items: center; flex-wrap: wrap;
  gap: .4rem; margin-top: 1rem; margin-bottom: 2rem;
}
.try-label { font-size: .8rem; color: #999; text-transform: uppercase; letter-spacing: .1em; margin-right: .2rem; }
.try-word {
  padding: .3rem .75rem;
  background: var(--cream); border: 1px solid var(--cream-d);
  border-radius: 2rem; font-size: .875rem;
  cursor: pointer; color: var(--b700);
  transition: all .15s; font-family: 'Noto Serif', serif;
}
.try-word:hover { background: var(--b100); border-color: var(--b300); }

/* ── Result grid ─────────────────────────────────────────────────────── */
.result-grid {
  display: grid;
  grid-template-columns: 1fr 320px;
  gap: 1.5rem;
  align-items: start;
  margin-top: .5rem;
}
@media (max-width: 900px) {
  .result-grid { grid-template-columns: 1fr; }
}

/* Word result card */
.word-result {
  background: var(--cream);
  border: 1.5px solid var(--cream-d);
  border-radius: var(--radius);
  overflow: hidden;
  transition: opacity .2s;
  min-height: 80px;
}
.word-result.has-result {
  box-shadow: var(--shadow);
}

.result-top {
  padding: 1.25rem 1.5rem 1rem;
  border-bottom: 1px solid var(--cream-d);
  display: flex; flex-wrap: wrap; align-items: baseline; gap: .75rem 1.25rem;
}
.result-word {
  font-family: 'Playfair Display', serif;
  font-size: 2.1rem; font-weight: 700;
  color: var(--b800); line-height: 1;
}
.result-ipa {
  font-size: 1.4rem; color: #555; line-height: 1;
}
.result-rank {
  font-size: .72rem; color: #aaa;
  margin-left: auto;
}

.spelling-insight {
  padding: .6rem 1.5rem;
  font-size: .85rem; color: #666;
  background: color-mix(in srgb, var(--b200) 20%, transparent);
  border-bottom: 1px solid var(--cream-d);
  line-height: 1.5;
}
.si-word { font-weight: 700; color: var(--b700); }

/* Phoneme breakdown */
.phoneme-breakdown {
  display: flex; flex-wrap: wrap;
  gap: .5rem;
  padding: 1.25rem 1.5rem 0;
}

.phoneme-token {
  display: flex; flex-direction: column; align-items: center;
  gap: .25rem;
  padding: .65rem .9rem;
  min-width: 54px;
  background: color-mix(in srgb, var(--tok-color) 7%, white);
  border: 1.5px solid color-mix(in srgb, var(--tok-color) 22%, transparent);
  border-radius: calc(var(--radius) * .85);
  cursor: pointer;
  transition: transform .15s, box-shadow .15s, background .15s, border-color .15s;
  position: relative;
}
.phoneme-token:hover {
  transform: translateY(-4px);
  box-shadow: 0 6px 18px color-mix(in srgb, var(--tok-color) 18%, transparent);
  background: color-mix(in srgb, var(--tok-color) 13%, white);
}
.phoneme-token.active {
  background: color-mix(in srgb, var(--tok-color) 16%, white);
  border-color: var(--tok-color);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--tok-color) 20%, transparent);
  transform: translateY(-4px);
}

.tok-sym {
  display: block;
  font-family: 'Noto Serif', serif;
  font-size: 1.9rem; font-weight: 600; line-height: 1;
  color: var(--tok-color);
}
.tok-label {
  display: block;
  font-size: .58rem; text-transform: uppercase; letter-spacing: .07em;
  color: var(--tok-color); opacity: .75;
  white-space: nowrap;
}

.breakdown-hint {
  padding: .6rem 1.5rem 1rem;
  font-size: .75rem; color: #bbb; text-align: center;
}

.fun-fact {
  margin: 0 1.5rem 1.25rem;
  padding: .6rem .9rem;
  background: color-mix(in srgb, var(--b300) 15%, transparent);
  border-left: 3px solid var(--b400);
  border-radius: 0 .4rem .4rem 0;
  font-size: .82rem; color: #555; line-height: 1.55;
}

/* ── Phoneme panel ───────────────────────────────────────────────────── */
.phoneme-panel {
  background: #fff;
  border: 1.5px solid var(--cream-d);
  border-radius: var(--radius);
  box-shadow: var(--shadow-lg);
  overflow: hidden;
  display: none;
  animation: panelIn .2s ease;
  position: sticky;
  top: 1.5rem;
}
.phoneme-panel.active { display: block; }

@keyframes panelIn {
  from { opacity: 0; transform: translateY(6px); }
  to   { opacity: 1; transform: translateY(0); }
}

.panel-glyph {
  font-family: 'Noto Serif', serif;
  font-size: 4.5rem; font-weight: 700; line-height: 1;
  padding: 1.5rem 2rem;
  text-align: center;
  border-bottom: 1px solid var(--cream-d);
}

.panel-body { padding: 1.25rem 1.5rem; }

.panel-header-row {
  display: flex; align-items: flex-start;
  gap: .6rem; flex-wrap: wrap;
  margin-bottom: .6rem;
}
.panel-name {
  font-family: 'Playfair Display', serif;
  font-size: 1.05rem; font-weight: 700;
  color: var(--b800); flex: 1;
}
.panel-cat-badge {
  display: inline-block;
  padding: .15rem .5rem;
  border-radius: .25rem;
  font-size: .68rem; font-weight: 600;
  text-transform: uppercase; letter-spacing: .07em;
  white-space: nowrap; flex-shrink: 0; margin-top: .1rem;
}
.panel-desc {
  font-size: .88rem; line-height: 1.65; color: #333;
  margin-bottom: .6rem;
}
.panel-cat-desc {
  font-size: .8rem; line-height: 1.55; color: #888;
  font-style: italic;
  margin-bottom: .6rem;
  padding-top: .4rem;
  border-top: 1px solid var(--cream-d);
}
.panel-stat {
  font-size: .75rem; color: #aaa; margin-bottom: .9rem;
}
.panel-words-label {
  font-size: .7rem; text-transform: uppercase;
  letter-spacing: .1em; color: #999; margin-bottom: .45rem;
}
.panel-words {
  display: flex; flex-wrap: wrap; gap: .35rem;
}
.panel-word {
  display: flex; align-items: center; gap: .35rem;
  padding: .22rem .55rem;
  background: var(--cream); border: 1px solid var(--cream-d);
  border-radius: .4rem;
  font-size: .78rem; cursor: pointer;
  transition: background .12s; color: var(--b800);
  font-weight: 600;
}
.panel-word:hover { background: var(--cream-d); }
.panel-word-ipa { font-family: 'Noto Serif', serif; color: #888; font-weight: 400; }

/* ── Phoneme Universe section ────────────────────────────────────────── */
.phoneme-section { background: var(--cream); padding: 5rem 2rem; }
.phoneme-section-inner { max-width: 1320px; margin: 0 auto; }

.section-title {
  font-family: 'Playfair Display', Georgia, serif;
  font-size: clamp(2rem, 5vw, 3rem);
  font-weight: 700; color: var(--b800); margin-bottom: .4rem;
}
.section-sub { color: #6b5a5c; margin-bottom: 2.5rem; font-size: .95rem; }

.phoneme-group { margin-bottom: 2.5rem; }
.phoneme-group-title {
  font-size: .72rem; font-weight: 600;
  text-transform: uppercase; letter-spacing: .18em;
  margin-bottom: .9rem; padding-bottom: .35rem;
  border-bottom: 2px solid currentColor;
  display: inline-block;
}
.phoneme-row {
  display: flex; flex-wrap: wrap; gap: .6rem; align-items: flex-end;
}

.ptile {
  position: relative;
  border-radius: calc(var(--radius) * .8);
  background: color-mix(in srgb, var(--tile-color) 8%, white);
  border: 1.5px solid color-mix(in srgb, var(--tile-color) 22%, transparent);
  box-shadow: var(--shadow);
  cursor: pointer;
  text-align: center;
  transition: transform .18s, box-shadow .18s, background .18s;
  padding: calc(.4rem + var(--sz) * .6rem) calc(.65rem + var(--sz) * .5rem);
  min-width: calc(2.8rem + var(--sz) * 2.5rem);
}
.ptile:hover {
  transform: translateY(-5px);
  box-shadow: var(--shadow-lg);
  background: color-mix(in srgb, var(--tile-color) 14%, white);
  z-index: 20;
}
.ptile.universe-highlight {
  border-color: var(--tile-color);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--tile-color) 25%, transparent), var(--shadow-lg);
}

.ptile-ch {
  display: block;
  font-family: 'Noto Serif', Georgia, serif;
  font-size: calc(1.5rem + var(--sz) * 2.8rem);
  line-height: 1.2;
  color: var(--tile-color); font-weight: 600;
}
.ptile-count {
  display: block; font-size: .62rem; color: #8a7070; margin-top: .12rem;
}

.ptile-popup {
  display: none;
  position: absolute; top: calc(100% + .5rem); left: 50%;
  transform: translateX(-50%);
  background: #fff; border-radius: var(--radius);
  padding: .75rem 1rem;
  box-shadow: var(--shadow-lg); border: 1px solid #e5dde0;
  z-index: 100; min-width: 175px; max-width: 240px;
  text-align: left; pointer-events: none;
}
.ptile:hover .ptile-popup { display: block; }
.popup-title { font-size: .62rem; text-transform: uppercase; letter-spacing: .1em; color: #9c8a8e; margin-bottom: .35rem; }
.popup-ex { font-size: .8rem; padding: .12rem 0; display: flex; gap: .35rem; align-items: baseline; }
.popup-word { color: #1c0d12; font-weight: 600; }
.popup-ipa  { font-family: 'Noto Serif', serif; color: var(--tile-color); }
.popup-tap  { font-size: .65rem; color: #bbb; margin-top: .35rem; }

/* ── Showcase section ────────────────────────────────────────────────── */
.showcase-section { background: var(--cream-d); padding: 5rem 2rem; }
.showcase-section-inner { max-width: 1300px; margin: 0 auto; }

.word-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: .5rem;
}
.wrow {
  display: flex; align-items: center; gap: .75rem;
  padding: .6rem .9rem;
  border-radius: calc(var(--radius) * .75);
  background: #fff; border: 1px solid #eddde3;
  transition: border-color .15s; cursor: pointer;
}
.wrow:hover { border-color: var(--b300); }
.wrow-rank { font-size: .68rem; color: #bba8b0; min-width: 2.2rem; text-align: right; }
.wrow-word { font-weight: 600; color: var(--b800); flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.wrow-ipa  { font-family: 'Noto Serif', serif; font-size: .88rem; color: #555; flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

/* ── Sources section ─────────────────────────────────────────────────── */
.sources-section { background: var(--b900); padding: 5rem 2rem; }
.sources-section .section-title { color: var(--cream); }
.sources-section .section-sub   { color: var(--b300); }

.sources-grid {
  display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 1.25rem;
}
.src-card {
  background: rgba(255,255,255,.05); border: 1px solid rgba(255,255,255,.1);
  border-radius: var(--radius); padding: 1.5rem; transition: background .2s;
}
.src-card:hover { background: rgba(255,255,255,.09); }
.src-tag  { font-family: 'JetBrains Mono', monospace; font-size: .75rem; color: var(--b300); letter-spacing: .07em; margin-bottom: .5rem; }
.src-num  { font-family: 'Playfair Display', serif; font-size: 2rem; font-weight: 700; color: #fff; line-height: 1; margin-bottom: .5rem; }
.src-desc { font-size: .875rem; color: var(--b200); line-height: 1.55; }

/* ── Footer ──────────────────────────────────────────────────────────── */
footer {
  background: var(--b950);
  border-top: 1px solid rgba(255,255,255,.05);
  padding: 2.5rem 2rem; text-align: center;
  color: var(--b400); font-size: .875rem;
  display: flex; flex-direction: column; align-items: center; gap: 1rem;
}
.footer-links { display: flex; gap: 1.5rem; flex-wrap: wrap; justify-content: center; }
footer a { color: var(--b300); text-decoration: none; }
footer a:hover { text-decoration: underline; }
.license-badge {
  display: inline-block; padding: .2rem .6rem;
  border-radius: .25rem; background: rgba(255,255,255,.06);
  font-family: 'JetBrains Mono', monospace; font-size: .72rem; letter-spacing: .05em;
}

/* ── Responsive ──────────────────────────────────────────────────────── */
@media (max-width: 640px) {
  .hero-title  { font-size: 3rem; }
  .result-top  { gap: .5rem; }
  .result-word { font-size: 1.6rem; }
  .result-ipa  { font-size: 1.1rem; }
  .ptile-popup { left: 0; transform: none; }
}
"#;

// ── Rendering ─────────────────────────────────────────────────────────────────

fn hero_bg_chars(stats: &Stats) -> String {
    let chars: Vec<char> = stats
        .phonemes
        .iter()
        .filter(|p| p.category != "supra" && p.category != "other")
        .take(28)
        .map(|p| p.ch)
        .collect();

    let mut html = String::from("<div class=\"hero-bg\" aria-hidden=\"true\">\n");
    for (i, ch) in chars.iter().enumerate() {
        let x = (i * 13 + 7) % 92;
        let y = (i * 17 + 5) % 82;
        let delay = (i * 3) % 22;
        let dur = 16 + (i % 7) * 4;
        let size = 7 + (i % 5) * 3;
        html.push_str(&format!(
            "  <span style=\"left:{x}%;top:{y}%;font-size:{size}rem;animation-duration:{dur}s;animation-delay:-{delay}s\">{ch}</span>\n"
        ));
    }
    html.push_str("</div>\n");
    html
}

fn render_hero(stats: &Stats) -> String {
    let bg = hero_bg_chars(stats);
    let total = fmt_num(stats.total_words);
    format!(
        r##"<header id="hero">
{bg}<div class="hero-content">
  <p class="hero-eyebrow">openepd · open english pronouncing dictionary</p>
  <h1 class="hero-title serif">The English<br>IPA Corpus</h1>
  <p class="hero-sub">
    {total} words fused from four open lexicons — now an interactive phonetics lesson.
    Type any word below to hear its phonemes.
  </p>
  <div class="hero-cta">
    <a href="#search" class="btn btn-primary">Explore a word</a>
    <a href="#phonemes" class="btn btn-ghost">Browse phonemes</a>
    <a href="https://github.com/JackDanger/open-english-pronouncing-dictionary" class="btn btn-ghost" target="_blank" rel="noopener">GitHub ↗</a>
  </div>
</div>
</header>
"##
    )
}

fn render_stats_band(stats: &Stats) -> String {
    let total = fmt_num(stats.total_words);
    let phonemes = stats.unique_phoneme_count;
    let avg = format!("{:.1}", stats.avg_ipa_len);
    let sources = stats.sources.len();
    format!(
        r#"<section id="stats" class="stats-band">
  <div class="stats-inner">
    <div class="stat-card">
      <span class="stat-num serif">{total}</span>
      <span class="stat-label">Words indexed</span>
    </div>
    <div class="stat-card">
      <span class="stat-num serif">{phonemes}</span>
      <span class="stat-label">IPA characters</span>
    </div>
    <div class="stat-card">
      <span class="stat-num serif">{sources}</span>
      <span class="stat-label">Source lexicons</span>
    </div>
    <div class="stat-card">
      <span class="stat-num serif">{avg}</span>
      <span class="stat-label">Avg phonemes/word</span>
    </div>
  </div>
</section>
"#
    )
}

fn render_search_section() -> String {
    let try_words = [
        "rhythm", "through", "thought", "colonel", "queue", "pneumonia",
        "bouquet", "receipt",
    ]
    .iter()
    .map(|w| format!("<button class=\"try-word\" onclick=\"searchAndShow({})\">{w}</button>", js_str(w)))
    .collect::<Vec<_>>()
    .join("\n      ");

    format!(
        r##"<section id="search" class="search-hero">
  <div class="search-hero-inner">
    <h2 class="search-headline serif">Type any English word.</h2>
    <p class="search-tagline">See exactly what sounds make it up — then click each sound to understand it.</p>

    <div class="search-wrap">
      <div class="search-box">
        <span class="search-icon" aria-hidden="true">&#9654;</span>
        <input id="q" type="search" autocomplete="off" spellcheck="false"
               placeholder="e.g. rhythm, colonel, through…"
               aria-label="Search for a word"
               oninput="handleInput(this)">
        <button class="search-clear" id="search-clear"
                onclick="clearAll()" aria-label="Clear search">&#x2715;</button>
      </div>
      <div id="suggestions" class="suggestions" role="listbox" aria-label="Word suggestions"></div>
    </div>

    <div class="try-words" aria-label="Example words to try">
      <span class="try-label">Try:</span>
      {try_words}
    </div>

    <div class="result-grid" id="result-grid">
      <div id="word-result" class="word-result" role="region" aria-label="Word analysis" aria-live="polite"></div>
      <div id="phoneme-panel" class="phoneme-panel" role="complementary" aria-label="Phoneme detail" aria-live="polite"></div>
    </div>
  </div>
</section>
"##
    )
}

fn render_phonemes_section(stats: &Stats) -> String {
    let order: &[(&str, &str)] = &[
        ("vowel", "Vowels"),
        ("stop", "Plosives"),
        ("fricative", "Fricatives"),
        ("nasal", "Nasals"),
        ("approx", "Approximants"),
        ("affricate", "Affricates"),
        ("supra", "Stress & Length"),
        ("other", "Other symbols"),
    ];

    let mut groups: BTreeMap<&str, Vec<&PhonemeInfo>> = BTreeMap::new();
    for p in &stats.phonemes {
        groups.entry(p.category).or_default().push(p);
    }

    let mut groups_html = String::new();
    for (slug, label) in order {
        let Some(items) = groups.get(slug) else {
            continue;
        };
        if items.is_empty() {
            continue;
        }
        let color = items[0].color;
        let mut tiles = String::new();
        for p in items {
            let ch = p.ch;
            let count_fmt = fmt_k(p.count);
            let sz = format!("{:.3}", p.size_ratio);
            let color = p.color;
            let ch_esc = esc(&ch.to_string());

            let mut examples_html = String::new();
            for (word, ipa) in &p.examples {
                examples_html.push_str(&format!(
                    "<div class=\"popup-ex\"><span class=\"popup-word\">{}</span><span class=\"popup-ipa\">{}</span></div>\n",
                    esc(word), esc(ipa)
                ));
            }

            let ch_js = js_str(&ch.to_string());
            tiles.push_str(&format!(
                "<div class=\"ptile\" data-ch=\"{ch_esc}\" style=\"--tile-color:{color};--sz:{sz};\" onclick=\"selectPhonemeFromUniverse({ch_js})\" aria-label=\"{ch_esc}: {count_fmt}\">\n  <span class=\"ptile-ch ipa-font\">{ch_esc}</span>\n  <span class=\"ptile-count\">{count_fmt}</span>\n  <div class=\"ptile-popup\"><div class=\"popup-title\">/{ch_esc}/</div>{examples_html}<div class=\"popup-tap\">Click to explore</div></div>\n</div>\n"
            ));
        }
        groups_html.push_str(&format!(
            "<div class=\"phoneme-group\">\n  <div class=\"phoneme-group-title\" style=\"color:{color}\">{label}</div>\n  <div class=\"phoneme-row\">\n{tiles}  </div>\n</div>\n"
        ));
    }

    format!(
        r#"<section id="phonemes" class="phoneme-section">
  <div class="phoneme-section-inner">
    <h2 class="section-title">Phoneme Universe</h2>
    <p class="section-sub">Every IPA sound in the corpus, sized by word-coverage. Hover for examples, click to explore in the search above.</p>
    {groups_html}
  </div>
</section>
"#
    )
}

fn render_showcase_section(stats: &Stats) -> String {
    let mut rows = String::new();
    for (word, ipa, rank) in &stats.showcase {
        let w_js = js_str(word);
        rows.push_str(&format!(
            "<div class=\"wrow\" onclick=\"searchAndShow({w_js})\" role=\"button\" tabindex=\"0\">\
             <span class=\"wrow-rank\">#{rank}</span>\
             <span class=\"wrow-word\">{}</span>\
             <span class=\"wrow-ipa ipa-font\">{}</span></div>\n",
            esc(word), esc(ipa)
        ));
    }
    format!(
        r#"<section id="top-words" class="showcase-section">
  <div class="showcase-section-inner">
    <h2 class="section-title">100 Most Common Words</h2>
    <p class="section-sub">Click any row to analyse that word.</p>
    <div class="word-grid">
{rows}    </div>
  </div>
</section>
"#
    )
}

fn source_description(tag: &str) -> &'static str {
    match tag {
        "cmu" => "Carnegie Mellon Pronouncing Dictionary — the gold standard for American English broad transcription. ARPABET-derived IPA, preferred for function words in their weak contextual forms.",
        "misaki_gold" => "Vetted near-IPA from the Kokoro TTS project (gold tier). Narrow vowel distinctions; tends to use citation/strong forms for function words.",
        "misaki_silver" => "Misaki silver tier — broader coverage than gold, slightly noisier. Excellent gap-fill for words absent from CMU.",
        "wikipron" => "WikiPron broad scrape from Wiktionary. Widest open coverage; captures regional variants.",
        "phonemicchart" => "Phonemic Chart corpus — legacy source from the pre-OpenEPD lexicon.",
        "wiktionary" => "Wiktionary IPA annotations — legacy source superseded by the WikiPron scrape.",
        _ => "Open-source IPA transcription source.",
    }
}

fn render_sources_section(stats: &Stats) -> String {
    let mut cards = String::new();
    for (tag, count) in &stats.sources {
        let desc = source_description(tag);
        let n = fmt_num(*count);
        cards.push_str(&format!(
            "<div class=\"src-card\">\n  <div class=\"src-tag mono\">{}</div>\n  <div class=\"src-num serif\">{n}</div>\n  <p class=\"src-desc\">{desc}</p>\n</div>\n",
            esc(tag)
        ));
    }
    format!(
        r#"<section id="sources" class="sources-section">
  <div style="max-width:1300px;margin:0 auto;padding:0">
    <h2 class="section-title">Source Lexicons</h2>
    <p class="section-sub">Each word carries provenance — which source contributed its transcription(s).</p>
    <div class="sources-grid">
{cards}    </div>
  </div>
</section>
"#
    )
}

fn render_footer() -> String {
    r#"<footer>
  <div class="footer-links">
    <a href="https://github.com/JackDanger/open-english-pronouncing-dictionary" target="_blank" rel="noopener">GitHub</a>
    <a href="https://crates.io/crates/open-english-pronouncing-dictionary" target="_blank" rel="noopener">crates.io</a>
    <a href="https://github.com/JackDanger/phonetics" target="_blank" rel="noopener">phonetics-rs</a>
  </div>
  <div>
    <span class="license-badge">CC BY-SA 4.0</span>
    &nbsp; Sources: Misaki · CMU · WikiPron
  </div>
  <div>Generated by <span class="mono">site-gen</span> (OpenEPD)</div>
</footer>
"#
    .to_owned()
}

// ── JS data: PHONEME_INFO + PHONEME_WORDS ─────────────────────────────────────

fn render_phoneme_data_js(stats: &Stats) -> String {
    let mut info = String::from("{\n");
    for p in &stats.phonemes {
        let ch_js = js_str(&p.ch.to_string());
        let name_js = js_str(p.name);
        let desc_js = js_str(p.desc);
        let ex_js = js_str(p.example_word);
        info.push_str(&format!(
            "  {ch_js}:{{name:{name_js},desc:{desc_js},example:{ex_js},category:{},color:{},wordCount:{}}},\n",
            js_str(p.category),
            js_str(p.color),
            p.count,
        ));
    }
    info.push('}');

    let mut words_map = String::from("{\n");
    for p in &stats.phonemes {
        let ch_js = js_str(&p.ch.to_string());
        let arr: String = p
            .examples
            .iter()
            .map(|(w, ipa)| format!("[{},{}]", js_str(w), js_str(ipa)))
            .collect::<Vec<_>>()
            .join(",");
        words_map.push_str(&format!("  {ch_js}:[{arr}],\n"));
    }
    words_map.push('}');

    format!("const PHONEME_INFO={info};\nconst PHONEME_WORDS={words_map};\n")
}

// ── Main JS: search + educational interactions ────────────────────────────────

fn render_scripts(stats: &Stats) -> String {
    // Compact search word array
    let mut word_json = String::from("[");
    for (i, (word, ipa)) in stats.search_words.iter().enumerate() {
        if i > 0 {
            word_json.push(',');
        }
        word_json.push('[');
        word_json.push_str(&js_str(word));
        word_json.push(',');
        word_json.push_str(&js_str(ipa));
        word_json.push(']');
    }
    word_json.push(']');

    let phoneme_data = render_phoneme_data_js(stats);

    format!(
        r#"<script>
/* ── Corpus data ──────────────────────────────────────────────── */
const WORDS={word_json};

{phoneme_data}

/* Category plain-English descriptions */
const CAT_DESC={{
  vowel:      "Vowels are made with an open vocal tract. The height and position of your tongue, and the shape of your lips, are the only things that change the sound.",
  stop:       "Stops (plosives) block all airflow completely — then release it in a tiny burst. Like a valve snapping open.",
  fricative:  "Fricatives squeeze air through a narrow gap, creating turbulence: hissing, buzzing, or hushing.",
  nasal:      "Nasals route air through the nose while the mouth is blocked. Humming produces a nasal sound.",
  approx:     "Approximants narrow the vocal tract without creating enough friction to produce turbulence. They glide between vowel-like and consonant-like.",
  affricate:  "Affricates start as a complete stop, then release as a fricative. Two sounds fused into one.",
  supra:      "Suprasegmentals mark properties of syllables rather than sounds: stress, length, and tone.",
  other:      "A transcription symbol used in phonetic notation."
}};

/* ── Helpers ───────────────────────────────────────────────────── */
function fmtK(n) {{
  return n>=1e6 ? (n/1e6).toFixed(1)+'M' : n>=1e3 ? Math.round(n/1e3)+'k' : String(n);
}}
function escHTML(s) {{
  return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
}}
function catShort(cat) {{
  return {{vowel:'vowel',stop:'stop',fricative:'fric.',nasal:'nasal',approx:'approx.',affricate:'aff.',supra:'',other:''}}[cat]||cat;
}}

/* ── State ─────────────────────────────────────────────────────── */
const qEl        = document.getElementById('q');
const clearBtn   = document.getElementById('search-clear');
const suggestEl  = document.getElementById('suggestions');
const resultEl   = document.getElementById('word-result');
const panelEl    = document.getElementById('phoneme-panel');
let debounce     = null;

/* ── Search ────────────────────────────────────────────────────── */
function handleInput(inp) {{
  const v = inp.value;
  clearBtn.classList.toggle('visible', v.length > 0);
  clearTimeout(debounce);
  const q = v.trim().toLowerCase();
  if (!q) {{ hideSuggestions(); return; }}

  /* exact match → show result immediately */
  const ex = WORDS.findIndex(function(w){{ return w[0]===q; }});
  if (ex>=0) {{ hideSuggestions(); showWordResult(q, WORDS[ex][1], ex); return; }}

  /* debounced suggestions */
  debounce = setTimeout(function(){{ showSuggestions(q); }}, 100);
}}

function showSuggestions(q) {{
  const hits=[];
  for(let i=0;i<WORDS.length&&hits.length<8;i++) {{
    if(WORDS[i][0].startsWith(q)) hits.push(i);
  }}
  if(!hits.length) {{
    suggestEl.innerHTML='<div class="suggestion-none">No match in top 10,000 words</div>';
    suggestEl.classList.add('active');
    return;
  }}
  suggestEl.innerHTML=hits.map(function(i){{
    return '<button class="suggestion-item" onclick="searchAndShow('+JSON.stringify(WORDS[i][0])+')">'
      +'<span class="sug-word">'+escHTML(WORDS[i][0])+'</span>'
      +'<span class="sug-ipa ipa-font">'+escHTML(WORDS[i][1])+'</span>'
      +'<span class="sug-rank">#'+(i+1)+'</span>'
      +'</button>';
  }}).join('');
  suggestEl.classList.add('active');
}}

function hideSuggestions() {{
  suggestEl.innerHTML=''; suggestEl.classList.remove('active');
}}

function clearAll() {{
  qEl.value=''; clearBtn.classList.remove('visible');
  hideSuggestions();
  resultEl.innerHTML=''; resultEl.classList.remove('has-result');
  panelEl.innerHTML=''; panelEl.classList.remove('active');
  document.querySelectorAll('.phoneme-token.active').forEach(function(el){{el.classList.remove('active');}});
  document.querySelectorAll('.ptile.universe-highlight').forEach(function(el){{el.classList.remove('universe-highlight');}});
}}

function searchAndShow(word) {{
  qEl.value=word; clearBtn.classList.add('visible');
  hideSuggestions();
  const q=word.toLowerCase();
  const i=WORDS.findIndex(function(w){{return w[0]===q;}});
  if(i>=0) {{
    showWordResult(word, WORDS[i][1], i);
  }} else {{
    /* word outside top 10k - still show IPA from PHONEME_WORDS if possible */
    resultEl.innerHTML='<div style="padding:1.5rem;color:#999;font-size:.9rem">&#34;'+escHTML(word)+'&#34; was not found in the top 10,000 most common words.</div>';
    resultEl.classList.add('has-result');
  }}
}}

/* ── Word result display ───────────────────────────────────────── */
function showWordResult(word, ipa, rank) {{
  panelEl.innerHTML=''; panelEl.classList.remove('active');
  document.querySelectorAll('.phoneme-token.active').forEach(function(el){{el.classList.remove('active');}});

  const chars = Array.from(ipa); /* proper Unicode split */
  const phonemeChars = chars.filter(function(c){{return c!=='ˈ'&&c!=='ˌ'&&c!=='ː';}});
  const lc = word.length, pc = phonemeChars.length;

  /* Phoneme breakdown tokens */
  const breakdown = chars.map(function(ch){{
    const info = PHONEME_INFO[ch] || {{name:'/'+ch+'/',category:'other',color:'#6b7280',wordCount:0}};
    const label = catShort(info.category);
    return '<button class="phoneme-token"'
      +' style="--tok-color:'+info.color+';"'
      +' aria-label="'+escHTML(info.name)+'"'
      +' title="'+escHTML(info.name)+'"'
      +' onclick="selectPhoneme(this,'+JSON.stringify(ch)+')">'
      +'<span class="tok-sym">'+escHTML(ch)+'</span>'
      +(label?'<span class="tok-label">'+label+'</span>':'')
      +'</button>';
  }}).join('');

  /* Spelling insight */
  let insight='';
  if(lc!==pc) {{
    const diff=lc-pc;
    const msg = diff>0
      ? lc+' letter'+(lc!==1?'s':'') +' → '+ pc+' sound'+(pc!==1?'s':'')
          +' &nbsp;·&nbsp; '+Math.abs(diff)+' letter'+(Math.abs(diff)!==1?'s':'')+' produce no extra sounds'
      : lc+' letters → '+pc+' sounds';
    insight='<div class="spelling-insight"><span class="si-word">'+escHTML(word)+'</span>: '+msg+'</div>';
  }}

  /* Fun fact */
  const fact = getFact(chars, word, ipa);

  resultEl.innerHTML =
    '<div class="result-top">'
    +'<span class="result-word serif">'+escHTML(word)+'</span>'
    +'<span class="result-ipa ipa-font">'+escHTML(ipa)+'</span>'
    +'<span class="result-rank">#'+(rank+1)+' most common</span>'
    +'</div>'
    +insight
    +'<div class="phoneme-breakdown" role="group" aria-label="Click each sound to explore">'+breakdown+'</div>'
    +'<p class="breakdown-hint">Click any sound above to learn how to make it</p>'
    +(fact?'<div class="fun-fact">'+fact+'</div>':'');

  resultEl.classList.add('has-result');

  /* Scroll result into view on mobile */
  if(window.innerWidth<768) {{
    setTimeout(function(){{resultEl.scrollIntoView({{behavior:'smooth',block:'nearest'}});}},50);
  }}
}}

/* ── Phoneme panel ─────────────────────────────────────────────── */
function selectPhoneme(btn, ch) {{
  document.querySelectorAll('.phoneme-token.active').forEach(function(el){{el.classList.remove('active');}});
  if(btn) btn.classList.add('active');
  renderPhonemePanel(ch);
}}

function selectPhonemeFromUniverse(ch) {{
  /* Jump to search section then reveal panel */
  document.getElementById('search').scrollIntoView({{behavior:'smooth',block:'start'}});
  setTimeout(function(){{renderPhonemePanel(ch);}}, 350);
}}

function renderPhonemePanel(ch) {{
  const info = PHONEME_INFO[ch] || {{
    name:'/'+ch+'/', desc:'An IPA symbol found in this corpus.',
    category:'other', color:'#6b7280', wordCount:0
  }};
  const words = PHONEME_WORDS[ch] || [];
  const catDesc = CAT_DESC[info.category] || '';

  panelEl.innerHTML =
    '<div class="panel-glyph ipa-font" style="background:'+info.color+'14;color:'+info.color+';">'+escHTML(ch)+'</div>'
    +'<div class="panel-body">'
    +'<div class="panel-header-row">'
    +'<span class="panel-name">'+escHTML(info.name)+'</span>'
    +'<span class="panel-cat-badge" style="background:'+info.color+'1a;color:'+info.color+';">'+escHTML(info.category)+'</span>'
    +'</div>'
    +'<p class="panel-desc">'+escHTML(info.desc)+'</p>'
    +(catDesc?'<p class="panel-cat-desc">'+escHTML(catDesc)+'</p>':'')
    +'<div class="panel-stat">'+fmtK(info.wordCount)+' words in this corpus use /'+escHTML(ch)+'/</div>'
    +(words.length?
      '<div class="panel-words-label">Words with /'+escHTML(ch)+'/</div>'
      +'<div class="panel-words">'
      +words.slice(0,20).map(function(w){{
        return '<button class="panel-word" onclick="searchAndShow('+JSON.stringify(w[0])+')">'
          +escHTML(w[0])
          +'<span class="panel-word-ipa ipa-font">'+escHTML(w[1])+'</span>'
          +'</button>';
      }}).join('')
      +'</div>'
    :'')
    +'</div>';

  panelEl.classList.add('active');

  /* Highlight tile in the universe section */
  document.querySelectorAll('.ptile.universe-highlight').forEach(function(el){{el.classList.remove('universe-highlight');}});
  const tile = document.querySelector('.ptile[data-ch="'+CSS.escape(ch)+'"]');
  if(tile) tile.classList.add('universe-highlight');
}}

/* ── Fun facts ─────────────────────────────────────────────────── */
function getFact(chars, word, ipa) {{
  const plain = chars.filter(function(c){{return c!=='ˈ'&&c!=='ˌ'&&c!=='ː';}});
  const uniq  = [...new Set(plain)];

  /* Schwa — most common English vowel */
  const schwaCount = chars.filter(function(c){{return c==='ə';}}).length;
  if(schwaCount>1) return 'The schwa /ə/ — the most common vowel in English — appears '+schwaCount+' times in this word. It\'s the sound of unstressed syllables in hundreds of common words.';
  if(schwaCount===1&&word.length>4) return 'The schwa /ə/ hides in this word. It\'s the most frequent vowel in English, appearing in every unstressed syllable — yet it has no unique letter, borrowed instead from whatever vowel the spelling uses.';

  /* Both TH sounds */
  if(chars.includes('θ')&&chars.includes('ð')) return 'This word contains both English "th" sounds: /θ/ (voiceless, as in "thin") and /ð/ (voiced, as in "this"). Most of the world\'s languages have neither.';

  /* Glottal stop */
  if(chars.includes('ʔ')) return 'This word contains a glottal stop /ʔ/ — the catch in the throat you make in "uh-oh". Many speakers insert it unconsciously before stressed vowels.';

  /* NG can\'t start words */
  if(chars.includes('ŋ')&&!word.startsWith('ng')) return 'The /ŋ/ sound ("ng") cannot begin a word in English — only appear mid-word or at the end. It\'s a positional constraint called a phonotactic rule.';

  /* Many vowels vs few */
  const vowelSet = 'iɪeɛæaɑɔoʊuəʌɜ';
  const vowels = plain.filter(function(c){{return vowelSet.includes(c);}});
  if(vowels.length===0) return 'Remarkably, "'+word+'" has no conventional vowel letters — yet English speakers produce what linguists call "syllabic consonants" that carry the syllable the way a vowel normally would.';

  /* Stress-only distinction */
  if(chars.includes('ˈ')) {{
    const stressed = ipa.split('ˈ');
    if(stressed.length>1) return 'The stress mark ˈ shows where emphasis falls. English uses stress to distinguish words: "REcord" (noun) vs "reCORD" (verb) have identical sounds — only the stress differs.';
  }}

  /* Long vowel */
  if(chars.includes('ː')) return 'The length mark ː means the preceding sound is held longer. English vowel length is contrastive: "ship" /ʃɪp/ vs "sheep" /ʃiːp/ — same sounds, different duration.';

  return null;
}}

/* ── Keyboard navigation ───────────────────────────────────────── */
qEl.addEventListener('keydown', function(e) {{
  if(e.key==='Escape') clearAll();
  if(e.key==='Enter') {{
    const q=qEl.value.trim().toLowerCase();
    const i=WORDS.findIndex(function(w){{return w[0]===q;}});
    if(i>=0){{ hideSuggestions(); showWordResult(q,WORDS[i][1],i); }}
  }}
}});

/* Make showcase rows keyboard-activatable */
document.querySelectorAll('.wrow[role=button]').forEach(function(el) {{
  el.addEventListener('keydown', function(e) {{
    if(e.key==='Enter'||e.key===' ') {{ e.preventDefault(); el.click(); }}
  }});
}});

/* Close suggestions on outside click */
document.addEventListener('click', function(e) {{
  if(!e.target.closest('.search-wrap')) hideSuggestions();
}});

/* Auto-populate with an interesting word on first load */
(function() {{
  var featured = ['rhythm','through','colonel','pneumonia','bouquet','queue'];
  var w = featured[Math.floor(Math.random()*featured.length)];
  var i = WORDS.findIndex(function(x){{return x[0]===w;}});
  if(i>=0) {{ qEl.value=w; clearBtn.classList.add('visible'); showWordResult(w,WORDS[i][1],i); }}
}})();
</script>
"#,
        word_json = word_json,
        phoneme_data = phoneme_data,
    )
}

// ── HTML assembly ──────────────────────────────────────────────────────────────

fn render_html(stats: &Stats) -> String {
    let mut html = String::new();

    html.push_str(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>The English IPA Corpus — OpenEPD</title>
<meta name="description" content="An interactive phonetics explorer: type any English word to see its IPA breakdown, then click each sound to learn how it is made.">
<meta property="og:title" content="The English IPA Corpus — OpenEPD">
<meta property="og:description" content="281k words · explore English phonetics interactively">
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Playfair+Display:ital,wght@0,400;0,700;0,900;1,400&family=Inter:wght@400;500;600&family=JetBrains+Mono:wght@400;500&family=Noto+Serif:wght@400;600&display=swap" rel="stylesheet">
<style>
"#);
    html.push_str(CSS);
    html.push_str("</style>\n</head>\n<body>\n\n");

    // Section order: hero → search (prominent) → stats → phoneme universe → top words → sources → footer
    html.push_str(&render_hero(stats));
    html.push_str(&render_search_section());
    html.push_str(&render_stats_band(stats));
    html.push_str(&render_phonemes_section(stats));
    html.push_str(&render_showcase_section(stats));
    html.push_str(&render_sources_section(stats));
    html.push_str(&render_footer());
    html.push('\n');
    html.push_str(&render_scripts(stats));

    html.push_str("</body>\n</html>\n");
    html
}

// ── Main ───────────────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: site-gen <corpus.json> <output-dir>");
        eprintln!();
        eprintln!("  site-gen ./data/openepd.json ./_site");
        std::process::exit(1);
    }

    let corpus_path = &args[1];
    let output_dir = &args[2];

    eprint!("Reading {corpus_path} … ");
    let json = fs::read_to_string(corpus_path)
        .unwrap_or_else(|e| panic!("Cannot read {corpus_path}: {e}"));
    eprintln!("{:.1} MB", json.len() as f64 / 1_048_576.0);

    eprint!("Parsing JSON … ");
    let root: serde_json::Value =
        serde_json::from_str(&json).expect("corpus is not valid JSON");
    let raw = root
        .as_object()
        .expect("corpus root must be a JSON object");
    eprintln!("{} entries", fmt_num(raw.len()));

    eprint!("Analysing corpus … ");
    let stats = analyze(raw);
    eprintln!(
        "{} words · {} phonemes · {:.1} avg IPA len",
        fmt_num(stats.total_words),
        stats.unique_phoneme_count,
        stats.avg_ipa_len
    );

    eprint!("Rendering HTML … ");
    let html = render_html(&stats);
    eprintln!("{:.0} KB", html.len() as f64 / 1_024.0);

    fs::create_dir_all(output_dir)
        .unwrap_or_else(|e| panic!("Cannot create {output_dir}: {e}"));

    let index_path = format!("{output_dir}/index.html");
    fs::write(&index_path, &html)
        .unwrap_or_else(|e| panic!("Cannot write {index_path}: {e}"));

    fs::write(format!("{output_dir}/.nojekyll"), b"" as &[u8])
        .unwrap_or_else(|e| panic!("Cannot write .nojekyll: {e}"));

    eprintln!("Done → {index_path}");
}
