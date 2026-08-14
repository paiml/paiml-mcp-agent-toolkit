//! SATD (Self-Admitted Technical Debt) integration in complexity context.

use crate::cli::{SatdOutputFormat, SatdSeverity};
use anyhow::Result;
use std::path::{Path, PathBuf};

use super::output::{format_satd_output, write_satd_output};

/// Handle SATD (Self-Admitted Technical Debt) analysis command
#[allow(clippy::too_many_arguments)]
/// Toyota Way: Extract Method - Handle SATD analysis (complexity <=8)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_satd(
    path: PathBuf,
    format: SatdOutputFormat,
    severity: Option<SatdSeverity>,
    critical_only: bool,
    include_tests: bool,
    strict: bool,
    evolution: bool,
    days: u32,
    metrics: bool,
    output: Option<PathBuf>,
    top_files: usize,
    fail_on_violation: bool,
    timeout: u64,
) -> Result<()> {
    // Without this, a nonexistent path walked to zero files and printed
    // "Found 0 SATD violations in 0 files" with exit 0 — a clean bill of
    // health for a tree that was never there.
    crate::cli::ensure_analysis_path_exists(&path)?;

    // Print analysis info
    print_satd_analysis_info(strict, timeout);

    // Run SATD analysis
    let mut result = run_satd_analysis(&path, include_tests, strict, timeout).await?;

    // Apply filters
    apply_satd_filters(&mut result, severity, critical_only, top_files);

    crate::status_eprintln!(
        "📊 Found {} SATD items in {} files",
        result.items.len(),
        result.files_with_debt
    );

    // Format and output results
    let content = format_satd_output(&result, format, metrics, evolution, days)?;
    write_satd_output(content, output).await?;

    // Check violations
    check_satd_violations(&result, fail_on_violation)?;

    Ok(())
}

/// Toyota Way Helper: Print SATD analysis info
fn print_satd_analysis_info(strict: bool, timeout: u64) {
    crate::status_eprintln!("🔍 Analyzing self-admitted technical debt...");
    crate::status_eprintln!("⏰ Analysis timeout set to {timeout} seconds");
    if strict {
        crate::status_eprintln!("📝 Using strict mode (only explicit SATD markers)");
    }
}

/// Toyota Way Helper: Run SATD analysis with timeout
///
/// The budget is enforced by `run_within_analysis_budget` — the single
/// implementation of what a `--timeout` on `AnalyzeCommands` means — rather
/// than by the private `tokio::time::timeout(_, async { … })` that used to sit
/// here. That form polls the analysis on the CALLER's task, so a scan that does
/// not yield never lets the timer fire: the same non-enforcement #929 found in
/// `analyze dead-code`, spelled differently.
async fn run_satd_analysis(
    path: &Path,
    include_tests: bool,
    strict: bool,
    timeout: u64,
) -> Result<crate::services::satd_detector::SATDAnalysisResult> {
    use crate::services::satd_detector::SATDDetector;

    let owned_path = path.to_path_buf();
    crate::cli::commands::analyze_commands::run_within_analysis_budget(
        "SATD analysis",
        timeout,
        async move {
            let detector = if strict {
                SATDDetector::new_strict()
            } else {
                SATDDetector::new()
            };
            detector
                .analyze_project(&owned_path, include_tests)
                .await
                .map_err(anyhow::Error::from)
        },
    )
    .await
}

/// Toyota Way Helper: Apply SATD filters
fn apply_satd_filters(
    result: &mut crate::services::satd_detector::SATDAnalysisResult,
    severity: Option<SatdSeverity>,
    critical_only: bool,
    top_files: usize,
) {
    use crate::services::satd_detector::Severity as DetectorSeverity;

    // Filter by severity
    if let Some(min_severity) = severity {
        let min_detector_severity = match min_severity {
            SatdSeverity::Critical => DetectorSeverity::Critical,
            SatdSeverity::High => DetectorSeverity::High,
            SatdSeverity::Medium => DetectorSeverity::Medium,
            SatdSeverity::Low => DetectorSeverity::Low,
        };
        result
            .items
            .retain(|item| item.severity >= min_detector_severity);
    }

    // Filter critical only
    if critical_only {
        result
            .items
            .retain(|item| item.severity == DetectorSeverity::Critical);
    }

    // Apply top files filter
    if top_files > 0 {
        filter_top_files(result, top_files);
    }
}

