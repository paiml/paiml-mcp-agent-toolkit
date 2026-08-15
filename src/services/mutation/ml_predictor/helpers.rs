#![cfg_attr(coverage_nightly, coverage(off))]
//! Helper functions for feature extraction.

/// Helper: Estimate nesting depth from source
pub(super) fn estimate_nesting_depth(source: &str) -> u32 {
    let mut max_depth: u32 = 0;
    let mut current_depth: u32 = 0;

    for ch in source.chars() {
        match ch {
            '{' => {
                current_depth += 1;
                max_depth = max_depth.max(current_depth);
            }
            '}' => {
                current_depth = current_depth.saturating_sub(1);
            }
            _ => {}
        }
    }

    max_depth
}

/// Helper: Count function parameters
#[allow(clippy::cast_possible_truncation)]
pub(super) fn count_parameters(source: &str) -> u32 {
    // Simple heuristic: count commas in first parentheses.
    //
    // The empty-parameter guard used to slice `start..start + end`, which stops
    // BEFORE the closing paren — so for `fn foo()` it produced `"("`, compared
    // that against `"()"`, never matched, and returned `0 commas + 1 = 1`
    // parameter for a function that takes none. The guard could not fire for
    // any input. Found by restoring tests that had not compiled in a long time;
    // `test_count_parameters_empty` asserts exactly this and was correct.
    //
    // Work on the text BETWEEN the parens instead, where "empty" is simply
    // "nothing there".
    if let Some(start) = source.find('(') {
        if let Some(end) = source.get(start..).unwrap_or_default().find(')') {
            let inner = source.get(start + 1..start + end).unwrap_or_default();
            if inner.trim().is_empty() {
                return 0;
            }
            return (inner.matches(',').count() + 1) as u32;
        }
    }
    0
}

/// Helper: Count unique variable identifiers (simple heuristic)
#[allow(clippy::cast_possible_truncation)]
pub(super) fn count_unique_variables(source: &str) -> u32 {
    use std::collections::HashSet;
    let mut variables = HashSet::new();

    // Simple heuristic: extract words that start with lowercase or underscore
    for token in source.split_whitespace() {
        // Remove common punctuation
        let cleaned = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');

        if !cleaned.is_empty() {
            let first_char = cleaned.chars().next().expect("checked is_empty");
            if first_char.is_lowercase() || first_char == '_' {
                // Skip keywords
                if !is_rust_keyword(cleaned) {
                    variables.insert(cleaned.to_string());
                }
            }
        }
    }

    variables.len() as u32
}

/// Helper: Check if word is a Rust keyword
pub(super) fn is_rust_keyword(word: &str) -> bool {
    matches!(
        word,
        "fn" | "let"
            | "mut"
            | "const"
            | "static"
            | "if"
            | "else"
            | "for"
            | "while"
            | "loop"
            | "match"
            | "return"
            | "break"
            | "continue"
            | "pub"
            | "use"
            | "mod"
            | "struct"
            | "enum"
            | "impl"
            | "trait"
            | "type"
            | "where"
            | "unsafe"
            | "async"
            | "await"
            | "move"
            | "ref"
            | "in"
            | "as"
            | "crate"
            | "super"
            | "self"
            | "Self"
            | "true"
            | "false"
    )
}
