// Pattern matching helpers for hygiene scoring

#![cfg_attr(coverage_nightly, coverage(off))]

pub(crate) fn matches_pattern(path: &str, pattern: &str) -> bool {
    if pattern.ends_with('/') {
        path.contains(pattern)
    } else if let Some(ext) = pattern.strip_prefix('*') {
        path.ends_with(ext)
    } else {
        path.contains(pattern)
    }
}
