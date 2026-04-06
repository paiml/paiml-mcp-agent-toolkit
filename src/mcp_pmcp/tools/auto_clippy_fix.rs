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

    // Filter by confidence level
    let engine = ClippyFixEngine::new();
    let filtered = filter_diagnostics(&engine, diagnostics, min_confidence, &fix_specific_codes);

    // Apply fixes or show what would be fixed
    let results = if is_dry_run {
        simulate_fixes(&engine, filtered).await?
    } else {
        apply_fixes(&engine, filtered).await?
    };

    Ok(create_fix_response(results, is_dry_run))
}

// Core helper functions: parsing, filtering, simulation, application, response creation
include!("auto_clippy_fix_core.rs");

// Unit tests: property tests, parsing, confidence, and filtering
include!("auto_clippy_fix_tests.rs");

// Async and integration tests: simulate, apply, response, edge cases
include!("auto_clippy_fix_tests_integration.rs");
