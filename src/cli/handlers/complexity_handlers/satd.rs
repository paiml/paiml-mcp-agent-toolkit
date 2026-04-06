//! SATD (Self-Admitted Technical Debt) integration in complexity context.

use crate::cli::{SatdOutputFormat, SatdSeverity};
use anyhow::Result;
use std::path::{Path, PathBuf};

use super::output::{format_satd_output, write_satd_output};

/// Handle SATD (Self-Admitted Technical Debt) analysis command
#[allow(clippy::too_many_arguments)]
/// Toyota Way: Extract Method - Handle SATD analysis (complexity <=8)
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
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    // Print analysis info
    print_satd_analysis_info(strict, timeout);

    // Run SATD analysis
    let mut result = run_satd_analysis(&path, include_tests, strict, timeout).await?;

    // Apply filters
    apply_satd_filters(&mut result, severity, critical_only, top_files);

    eprintln!(
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
    eprintln!("🔍 Analyzing self-admitted technical debt...");
    eprintln!("⏰ Analysis timeout set to {timeout} seconds");
    if strict {
        eprintln!("📝 Using strict mode (only explicit SATD markers)");
    }
}

/// Toyota Way Helper: Run SATD analysis with timeout
async fn run_satd_analysis(
    path: &Path,
    include_tests: bool,
    strict: bool,
    timeout: u64,
) -> Result<crate::services::satd_detector::SATDAnalysisResult> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    use crate::services::satd_detector::SATDDetector;

    // Create detector
    let detector = if strict {
        SATDDetector::new_strict()
    } else {
        SATDDetector::new()
    };

    // Run with timeout
    let timeout_duration = tokio::time::Duration::from_secs(timeout);
    let result = tokio::time::timeout(timeout_duration, async {
        detector.analyze_project(path, include_tests).await
    })
    .await
    .map_err(|_| anyhow::anyhow!("SATD analysis timed out after {timeout} seconds"))??;

    Ok(result)
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
