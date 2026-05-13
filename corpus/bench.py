#!/usr/bin/env python3
"""
Quality check for the built OpenEPD corpus.

Reports per-source coverage and runs a hand-curated check set
covering canonical phonetic distinctions (narrow vowel pairs, function
words, affricates, diphthongs). Run after `build.py`:

    corpus/venv/bin/python corpus/bench.py
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

CORPUS_PATH = Path(__file__).resolve().parent.parent / "data" / "openepd.json"

# Mirrors phonetics-rs::Corpus::preferred_ipa: exact match per
# preference, then prefix fallback within the same preference.
SOURCE_PREFERENCE = [
    "cmu", "misaki_gold", "misaki_silver",
    "phonemicchart", "wiktionary", "wikipron",
]

# Hand-curated check set. Each entry is (word, must-contain-substring
# after stress strip + g→ɡ fold). These probe the kinds of distinctions
# the corpus exists to make.
CHECK_SET = [
    # Narrow vowel pairs the old ARPABET-derived corpus collapsed:
    ("stupid",  "stup"),     # /u/ tense, not the /ʊ/ that ARPABET lumps in
    ("hid",     "hɪd"),
    ("hits",    "hɪts"),
    ("justice", "dʒʌs"),
    ("love",    "lʌv"),
    # Affricates as 2-char IPA sequences:
    ("just",    "dʒ"),
    ("chair",   "tʃ"),
    # Diphthongs:
    ("game",    "eɪ"),
    ("came",    "eɪ"),
    ("dupe",    "dup"),
    # Function-word weak forms — critical because targets transcribed
    # for downstream search need the form a speaker actually produces:
    ("a",       "ə"),
    ("the",     "ðə"),
    # Long-tail / proper-noun coverage:
    ("california", "kæl"),
    ("supercalifragilisticexpialidocious", ""),
]


def best_ipa(entry: dict) -> str | None:
    ipa_map = entry.get("ipa", {})
    if not ipa_map:
        return None
    for pref in SOURCE_PREFERENCE:
        if pref in ipa_map:
            return ipa_map[pref]
        for src, ipa in ipa_map.items():
            if src.startswith(pref):
                return ipa
    return next(iter(ipa_map.values()))


def main() -> int:
    if not CORPUS_PATH.exists():
        print(f"corpus not found at {CORPUS_PATH} — run build.py first",
              file=sys.stderr)
        return 2
    corpus = json.loads(CORPUS_PATH.read_text())
    size_mb = CORPUS_PATH.stat().st_size / 1e6
    print(f"corpus: {len(corpus):>7} unique words, {size_mb:.1f} MB")
    print()

    print("source coverage (entries with that source present):")
    counts: dict[str, int] = {}
    for entry in corpus.values():
        for src in entry.get("ipa", {}):
            counts[src] = counts.get(src, 0) + 1
    for src, n in sorted(counts.items(), key=lambda kv: -kv[1]):
        print(f"  {src:18s} {n:>7}")
    print()

    # Normalize for substring check: strip stress, fold g→ɡ so the
    # check is independent of which source happened to win.
    def norm(s: str) -> str:
        return s.replace("ˈ", "").replace("ˌ", "").replace("g", "ɡ")

    print("check set (expected substring in preferred IPA, "
          "stress stripped, g→ɡ folded):")
    print(f"  {'word':<40s} {'preferred IPA':<30s} match? expected")
    failures = 0
    for word, expected in CHECK_SET:
        ipa = best_ipa(corpus[word]) if word in corpus else None
        ok = ipa is not None and (not expected or expected in norm(ipa))
        if not ok:
            failures += 1
        flag = "✓" if ok else "✗"
        print(f"  {word:<40s} {(ipa or '—'):<30s} {flag}      "
              f"{expected!r}")
    print()
    if failures:
        print(f"FAIL: {failures}/{len(CHECK_SET)} check-set entries failed")
        return 1
    print(f"OK: {len(CHECK_SET)}/{len(CHECK_SET)} check-set entries passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
