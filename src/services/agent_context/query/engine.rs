#![cfg_attr(coverage_nightly, coverage(off))]

use super::types::{CaseSensitivity, QueryOptions, QueryResult, SearchMode};
use crate::services::agent_context::{AgentContextIndex, FunctionEntry};
use regex::Regex;
use std::collections::HashSet;

/// Check if function is a test (test_ prefix or in tests/ directory)
pub(super) fn is_test_function(func: &FunctionEntry) -> bool {
    func.function_name.starts_with("test_")
        || func.file_path.starts_with("tests/")
        || func.file_path.contains("/tests/")
        || func.file_path.contains("_tests.")
        || func.file_path.contains("_test.")
}

/// Parse `file:` and `fn:` prefixes from a query string.
///
/// Extracts optional scope filters:
/// - `file:query.rs error handling` -> (Some("query.rs"), None, "error handling")
/// - `fn:handle_ auth` -> (None, Some("handle_"), "auth")
/// - `file:foo.rs fn:bar baz` -> (Some("foo.rs"), Some("bar"), "baz")
pub(super) fn parse_query_prefixes(query: &str) -> (Option<String>, Option<String>, String) {
    let mut file_filter = None;
    let mut fn_filter = None;
    let mut remaining_parts = Vec::new();

    for token in query.split_whitespace() {
        if let Some(pattern) = token.strip_prefix("file:") {
            if !pattern.is_empty() {
                file_filter = Some(pattern.to_string());
            }
        } else if let Some(pattern) = token.strip_prefix("fn:") {
            if !pattern.is_empty() {
                fn_filter = Some(pattern.to_string());
            }
        } else {
            remaining_parts.push(token);
        }
    }

    let remaining = remaining_parts.join(" ");
    (file_filter, fn_filter, remaining)
}

/// Simple glob matching: supports `*` and `**` patterns
pub(super) fn glob_matches(pattern: &str, path: &str) -> bool {
    if pattern.contains('*') {
        // Convert glob to regex: handle ** before * using placeholder to avoid
        // the second replace clobbering the .* from the first
        let regex_str = pattern
            .replace('.', "\\.")
            .replace("**/", "\x00GLOBSTAR\x00")
            .replace("**", "\x00GLOBSTAR2\x00")
            .replace('*', "[^/]*")
            .replace("\x00GLOBSTAR\x00", "(.*/)?")
            .replace("\x00GLOBSTAR2\x00", ".*");
        Regex::new(&format!("^{regex_str}$")).is_ok_and(|re| re.is_match(path))
    } else {
        path.contains(pattern)
    }
}

// --- impl AgentContextIndex: public query API ---
include!("engine_query.rs");

// --- impl AgentContextIndex: relevance scoring and filtering ---
include!("engine_scoring.rs");

// --- impl AgentContextIndex: regex and literal search modes ---
include!("engine_search.rs");

#[cfg(test)]
#[path = "path_pattern_glob_tests.rs"]
mod path_pattern_glob_tests;
