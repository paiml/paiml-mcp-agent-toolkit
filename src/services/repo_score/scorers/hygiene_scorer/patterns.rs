// Pattern matching helpers for hygiene scoring

#![cfg_attr(coverage_nightly, coverage(off))]

pub(crate) fn matches_pattern(path: &str, pattern: &str) -> bool {
    if pattern.ends_with('/') {
        // #241: Match directory patterns against path components, not substrings.
        // "out/" should match "/out/foo" but NOT "/dropout/foo".
        let dir_name = pattern.trim_end_matches('/');
        path.split('/').any(|component| component == dir_name)
    } else if let Some(ext) = pattern.strip_prefix('*') {
        path.ends_with(ext)
    } else {
        // Match exact filename against path components
        path.split('/').any(|component| component == pattern)
    }
}
