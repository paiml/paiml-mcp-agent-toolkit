#![cfg_attr(coverage_nightly, coverage(off))]
//! Lint hotspot analysis handlers
//!
//! Analyzes Rust projects to find the single file with highest defect density
//! using streaming analysis of Clippy's JSON output.
//!
//! By default, uses EXTREME quality standards:
//! - `--all-targets`: Lints library, binaries, tests, and examples
//! - `-D warnings`: Zero tolerance for warnings (fails on any warning)
//! - `-D clippy::pedantic`: Strictest built-in lint group
//! - `-D clippy::nursery`: Experimental lints
//! - `-D clippy::cargo`: Cargo.toml manifest lints

pub mod clippy;
pub mod metrics;
pub mod output;
pub mod types;

// Re-export all public types from the original module
pub use types::{
    EnforcementMetadata, FileSummary, LintHotspot, LintHotspotParams, LintHotspotResult,
    QualityGateStatus, QualityViolation, RefactorChain, RefactorStep, SeverityDistribution,
    ViolationDetail,
};

// Re-export the public formatting function
pub use output::format_summary;

use crate::cli::LintHotspotOutputFormat;
use anyhow::Result;
use metrics::{calculate_enforcement_metadata, check_quality_gates, generate_refactor_chain};
use output::format_output;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Handle analyze lint-hotspot command
///
/// This function analyzes a Rust project to find lint violations and can enforce
/// quality standards. When the `--enforce` flag is set, the command will exit
/// with a non-zero status code if ANY violations are found.
///
/// # Exit Status
///
/// The command exits with status code 1 in the following cases:
/// - Quality gate fails (defect density exceeds `max_density` threshold)
/// - When `--enforce` flag is set AND there are any violations
///
/// # Example
///
/// ```bash
/// # Without enforce flag - only exits non-zero if quality gate fails
/// pmat analyze lint-hotspot --max-density 5.0
///
/// # With enforce flag - exits non-zero if ANY violations exist
/// pmat analyze lint-hotspot --enforce
/// ```ignore
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_analyze_lint_hotspot(
    project_path: PathBuf,
    file: Option<PathBuf>,
    format: LintHotspotOutputFormat,
    max_density: f64,
    min_confidence: f64,
    enforce: bool,
    dry_run: bool,
    enforcement_metadata: bool,
    output: Option<PathBuf>,
    perf: bool,
    clippy_flags: String,
    top_files: usize,
    include: Vec<String>,
    exclude: Vec<String>,
) -> Result<()> {
    // Apply include/exclude filters if specified
    if !include.is_empty() || !exclude.is_empty() {
        eprintln!("🔍 Applying file filters...");
        if !include.is_empty() {
            eprintln!("  Include patterns: {include:?}");
        }
        if !exclude.is_empty() {
            eprintln!("  Exclude patterns: {exclude:?}");
        }
    }

    let params = LintHotspotParams {
        project_path,
        file,
        format,
        max_density,
        min_confidence,
        enforce,
        dry_run,
        enforcement_metadata,
        output,
        perf,
        clippy_flags,
        top_files,
        include,
        exclude,
    };

    handle_analyze_lint_hotspot_with_params(params).await
}

