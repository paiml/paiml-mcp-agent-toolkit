//! Clippy fix PREVIEW, behind `pmat analyze clippy`.
//!
//! This module does not modify source. Nothing under
//! `src/services/clippy_fix/` calls `fs::write` — there is no writer at all —
//! so every fix it can describe is a description, never an edit (#1086).
//!
//! A+ Code Standard: ALL functions <=10 complexity

use crate::services::clippy_fix::{ClippyDiagnostic, ClippyFixEngine, ConfidenceLevel};
use anyhow::Result;
use pmcp::ToolResult;
use serde_json::{json, Value};

/// Preview the clippy warnings that clear a confidence bar. Writes nothing.
///
/// `dry_run` is accepted and IGNORED — see the comment above the call to
/// `simulate_fixes` below.
///
/// Complexity: below the A+ ceiling of 10 (one branch fewer than before #1086,
/// which removed the apply/dry-run fork).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn auto_clippy_fix(
    project_path: Option<String>,
    confidence_level: Option<String>,
    dry_run: Option<bool>,
    fix_specific_codes: Option<Vec<String>>,
) -> Result<ToolResult> {
    let path = project_path.unwrap_or_else(|| ".".to_string());
    let min_confidence = parse_confidence_level(&confidence_level)?;

    // Run clippy and get diagnostics
    let diagnostics = run_clippy_analysis(&path).await?;

    // How many clippy actually REPORTED, before any filtering of ours.
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

    // PREVIEW IS THE ONLY MODE, and `dry_run` therefore selects nothing.
    //
    // The branch this replaces called an `apply_fixes` that never wrote a byte:
    // it built a modified string in memory, dropped it, and still returned
    // `"action": "applied"` with a non-zero `successful_fixes` and a named
    // `fixed_files` list over a byte-identical tree (#1086). A caller had every
    // reason to believe its source had been rewritten.
    //
    // Adding the missing write is not the fix either. The transform underneath
    // is `source.replace("return ", "")` over the whole file
    // (`ClippyFixEngine::apply_fix_internal`), which consults no span at all and
    // strikes that substring inside a string literal or a comment as readily as
    // at the `return` statement clippy flagged. A real span-based rewriter is a
    // separate piece of work.
    //
    // `dry_run` stays in the signature so `analyze clippy --dry-run` keeps
    // parsing unchanged; both values now produce the same preview.
    let _ = dry_run;
    let results = simulate_fixes(&engine, filtered).await?;

    Ok(create_fix_response(
        results,
        &census.with_eligible(eligible),
    ))
}

// Core helper functions: parsing, filtering, preview, response creation
include!("auto_clippy_fix_core.rs");

// Unit tests: property tests, parsing, confidence, and filtering
include!("auto_clippy_fix_tests.rs");

// Async and integration tests: preview, response, edge cases
include!("auto_clippy_fix_tests_integration.rs");
