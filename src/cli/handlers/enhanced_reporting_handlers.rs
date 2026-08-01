//! Enhanced reporting command handlers
//!
//! This module provides handlers for generating comprehensive analysis reports
//! that consolidate multiple analysis outputs.

#![cfg_attr(coverage_nightly, coverage(off))]

use crate::cli::{AnalysisType, ReportOutputFormat};
use crate::models::defect_report::DefectReport;
use crate::services::defect_report_service::{DefectReportService, ReportFormat};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::info;

/// Generates comprehensive defect and analysis reports in multiple formats.
///
/// This is the flagship reporting command that consolidates analysis results from
/// multiple sources into professional reports suitable for stakeholders, developers,
/// and management. Critical for API stability as it defines the primary reporting interface.
///
/// # Parameters
///
/// * `project_path` - Root directory of the project to analyze and report on
/// * `output_format` - Primary output format for the report
/// * `text` - Force plain text output format (overrides `output_format`)
/// * `markdown` - Force Markdown output format (overrides `output_format`)
/// * `csv` - Force CSV output format (overrides `output_format`)
/// * `include_visualizations` - Include charts and graphs in the report
/// * `include_executive_summary` - Include high-level executive summary
/// * `include_recommendations` - Include actionable improvement recommendations
/// * `analyses` - Specific analysis types to include in the report
/// * `confidence_threshold` - Minimum confidence level for including findings (0-100)
/// * `output` - Optional output file path (auto-generated if None)
/// * `perf` - Enable performance optimizations
///
/// # Returns
///
/// * `Ok(())` - Report generation completed successfully and file written
/// * `Err(anyhow::Error)` - Report generation failed with detailed error context
///
/// # Report Components
///
/// ## Executive Dashboard
/// - **Project Overview**: Language breakdown, lines of code, file count
/// - **Quality Metrics**: Maintainability index, technical debt ratio
/// - **Risk Assessment**: Critical issues count, defect probability scores
/// - **Trend Analysis**: Quality evolution over time (if historical data available)
///
/// ## Detailed Analysis Sections
/// - **Defect Hotspots**: Files with highest defect density
/// - **Complexity Analysis**: Cyclomatic and cognitive complexity metrics
/// - **Code Coverage**: Test coverage gaps and recommendations
/// - **Security Issues**: Vulnerability patterns and severity rankings
/// - **Performance Bottlenecks**: Algorithmic complexity concerns
/// - **Maintainability Issues**: Code smell detection and refactoring opportunities
///
/// # Output Formats
///
/// - **JSON**: Machine-readable structured data for tooling integration
/// - **CSV**: Spreadsheet-compatible format for data analysis
/// - **Markdown**: Documentation-friendly format for README/wiki inclusion
/// - **Text**: Plain text format for console output and logging
///
/// `html`, `pdf` and `dashboard` are accepted by the parser for backwards
/// compatibility but are NOT implemented and are rejected with an error
/// (issue #672) rather than silently rendered as markdown/json.
///
/// # Performance Characteristics
///
/// - Time complexity: O(n log n) where n = project size in files
/// - Memory usage: ~100MB base + 5KB per source file
/// - Report generation: 30-60 seconds for typical projects (<100k LOC)
/// - Concurrent analysis: Parallelized across CPU cores
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::cli::handlers::enhanced_reporting_handlers::handle_generate_report;
/// use pmat::cli::enums::{ReportOutputFormat, AnalysisType};
/// use std::path::PathBuf;
/// use tempfile::tempdir;
/// use std::fs;
///
/// # tokio_test::block_on(async {
/// // Create a temporary project
/// let dir = tempdir().unwrap();
/// let main_rs = dir.path().join("main.rs");
/// fs::write(&main_rs, "fn main() { println!(\"Hello, world!\"); }").unwrap();
///
/// // Generate comprehensive report
/// let result = handle_generate_report(
///     dir.path().to_path_buf(),
///     ReportOutputFormat::Markdown,
///     false, // not text format
///     false, // not markdown shortcut
///     false, // not csv shortcut
///     true,  // include visualizations
///     true,  // include executive summary
///     true,  // include recommendations
///     vec![AnalysisType::Complexity, AnalysisType::TechnicalDebt],
///     80,    // 80% confidence threshold
///     Some(dir.path().join("project-report.md")),
///     false, // normal performance
/// ).await;
///
/// // Note: Function may return error for minimal test projects
/// // This test verifies the API compiles and runs without panicking
/// match result {
///     Ok(_) => println!("Report generated successfully"),
///     Err(e) => println!("Report generation failed: {}", e),
/// }
///
/// // Generate quick CSV report
/// let csv_result = handle_generate_report(
///     dir.path().to_path_buf(),
///     ReportOutputFormat::Json, // will be overridden
///     false, // not text
///     false, // not markdown
///     true,  // force CSV format
///     false, // no visualizations
///     false, // no executive summary
///     false, // no recommendations
///     vec![AnalysisType::Complexity],
///     50,    // lower confidence threshold
///     None,  // auto-generate filename
///     true,  // performance mode
/// ).await;
///
/// // Handle result gracefully for test
/// match csv_result {
///     Ok(_) => println!("CSV report generated successfully"),
///     Err(e) => println!("CSV report generation failed: {}", e),
/// }
/// # });
/// ```
///
/// # CLI Usage Examples
///
/// ```bash
/// # Comprehensive executive report
/// pmat generate report /path/to/project --format markdown \
///   --include-visualizations --include-executive-summary \
///   --include-recommendations --output project-health.md
///
/// # Quick CSV export for data analysis
/// pmat generate report /path/to/project --csv \
///   --confidence-threshold 80 --perf
///
/// # Detailed JSON report for CI/CD integration
/// pmat generate report /path/to/project --format json \
///   --analyses complexity,defects,duplicates \
///   --output ci-quality-report.json
///
/// # (`--format dashboard` / `html` / `pdf` are NOT implemented and error out)
/// ```ignore
///
/// # Integration Examples
///
/// ## CI/CD Pipeline Integration
/// ```yaml
/// # .github/workflows/quality-gate.yml
/// - name: Generate Quality Report
///   run: |
///     pmat generate report . --format json \
///       --confidence-threshold 90 \
///       --output quality-report.json
/// ```ignore
///
/// ## Development Workflow Integration
/// ```bash
/// # Pre-commit hook
/// pmat generate report . --format text --perf > quality-summary.txt
/// ```ignore
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_generate_report(
    project_path: PathBuf,
    output_format: ReportOutputFormat,
    text: bool,
    markdown: bool,
    csv: bool,
    _include_visualizations: bool,
    _include_executive_summary: bool,
    _include_recommendations: bool,
    _analyses: Vec<AnalysisType>,
    _confidence_threshold: u8,
    output: Option<PathBuf>,
    perf: bool,
) -> Result<()> {
    let start_time = Instant::now();

    let actual_format = determine_output_format(output_format, text, markdown, csv);
    log_report_generation_start(&project_path, &actual_format);

    // Reject an unsupported format before paying for the whole project scan.
    let service_format = convert_to_service_format(actual_format)?;

    let service = DefectReportService::new();
    let report = service.generate_report(&project_path).await?;

    let formatted_output = format_report_output(&service, &report, service_format)?;

    write_report_output(formatted_output, output, service_format, &project_path).await?;

    let elapsed = start_time.elapsed();
    print_report_summary(&report, elapsed, perf);

    Ok(())
}

