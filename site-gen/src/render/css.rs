//! All CSS for the static site, kept as one literal so we can embed
//! it in `<style>` without runtime concatenation. The file it includes
//! is plain CSS — diff-friendly, syntax-highlight-friendly, and
//! editor-IDE-friendly outside of a Rust string literal.

pub const CSS: &str = include_str!("css.css");
