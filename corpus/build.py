#!/usr/bin/env python3
"""
Build OpenEPD — the open English pronouncing dictionary.

Fuses four open sources into a single per-word JSON dictionary with
canonical IPA, optional variant transcriptions, frequency-derived
rarity rank, and source provenance.

Sources (preference order, highest first):

  misaki_gold    — Misaki US gold dictionary (Apache 2.0). ~90k vetted
                   entries in a near-IPA convention used by Kokoro TTS.
  cmu            — CMUdict 0.7b (BSD). ~135k entries in ARPABET; we
                   convert to IPA. Wins on coverage of proper nouns.
  misaki_silver  — Misaki US silver dictionary (Apache 2.0). ~93k
                   additional entries, less vetted than gold.
  wikipron       — WikiPron eng_latn_us_broad scrape (CC-BY-SA).
                   ~101k entries from Wiktionary, captures pronunciation
                   variants.

Output license is CC-BY-SA 4.0 because WikiPron inherits Wiktionary's
copyleft. To produce a permissive (Apache 2.0) build, omit WikiPron.

Run from the repo root:

    corpus/venv/bin/python corpus/build.py

Writes data/openepd.json.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from collections import defaultdict
from typing import Iterable

import wordfreq

ROOT = Path(__file__).resolve().parent
SRC = ROOT / "sources"
OUT = ROOT.parent / "data" / "openepd.json"


# ---------------------------------------------------------------------------
# Source 1: Misaki gold/silver — near-IPA → canonical IPA
# ---------------------------------------------------------------------------
#
# Misaki uses some custom symbols for ML tokenization efficiency. Expand
# them back to standard IPA. See misaki/EN_PHONES.md for the full table.

MISAKI_TO_IPA = [
    # Order matters: longer keys first so substring substitutions don't
    # collide. Diphthong letters expand into two-char IPA sequences.
    ("ʤ", "dʒ"),   # affricate (compound) → component pair
    ("ʧ", "tʃ"),
    ("A", "eɪ"),   # custom diphthong letters (Misaki-only)
    ("I", "aɪ"),
    ("W", "aʊ"),
    ("Y", "ɔɪ"),
    ("O", "oʊ"),   # US-only diphthong
    ("ᵊ", "ə"),    # muted schwa → schwa
    ("ᵻ", "ɪ"),    # near-schwa "i" → ɪ (broad approximation)
]


def misaki_to_ipa(ps: str) -> str:
    """Expand Misaki's near-IPA tokens into canonical IPA."""
    out = ps
    for old, new in MISAKI_TO_IPA:
        out = out.replace(old, new)
    return out


# ---------------------------------------------------------------------------
# Source 2: CMUdict 0.7b — ARPABET → IPA
# ---------------------------------------------------------------------------
#
# CMUdict uses ARPABET with optional stress digits on vowels:
#
#   HELLO  HH AH0 L OW1
#
# Each phoneme is space-separated. Stress digits: 0 = unstressed,
# 1 = primary, 2 = secondary. We emit /ˈ/ before the syllable containing
# the primary-stressed vowel and /ˌ/ before secondary. Reduced vowels
# (AH0, ER0) get the central counterparts /ə/, /ɚ/ as is conventional.

ARPABET_VOWELS = {
    "AA": "ɑ",
    "AE": "æ",
    "AH": "ʌ",    # stressed; AH0 handled separately as /ə/
    "AO": "ɔ",
    "AW": "aʊ",
    "AY": "aɪ",
    "EH": "ɛ",
    "ER": "ɝ",    # stressed; ER0 → /ɚ/
    "EY": "eɪ",
    "IH": "ɪ",
    "IY": "i",
    "OW": "oʊ",
    "OY": "ɔɪ",
    "UH": "ʊ",
    "UW": "u",
}