/// Toyota Way Helper: Check SATD violations
fn check_satd_violations(
    result: &crate::services::satd_detector::SATDAnalysisResult,
    fail_on_violation: bool,
) -> Result<()> {
    if fail_on_violation && !result.items.is_empty() {
        eprintln!(
            "\n❌ SATD violations found: {} technical debt items",
            result.items.len()
        );
        std::process::exit(1);
    }
    Ok(())
}

/// Toyota Way Helper: Filter to top N files with most SATD
fn filter_top_files(
    result: &mut crate::services::satd_detector::SATDAnalysisResult,
    top_files: usize,
) {
    use std::collections::HashMap;

    // Count items per file
    let mut file_counts: HashMap<std::path::PathBuf, usize> = HashMap::new();
    for item in &result.items {
        *file_counts.entry(item.file.clone()).or_insert(0) += 1;
    }

    // Sort and select top files
    let mut sorted_files: Vec<_> = file_counts.into_iter().collect();
    sorted_files.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

    let top_file_paths: std::collections::HashSet<_> = sorted_files
        .into_iter()
        .take(top_files)
        .map(|(path, _)| path)
        .collect();

    // Keep only items from top files
    result
        .items
        .retain(|item| top_file_paths.contains(&item.file));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::satd_detector::{
        DebtCategory, SATDAnalysisResult, SATDSummary, Severity as DetectorSeverity, TechnicalDebt,
    };
    use chrono::Utc;
    use std::collections::HashMap;

    fn debt(file: &str, line: u32, severity: DetectorSeverity) -> TechnicalDebt {
        TechnicalDebt {
            category: DebtCategory::Requirement,
            severity,
            text: "TODO: thing".into(),
            file: PathBuf::from(file),
            line,
            column: 0,
            context_hash: [0u8; 16],
        }
    }

    fn make_result(items: Vec<TechnicalDebt>) -> SATDAnalysisResult {
        let files_with_debt = items
            .iter()
            .map(|i| &i.file)
            .collect::<std::collections::HashSet<_>>()
            .len();
        SATDAnalysisResult {
            total_files_analyzed: 100,
            files_with_debt,
            analysis_timestamp: Utc::now(),
            summary: SATDSummary {
                total_items: items.len(),
                by_severity: HashMap::new(),
                by_category: HashMap::new(),
                files_with_satd: files_with_debt,
                avg_age_days: 0.0,
            },
            items,
        }
    }

    // ── print_satd_analysis_info ────────────────────────────────────────────

    #[test]
    fn test_print_satd_analysis_info_strict_no_panic() {
        // println side-effect — exercise both arms
        print_satd_analysis_info(true, 30);
        print_satd_analysis_info(false, 60);
    }

    // ── apply_satd_filters: severity ────────────────────────────────────────

    #[test]
    fn test_apply_satd_filters_severity_critical_keeps_only_critical() {
        let mut r = make_result(vec![
            debt("a.rs", 1, DetectorSeverity::Critical),
            debt("a.rs", 2, DetectorSeverity::High),
            debt("a.rs", 3, DetectorSeverity::Low),
        ]);
        apply_satd_filters(&mut r, Some(SatdSeverity::Critical), false, 0);
        assert_eq!(r.items.len(), 1);
        assert_eq!(r.items[0].severity, DetectorSeverity::Critical);
    }

    #[test]
    fn test_apply_satd_filters_severity_high_keeps_high_and_critical() {
        let mut r = make_result(vec![
            debt("a.rs", 1, DetectorSeverity::Critical),
            debt("a.rs", 2, DetectorSeverity::High),
            debt("a.rs", 3, DetectorSeverity::Medium),
        ]);
        apply_satd_filters(&mut r, Some(SatdSeverity::High), false, 0);
        assert_eq!(r.items.len(), 2);
    }

    #[test]
    fn test_apply_satd_filters_severity_low_keeps_all() {
        let mut r = make_result(vec![
            debt("a.rs", 1, DetectorSeverity::Critical),
            debt("a.rs", 2, DetectorSeverity::Medium),
            debt("a.rs", 3, DetectorSeverity::Low),
        ]);
        apply_satd_filters(&mut r, Some(SatdSeverity::Low), false, 0);
        assert_eq!(r.items.len(), 3);
    }

    #[test]
    fn test_apply_satd_filters_severity_medium_keeps_med_high_critical() {
        let mut r = make_result(vec![
            debt("a.rs", 1, DetectorSeverity::Critical),
            debt("a.rs", 2, DetectorSeverity::High),
            debt("a.rs", 3, DetectorSeverity::Medium),
            debt("a.rs", 4, DetectorSeverity::Low),
        ]);
        apply_satd_filters(&mut r, Some(SatdSeverity::Medium), false, 0);
        assert_eq!(r.items.len(), 3);
    }

    // ── apply_satd_filters: critical_only ───────────────────────────────────

    #[test]
    fn test_apply_satd_filters_critical_only_drops_others() {
        let mut r = make_result(vec![
            debt("a.rs", 1, DetectorSeverity::Critical),
            debt("a.rs", 2, DetectorSeverity::High),
        ]);
        apply_satd_filters(&mut r, None, true, 0);
        assert_eq!(r.items.len(), 1);
        assert_eq!(r.items[0].severity, DetectorSeverity::Critical);
    }

    // ── apply_satd_filters: top_files ───────────────────────────────────────

    #[test]
    fn test_apply_satd_filters_top_files_keeps_n_files() {
        // 3 files: a.rs has 3 items, b.rs has 2, c.rs has 1. Top 2 → keep a.rs and b.rs.
        let mut r = make_result(vec![
            debt("a.rs", 1, DetectorSeverity::Low),
            debt("a.rs", 2, DetectorSeverity::Low),
            debt("a.rs", 3, DetectorSeverity::Low),
            debt("b.rs", 1, DetectorSeverity::Low),
            debt("b.rs", 2, DetectorSeverity::Low),
            debt("c.rs", 1, DetectorSeverity::Low),
        ]);
        apply_satd_filters(&mut r, None, false, 2);
        assert_eq!(r.items.len(), 5);
        assert!(!r.items.iter().any(|i| i.file == Path::new("c.rs")));
    }

    #[test]
    fn test_apply_satd_filters_top_files_zero_keeps_all() {
        // top_files=0 → no filter applied
        let mut r = make_result(vec![
            debt("a.rs", 1, DetectorSeverity::Low),
            debt("b.rs", 1, DetectorSeverity::Low),
            debt("c.rs", 1, DetectorSeverity::Low),
        ]);
        apply_satd_filters(&mut r, None, false, 0);
        assert_eq!(r.items.len(), 3);
    }

    // ── filter_top_files ────────────────────────────────────────────────────

    #[test]
    fn test_filter_top_files_picks_highest_count() {
        let mut r = make_result(vec![
            debt("a.rs", 1, DetectorSeverity::Low),
            debt("b.rs", 1, DetectorSeverity::Low),
            debt("b.rs", 2, DetectorSeverity::Low),
            debt("b.rs", 3, DetectorSeverity::Low),
        ]);
        filter_top_files(&mut r, 1);
        // Only b.rs (3 items) survives
        assert_eq!(r.items.len(), 3);
        assert!(r.items.iter().all(|i| i.file == Path::new("b.rs")));
    }

    #[test]
    fn test_filter_top_files_n_larger_than_files_keeps_all() {
        let mut r = make_result(vec![
            debt("a.rs", 1, DetectorSeverity::Low),
            debt("b.rs", 1, DetectorSeverity::Low),
        ]);
        filter_top_files(&mut r, 100);
        assert_eq!(r.items.len(), 2);
    }

    // ── check_satd_violations ───────────────────────────────────────────────
    // (calls process::exit on failure — only test the non-failure paths)

    #[test]
    fn test_check_satd_violations_disabled_flag_returns_ok() {
        // fail_on_violation = false → never exits, returns Ok regardless of items
        let r = make_result(vec![debt("a.rs", 1, DetectorSeverity::High)]);
        assert!(check_satd_violations(&r, false).is_ok());
    }

    #[test]
    fn test_check_satd_violations_empty_items_returns_ok() {
        // fail_on_violation = true but no items → returns Ok
        let r = make_result(vec![]);
        assert!(check_satd_violations(&r, true).is_ok());
    }
}
