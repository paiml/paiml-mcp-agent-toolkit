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
/// * `output` - Optional output file path; when None the report is written to
///   stdout and NO file is created (a measurement tool must not write into the
///   tree it measures — the old auto-named artifact inflated that tree's TDG)
/// * `perf` - Enable performance optimizations
///
/// # Returns
///
/// * `Ok(())` - Report generation completed successfully
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
/// - **HTML**: Web-ready format with embedded visualizations (legacy)
/// - **PDF**: Print-ready format for formal reports (legacy)
/// - **Dashboard**: Interactive web dashboard format (legacy)
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
///     false, // include visualizations (rejected: not implemented for any format)
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
///     None,  // no --output: report goes to stdout, no file is created
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
///   --include-executive-summary \
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
/// # Management dashboard (legacy HTML format)
/// pmat generate report /path/to/project --format dashboard \
///   --include-executive-summary
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
    include_visualizations: bool,
    _include_executive_summary: bool,
    _include_recommendations: bool,
    _analyses: Vec<AnalysisType>,
    _confidence_threshold: u8,
    output: Option<PathBuf>,
    perf: bool,
) -> Result<()> {
    // A nonexistent --project-path used to produce a green 0-defect report with
    // exit 0, shape-identical to a real report on an empty project: nothing in
    // the pipeline can tell "no defects" from "no such directory". Fail like
    // deps-audit and five-whys already do.
    crate::cli::ensure_analysis_path_exists(&project_path)?;

    let start_time = Instant::now();

    let actual_format = determine_output_format(output_format, text, markdown, csv);
    reject_unimplemented_visualizations(include_visualizations)?;
    log_report_generation_start(&project_path, &actual_format);

    let service = DefectReportService::new();
    let report = service.generate_report(&project_path).await?;

    let service_format = convert_to_service_format(actual_format)?;
    let formatted_output = format_report_output(&service, &report, service_format)?;

    write_report_output(formatted_output, output).await?;

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
/// Map a declared `--output-format` to an emitter, or reject it.
///
/// #672: html, pdf and dashboard were silently rewritten to Markdown/Json, so
/// `--format html` produced a file containing no markup and the user was never
/// told. A declared format must produce that format or be refused -- silently
/// emitting a different one is the defect. (A merge reverted this once; the
/// test in tests_report_format_fidelity.rs pins it.)
fn convert_to_service_format(actual_format: ReportOutputFormat) -> Result<ReportFormat> {
    Ok(match actual_format {
        ReportOutputFormat::Json => ReportFormat::Json,
        ReportOutputFormat::Csv => ReportFormat::Csv,
        ReportOutputFormat::Markdown => ReportFormat::Markdown,
        ReportOutputFormat::Text => ReportFormat::Text,
        unsupported => anyhow::bail!(
            "--format {} is not implemented for `pmat report` (it previously emitted \
             plain text, not {}). Supported formats: json, csv, markdown, text.",
            format!("{unsupported:?}").to_lowercase(),
            format!("{unsupported:?}").to_lowercase(),
        ),
    })
}

/// Refuse `--include-visualizations` rather than accepting it and doing nothing.
///
/// The flag was taken as `_include_visualizations` and dropped: `pmat report -f
/// markdown` and `pmat report -f markdown --include-visualizations` produced
/// byte-identical output, and the same held for json, csv and text. The three
/// formats that could plausibly carry a chart (html, pdf, dashboard) are
/// themselves rejected by `convert_to_service_format` (#672), so there is no
/// format left for this flag to affect. Same rule as that rejection: a declared
/// option must do what it says or be refused, never silently no-op.
fn reject_unimplemented_visualizations(include_visualizations: bool) -> Result<()> {
    if include_visualizations {
        anyhow::bail!(
            "--include-visualizations is not implemented for `pmat report`: it changed \
             nothing in json, csv, markdown or text, and the formats that could embed \
             charts (html, pdf, dashboard) are not implemented either. Re-run without it."
        );
    }
    Ok(())
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

/// Where a generated report is delivered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReportSink {
    /// Explicit `--output <PATH>` given by the caller.
    File(PathBuf),
    /// No `--output`: the report goes to stdout and NOTHING is written to disk.
    Stdout,
}

/// Decide where the report goes.
///
/// Round-3 defect: with no `--output` the command used to drop an
/// auto-named `defect-report-<timestamp>.<ext>` artifact into the working
/// directory — which for the common `pmat report -p .` invocation is the tree
/// being measured. TDG scores `.md` files, so pmat graded its own output as
/// project source and the score climbed on every run: on one fixture
/// 74.78 (B-) with 0 reports present -> 84.83 (B+) -> 88.18 (A-) -> 89.86 (A-)
/// -> 90.87 (A) after four invocations, without a line of source changing.
/// A measurement tool must not write into the tree it measures, so an
/// unrequested artifact is never created; stdout is the documented usage
/// (`pmat report --json > defect-report.json`).
pub(crate) fn report_sink(output: Option<PathBuf>) -> ReportSink {
    output.map_or(ReportSink::Stdout, ReportSink::File)
}

/// Write report output to the explicit `--output` file, else to stdout.
async fn write_report_output(formatted_output: String, output: Option<PathBuf>) -> Result<()> {
    match report_sink(output) {
        ReportSink::File(output_path) => {
            tokio::fs::write(&output_path, &formatted_output).await?;
            eprintln!("📄 Report saved to: {}", output_path.display());
        }
        ReportSink::Stdout => println!("{formatted_output}"),
    }
    Ok(())
}

/// Print comprehensive report summary (cognitive complexity ≤8)
fn print_report_summary(report: &DefectReport, elapsed: std::time::Duration, perf: bool) {
    info!("✅ Report generation completed in {:?}", elapsed);
    info!("📊 Total Defects: {}", report.summary.total_defects);
    info!("📁 Files with defects: {}", report.file_index.len());

    print_severity_summary(report);

    for line in report_perf_lines(report, elapsed, perf) {
        eprintln!("{line}");
    }
}

/// The lines `pmat report --perf` adds, in emission order; empty without the flag.
///
/// This used to be a single `info!("⚡ Performance: {:.0} files/second", …)`,
/// which the default `warn`-level EnvFilter discards, so `pmat report --perf`
/// was byte-identical to `pmat report` at json, text, markdown and csv alike —
/// the same defect `analyze big-o --perf` had, and the reason the analyze fix's
/// verifier came back. The wording and the stderr sink now come from the one
/// `--perf` implementation (`perf_report`), so this command cannot drift from
/// the thirteen `analyze` ones. Returning the lines rather than printing them
/// keeps the flag's effect assertable in a unit test.
pub(crate) fn report_perf_lines(
    report: &DefectReport,
    elapsed: std::time::Duration,
    perf: bool,
) -> Vec<String> {
    if !perf {
        return Vec::new();
    }
    let files_per_sec = report.metadata.total_files_analyzed as f64 / elapsed.as_secs_f64();
    crate::cli::handlers::analysis_handlers::perf_report::readout_lines(
        "report",
        elapsed,
        perf,
        &[("throughput", format!("{files_per_sec:.0} files/second"))],
    )
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

    #[test]
    fn test_enhanced_reporting_handlers_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }

    /// Regression: `pmat report` with no `--output` must not create a file.
    /// It used to auto-write `defect-report-<ts>.md` into the working
    /// directory — usually the tree being measured — which TDG then graded as
    /// project source (74.78 B- -> 90.87 A over four runs on one fixture).
    fn report_with_files(total_files_analyzed: usize) -> DefectReport {
        use crate::models::defect_report::{DefectSummary, ReportMetadata};
        use std::collections::BTreeMap;
        DefectReport {
            metadata: ReportMetadata {
                tool: "pmat".to_string(),
                version: "test".to_string(),
                generated_at: chrono::Utc::now(),
                project_root: PathBuf::from("."),
                total_files_analyzed,
                analysis_duration_ms: 0,
            },
            defects: vec![],
            summary: DefectSummary {
                total_defects: 0,
                by_severity: BTreeMap::new(),
                by_category: BTreeMap::new(),
                hotspot_files: vec![],
            },
            file_index: BTreeMap::new(),
        }
    }

    /// Regression: `pmat report --perf` was byte-identical to `pmat report` at
    /// json, text, markdown and csv, because its only perf line was an `info!`
    /// the default `warn` EnvFilter discards. The flag must add a readout that
    /// reaches the user, and add nothing when it is absent.
    #[test]
    fn test_perf_adds_a_readout_the_default_log_filter_cannot_swallow() {
        let report = report_with_files(20);
        let elapsed = std::time::Duration::from_millis(100);

        assert!(
            report_perf_lines(&report, elapsed, false).is_empty(),
            "`pmat report` without --perf must stay byte-identical"
        );

        let lines = report_perf_lines(&report, elapsed, true);
        assert!(
            !lines.is_empty(),
            "--perf produced no readout at all, which is the defect: the flag \
             parsed and changed nothing"
        );
        let joined = lines.join("\n");
        assert!(joined.contains("perf: report"), "got: {joined}");
        assert!(
            joined.contains("200 files/second"),
            "the throughput measurement must survive: 20 files / 0.1s, got: {joined}"
        );
    }

    /// The readout must be the shared one, not a fourth wording. Same shape as
    /// the `analyze` router's line, so a caller can grep one prefix.
    #[test]
    fn test_perf_readout_matches_the_shared_implementation() {
        let elapsed = std::time::Duration::from_millis(100);
        let lines = report_perf_lines(&report_with_files(1), elapsed, true);
        assert_eq!(
            lines[0],
            crate::cli::handlers::analysis_handlers::perf_report::format_line("report", elapsed),
        );
    }

    #[test]
    fn test_report_sink_defaults_to_stdout_not_a_generated_file() {
        assert_eq!(report_sink(None), ReportSink::Stdout);
    }

    #[test]
    fn test_report_sink_honors_explicit_output_path() {
        let path = PathBuf::from("/tmp/explicit-report.md");
        assert_eq!(report_sink(Some(path.clone())), ReportSink::File(path));
    }

    /// Count `defect-report-*` artifacts sitting in the working directory.
    fn auto_artifacts_in_cwd() -> Vec<PathBuf> {
        let cwd = std::env::current_dir().expect("cwd");
        std::fs::read_dir(cwd)
            .expect("read_dir")
            .filter_map(std::result::Result::ok)
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("defect-report-"))
            })
            .collect()
    }

    /// The default path must leave the working (== analysed) directory
    /// unchanged, so a second measurement of the same tree sees the same files.
    /// Pre-fix this call dropped `defect-report-<ts>.<ext>` next to the source.
    #[tokio::test]
    async fn test_write_report_output_creates_no_artifact_without_output_flag() {
        let before = auto_artifacts_in_cwd();

        write_report_output("# report body\n".to_string(), None)
            .await
            .expect("write");

        let after = auto_artifacts_in_cwd();
        assert_eq!(
            before.len(),
            after.len(),
            "report must not drop an artifact into the tree it measures (before={before:?}, after={after:?})"
        );
    }

    /// `pmat report -p /does/not/exist -f json` used to print a clean
    /// 0-defect document and exit 0 — shape-identical to a real report on an
    /// empty project, with nothing on stderr. A path that is not there is an
    /// error, not a clean bill of health.
    #[tokio::test]
    async fn test_generate_report_rejects_a_nonexistent_project_path() {
        let missing = PathBuf::from("/does/not/exist-9f3a-pmat");
        let result = handle_generate_report(
            missing,
            ReportOutputFormat::Json,
            false,
            false,
            false,
            false,
            false,
            false,
            vec![],
            0,
            None,
            false,
        )
        .await;

        let err = result.expect_err("a nonexistent project path must be an error");
        assert!(
            err.to_string().contains("not found"),
            "unexpected error: {err}"
        );
    }

    /// `--include-visualizations` was taken as `_include_visualizations` and
    /// dropped, so `-f markdown` and `-f markdown --include-visualizations`
    /// produced byte-identical output. A flag that cannot do anything must say
    /// so, like `--format html` already does.
    #[test]
    fn test_include_visualizations_is_rejected_not_silently_ignored() {
        assert!(reject_unimplemented_visualizations(false).is_ok());

        let err = reject_unimplemented_visualizations(true)
            .expect_err("--include-visualizations affects no format and must be refused");
        assert!(
            err.to_string().contains("--include-visualizations"),
            "the error must name the flag: {err}"
        );
    }

    /// End-to-end on the handler: the flag must never reach a successful run.
    #[tokio::test]
    async fn test_generate_report_refuses_the_visualizations_flag() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("lib.rs"), "fn main() {}").expect("write");

        let result = handle_generate_report(
            dir.path().to_path_buf(),
            ReportOutputFormat::Markdown,
            false,
            false,
            false,
            true, // --include-visualizations
            false,
            false,
            vec![],
            50,
            None,
            false,
        )
        .await;

        let err = result.expect_err("--include-visualizations must not succeed as a no-op");
        assert!(
            err.to_string().contains("not implemented"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn test_write_report_output_writes_explicit_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("out.md");
        write_report_output("# body\n".to_string(), Some(target.clone()))
            .await
            .expect("write");
        assert_eq!(
            std::fs::read_to_string(&target).expect("read back"),
            "# body\n"
        );
    }
}
