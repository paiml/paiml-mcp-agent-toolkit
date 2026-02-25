#![cfg_attr(coverage_nightly, coverage(off))]

use super::types::QueryResult;

/// Format results as JSON
pub fn format_json(results: &[QueryResult]) -> Result<String, String> {
    serde_json::to_string_pretty(results).map_err(|e| format!("JSON serialization failed: {e}"))
}

// Shared helper functions: coverage metrics, truncation, rich metrics builders,
// call graph formatting, fault lines, match highlighting, and source rendering.
include!("formatters_helpers.rs");

// Markdown output formatting: build_quality_md, push_churn_md, format_md_details,
// and the public format_markdown() function.
include!("formatters_markdown.rs");

// Colorized terminal text formatting: format_text_with_code (with source code),
// build_text_metrics, format_text_details, and the public format_text() function.
include!("formatters_colorized.rs");

// Unit tests for all formatter functions.
include!("formatters_tests.rs");