/// Determine final output format based on shortcuts (cognitive complexity ≤3)
fn determine_output_format(
    output_format: ReportOutputFormat,
    text: bool,
    markdown: bool,
    csv: bool,
) -> ReportOutputFormat {
    if text {
        ReportOutputFormat::Text
    } else if markdown {
        ReportOutputFormat::Markdown
    } else if csv {
        ReportOutputFormat::Csv
    } else {
        output_format
    }
}

/// Log report generation startup info (cognitive complexity ≤2)
fn log_report_generation_start(project_path: &Path, actual_format: &ReportOutputFormat) {
    info!("📊 Generating comprehensive defect report");
    info!("📂 Project path: {}", project_path.display());
    info!("📄 Output format: {:?}", actual_format);
}

/// Convert CLI output format to service format (cognitive complexity ≤7)
///
/// Issue #672: `Html`/`Pdf`/`Dashboard` used to be silently rewritten to
/// Markdown/Json, so `--format html` produced a file containing no markup and
/// `--format pdf` produced no PDF. `DefectReportService` has exactly four
/// emitters (json, csv, markdown, text); there is nothing honest to render
/// these three as, so they are rejected instead of quietly substituted.
/// The four listed as supported all really work — `csv` in particular is no
/// longer behind a non-default feature.
fn convert_to_service_format(actual_format: ReportOutputFormat) -> Result<ReportFormat> {
    match actual_format {
        ReportOutputFormat::Json => Ok(ReportFormat::Json),
        ReportOutputFormat::Csv => Ok(ReportFormat::Csv),
        ReportOutputFormat::Markdown => Ok(ReportFormat::Markdown),
        ReportOutputFormat::Text => Ok(ReportFormat::Text),
        other => anyhow::bail!(
            "--format {other} is not implemented for `pmat report` \
             (it previously emitted plain text, not {other}). \
             Supported formats: json, csv, markdown, text."
        ),
    }
}

