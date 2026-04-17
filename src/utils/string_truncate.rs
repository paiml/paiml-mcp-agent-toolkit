//! Safe UTF-8 string truncation.
//!
//! Raw byte-index slicing (`&s[..n]`) panics when `n` lands inside a multi-byte
//! UTF-8 sequence. These helpers truncate at the nearest char boundary ≤ `n`,
//! which is what callers actually want for display truncation.

/// Return a prefix of `s` that is at most `max_bytes` bytes long, truncated at
/// a char boundary. If `s.len() <= max_bytes`, returns `s` unchanged.
pub fn truncate_at_char_boundary(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Truncate with a trailing ellipsis (`...`) if the string exceeds `max_bytes`.
/// The returned string's byte length is at most `max_bytes + 3`. Safe for UTF-8.
pub fn truncate_with_ellipsis(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    format!("{}...", truncate_at_char_boundary(s, max_bytes))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn ascii_under_limit_unchanged() {
        assert_eq!(truncate_at_char_boundary("hello", 10), "hello");
    }

    #[test]
    fn ascii_over_limit_truncated() {
        assert_eq!(truncate_at_char_boundary("hello world", 5), "hello");
    }

    #[test]
    fn em_dash_does_not_panic() {
        // GH-291 regression: em-dash is 3 UTF-8 bytes (E2 80 94).
        let s = "refactor: split foo — bar";
        let t = truncate_at_char_boundary(s, 20);
        assert!(t.len() <= 20);
        assert!(!t.ends_with(char::REPLACEMENT_CHARACTER));
    }

    #[test]
    fn truncation_at_char_boundary() {
        let s = "a——b"; // a + em-dash + em-dash + b (1 + 3 + 3 + 1)
        assert_eq!(truncate_at_char_boundary(s, 2), "a");
        assert_eq!(truncate_at_char_boundary(s, 4), "a—");
        assert_eq!(truncate_at_char_boundary(s, 7), "a——");
        assert_eq!(truncate_at_char_boundary(s, 8), "a——b");
    }

    #[test]
    fn ellipsis_under_limit_no_change() {
        assert_eq!(truncate_with_ellipsis("short", 100), "short");
    }

    #[test]
    fn ellipsis_over_limit_appended() {
        assert_eq!(truncate_with_ellipsis("hello world", 5), "hello...");
    }

    #[test]
    fn ellipsis_preserves_char_boundary() {
        let s = "foo — bar baz quux";
        let out = truncate_with_ellipsis(s, 6);
        assert!(out.ends_with("..."));
        assert!(out.is_char_boundary(out.len() - 3));
    }

    #[test]
    fn empty_string() {
        assert_eq!(truncate_at_char_boundary("", 10), "");
        assert_eq!(truncate_with_ellipsis("", 10), "");
    }

    #[test]
    fn zero_max() {
        assert_eq!(truncate_at_char_boundary("hello", 0), "");
    }
}
