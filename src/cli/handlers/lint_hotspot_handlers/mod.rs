#![cfg_attr(coverage_nightly, coverage(off))]
//! Lint hotspot analysis handlers
//!
//! Analyzes Rust projects to find the single file with highest defect density
//! using streaming analysis of Clippy's JSON output.
//!
//! By default, uses EXTREME quality standards:
//! - `--all-targets`: Lints library, binaries, tests, and examples. Because the
//!   same source file is compiled once per target, cargo emits each finding
//!   once per target; identical findings are collapsed so the reported count is
//!   the number of distinct violations, not the number of compilations.
//! - `-W warnings`, `-W clippy::pedantic`, `-W clippy::nursery`,
//!   `-W clippy::cargo` (see the `--clippy-flags` default). These are rustc
//!   flags and are passed after the `--` separator.
//!
//! If `cargo clippy` cannot complete, this command returns an error. It never
//! reports a clean project it did not establish.

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
/// quality standards. `--enforce` reports the breach and makes the exit code
/// follow the quality gate; it does not lower the threshold.
///
/// # Exit Status
///
/// The command exits with status code 1 when the quality gate fails, i.e. when
/// the measured defect density exceeds `max_density` (or a single file carries
/// more than 50 violations) — with or without `--enforce`.
///
/// # Example
///
/// ```bash
/// # Exits non-zero only if the measured density exceeds --max-density
/// pmat analyze lint-hotspot --max-density 5.0
///
/// # Same gate, plus the enforcement metadata and refactor chain in the report
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
        crate::status_eprintln!("🔍 Applying file filters...");
        if !include.is_empty() {
            crate::status_eprintln!("  Include patterns: {include:?}");
        }
        if !exclude.is_empty() {
            crate::status_eprintln!("  Exclude patterns: {exclude:?}");
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

    // `None` = clippy ran to completion and found nothing. An unusable run is an
    // Err and propagates: #679 shipped a version that turned "cargo rejected our
    // argv" into "project is clean", which is the one outcome a linter must
    // never invent.
    let Some(mut result) = run_analysis_by_mode(&params).await? else {
        return report_measured_clean(&params).await;
    };

    apply_file_filters(&mut result, &params)?;

    let final_result = build_final_result(result, &params)?;

    output_results(&final_result, &params, start_time.elapsed()).await?;

    execute_enforcement_if_needed(&final_result, &params);

    check_exit_conditions(&final_result, &params);

    Ok(())
}

/// Write the machine/human "clean project" report to stdout (or `--output`).
///
/// Uses the same sink as `output_results` so a clean run and a dirty run of the
/// same command land in the same place.
async fn emit_clean_output(params: &LintHotspotParams, elapsed: std::time::Duration) -> Result<()> {
    let content = output::format_clean_output(&params.format, params.perf, elapsed)?;
    write_output(&content, params).await
}

/// Single stdout/`--output` sink for every lint-hotspot report.
async fn write_output(content: &str, params: &LintHotspotParams) -> Result<()> {
    if let Some(output_path) = &params.output {
        tokio::fs::write(output_path, content).await?;
    } else {
        println!("{content}");
    }
    Ok(())
}

/// Log analysis start message.
///
/// Progress chatter is suppressed for every machine-readable format, not just
/// `json`: `enforcement-json` and `sarif` are parsed by tools too, and their
/// stderr should not differ from `json`'s for the same run.
fn log_analysis_start(format: &LintHotspotOutputFormat) {
    if !is_machine_format(format) {
        crate::status_eprintln!("🔍 Running Clippy analysis...");
    }
}

/// True for formats consumed by tools rather than humans.
fn is_machine_format(format: &LintHotspotOutputFormat) -> bool {
    matches!(
        format,
        LintHotspotOutputFormat::Json
            | LintHotspotOutputFormat::EnforcementJson
            | LintHotspotOutputFormat::Sarif
    )
}

