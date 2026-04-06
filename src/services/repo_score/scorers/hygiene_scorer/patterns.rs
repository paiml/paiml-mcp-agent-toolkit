// Pattern matching helpers for hygiene scoring

#![cfg_attr(coverage_nightly, coverage(off))]

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn matches_pattern(path: &str, pattern: &str) -> bool {
    debug_assert!(!path.is_empty(), "path must not be empty");
    if pattern.ends_with('/') {
        // #241: Match directory patterns against path components, not substrings.
        // "out/" should match "./out/foo" but NOT "./dropout/foo".
        // Only match as a non-terminal component (i.e., there's a path after it).
        let dir_name = pattern.trim_end_matches('/');
        let components: Vec<&str> = path.split('/').collect();
        components
            .iter()
            .enumerate()
            .any(|(i, component)| *component == dir_name && i < components.len() - 1)
    } else if let Some(ext) = pattern.strip_prefix('*') {
        path.ends_with(ext)
    } else if pattern.starts_with('.') {
        // Dotfile/extension patterns: match as suffix on any path component
        // ".sublime-project" matches "project.sublime-project"
        // ".DS_Store" matches ".DS_Store"
        path.split('/')
            .any(|component| component.ends_with(pattern) || component == pattern)
    } else {
        // Exact filename match against path components
        path.split('/').any(|component| component == pattern)
    }
}