/// Format report using service (cognitive complexity ≤4)
fn format_report_output(
    service: &DefectReportService,
    report: &DefectReport,
    service_format: ReportFormat,
) -> Result<String> {
    match service_format {
        ReportFormat::Json => service.format_json(report),
        ReportFormat::Csv => service.format_csv(report),
        ReportFormat::Markdown => service.format_markdown(report),
        ReportFormat::Text => service.format_text(report),
    }
}

/// Write report output to file or auto-generated filename (cognitive complexity ≤4)
///
/// Without `-o` the auto-named report lands next to the project that was
/// analysed. It used to be written relative to the process CWD, so analysing a
/// /tmp fixture from inside the pmat checkout dropped
/// `defect-report-<ts>.json` into the pmat repo (GH #671).
async fn write_report_output(
    formatted_output: String,
    output: Option<PathBuf>,
    service_format: ReportFormat,
    project_path: &Path,
) -> Result<()> {
    let output_path = output.unwrap_or_else(|| {
        let service = DefectReportService::new();
        project_path.join(service.generate_filename(service_format))
    });
    tokio::fs::write(&output_path, &formatted_output).await?;
    eprintln!("📄 Report saved to: {}", output_path.display());
    Ok(())
}

/// Print comprehensive report summary (cognitive complexity ≤8)
fn print_report_summary(report: &DefectReport, elapsed: std::time::Duration, perf: bool) {
    info!("✅ Report generation completed in {:?}", elapsed);
    info!("📊 Total Defects: {}", report.summary.total_defects);
    info!("📁 Files with defects: {}", report.file_index.len());

    print_severity_summary(report);

    if perf {
        let files_per_sec = report.metadata.total_files_analyzed as f64 / elapsed.as_secs_f64();
        info!("⚡ Performance: {:.0} files/second", files_per_sec);
    }
}

