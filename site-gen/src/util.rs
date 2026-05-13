//! Small formatting helpers used across the render modules.

/// Insert thousands separators: `141851` → `"141,851"`.
pub fn fmt_num(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

/// Compact: 1_234 → "1k", 1_234_567 → "1.2M".
pub fn fmt_k(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.0}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thousands_separator() {
        assert_eq!(fmt_num(1), "1");
        assert_eq!(fmt_num(1234), "1,234");
        assert_eq!(fmt_num(141_851), "141,851");
        assert_eq!(fmt_num(1_234_567), "1,234,567");
    }

    #[test]
    fn k_compression() {
        assert_eq!(fmt_k(50), "50");
        assert_eq!(fmt_k(999), "999");
        assert_eq!(fmt_k(1234), "1k");
        assert_eq!(fmt_k(141_851), "142k");
        assert_eq!(fmt_k(1_234_567), "1.2M");
    }
}
