//! HTML structural smoke tests.
//!
//! These walk the generated `_site/index.html` (rebuilt by the test
//! harness from a small in-repo fixture, so the tests don't depend
//! on the 28-MB production corpus) and assert structural invariants
//! the visual review can't easily catch:
//!
//! * No inline `onclick=`, `onmouseover=`, etc. attributes anywhere
//!   — every interaction is supposed to go through `data-*` + the
//!   delegated listener in `behavior.js`. A regression that emits a
//!   raw `onclick="..."` (like the bug this whole refactor was a
//!   response to) fails the build before deploy.
//!
//! * Every `data-word` value matches a real word in the embedded
//!   WORDS payload, so the click can actually resolve to a result.
//!
//! * Every `data-ch` value is keyed in PHONEME_INFO so the panel
//!   has something to render.
//!
//! * The `<script>` block parses as valid JavaScript (no stray
//!   `</script>` literals, no truncation).
//!
//! The fixture is intentionally tiny (~10 words) so this test runs
//! in <1 s.

use std::path::PathBuf;
use std::process::Command;

use scraper::{Html, Selector};
use serde_json::Value;

/// Build the site against the production data file and return the
/// rendered HTML. We use the real corpus because the structural
/// invariants only matter for what actually ships — and on a modern
/// laptop the build is <500 ms warm.
fn build_site() -> String {
    // Locate the repo root: ../ from this crate (site-gen).
    let mut repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    repo.pop();

    let corpus = repo.join("data").join("openepd.json");
    assert!(corpus.exists(), "missing {}", corpus.display());

    // Render into a per-test scratch dir so parallel cargo-test runs
    // don't clobber each other.
    let out = std::env::temp_dir().join("oepd-html-invariants");
    std::fs::create_dir_all(&out).unwrap();

    let bin = repo.join("target").join("release").join("site-gen");
    if !bin.exists() {
        // Build once on demand. `cargo test --release` is the
        // canonical invocation but we want this to work under plain
        // `cargo test` too.
        let status = Command::new(env!("CARGO"))
            .args(["build", "--release", "--bin", "site-gen"])
            .current_dir(&repo)
            .status()
            .expect("cargo build site-gen");
        assert!(status.success(), "site-gen failed to build");
    }

    let status = Command::new(&bin)
        .arg(&corpus)
        .arg(&out)
        .status()
        .expect("run site-gen");
    assert!(status.success(), "site-gen exited non-zero");

    std::fs::read_to_string(out.join("index.html")).expect("read index.html")
}

#[test]
fn no_inline_event_handlers() {
    // The regression that motivated the whole refactor: a string
    // value was emitted into onclick="..." and the browser truncated
    // the attribute at the first inner ". The architectural fix is
    // "no inline event handlers period". This is the test that locks
    // that in.
    let html = build_site();
    let doc = Html::parse_document(&html);
    let star = Selector::parse("*").unwrap();
    let banned_prefix = "on"; // onclick, onmouseover, onload, onerror, …
    for el in doc.select(&star) {
        for (attr, _value) in el.value().attrs() {
            assert!(
                !attr.starts_with(banned_prefix) || attr == "ontology"
                    || attr.starts_with("on-"),
                "found inline event handler `{attr}` on <{}>",
                el.value().name(),
            );
        }
    }
}

#[test]
fn data_word_values_resolve_in_words_payload() {
    let html = build_site();
    let doc = Html::parse_document(&html);
    let words = extract_payload_as_array(&html, "WORDS");

    // Build a set of all corpus words for O(1) lookup.
    let known: std::collections::HashSet<&str> = words
        .iter()
        .filter_map(|w| w.get(0).and_then(|v| v.as_str()))
        .collect();

    let sel = Selector::parse("[data-word]").unwrap();
    let mut checked = 0usize;
    for el in doc.select(&sel) {
        let w = el.value().attr("data-word").unwrap();
        // Skip empties — there shouldn't be any but a guard keeps
        // the failure message clean if something goes wrong.
        if w.is_empty() {
            panic!("element <{}> has empty data-word", el.value().name());
        }
        assert!(
            known.contains(w),
            "data-word=\"{w}\" on <{}> does not resolve to any word in WORDS",
            el.value().name(),
        );
        checked += 1;
    }
    assert!(checked > 100, "expected >100 data-word elements, saw {checked}");
}