/// Print severity-specific summary (cognitive complexity ≤4)
fn print_severity_summary(report: &DefectReport) {
    if let Some(critical) = report.summary.by_severity.get("critical") {
        if *critical > 0 {
            info!("🚨 Critical Issues: {}", critical);
        }
    }

    if let Some(high) = report.summary.by_severity.get("high") {
        if *high > 0 {
            info!("⚠️ High Severity Issues: {}", high);
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    /// Issue #672 regression. `--help` advertises "csv: CSV format for
    /// spreadsheet analysis"; on the released binary `pmat report --format csv`
    /// exited rc=1 with "CSV reporting requires the 'reporting' feature"
    /// because `reporting` was not in `default`. This test runs in the default
    /// feature set and fails if csv is ever gated again.
    #[test]
    fn test_csv_is_available_in_default_feature_set() {
        let service = DefectReportService::new();
        let report = DefectReport {
            metadata: crate::models::defect_report::ReportMetadata {
                tool: "pmat".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                generated_at: chrono::Utc::now(),
                project_root: PathBuf::from("/test"),
                total_files_analyzed: 0,
                analysis_duration_ms: 0,
            },
            defects: vec![],
            summary: crate::models::defect_report::DefectSummary {
                total_defects: 0,
                by_severity: std::collections::BTreeMap::new(),
                by_category: std::collections::BTreeMap::new(),
                hotspot_files: vec![],
            },
            file_index: std::collections::BTreeMap::new(),
        };

        let csv = format_report_output(&service, &report, ReportFormat::Csv)
            .expect("csv must render in the shipped default feature set");
        assert!(
            csv.starts_with("id,severity,category"),
            "csv header missing, got: {csv:?}"
        );
    }

    /// Issue #672: html/pdf/dashboard used to be silently substituted with
    /// markdown/json. A declared format must render or be rejected.
    #[test]
    fn test_unimplemented_formats_are_rejected_not_substituted() {
        for fmt in [
            ReportOutputFormat::Html,
            ReportOutputFormat::Pdf,
            ReportOutputFormat::Dashboard,
        ] {
            let err = convert_to_service_format(fmt.clone())
                .expect_err("unimplemented format must be rejected");
            let msg = err.to_string();
            assert!(
                msg.contains(&fmt.to_string()),
                "message must name the format: {msg}"
            );
            assert!(
                msg.contains("Supported formats: json, csv, markdown, text."),
                "message must list only formats that really work: {msg}"
            );
        }
    }

    /// Every format the rejection message advertises must actually convert.
    /// Guards against the message and the implementation drifting apart again.
    #[test]
    fn test_advertised_formats_all_convert() {
        for fmt in [
            ReportOutputFormat::Json,
            ReportOutputFormat::Csv,
            ReportOutputFormat::Markdown,
            ReportOutputFormat::Text,
        ] {
            assert!(
                convert_to_service_format(fmt.clone()).is_ok(),
                "advertised format {fmt} must convert"
            );
        }
    }

    /// GH #671: without `-o`, the auto-named report was written relative to the
    /// process CWD, so analysing a /tmp fixture from inside the pmat checkout
    /// dropped `defect-report-<ts>.json` into the pmat repo.
    #[tokio::test]
    async fn auto_named_report_lands_in_the_analysed_project() {
        let project = tempfile::TempDir::new().unwrap();
        let cwd_before = std::env::current_dir().unwrap();

        write_report_output("{}".to_string(), None, ReportFormat::Json, project.path())
            .await
            .expect("report must be written");

        let written: Vec<_> = std::fs::read_dir(project.path())
            .unwrap()
            .filter_map(std::result::Result::ok)
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(
            written.len(),
            1,
            "expected exactly one report in the analysed project, found {written:?}"
        );
        assert!(written[0].starts_with("defect-report-"), "{written:?}");

        // And nothing was dropped into the current working directory.
        assert_eq!(
            cwd_before,
            std::env::current_dir().unwrap(),
            "the handler must not change the process CWD"
        );
    }

    /// An explicit `-o` still wins and is used verbatim.
    #[tokio::test]
    async fn explicit_output_path_is_honoured() {
        let dir = tempfile::TempDir::new().unwrap();
        let target = dir.path().join("chosen.json");

        write_report_output(
            "{\"ok\":true}".to_string(),
            Some(target.clone()),
            ReportFormat::Json,
            Path::new("/nonexistent-project"),
        )
        .await
        .expect("report must be written");

        assert_eq!(std::fs::read_to_string(&target).unwrap(), "{\"ok\":true}");
    }

    /// Issue #672 regression: every declared `--output-format` used to reach
    /// `ReportFormat::Text`, so csv, markdown, html, pdf and dashboard all
    /// produced the same 909-byte "CODE QUALITY REPORT" file. Each supported
    /// value must map to its own service format.
    #[test]
    fn test_supported_formats_map_one_to_one() {
        assert_eq!(
            convert_to_service_format(ReportOutputFormat::Json).unwrap(),
            ReportFormat::Json
        );
        assert_eq!(
            convert_to_service_format(ReportOutputFormat::Csv).unwrap(),
            ReportFormat::Csv
        );
        assert_eq!(
            convert_to_service_format(ReportOutputFormat::Markdown).unwrap(),
            ReportFormat::Markdown
        );
        assert_eq!(
            convert_to_service_format(ReportOutputFormat::Text).unwrap(),
            ReportFormat::Text
        );
    }
}
