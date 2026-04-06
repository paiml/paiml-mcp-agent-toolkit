#![cfg_attr(coverage_nightly, coverage(off))]
// Output formatters for Five Whys analysis
//
// GREEN PHASE: Minimal implementation for test formats
//
// Split into submodules:
//   - debug_formatters_text.rs: Human-readable text formatter
//   - debug_formatters_markdown.rs: Markdown formatter
//   - debug_formatters_tests.rs: All unit tests

use crate::models::debug_analysis::*;
use anyhow::Result;

/// Format analysis as JSON
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_json(analysis: &DebugAnalysis) -> Result<String> {
    let json = serde_json::to_string_pretty(analysis)?;
    Ok(json)
}

include!("debug_formatters_text.rs");
include!("debug_formatters_markdown.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    include!("debug_formatters_tests.rs");
}
