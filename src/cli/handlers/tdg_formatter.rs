#![cfg_attr(coverage_nightly, coverage(off))]
//! Toyota Way: TDG (Technical Debt Gradient) Formatting Handler
//! Complexity: Reduced from 16 to individual functions <=8
//! Purpose: TDG report formatting with clean separation of concerns

/// Toyota Way: Single Responsibility - Format TDG summary as markdown
/// Extracted from stubs.rs to reduce complexity and improve maintainability
///
/// # Parameters
///
/// * `summary` - TDG analysis summary
/// * `include_components` - Whether to include component details
///
/// # Returns
///
/// Formatted markdown string
#[must_use]
pub fn format_markdown_output(
    summary: &crate::models::tdg::TDGSummary,
    include_components: bool,
) -> String {
    let mut md = String::new();

    // Header and summary
    add_header_and_summary(&mut md, summary);

    // Hotspots section
    if !summary.hotspots.is_empty() {
        add_hotspots_section(&mut md, &summary.hotspots);
    }

    // Components section if requested
    if include_components {
        add_components_section(&mut md);
    }

    md
}

// Helper functions for markdown section generation (header, summary, hotspots, components)
include!("tdg_formatter_helpers.rs");

// Unit tests for TDG formatter markdown output
include!("tdg_formatter_tests.rs");

// Property-based tests using proptest for TDG formatter invariants
include!("tdg_formatter_proptests.rs");