/// Run analysis based on single file or project mode
///
/// `Ok(None)` means "clippy ran and reported nothing", never "we could not
/// measure" — the latter is an `Err`.
async fn run_analysis_by_mode(params: &LintHotspotParams) -> Result<Option<LintHotspotResult>> {
    if let Some(ref file_path) = params.file {
        log_single_file_mode(file_path, &params.format);
        clippy::run_clippy_analysis_single_file(
            &params.project_path,
            file_path,
            &params.clippy_flags,
        )
        .await
        .map(Some)
    } else {
        clippy::run_clippy_analysis(&params.project_path, &params.clippy_flags).await
    }
}

/// Emit an explicitly empty, well-formed result for a project clippy actually
/// measured and found clean.
///
/// Before this, the clean path wrote a line to STDERR and produced NOTHING on
/// stdout, so `--format json` (a declared format) yielded an empty document.
async fn report_measured_clean(params: &LintHotspotParams) -> Result<()> {
    use crate::cli::colors as c;

    let content = output::format_clean_result(&params.format)?;
    if let Some(output_path) = &params.output {
        tokio::fs::write(output_path, &content).await?;
    } else {
        println!("{content}");
    }
    crate::status_eprintln!(
        "{}",
        c::pass("cargo clippy completed and reported no lint violations")
    );
    Ok(())
}