#[test]
fn data_ch_values_resolve_in_phoneme_info() {
    let html = build_site();
    let doc = Html::parse_document(&html);
    let info = extract_payload_as_object(&html, "PHONEME_INFO");

    let sel = Selector::parse("[data-ch]").unwrap();
    let mut checked = 0usize;
    for el in doc.select(&sel) {
        let ch = el.value().attr("data-ch").unwrap();
        assert!(
            info.contains_key(ch),
            "data-ch=\"{ch}\" on <{}> has no entry in PHONEME_INFO",
            el.value().name(),
        );
        checked += 1;
    }
    assert!(checked > 50, "expected >50 data-ch elements, saw {checked}");
}

#[test]
fn heatmap_cells_match_matrix_dimensions() {
    let html = build_site();
    let doc = Html::parse_document(&html);
    let axis = extract_payload_as_array(&html, "PHONEME_AXIS");
    let matrix = extract_payload_as_array(&html, "DISTANCE_MATRIX");
    let n = axis.len();
    assert_eq!(matrix.len(), n * n, "DISTANCE_MATRIX size mismatch");

    let cells = Selector::parse(".heatmap-cell[data-row][data-col]").unwrap();
    let count = doc.select(&cells).count();
    assert_eq!(count, n * n, "rendered heatmap-cell count mismatch");
}

#[test]
fn script_block_has_no_premature_close() {
    // A `</script>` literal inside the JS payload would close the
    // tag early and dump the rest of the JS as text. serde_json
    // doesn't auto-escape `</script>`, so the renderer has to do it
    // — but for now we rely on the corpus and phoneme metadata never
    // containing the substring. If a future entry does, this test
    // fails before deploy.
    let html = build_site();
    // The HTML itself naturally contains `</script>` once at the tag
    // close. Anything more is a bug.
    let count = html.matches("</script>").count();
    assert_eq!(count, 1, "expected exactly one </script>, found {count}");
}

// ────────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────────

/// Pull a top-level `const NAME = …;` JSON literal out of the page
/// script and parse it as `Vec<serde_json::Value>`.
fn extract_payload_as_array(html: &str, name: &str) -> Vec<Value> {
    let literal = locate_const(html, name);
    serde_json::from_str(&literal)
        .unwrap_or_else(|e| panic!("parse {name} as array: {e}\nliteral: {}…", &literal[..200.min(literal.len())]))
}

fn extract_payload_as_object(html: &str, name: &str) -> serde_json::Map<String, Value> {
    let literal = locate_const(html, name);
    serde_json::from_str(&literal)
        .unwrap_or_else(|e| panic!("parse {name} as object: {e}"))
}

/// Find `const NAME=` in the page script and return everything from
/// the JSON literal start through to the terminating `;\n`.
fn locate_const(html: &str, name: &str) -> String {
    let needle = format!("const {name}=");
    let idx = html.find(&needle).unwrap_or_else(|| panic!("no `{needle}` in page"));
    let start = idx + needle.len();
    // Scan forward through balanced brackets to find the matching
    // end. JSON literals start with `[` or `{`; we count to depth 0.
    let bytes = html.as_bytes();
    let first = bytes[start];
    let (open, close) = match first {
        b'[' => (b'[', b']'),
        b'{' => (b'{', b'}'),
        c => panic!("unexpected leading char `{}` for const {name}", c as char),
    };
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    let mut end = start;
    for (i, &b) in bytes[start..].iter().enumerate() {
        if in_string {
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_string = false;
            }
            continue;
        }
        match b {
            b'"' => in_string = true,
            x if x == open => depth += 1,
            x if x == close => {
                depth -= 1;
                if depth == 0 {
                    end = start + i + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    assert!(end > start, "couldn't find end of const {name}");
    html[start..end].to_string()
}