ARPABET_CONSONANTS = {
    "B":  "b",
    "CH": "tʃ",
    "D":  "d",
    "DH": "ð",
    "F":  "f",
    "G":  "ɡ",     # U+0261 (IPA hard g), not ASCII g
    "HH": "h",
    "JH": "dʒ",
    "K":  "k",
    "L":  "l",
    "M":  "m",
    "N":  "n",
    "NG": "ŋ",
    "P":  "p",
    "R":  "ɹ",
    "S":  "s",
    "SH": "ʃ",
    "T":  "t",
    "TH": "θ",
    "V":  "v",
    "W":  "w",
    "Y":  "j",
    "Z":  "z",
    "ZH": "ʒ",
}

ARPABET_STRESS_RE = re.compile(r"^([A-Z]+)(\d?)$")


def arpabet_to_ipa(arpa_tokens: list[str]) -> str:
    """
    Convert a list of ARPABET tokens (with optional stress digits) into
    a stress-marked IPA string.

    Strategy: insert /ˈ/ once, immediately before the primary-stressed
    vowel; insert /ˌ/ before each secondary-stressed vowel.
    """
    out: list[str] = []
    for tok in arpa_tokens:
        m = ARPABET_STRESS_RE.match(tok)
        if not m:
            # Unknown token — drop it; cmudict is hand-curated so this
            # should never fire on real entries.
            continue
        phone, stress = m.group(1), m.group(2)
        if phone in ARPABET_VOWELS:
            # Reduced vowel handling: AH0 → /ə/, ER0 → /ɚ/. Everything
            # else uses its full quality regardless of stress (English
            # speakers don't reduce other vowels in CMU's convention).
            if phone == "AH" and stress == "0":
                ipa = "ə"
            elif phone == "ER" and stress == "0":
                ipa = "ɚ"
            else:
                ipa = ARPABET_VOWELS[phone]
            if stress == "1":
                out.append("ˈ")
            elif stress == "2":
                out.append("ˌ")
            out.append(ipa)
        elif phone in ARPABET_CONSONANTS:
            out.append(ARPABET_CONSONANTS[phone])
        # else: drop unknown phone (shouldn't happen for cmudict 0.7b)
    return "".join(out)


# ---------------------------------------------------------------------------
# Source loaders
# ---------------------------------------------------------------------------

def load_misaki(name: str) -> dict[str, str]:
    """Load a Misaki JSON dict; expand to canonical IPA.

    Most values are plain IPA strings, but ~1% are POS-conditioned
    homograph maps like `{'DEFAULT': 'ˈɑˌɑ', 'NOUN': None}` — we take
    the DEFAULT for those, since we don't carry POS context.
    """
    path = SRC / name
    print(f"  loading {path.name}…", file=sys.stderr)
    raw = json.loads(path.read_text())
    out: dict[str, str] = {}
    for word, value in raw.items():
        if isinstance(value, dict):
            ipa = value.get("DEFAULT")
            if not ipa:
                continue
        else:
            ipa = value
        canon = misaki_to_ipa(ipa)
        # Skip entries that still contain capital diphthong letters
        # — those are silver-dict transcription bugs (e.g.
        # 'ˌæbəsᵻnˈAʃən' where a capital A leaked through). The gold
        # dict shouldn't have these, but silver does.
        if any(c.isupper() for c in canon):
            continue
        out[word.lower()] = canon
    print(f"    → {len(out)} entries", file=sys.stderr)
    return out


def load_cmudict() -> dict[str, list[str]]:
    """
    Load CMUdict 0.7b. Returns word → list of IPA variants (CMU
    occasionally has multiple pronunciations per word, marked with
    `word(2)`, `word(3)`, etc.).
    """
    path = SRC / "cmudict.dict"
    print(f"  loading {path.name}…", file=sys.stderr)
    out: dict[str, list[str]] = defaultdict(list)
    for line in path.read_text().splitlines():
        # Comment / blank
        if not line or line.startswith(";;;"):
            continue
        # The cmusphinx fork uses lowercase words and trailing comments
        # after `#`. Split off the comment first.
        line = line.split("#", 1)[0].rstrip()
        if not line:
            continue
        parts = line.split()
        if len(parts) < 2:
            continue
        word_with_variant = parts[0]
        # word(2), word(3) → strip the variant marker
        word = re.sub(r"\(\d+\)$", "", word_with_variant)
        ipa = arpabet_to_ipa(parts[1:])
        if ipa:
            out[word.lower()].append(ipa)
    print(f"    → {sum(len(v) for v in out.values())} entries "
          f"({len(out)} unique words)", file=sys.stderr)
    return dict(out)