/// Log single file analysis mode
fn log_single_file_mode(file_path: &Path, format: &LintHotspotOutputFormat) {
    if !is_machine_format(format) {
        crate::status_eprintln!("📄 Analyzing single file: {}", file_path.display());
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

    write_output(&output_content, params).await
}

/// Report that the gate is blocking.
///
/// This used to announce "executing refactor chain..." and then admit
/// "Enforcement execution not yet implemented": a released command advertising
/// a step it never took. `--enforce` is a gate — it reports the breach and sets
/// the exit code; the refactor chain it computes is printed in the report for a
/// human or tool to apply.
fn execute_enforcement_if_needed(final_result: &LintHotspotResult, params: &LintHotspotParams) {
    if let Some(notice) = enforcement_notice(final_result, params) {
        eprintln!("{notice}");
    }
}

/// The message `--enforce` prints when the gate is blocking, or `None`.
///
/// Split out so the released text can be asserted: it must describe what the
/// command actually did, never a step it does not perform.
fn enforcement_notice(
    final_result: &LintHotspotResult,
    params: &LintHotspotParams,
) -> Option<String> {
    if params.enforce && !params.dry_run && final_result.quality_gate.blocking {
        Some("🚨 Quality gate is blocking - see the refactor chain in the report above".to_string())
    } else {
        None
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
///
/// The exit code follows the quality gate, which is what applies
/// `--max-density`. `--enforce` used to OR in "any violation at all", so
/// `--max-density 100` still exited 1 on a project measured at 0.72 while the
/// same run's `enforcement-json` reported `quality_gate.passed = true`. The
/// exit code and the machine-readable report now come from the same decision.
fn should_exit_with_error(final_result: &LintHotspotResult, _params: &LintHotspotParams) -> bool {
    !final_result.quality_gate.passed
}

/// Log enforcement failure message if conditions are met
fn log_enforcement_failure_if_needed(final_result: &LintHotspotResult, params: &LintHotspotParams) {
    if params.enforce && !final_result.quality_gate.passed {
        eprintln!("\n❌ Enforcement failed: quality gate breached");
        for violation in &final_result.quality_gate.violations {
            eprintln!(
                "   {}: {:.2} exceeds {:.2}",
                violation.rule, violation.actual, violation.threshold
            );
        }
    }
}

// Tests extracted to lint_hotspot_handlers_tests.rs for file health compliance (CB-040)
//
// #701: these fragments sat behind the deliberately-non-compiling
// `broken-tests` feature for so long that they silently drifted off the real
// types — every `DiagnosticSpan` literal still set a `_text` field that
// `types.rs` had dropped. Quarantine hid that: the fragments compiled in no
// profile, so nothing ever told us they were stale, yet they kept being edited
// (see the #698 comment in part3). Re-enabled under plain `cfg(test)` so the
// compiler keeps them honest from here on.
#[cfg(test)]
#[path = "../lint_hotspot_handlers_tests.rs"]
mod tests;

#[cfg(test)]
mod pure_helper_tests {
    //! Wave 39 PR18 — pure-helper coverage for lint_hotspot_handlers/mod.rs
    //! (160 missed pre-wave). Async handlers + clippy invocation are
    //! disqualified per spec §4.11 (shell out to cargo). The pure helpers
    //! `apply_file_filters` + `filter_violations` + `recalculate_hotspot_metrics`
    //! + `should_exit_with_error` are testable.
    use super::*;
    use crate::cli::LintHotspotOutputFormat;
    use std::collections::HashMap;

    fn make_violation(file: &str, line: u32, severity: &str) -> ViolationDetail {
        ViolationDetail {
            file: PathBuf::from(file),
            line,
            column: 0,
            end_line: line,
            end_column: 0,
            lint_name: "test_lint".to_string(),
            message: "test".to_string(),
            severity: severity.to_string(),
            suggestion: None,
            machine_applicable: false,
        }
    }

    fn make_hotspot(file: &str, sloc: usize, total: usize) -> LintHotspot {
        LintHotspot {
            file: PathBuf::from(file),
            defect_density: total as f64 / sloc.max(1) as f64,
            total_violations: total,
            sloc,
            severity_distribution: SeverityDistribution::default(),
            top_lints: vec![],
            detailed_violations: (0..total)
                .map(|i| make_violation(file, i as u32, "warning"))
                .collect(),
        }
    }

    fn make_result(hotspot: LintHotspot, all: Vec<ViolationDetail>) -> LintHotspotResult {
        LintHotspotResult {
            hotspot,
            all_violations: all,
            summary_by_file: HashMap::new(),
            total_project_violations: 0,
            enforcement: None,
            refactor_chain: None,
            quality_gate: QualityGateStatus {
                passed: true,
                violations: vec![],
                blocking: false,
            },
        }
    }

    fn make_params(include: Vec<String>, exclude: Vec<String>) -> LintHotspotParams {
        LintHotspotParams {
            project_path: PathBuf::from("/tmp"),
            file: None,
            format: LintHotspotOutputFormat::Json,
            max_density: 0.1,
            min_confidence: 0.5,
            enforce: false,
            dry_run: false,
            enforcement_metadata: false,
            output: None,
            perf: false,
            clippy_flags: String::new(),
            top_files: 10,
            include,
            exclude,
        }
    }

    // ── apply_file_filters ──────────────────────────────────────────────────

    #[test]
    fn test_apply_file_filters_empty_include_exclude_short_circuit() {
        // PIN: empty include AND empty exclude → early return Ok, no mutation.
        let mut result = make_result(make_hotspot("src/foo.rs", 100, 5), vec![]);
        let params = make_params(vec![], vec![]);
        let result_ok = apply_file_filters(&mut result, &params);
        assert!(result_ok.is_ok());
        // Hotspot violations unchanged.
        assert_eq!(result.hotspot.detailed_violations.len(), 5);
    }

    #[test]
    fn test_apply_file_filters_invalid_pattern_returns_err() {
        // FileFilter::new requires valid patterns; an unparseable glob errors.
        let mut result = make_result(make_hotspot("src/foo.rs", 100, 5), vec![]);
        let params = make_params(vec!["[invalid".to_string()], vec![]);
        let r = apply_file_filters(&mut result, &params);
        assert!(r.is_err());
    }

    // ── recalculate_hotspot_metrics ─────────────────────────────────────────

    #[test]
    fn test_recalculate_hotspot_metrics_recomputes_density() {
        let mut result = make_result(make_hotspot("src/foo.rs", 100, 5), vec![]);
        // Drop one violation directly.
        result.hotspot.detailed_violations.pop();
        recalculate_hotspot_metrics(&mut result);
        assert_eq!(result.hotspot.total_violations, 4);
        assert!((result.hotspot.defect_density - 0.04).abs() < 1e-9);
    }

    #[test]
    fn test_recalculate_hotspot_metrics_zero_sloc_defect_density_unchanged() {
        // PIN: when sloc == 0, defect_density is NOT updated (avoids div/0).
        let mut result = make_result(make_hotspot("src/foo.rs", 0, 5), vec![]);
        let original_density = result.hotspot.defect_density;
        // Empty out violations.
        result.hotspot.detailed_violations.clear();
        recalculate_hotspot_metrics(&mut result);
        assert_eq!(result.hotspot.total_violations, 0);
        assert_eq!(result.hotspot.defect_density, original_density);
    }

    // ── should_exit_with_error ──────────────────────────────────────────────

    #[test]
    fn test_should_exit_quality_gate_failed() {
        let mut result = make_result(make_hotspot("src/foo.rs", 100, 5), vec![]);
        result.quality_gate.passed = false;
        let params = make_params(vec![], vec![]);
        assert!(should_exit_with_error(&result, &params));
    }

    #[test]
    fn test_should_exit_quality_gate_passed_no_enforce() {
        let result = make_result(make_hotspot("src/foo.rs", 100, 5), vec![]);
        let params = make_params(vec![], vec![]);
        assert!(!should_exit_with_error(&result, &params));
    }

    #[test]
    fn test_should_exit_enforce_with_violations_under_threshold_passes() {
        // Regression: `--enforce` used to OR in "any violation at all", so a
        // project measured well under --max-density still exited 1 while the
        // same run's enforcement-json said quality_gate.passed = true. This
        // test previously PINNED that contradiction.
        let mut result = make_result(make_hotspot("src/foo.rs", 100, 5), vec![]);
        result.total_project_violations = 3;
        let mut params = make_params(vec![], vec![]);
        params.enforce = true;
        assert!(
            !should_exit_with_error(&result, &params),
            "exit code must follow quality_gate.passed, which already applies --max-density"
        );
    }

    #[test]
    fn test_should_exit_enforce_follows_failing_gate() {
        let mut result = make_result(make_hotspot("src/foo.rs", 100, 5), vec![]);
        result.total_project_violations = 3;
        result.quality_gate.passed = false;
        let mut params = make_params(vec![], vec![]);
        params.enforce = true;
        assert!(should_exit_with_error(&result, &params));
    }

    #[test]
    fn test_should_exit_enforce_with_no_violations_passes() {
        let mut result = make_result(make_hotspot("src/foo.rs", 100, 5), vec![]);
        result.total_project_violations = 0;
        let mut params = make_params(vec![], vec![]);
        params.enforce = true;
        assert!(!should_exit_with_error(&result, &params));
    }

    // ── enforcement_notice ──────────────────────────────────────────────────

    #[test]
    fn test_enforcement_notice_never_announces_an_unimplemented_step() {
        // Regression: --enforce printed "executing refactor chain..." followed
        // by "Enforcement execution not yet implemented" in the shipped binary.
        let mut result = make_result(make_hotspot("src/foo.rs", 100, 5), vec![]);
        result.quality_gate.blocking = true;
        let mut params = make_params(vec![], vec![]);
        params.enforce = true;
        let notice = enforcement_notice(&result, &params).expect("blocking gate must be reported");
        assert!(!notice.contains("not yet implemented"), "{notice}");
        assert!(!notice.contains("executing refactor chain"), "{notice}");
    }

    #[test]
    fn test_enforcement_notice_silent_when_gate_not_blocking() {
        let result = make_result(make_hotspot("src/foo.rs", 100, 5), vec![]);
        let mut params = make_params(vec![], vec![]);
        params.enforce = true;
        assert!(enforcement_notice(&result, &params).is_none());
    }

    // ── generate_enforcement_metadata_if_needed ─────────────────────────────

    #[test]
    fn test_generate_enforcement_metadata_none_when_neither_flag_set() {
        let hotspot = make_hotspot("src/foo.rs", 100, 5);
        let params = make_params(vec![], vec![]);
        assert!(generate_enforcement_metadata_if_needed(&hotspot, &params).is_none());
    }

    #[test]
    fn test_generate_enforcement_metadata_some_when_metadata_flag() {
        let hotspot = make_hotspot("src/foo.rs", 100, 5);
        let mut params = make_params(vec![], vec![]);
        params.enforcement_metadata = true;
        assert!(generate_enforcement_metadata_if_needed(&hotspot, &params).is_some());
    }

    #[test]
    fn test_generate_enforcement_metadata_some_when_enforce_flag() {
        let hotspot = make_hotspot("src/foo.rs", 100, 5);
        let mut params = make_params(vec![], vec![]);
        params.enforce = true;
        assert!(generate_enforcement_metadata_if_needed(&hotspot, &params).is_some());
    }
}
