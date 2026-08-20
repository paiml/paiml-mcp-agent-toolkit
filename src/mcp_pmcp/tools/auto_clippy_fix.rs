//! MCP Tool for Automated Clippy Fix
//!
//! A+ Code Standard: ALL functions <=10 complexity
//! MCP-First Dogfooding: Primary interface for clippy fixes

use crate::services::clippy_fix::{ClippyDiagnostic, ClippyFixEngine, ConfidenceLevel};
use anyhow::Result;
use pmcp::ToolResult;
use serde_json::{json, Value};

/// Auto-fix clippy warnings with confidence-based filtering
///
/// Complexity: 8 (within A+ standard <=10)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn auto_clippy_fix(
    project_path: Option<String>,
    confidence_level: Option<String>,
    dry_run: Option<bool>,
    fix_specific_codes: Option<Vec<String>>,
) -> Result<ToolResult> {
    let path = project_path.unwrap_or_else(|| ".".to_string());
    let min_confidence = parse_confidence_level(&confidence_level)?;
    let is_dry_run = dry_run.unwrap_or(false);

    // Run clippy and get diagnostics
    let diagnostics = run_clippy_analysis(&path).await?;

    // How many clippy actually REPORTED, before any of ours filtering.
    //
    // Everything downstream counts the FILTERED list, so on a crate where
    // `cargo clippy` emits 76 warnings this reported
    //
    //     "total_diagnostics": 0, "action": "applied",
    //     "message": "🔧 Clippy fixes applied successfully"
    //
    // and exited 0. The default is `--confidence high`, and `default_confidence`
    // rates anything without an explicit rule as Medium (a suggestion exists) or
    // Low (none) — so unless a lint is named in `confidence_rules`, every
    // diagnostic is dropped here. All 76 were: 75 `clippy::collapsible_if` and
    // one `dead_code`. "None were auto-fixable at this confidence" and "the
    // crate is clippy-clean" are opposite claims, and only the second was
    // reported.
    let census = DiagnosticCensus {
        found: diagnostics.len(),
        eligible: 0,
        min_confidence: format!("{min_confidence:?}"),
    };

    // Filter by confidence level
    let engine = ClippyFixEngine::new();
    let filtered = filter_diagnostics(&engine, diagnostics, min_confidence, &fix_specific_codes);
    let eligible = filtered.len();

    // Apply fixes or show what would be fixed
    let results = if is_dry_run {
        simulate_fixes(&engine, filtered).await?
    } else {
        apply_fixes(&engine, filtered).await?
    };

    Ok(create_fix_response(
        results,
        is_dry_run,
        &census.with_eligible(eligible),
    ))
}

// Core helper functions: parsing, filtering, simulation, application, response creation
include!("auto_clippy_fix_core.rs");

// Unit tests: property tests, parsing, confidence, and filtering
include!("auto_clippy_fix_tests.rs");

// Async and integration tests: simulate, apply, response, edge cases
include!("auto_clippy_fix_tests_integration.rs");