# WikiPron's "broad" scrape is actually narrow-ish: it preserves
# combining diacritics (nasalization, syllabicity, no-audible-release),
# tied-bar markers (d͡ʒ for affricates), and length marks (ː). The
# phonetics tokenizer expects broad IPA. Strip the narrowing back out
# so each character of the resulting IPA is a real broad phoneme.
WIKIPRON_STRIP_RE = re.compile(
    "["
    "̀-ͯ"  # combining diacritics (full range)
    "ː"         # ː — length mark
    "]"
)


def load_wikipron() -> dict[str, list[str]]:
    """
    Load WikiPron eng_latn_us_broad. Returns word → list of IPA
    variants. Each line is `word\\tphone phone phone`.
    """
    path = SRC / "wikipron_us_broad.tsv"
    print(f"  loading {path.name}…", file=sys.stderr)
    out: dict[str, list[str]] = defaultdict(list)
    for line in path.read_text().splitlines():
        if not line:
            continue
        try:
            word, phones = line.split("\t", 1)
        except ValueError:
            continue
        # Phones are space-separated; join them into a continuous IPA
        # string. Then strip narrow-IPA decoration to leave only the
        # broad phoneme letters and stress marks.
        ipa = WIKIPRON_STRIP_RE.sub("", phones.replace(" ", ""))
        if not ipa:
            continue
        out[word.lower()].append(ipa)
    print(f"    → {sum(len(v) for v in out.values())} entries "
          f"({len(out)} unique words)", file=sys.stderr)
    return dict(out)


# ---------------------------------------------------------------------------
# Merge + frequency
# ---------------------------------------------------------------------------

# Contraction-fragment words like `'s`, `'t`, `'d`, `o'` carry
# single-character IPAs (`s`, `t`, `d`, etc.) in Misaki's dictionaries.
# Each fragment is a real morpheme but not a Mad Gab clue word — they
# pollute the trie with 1-char "words" the search will splice into
# clue phrases. Drop them at merge time.
def is_contraction_fragment(word: str) -> bool:
    return (word.startswith("'") or word.endswith("'")) and len(word) <= 3


# Words with a legitimate 1-char IPA — vowel/consonant interjections
# and the indefinite article. Other 1-char IPAs (typically WikiPron's
# letter-name suffix pronunciations like `s` for the letter S, `d` for
# D) are dropped to keep the trie from matching at every isolated
# phoneme position in a target stream.
LEGITIMATE_SINGLE_CHAR_IPA_WORDS = {
    "a", "ah", "ahh", "aw", "awe", "ah-ah", "eh", "er", "ee",
    "ooh", "oooh", "oh", "ou", "uh", "uhh", "ur", "ure",
    "mm", "sh", "shh",
}


def strip_stress(ipa: str) -> str:
    return ipa.replace("ˈ", "").replace("ˌ", "")


def is_noise_transcription(word: str, ipa: str) -> bool:
    """Drop transcriptions that fragment the trie at single phonemes."""
    return (len(strip_stress(ipa)) <= 1
            and word not in LEGITIMATE_SINGLE_CHAR_IPA_WORDS)


