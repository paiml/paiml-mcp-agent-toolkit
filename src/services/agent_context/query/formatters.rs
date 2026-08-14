#![cfg_attr(coverage_nightly, coverage(off))]

use super::types::QueryResult;

// ── Colour ──────────────────────────────────────────────────────────────────
//
// `format_text` / `format_text_with_code` — the printer behind `pmat query` —
// interpolated raw `"\x1b[36m"` literals, a THIRD private copy of the palette
// after `cli::colors` and `query_handler::options`. `pmat query 'error
// handling' --color never | cat` still wrote 20 escape-bearing lines, and so
// did `NO_COLOR=1 pmat query …`: the flag was wired to nothing here at all.
//
// These aliases are `cli::colors::Sgr`, whose `Display` consults
// `colors_enabled()`, so `{CYAN}` in a format string is now gated by
// construction. They live in this parent module because the printers are
// `include!`d and cannot carry `use` items of their own.
use crate::cli::colors::Sgr;

const RESET: Sgr = crate::cli::colors::RESET;
const BOLD: Sgr = crate::cli::colors::BOLD;
const DIM: Sgr = crate::cli::colors::DIM;
const RED: Sgr = crate::cli::colors::RED;
const GREEN: Sgr = crate::cli::colors::GREEN;
const YELLOW: Sgr = crate::cli::colors::YELLOW;
const MAGENTA: Sgr = crate::cli::colors::MAGENTA;
const CYAN: Sgr = crate::cli::colors::CYAN;
const BOLD_RED: Sgr = crate::cli::colors::BOLD_RED;
const BOLD_GREEN: Sgr = crate::cli::colors::BOLD_GREEN;
const BOLD_YELLOW: Sgr = crate::cli::colors::BOLD_YELLOW;
const BOLD_MAGENTA: Sgr = Sgr::new("\x1b[1;35m");
const BOLD_CYAN: Sgr = crate::cli::colors::BOLD_CYAN;
const BOLD_WHITE: Sgr = crate::cli::colors::BOLD_WHITE;
const DIM_CYAN: Sgr = crate::cli::colors::DIM_CYAN;
const ITALIC_WHITE: Sgr = Sgr::new("\x1b[3;37m");
/// Bold on a yellow background — the `--literal`/`--regex` match highlight.
const BG_YELLOW_BOLD: Sgr = Sgr::new("\x1b[1;43m");
/// Bright red, bold — the fault-annotation marker.
const BRIGHT_RED_BOLD: Sgr = Sgr::new("\x1b[1;91m");

/// Format results as JSON
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