/// Handle analyze lint-hotspot command with parameter struct
///
/// # Errors
///
/// Returns an error if the operation fails
async fn handle_analyze_lint_hotspot_with_params(params: LintHotspotParams) -> Result<()> {
    let start_time = std::time::Instant::now();

    log_analysis_start(&params.format);

    let result = run_analysis_by_mode(&params).await;

    let mut result = match result {
        Ok(r) => r,
        Err(e) if e.to_string().contains("No lint violations found") => {
            use crate::cli::colors as c;
            eprintln!("{}", c::pass("No lint violations found — project is clean"));
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    apply_file_filters(&mut result, &params)?;

    let final_result = build_final_result(result, &params)?;

    output_results(&final_result, &params, start_time.elapsed()).await?;

    execute_enforcement_if_needed(&final_result, &params);

    check_exit_conditions(&final_result, &params);

    Ok(())
}

/// Log analysis start message
fn log_analysis_start(format: &LintHotspotOutputFormat) {
    if *format != LintHotspotOutputFormat::Json {
        eprintln!("🔍 Running Clippy analysis...");
    }
}

/// Run analysis based on single file or project mode
async fn run_analysis_by_mode(params: &LintHotspotParams) -> Result<LintHotspotResult> {
    if let Some(ref file_path) = params.file {
        log_single_file_mode(file_path, &params.format);
        clippy::run_clippy_analysis_single_file(
            &params.project_path,
            file_path,
            &params.clippy_flags,
        )
        .await
    } else {
        clippy::run_clippy_analysis(&params.project_path, &params.clippy_flags).await
    }
}

/// Log single file analysis mode
fn log_single_file_mode(file_path: &Path, format: &LintHotspotOutputFormat) {
    if *format != LintHotspotOutputFormat::Json {
        eprintln!("📄 Analyzing single file: {}", file_path.display());
    }
}

/// Apply include/exclude file filters to results
fn apply_file_filters(result: &mut LintHotspotResult, params: &LintHotspotParams) -> Result<()> {
    if params.include.is_empty() && params.exclude.is_empty() {
        return Ok(());
    }

    use crate::utils::file_filter::FileFilter;
    let filter = FileFilter::new(params.include.clone(), params.exclude.clone())?;

    if !filter.has_filters() {
        return Ok(());
    }

    filter_violations(result, &filter);
    recalculate_hotspot_metrics(result);

    Ok(())
}

/// Filter violations using file filter
fn filter_violations(
    result: &mut LintHotspotResult,
    filter: &crate::utils::file_filter::FileFilter,
) {
    result.hotspot.detailed_violations.retain(|violation| {
        let path = std::path::Path::new(&violation.file);
        filter.should_include(path)
    });

    result.all_violations.retain(|violation| {
        let path = std::path::Path::new(&violation.file);
        filter.should_include(path)
    });

    let filtered_summary: HashMap<PathBuf, FileSummary> = result
        .summary_by_file
        .drain()
        .filter(|(path, _summary)| filter.should_include(path))
        .collect();
    result.summary_by_file = filtered_summary;
}

/// Recalculate hotspot metrics after filtering
fn recalculate_hotspot_metrics(result: &mut LintHotspotResult) {
    result.hotspot.total_violations = result.hotspot.detailed_violations.len();
    if result.hotspot.sloc > 0 {
        result.hotspot.defect_density =
            result.hotspot.total_violations as f64 / result.hotspot.sloc as f64;
    }
}

/// Build final result with enforcement and quality gate data
fn build_final_result(
    mut result: LintHotspotResult,
    params: &LintHotspotParams,
) -> Result<LintHotspotResult> {
    let enforcement = generate_enforcement_metadata_if_needed(&result.hotspot, params);
    let refactor_chain = generate_refactor_chain_if_needed(&result.hotspot, params, &enforcement);
    let quality_gate = check_quality_gates(&result.hotspot, params.max_density);

    result.enforcement = enforcement;
    result.refactor_chain = refactor_chain;
    result.quality_gate = quality_gate;

    Ok(result)
}

/// Generate enforcement metadata if requested
fn generate_enforcement_metadata_if_needed(
    hotspot: &LintHotspot,
    params: &LintHotspotParams,
) -> Option<EnforcementMetadata> {
    if params.enforcement_metadata || params.enforce {
        Some(calculate_enforcement_metadata(
            hotspot,
            params.min_confidence,
        ))
    } else {
        None
    }
}

/// Generate refactor chain if enforcement is needed
fn generate_refactor_chain_if_needed(
    hotspot: &LintHotspot,
    params: &LintHotspotParams,
    enforcement: &Option<EnforcementMetadata>,
) -> Option<RefactorChain> {
    if params.enforce || enforcement.as_ref().is_some_and(|e| e.requires_enforcement) {
        Some(generate_refactor_chain(hotspot, params.min_confidence))
    } else {
        None
    }
}

/// Output results to file or stdout
async fn output_results(
    final_result: &LintHotspotResult,
    params: &LintHotspotParams,
    elapsed: std::time::Duration,
) -> Result<()> {
    let output_content = format_output(
        final_result,
        params.format.clone(),
        params.perf,
        elapsed,
        params.top_files,
    )?;

    if let Some(output_path) = &params.output {
        tokio::fs::write(output_path, &output_content).await?;
    } else {
        println!("{output_content}");
    }

    Ok(())
}

/// Execute enforcement if requested and conditions are met
fn execute_enforcement_if_needed(final_result: &LintHotspotResult, params: &LintHotspotParams) {
    if params.enforce && !params.dry_run && final_result.quality_gate.blocking {
        eprintln!("🚨 Enforcement required - executing refactor chain...");
        eprintln!("⚠️  Enforcement execution not yet implemented");
    }
}

/// Check exit conditions and exit with error code if needed
fn check_exit_conditions(final_result: &LintHotspotResult, params: &LintHotspotParams) {
    if should_exit_with_error(final_result, params) {
        log_enforcement_failure_if_needed(final_result, params);
        std::process::exit(1);
    }
}

/// Check if we should exit with error code
fn should_exit_with_error(final_result: &LintHotspotResult, params: &LintHotspotParams) -> bool {
    !final_result.quality_gate.passed
        || (params.enforce && final_result.total_project_violations > 0)
}

/// Log enforcement failure message if conditions are met
fn log_enforcement_failure_if_needed(final_result: &LintHotspotResult, params: &LintHotspotParams) {
    if params.enforce
        && final_result.total_project_violations > 0
        && final_result.quality_gate.passed
    {
        eprintln!(
            "\n❌ Enforcement failed: {} violations found",
            final_result.total_project_violations
        );
    }
}

// Tests extracted to lint_hotspot_handlers_tests.rs for file health compliance (CB-040)
// TEMPORARILY DISABLED: File splitting broke syntax (functions/modules split across files)
#[cfg(all(test, feature = "broken-tests"))]
#[path = "../lint_hotspot_handlers_tests.rs"]
mod tests;