def merge_sources(
    misaki_gold: dict[str, str],
    cmu: dict[str, list[str]],
    misaki_silver: dict[str, str],
    wikipron: dict[str, list[str]],
) -> dict[str, dict[str, str]]:
    """
    Per-word merge with preference order misaki_gold > cmu > misaki_silver
    > wikipron. Returns word → { source_label: ipa_string }.

    The resulting schema is compatible with phonetics-rs's
    Corpus::from_json: a dict per word with an `ipa` sub-dict keyed by
    source name.
    """
    words: set[str] = set()
    words.update(misaki_gold)
    words.update(cmu)
    words.update(misaki_silver)
    words.update(wikipron)
    words = {w for w in words if not is_contraction_fragment(w)}

    merged: dict[str, dict[str, str]] = {}
    for w in words:
        entry: dict[str, str] = {}

        def add(label: str, ipa: str) -> None:
            if not is_noise_transcription(w, ipa):
                entry[label] = ipa

        # Layer 1: Misaki gold (vetted, narrow-ish IPA)
        if w in misaki_gold:
            add("misaki_gold", misaki_gold[w])
        # Layer 2: CMU. If multiple variants, the first is the primary;
        # any additional are stored under cmu2, cmu3, ….
        if w in cmu:
            variants = cmu[w]
            add("cmu", variants[0])
            for i, v in enumerate(variants[1:], start=2):
                add(f"cmu{i}", v)
        # Layer 3: Misaki silver (only if neither gold nor cmu
        # contributed anything — silver entries can have transcription
        # bugs, so we use them only as gap-fill).
        if (w in misaki_silver
                and "misaki_gold" not in entry and "cmu" not in entry):
            add("misaki_silver", misaki_silver[w])
        # Layer 4: WikiPron — always include if present, primary + 2
        # variants. Captures pronunciation variation.
        if w in wikipron:
            variants = wikipron[w]
            add("wikipron", variants[0])
            for i, v in enumerate(variants[1:3], start=2):
                add(f"wikipron{i}", v)

        if entry:
            merged[w] = entry
    return merged


def attach_rarity(merged: dict[str, dict[str, str]]) -> dict[str, dict]:
    """
    Convert flat per-word IPA dicts into the phonetics-rs Corpus schema:

        { word: { "rarity": <rank>, "ipa": { ... }, "alt_display": "WORD" } }

    Rank is by descending Zipf frequency from the wordfreq package:
    rank 0 is the most common word, higher = rarer. Words not in
    wordfreq sort to the end (assigned a uniform high rank).
    """
    # Score every word with Zipf frequency; words wordfreq doesn't know
    # get 0.0 and sort to the end.
    scored = [
        (-wordfreq.zipf_frequency(w, "en"), w) for w in merged
    ]
    scored.sort()  # ascending by negative zipf → descending by zipf

    out: dict[str, dict] = {}
    for rank, (_, w) in enumerate(scored):
        entry = merged[w]
        out[w] = {
            "rarity": float(rank),
            "ipa": entry,
            "alt_display": w.upper(),
        }
    return out


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> int:
    print("loading sources…", file=sys.stderr)
    misaki_gold = load_misaki("us_gold.json")
    misaki_silver = load_misaki("us_silver.json")
    cmu = load_cmudict()
    wikipron = load_wikipron()

    print("merging…", file=sys.stderr)
    merged = merge_sources(misaki_gold, cmu, misaki_silver, wikipron)
    print(f"  → {len(merged)} unique words", file=sys.stderr)

    print("attaching rarity…", file=sys.stderr)
    final = attach_rarity(merged)

    # Stats
    n_gold = sum(1 for v in final.values() if "misaki_gold" in v["ipa"])
    n_cmu = sum(1 for v in final.values() if "cmu" in v["ipa"])
    n_silver = sum(1 for v in final.values() if "misaki_silver" in v["ipa"])
    n_wp = sum(1 for v in final.values() if "wikipron" in v["ipa"])
    print(f"  misaki_gold: {n_gold}", file=sys.stderr)
    print(f"  cmu:         {n_cmu}", file=sys.stderr)
    print(f"  misaki_silver: {n_silver}", file=sys.stderr)
    print(f"  wikipron:    {n_wp}", file=sys.stderr)

    print(f"writing {OUT}…", file=sys.stderr)
    OUT.parent.mkdir(parents=True, exist_ok=True)
    # ensure_ascii=False keeps IPA characters as readable UTF-8
    # (file is ~30% smaller than escaping every non-ASCII byte).
    with OUT.open("w", encoding="utf-8") as f:
        json.dump(final, f, ensure_ascii=False, separators=(",", ":"))
    size_mb = OUT.stat().st_size / 1024 / 1024
    print(f"  → {size_mb:.1f} MB", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
