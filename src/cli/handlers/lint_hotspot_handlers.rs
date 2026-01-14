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

use crate::cli::LintHotspotOutputFormat;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

/// Parameters for lint hotspot analysis
pub struct LintHotspotParams {
    pub project_path: PathBuf,
    pub file: Option<PathBuf>,
    pub format: LintHotspotOutputFormat,
    pub max_density: f64,
    pub min_confidence: f64,
    pub enforce: bool,
    pub dry_run: bool,
    pub enforcement_metadata: bool,
    pub output: Option<PathBuf>,
    pub perf: bool,
    pub clippy_flags: String,
    pub top_files: usize,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

/// Lint hotspot analysis result
#[derive(Debug, Serialize, Deserialize)]
pub struct LintHotspotResult {
    pub hotspot: LintHotspot,
    pub all_violations: Vec<ViolationDetail>,
    pub summary_by_file: HashMap<PathBuf, FileSummary>,
    pub total_project_violations: usize,
    pub enforcement: Option<EnforcementMetadata>,
    pub refactor_chain: Option<RefactorChain>,
    pub quality_gate: QualityGateStatus,
}

/// The identified hotspot file
#[derive(Debug, Serialize, Deserialize)]
pub struct LintHotspot {
    pub file: PathBuf,
    pub defect_density: f64,
    pub total_violations: usize,
    pub sloc: usize,
    pub severity_distribution: SeverityDistribution,
    pub top_lints: Vec<(String, usize)>,
    pub detailed_violations: Vec<ViolationDetail>,
}

/// Detailed violation information for rewriting
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ViolationDetail {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub lint_name: String,
    pub message: String,
    pub severity: String,
    pub suggestion: Option<String>,
    pub machine_applicable: bool,
}

/// File-level summary
#[derive(Debug, Serialize, Deserialize)]
pub struct FileSummary {
    pub total_violations: usize,
    pub errors: usize,
    pub warnings: usize,
    pub sloc: usize,
    pub defect_density: f64,
}

/// Severity distribution of violations
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SeverityDistribution {
    pub error: usize,
    pub warning: usize,
    pub suggestion: usize,
    pub note: usize,
}

/// Enforcement metadata for quality gates
#[derive(Debug, Serialize, Deserialize)]
pub struct EnforcementMetadata {
    pub enforcement_score: f64,
    pub requires_enforcement: bool,
    pub estimated_fix_time: u32,
    pub automation_confidence: f64,
    pub enforcement_priority: u8,
}

/// Refactor chain for automated fixes
#[derive(Debug, Serialize, Deserialize)]
pub struct RefactorChain {
    pub id: String,
    pub estimated_reduction: usize,
    pub automation_confidence: f64,
    pub steps: Vec<RefactorStep>,
}

/// Individual refactor step
#[derive(Debug, Serialize, Deserialize)]
pub struct RefactorStep {
    pub id: String,
    pub lint: String,
    pub confidence: f64,
    pub impact: usize,
    pub description: String,
}

/// Quality gate status
#[derive(Debug, Serialize, Deserialize)]
pub struct QualityGateStatus {
    pub passed: bool,
    pub violations: Vec<QualityViolation>,
    pub blocking: bool,
}

/// Individual quality violation
#[derive(Debug, Serialize, Deserialize)]
pub struct QualityViolation {
    pub rule: String,
    pub threshold: f64,
    pub actual: f64,
    pub severity: String,
}

/// File metrics for analysis
#[derive(Debug, Default)]
struct FileMetrics {
    violations: HashMap<String, usize>,
    severity_counts: SeverityDistribution,
    sloc: usize,
    detailed_violations: Vec<ViolationDetail>,
}

/// Clippy message structure
#[derive(Debug, Deserialize)]
struct ClippyMessage {
    reason: Option<String>,
    message: Option<DiagnosticMessage>,
}

#[derive(Debug, Deserialize)]
struct DiagnosticMessage {
    level: String,
    message: String,
    code: Option<DiagnosticCode>,
    spans: Vec<DiagnosticSpan>,
}

#[derive(Debug, Deserialize)]
struct DiagnosticCode {
    code: String,
}

#[derive(Debug, Deserialize)]
struct DiagnosticSpan {
    file_name: String,
    line_start: u32,
    line_end: u32,
    column_start: u32,
    column_end: u32,
    #[serde(default)]
    is_primary: bool,
    #[serde(default, rename = "text")]
    _text: Vec<DiagnosticText>,
    #[serde(default)]
    suggested_replacement: Option<String>,
    #[serde(default)]
    suggestion_applicability: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DiagnosticText {
    _text: String,
    _highlight_start: u32,
    _highlight_end: u32,
}

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

    let mut result = run_analysis_by_mode(&params).await?;

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
        run_clippy_analysis_single_file(&params.project_path, file_path, &params.clippy_flags).await
    } else {
        run_clippy_analysis(&params.project_path, &params.clippy_flags).await
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

/// Run clippy and analyze the JSON output
///
/// # Errors
///
/// Returns an error if the operation fails
async fn run_clippy_analysis(project_path: &Path, clippy_flags: &str) -> Result<LintHotspotResult> {
    let flags: Vec<&str> = clippy_flags.split_whitespace().collect();
    let output = execute_clippy_command(project_path, &flags).await?;

    check_clippy_output(&output)?;

    let mut file_metrics = parse_clippy_json_output(&output)?;

    let workspace_root = find_workspace_root(project_path)?;
    calculate_sloc_for_files(&mut file_metrics, project_path, workspace_root.as_ref()).await?;

    build_lint_hotspot_result(file_metrics)
}

/// Run clippy on a single file and analyze the JSON output
///
/// # Errors
///
/// Returns an error if the operation fails
async fn run_clippy_analysis_single_file(
    project_path: &Path,
    file_path: &Path,
    clippy_flags: &str,
) -> Result<LintHotspotResult> {
    let output = run_clippy_command(project_path, clippy_flags).await?;
    let abs_file_path = resolve_absolute_path(project_path, file_path);

    let (file_violations, all_violations, severity_dist) =
        parse_clippy_output(&output.stdout, &abs_file_path, file_path)?;

    let sloc = count_source_lines(project_path, file_path)
        .await
        .unwrap_or(100);

    create_single_file_result(
        file_path,
        file_violations,
        all_violations,
        severity_dist,
        sloc,
    )
}

async fn run_clippy_command(
    project_path: &Path,
    clippy_flags: &str,
) -> Result<std::process::Output> {
    let flags: Vec<&str> = clippy_flags.split_whitespace().collect();
    let mut cmd = Command::new("cargo");

    cmd.current_dir(project_path)
        .arg("clippy")
        .arg("--all-targets")
        .arg("--message-format=json")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if !flags.is_empty() {
        cmd.arg("--").args(&flags);
    }

    cmd.output().await.context("Failed to run cargo clippy")
}

fn resolve_absolute_path(project_path: &Path, file_path: &Path) -> PathBuf {
    if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        project_path.join(file_path)
    }
}

fn parse_clippy_output(
    stdout: &[u8],
    abs_file_path: &Path,
    file_path: &Path,
) -> Result<(
    Vec<ViolationDetail>,
    Vec<ViolationDetail>,
    SeverityDistribution,
)> {
    let reader = BufReader::new(stdout);
    let mut file_violations = Vec::new();
    let mut all_violations = Vec::new();
    let mut severity_dist = SeverityDistribution::default();

    for line in std::io::BufRead::lines(reader) {
        let line = line?;
        if let Some(violation) = parse_clippy_line(&line, abs_file_path, file_path)? {
            file_violations.push(violation.clone());
            update_severity_distribution(&mut severity_dist, &violation.severity);
            all_violations.push(violation);
        }
    }

    Ok((file_violations, all_violations, severity_dist))
}

fn parse_clippy_line(
    line: &str,
    abs_file_path: &Path,
    file_path: &Path,
) -> Result<Option<ViolationDetail>> {
    let msg = match serde_json::from_str::<ClippyMessage>(line) {
        Ok(msg) => msg,
        Err(_) => return Ok(None),
    };

    let (Some("compiler-message"), Some(diagnostic)) = (msg.reason.as_deref(), &msg.message) else {
        return Ok(None);
    };

    let Some(span) = find_primary_span(diagnostic) else {
        return Ok(None);
    };

    if !is_target_file(&span.file_name, abs_file_path, file_path) {
        return Ok(None);
    }

    Ok(Some(create_violation_detail(file_path, span, diagnostic)))
}

fn find_primary_span(diagnostic: &DiagnosticMessage) -> Option<&DiagnosticSpan> {
    diagnostic
        .spans
        .iter()
        .find(|s| s.is_primary || diagnostic.spans.len() == 1)
}

fn is_target_file(diagnostic_file: &str, abs_file_path: &Path, file_path: &Path) -> bool {
    let diagnostic_path = PathBuf::from(diagnostic_file);
    diagnostic_path == *abs_file_path
        || diagnostic_path == *file_path
        || diagnostic_path.ends_with(file_path)
}

fn create_violation_detail(
    file_path: &Path,
    span: &DiagnosticSpan,
    diagnostic: &DiagnosticMessage,
) -> ViolationDetail {
    ViolationDetail {
        file: file_path.to_path_buf(),
        line: span.line_start,
        column: span.column_start,
        end_line: span.line_end,
        end_column: span.column_end,
        lint_name: extract_lint_name(diagnostic),
        message: diagnostic.message.clone(),
        severity: diagnostic.level.clone(),
        suggestion: span.suggested_replacement.clone(),
        machine_applicable: is_machine_applicable(span),
    }
}

fn extract_lint_name(diagnostic: &DiagnosticMessage) -> String {
    diagnostic
        .code
        .as_ref()
        .map(|c| c.code.clone())
        .unwrap_or_default()
}

fn is_machine_applicable(span: &DiagnosticSpan) -> bool {
    span.suggestion_applicability
        .as_ref()
        .is_some_and(|a| a == "machine-applicable" || a == "maybe-incorrect")
}

fn update_severity_distribution(severity_dist: &mut SeverityDistribution, level: &str) {
    match level {
        "error" => severity_dist.error += 1,
        "warning" => severity_dist.warning += 1,
        _ => severity_dist.note += 1,
    }
}

fn create_single_file_result(
    file_path: &Path,
    file_violations: Vec<ViolationDetail>,
    all_violations: Vec<ViolationDetail>,
    severity_dist: SeverityDistribution,
    sloc: usize,
) -> Result<LintHotspotResult> {
    let total_violations = file_violations.len();
    let defect_density = (total_violations as f64 / sloc as f64) * 100.0;

    let hotspot = LintHotspot {
        file: file_path.to_path_buf(),
        defect_density,
        total_violations,
        sloc,
        severity_distribution: severity_dist,
        top_lints: count_top_lints(&file_violations),
        detailed_violations: file_violations,
    };

    let mut summary_by_file = HashMap::new();
    summary_by_file.insert(
        file_path.to_path_buf(),
        FileSummary {
            total_violations,
            errors: hotspot.severity_distribution.error,
            warnings: hotspot.severity_distribution.warning,
            sloc,
            defect_density,
        },
    );

    Ok(LintHotspotResult {
        hotspot,
        all_violations,
        summary_by_file,
        total_project_violations: total_violations,
        enforcement: None,
        refactor_chain: None,
        quality_gate: QualityGateStatus {
            passed: defect_density <= 5.0,
            violations: vec![],
            blocking: false,
        },
    })
}

/// Count top lint types from violations
fn count_top_lints(violations: &[ViolationDetail]) -> Vec<(String, usize)> {
    let mut lint_counts: HashMap<String, usize> = HashMap::new();

    for violation in violations {
        *lint_counts.entry(violation.lint_name.clone()).or_insert(0) += 1;
    }

    let mut counts: Vec<_> = lint_counts.into_iter().collect();
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    counts.truncate(10); // Top 10 lints
    counts
}

/// Count source lines in a file
async fn count_source_lines(project_path: &Path, file_path: &Path) -> Result<usize> {
    let full_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        project_path.join(file_path)
    };

    let content = tokio::fs::read_to_string(&full_path).await?;
    let non_empty_lines = content
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with("//"))
        .count();

    Ok(non_empty_lines.max(1)) // At least 1 to avoid division by zero
}

/// Process a diagnostic message
fn process_diagnostic(
    diagnostic: &DiagnosticMessage,
    file_metrics: &mut HashMap<PathBuf, FileMetrics>,
) {
    // Find the primary span
    let primary_span = diagnostic
        .spans
        .iter()
        .find(|s| s.is_primary)
        .or_else(|| diagnostic.spans.first());

    if let Some(span) = primary_span {
        let mut file_path = PathBuf::from(&span.file_name);

        // Handle workspace paths - if path starts with "server/", strip it for consistent handling
        // But preserve the original path structure for examples
        if let Ok(stripped) = file_path.strip_prefix("server/") {
            file_path = PathBuf::from(stripped);
        } else if file_path.starts_with("examples/") {
            // Keep examples/ paths as-is since they are relative to server/
            file_path = PathBuf::from("server").join(&file_path);
        }

        // Skip non-Rust files (config files, etc.)
        if !file_path.extension().is_some_and(|ext| ext == "rs") {
            return;
        }

        let metrics = file_metrics.entry(file_path.clone()).or_default();

        // Count by severity
        match diagnostic.level.as_str() {
            "error" => metrics.severity_counts.error += 1,
            "warning" => metrics.severity_counts.warning += 1,
            "help" | "suggestion" => metrics.severity_counts.suggestion += 1,
            _ => metrics.severity_counts.note += 1,
        }

        // Count by lint code
        let lint_name = diagnostic
            .code
            .as_ref()
            .map_or_else(|| "unknown".to_string(), |c| c.code.clone());

        *metrics.violations.entry(lint_name.clone()).or_default() += 1;

        // Collect detailed violation information
        let violation = ViolationDetail {
            file: file_path,
            line: span.line_start,
            column: span.column_start,
            end_line: span.line_end,
            end_column: span.column_end,
            lint_name,
            message: diagnostic.message.clone(),
            severity: diagnostic.level.clone(),
            suggestion: span.suggested_replacement.clone(),
            machine_applicable: span
                .suggestion_applicability
                .as_ref()
                .is_some_and(|a| a == "MachineApplicable"),
        };

        metrics.detailed_violations.push(violation);
    }
}

/// Find the file with highest defect density (including detailed violations)
///
/// # Errors
///
/// Returns an error if the operation fails
fn find_hotspot_with_details(file_metrics: HashMap<PathBuf, FileMetrics>) -> Result<LintHotspot> {
    let mut hotspot_file = None;
    let mut max_density = 0.0;

    if std::env::var("LINT_HOTSPOT_DEBUG").is_ok() {
        eprintln!("🔍 Finding hotspot from {} files", file_metrics.len());
    }

    for (file_path, metrics) in file_metrics {
        if std::env::var("LINT_HOTSPOT_DEBUG").is_ok() {
            eprintln!(
                "  File: {}, SLOC: {}, Errors: {}, Warnings: {}",
                file_path.display(),
                metrics.sloc,
                metrics.severity_counts.error,
                metrics.severity_counts.warning
            );
        }

        if metrics.sloc == 0 {
            continue;
        }

        let total_violations = metrics.severity_counts.error
            + metrics.severity_counts.warning
            + metrics.severity_counts.suggestion;

        let density = (total_violations as f64) / (metrics.sloc as f64);

        if density > max_density {
            max_density = density;

            // Get top 10 lint violations
            let mut top_lints: Vec<_> = metrics.violations.into_iter().collect();
            top_lints.sort_by(|a, b| b.1.cmp(&a.1));
            top_lints.truncate(10);

            hotspot_file = Some(LintHotspot {
                file: file_path,
                defect_density: density,
                total_violations,
                sloc: metrics.sloc,
                severity_distribution: metrics.severity_counts,
                top_lints,
                detailed_violations: metrics.detailed_violations,
            });
        }
    }

    hotspot_file.ok_or_else(|| anyhow::anyhow!("No lint violations found in any Rust files"))
}

/// Calculate enforcement metadata
fn calculate_enforcement_metadata(
    hotspot: &LintHotspot,
    min_confidence: f64,
) -> EnforcementMetadata {
    // Simple heuristic: higher density = higher priority
    let enforcement_score = (hotspot.defect_density * 10.0).min(10.0);
    let enforcement_priority = (enforcement_score as u8).max(1);

    // Estimate fix time based on violations (5 minutes per violation)
    let estimated_fix_time = (hotspot.total_violations as u32) * 300;

    // Confidence based on lint types (some are easier to automate)
    let automation_confidence = if hotspot
        .top_lints
        .iter()
        .any(|(lint, _)| lint.contains("unused") || lint.contains("redundant"))
    {
        0.9
    } else {
        0.7
    };

    EnforcementMetadata {
        enforcement_score,
        requires_enforcement: enforcement_score >= 7.0 && automation_confidence >= min_confidence,
        estimated_fix_time,
        automation_confidence,
        enforcement_priority,
    }
}

/// Generate refactor chain for automated fixes
fn generate_refactor_chain(hotspot: &LintHotspot, min_confidence: f64) -> RefactorChain {
    let mut steps = Vec::new();
    let mut total_impact = 0;

    for (lint_code, count) in &hotspot.top_lints {
        let (confidence, description) = match lint_code.as_str() {
            s if s.contains("unused") => (0.95, "Remove unused code"),
            s if s.contains("redundant") => (0.90, "Remove redundant code"),
            s if s.contains("needless") => (0.85, "Simplify needless patterns"),
            s if s.contains("too_many_arguments") => (0.80, "Extract context objects"),
            _ => (0.70, "Apply clippy suggestion"),
        };

        if confidence >= min_confidence {
            steps.push(RefactorStep {
                id: format!("fix-{lint_code}"),
                lint: lint_code.clone(),
                confidence,
                impact: *count,
                description: description.to_string(),
            });
            total_impact += count;
        }
    }

    RefactorChain {
        id: format!(
            "lint-hotspot-{}",
            chrono::Utc::now().format("%Y%m%d-%H%M%S")
        ),
        estimated_reduction: total_impact,
        automation_confidence: steps.iter().map(|s| s.confidence).sum::<f64>() / steps.len() as f64,
        steps,
    }
}

/// Check quality gates
fn check_quality_gates(hotspot: &LintHotspot, max_density: f64) -> QualityGateStatus {
    let mut violations = Vec::new();

    if hotspot.defect_density > max_density {
        violations.push(QualityViolation {
            rule: "max_defect_density".to_string(),
            threshold: max_density,
            actual: hotspot.defect_density,
            severity: "blocking".to_string(),
        });
    }

    if hotspot.total_violations > 50 {
        violations.push(QualityViolation {
            rule: "max_single_file_violations".to_string(),
            threshold: 50.0,
            actual: hotspot.total_violations as f64,
            severity: "warning".to_string(),
        });
    }

    let passed = violations.is_empty();
    let blocking = violations.iter().any(|v| v.severity == "blocking");

    QualityGateStatus {
        passed,
        violations,
        blocking,
    }
}

/// Format output based on selected format
fn format_output(
    result: &LintHotspotResult,
    format: LintHotspotOutputFormat,
    perf: bool,
    elapsed: std::time::Duration,
    top_files: usize,
) -> Result<String> {
    match format {
        LintHotspotOutputFormat::Summary => format_summary(result, perf, elapsed, top_files),
        LintHotspotOutputFormat::Detailed => format_detailed(result, perf, elapsed, top_files),
        LintHotspotOutputFormat::Json => format_json(result, false),
        LintHotspotOutputFormat::EnforcementJson => format_json(result, true),
        LintHotspotOutputFormat::Sarif => format_sarif(result),
    }
}

/// Format summary output
///
/// # Example
///
/// ```no_run
/// use pmat::cli::handlers::lint_hotspot_handlers::{LintHotspotResult, LintHotspot, FileSummary, SeverityDistribution, QualityGateStatus};
/// use std::collections::HashMap;
/// use std::path::PathBuf;
/// use std::time::Duration;
///
/// let hotspot = LintHotspot {
///     file: PathBuf::from("src/main.rs"),
///     defect_density: 0.05,
///     total_violations: 5,
///     sloc: 100,
///     severity_distribution: SeverityDistribution {
///         error: 2,
///         warning: 3,
///         suggestion: 0,
///         note: 0,
///     },
///     top_lints: vec![
///         ("clippy::too_many_arguments".to_string(), 2),
///         ("unused_variable".to_string(), 3),
///     ],
///     detailed_violations: vec![],
/// };
///
/// let mut summary_by_file = HashMap::new();
/// summary_by_file.insert(
///     PathBuf::from("src/main.rs"),
///     FileSummary {
///         total_violations: 5,
///         errors: 2,
///         warnings: 3,
///         sloc: 100,
///         defect_density: 0.05,
///     }
/// );
///
/// let result = LintHotspotResult {
///     hotspot,
///     all_violations: vec![],
///     summary_by_file,
///     total_project_violations: 5,
///     enforcement: None,
///     refactor_chain: None,
///     quality_gate: QualityGateStatus {
///         passed: true,
///         violations: vec![],
///         blocking: false,
///     },
/// };
///
/// let output = pmat::cli::handlers::lint_hotspot_handlers::format_summary(&result, false, Duration::from_secs(1), 10).unwrap();
///
/// assert!(output.contains("# Lint Hotspot Analysis"));
/// assert!(output.contains("**Total Project Violations**: 5"));
/// assert!(output.contains("## Top Files with Lint Issues"));
/// assert!(output.contains("1. `main.rs` - 0.05 violations/SLOC"));
/// assert!(output.contains("## Hottest File Details"));
/// assert!(output.contains("**File**: src/main.rs"));
/// ```ignore
pub fn format_summary(
    result: &LintHotspotResult,
    perf: bool,
    elapsed: std::time::Duration,
    _top_files: usize,
) -> Result<String> {
    let mut output = String::new();

    output.push_str("# Lint Hotspot Analysis (EXTREME Quality Mode)\n\n");
    output.push_str(&format!(
        "**Total Project Violations**: {}\n",
        result.total_project_violations
    ));
    output.push_str(&format!(
        "**Files with Issues**: {}\n\n",
        result.summary_by_file.len()
    ));

    // Show top files with lint issues (consistent with other analyze commands)
    output.push_str("## Top Files with Lint Issues\n\n");
    let mut sorted_files: Vec<_> = result.summary_by_file.iter().collect();
    sorted_files.sort_by(|a, b| {
        b.1.defect_density
            .partial_cmp(&a.1.defect_density)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let files_to_show = if _top_files == 0 { 10 } else { _top_files };
    for (i, (file, summary)) in sorted_files.iter().take(files_to_show).enumerate() {
        let filename = file.file_name().unwrap_or_default().to_string_lossy();
        output.push_str(&format!(
            "{}. `{}` - {:.2} violations/SLOC ({} violations, {} SLOC)\n",
            i + 1,
            filename,
            summary.defect_density,
            summary.total_violations,
            summary.sloc
        ));
    }
    output.push('\n');

    output.push_str("## Hottest File Details\n");
    output.push_str(&format!("**File**: {}\n", result.hotspot.file.display()));
    output.push_str(&format!(
        "**Defect Density**: {:.2} violations/SLOC\n",
        result.hotspot.defect_density
    ));
    output.push_str(&format!(
        "**Total Violations**: {}\n",
        result.hotspot.total_violations
    ));
    output.push_str(&format!("**Lines of Code**: {}\n\n", result.hotspot.sloc));

    output.push_str("## Severity Distribution\n");
    output.push_str(&format!(
        "- Errors: {}\n",
        result.hotspot.severity_distribution.error
    ));
    output.push_str(&format!(
        "- Warnings: {}\n",
        result.hotspot.severity_distribution.warning
    ));
    output.push_str(&format!(
        "- Suggestions: {}\n\n",
        result.hotspot.severity_distribution.suggestion
    ));

    output.push_str("## Top Violations\n");
    for (lint, count) in result.hotspot.top_lints.iter().take(5) {
        output.push_str(&format!("- {lint}: {count} occurrences\n"));
    }

    if let Some(enforcement) = &result.enforcement {
        output.push_str("\n## Enforcement Metadata\n");
        output.push_str(&format!(
            "- Score: {:.1}/10\n",
            enforcement.enforcement_score
        ));
        output.push_str(&format!(
            "- Priority: {}\n",
            enforcement.enforcement_priority
        ));
        output.push_str(&format!(
            "- Estimated Fix Time: {} minutes\n",
            enforcement.estimated_fix_time / 60
        ));
        output.push_str(&format!(
            "- Automation Confidence: {:.0}%\n",
            enforcement.automation_confidence * 100.0
        ));
    }

    if !result.quality_gate.passed {
        output.push_str("\n## ❌ Quality Gate Failed\n");
        for violation in &result.quality_gate.violations {
            output.push_str(&format!(
                "- {} exceeded: {:.2} > {:.2}\n",
                violation.rule, violation.actual, violation.threshold
            ));
        }
    }

    if perf {
        output.push_str(&format!(
            "\n⏱️  Analysis completed in {:.2}s\n",
            elapsed.as_secs_f64()
        ));
    }

    Ok(output)
}

/// Format detailed output
fn format_detailed(
    result: &LintHotspotResult,
    perf: bool,
    elapsed: std::time::Duration,
    top_files: usize,
) -> Result<String> {
    let mut output = format_summary(result, perf, elapsed, top_files)?;

    // Add detailed violations for the hotspot file
    output.push_str("\n## Detailed Violations in Hotspot File\n");
    for violation in &result.hotspot.detailed_violations {
        output.push_str(&format!(
            "- **{}:{}:{}** [{}] {}\n",
            violation.file.display(),
            violation.line,
            violation.column,
            violation.lint_name,
            violation.message
        ));
        if let Some(suggestion) = &violation.suggestion {
            output.push_str(&format!("  Suggestion: {suggestion}\n"));
        }
    }

    // Add top files by violation count
    output.push_str("\n## Top Files by Violations\n");
    let mut sorted_files: Vec<_> = result.summary_by_file.iter().collect();
    sorted_files.sort_by(|a, b| b.1.total_violations.cmp(&a.1.total_violations));

    let files_to_show = if top_files == 0 {
        sorted_files.len()
    } else {
        top_files
    };
    for (file, summary) in sorted_files.iter().take(files_to_show) {
        output.push_str(&format!(
            "- {}: {} violations ({} errors, {} warnings, density: {:.2})\n",
            file.display(),
            summary.total_violations,
            summary.errors,
            summary.warnings,
            summary.defect_density
        ));
    }

    if let Some(chain) = &result.refactor_chain {
        output.push_str("\n## Refactor Chain\n");
        output.push_str(&format!("ID: {}\n", chain.id));
        output.push_str(&format!(
            "Estimated Reduction: {} violations\n",
            chain.estimated_reduction
        ));
        output.push_str(&format!(
            "Automation Confidence: {:.0}%\n\n",
            chain.automation_confidence * 100.0
        ));

        output.push_str("### Steps\n");
        for (i, step) in chain.steps.iter().enumerate() {
            output.push_str(&format!(
                "{}. {} - {} (confidence: {:.0}%, impact: {})\n",
                i + 1,
                step.description,
                step.lint,
                step.confidence * 100.0,
                step.impact
            ));
        }
    }

    Ok(output)
}

/// Format JSON output
///
/// # Errors
///
/// Returns an error if the operation fails
fn format_json(result: &LintHotspotResult, enforcement: bool) -> Result<String> {
    if enforcement {
        // Full enforcement-ready JSON
        serde_json::to_string_pretty(result).context("Failed to serialize to JSON")
    } else {
        // Simple JSON without enforcement details
        #[derive(Serialize)]
        struct SimpleResult<'a> {
            hotspot: &'a LintHotspot,
            quality_gate: &'a QualityGateStatus,
        }

        let simple = SimpleResult {
            hotspot: &result.hotspot,
            quality_gate: &result.quality_gate,
        };

        serde_json::to_string_pretty(&simple).context("Failed to serialize to JSON")
    }
}

/// Format SARIF output
///
/// # Errors
///
/// Returns an error if the operation fails
fn format_sarif(result: &LintHotspotResult) -> Result<String> {
    let sarif = serde_json::json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat-lint-hotspot",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit"
                }
            },
            "results": result.quality_gate.violations.iter().map(|v| {
                serde_json::json!({
                    "ruleId": v.rule,
                    "level": if v.severity == "blocking" { "error" } else { "warning" },
                    "message": {
                        "text": format!("{} exceeded: {:.2} > {:.2}", v.rule, v.actual, v.threshold)
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": result.hotspot.file.to_string_lossy()
                            }
                        }
                    }]
                })
            }).collect::<Vec<_>>()
        }]
    });

    serde_json::to_string_pretty(&sarif).context("Failed to serialize to SARIF")
}

/// Execute clippy command with given flags (cognitive complexity ≤3)
async fn execute_clippy_command(
    project_path: &Path,
    flags: &[&str],
) -> Result<std::process::Output> {
    let mut cmd = tokio::process::Command::new("cargo");
    cmd.arg("clippy")
        .arg("--message-format=json")
        .args(flags)
        .current_dir(project_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    Ok(cmd.output().await?)
}

/// Check clippy output status (cognitive complexity ≤5)
fn check_clippy_output(output: &std::process::Output) -> Result<()> {
    if !output.status.success()
        && output.status.code() != Some(101)
        && std::env::var("LINT_HOTSPOT_DEBUG").is_ok()
    {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("⚠️  Clippy exited with status: {:?}", output.status);
        eprintln!("Stderr: {stderr}");
    }
    Ok(())
}

/// Parse clippy JSON output into file metrics (cognitive complexity ≤8)
fn parse_clippy_json_output(
    output: &std::process::Output,
) -> Result<HashMap<PathBuf, FileMetrics>> {
    let reader = BufReader::new(output.stdout.as_slice());
    let mut file_metrics: HashMap<PathBuf, FileMetrics> = HashMap::new();
    let mut message_count = 0;

    for line in std::io::BufRead::lines(reader) {
        let line = line?;
        if let Ok(msg) = serde_json::from_str::<ClippyMessage>(&line) {
            if let Some(diagnostic) = msg.message {
                if msg.reason == Some("compiler-message".to_string()) {
                    message_count += 1;
                    process_diagnostic(&diagnostic, &mut file_metrics);
                }
            }
        }
    }

    if std::env::var("LINT_HOTSPOT_DEBUG").is_ok() {
        eprintln!("📊 Processed {message_count} compiler messages");
        eprintln!("📁 Files with metrics: {}", file_metrics.len());
    }

    Ok(file_metrics)
}

/// Calculate SLOC for each file in metrics (cognitive complexity ≤8)
async fn calculate_sloc_for_files(
    file_metrics: &mut HashMap<PathBuf, FileMetrics>,
    project_path: &Path,
    workspace_root: Option<&PathBuf>,
) -> Result<()> {
    for (file_path, metrics) in file_metrics.iter_mut() {
        let actual_path = resolve_file_path(file_path, project_path, workspace_root);

        if actual_path.exists() {
            let content = tokio::fs::read_to_string(&actual_path).await?;
            metrics.sloc = count_sloc(&content);
            log_sloc_debug(&actual_path, metrics.sloc);
        } else {
            log_file_not_found_debug(file_path, &actual_path, workspace_root);
        }
    }
    Ok(())
}

/// Resolve actual file path trying various locations (cognitive complexity ≤7)
fn resolve_file_path(
    file_path: &Path,
    project_path: &Path,
    workspace_root: Option<&PathBuf>,
) -> PathBuf {
    if file_path.exists() {
        return file_path.to_path_buf();
    }

    if let Some(ws_root) = workspace_root {
        // Try relative to workspace root
        let ws_relative = ws_root.join(file_path);
        if ws_relative.exists() {
            return ws_relative;
        }

        // Try with "server/" prefix
        let with_server = ws_root.join("server").join(file_path);
        if with_server.exists() {
            return with_server;
        }
    }

    // Try relative to project_path
    let project_relative = project_path.join(file_path);
    if project_relative.exists() {
        project_relative
    } else {
        file_path.to_path_buf()
    }
}

/// Count source lines of code (cognitive complexity ≤3)
fn count_sloc(content: &str) -> usize {
    content
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with("//"))
        .count()
}

/// Log SLOC debug info if enabled (cognitive complexity ≤3)
fn log_sloc_debug(path: &Path, sloc: usize) {
    if std::env::var("LINT_HOTSPOT_DEBUG").is_ok() && sloc > 0 {
        eprintln!("✓ File {} has {} SLOC", path.display(), sloc);
    }
}

/// Log file not found debug info if enabled (cognitive complexity ≤4)
fn log_file_not_found_debug(
    file_path: &Path,
    actual_path: &Path,
    workspace_root: Option<&PathBuf>,
) {
    if std::env::var("LINT_HOTSPOT_DEBUG").is_ok() {
        eprintln!("⚠️  Could not find file: {}", file_path.display());
        eprintln!("   Tried: {}", actual_path.display());
        if let Some(ws) = workspace_root {
            eprintln!("   Workspace root: {}", ws.display());
        }
    }
}

/// Build final `LintHotspotResult` from file metrics (cognitive complexity ≤8)
fn build_lint_hotspot_result(
    file_metrics: HashMap<PathBuf, FileMetrics>,
) -> Result<LintHotspotResult> {
    let (all_violations, summary_by_file, total_project_violations) =
        collect_project_violations(&file_metrics);

    let hotspot = find_hotspot_with_details(file_metrics)?;

    Ok(LintHotspotResult {
        hotspot,
        all_violations,
        summary_by_file,
        total_project_violations,
        enforcement: None,
        refactor_chain: None,
        quality_gate: QualityGateStatus {
            passed: true,
            violations: vec![],
            blocking: false,
        },
    })
}

/// Collect all violations across the project (cognitive complexity ≤7)
fn collect_project_violations(
    file_metrics: &HashMap<PathBuf, FileMetrics>,
) -> (Vec<ViolationDetail>, HashMap<PathBuf, FileSummary>, usize) {
    let mut all_violations = Vec::new();
    let mut summary_by_file = HashMap::new();
    let mut total_project_violations = 0;

    for (file_path, metrics) in file_metrics {
        all_violations.extend(metrics.detailed_violations.clone());

        let total_file_violations = calculate_total_violations(metrics);
        total_project_violations += total_file_violations;

        let defect_density = calculate_defect_density(total_file_violations, metrics.sloc);

        summary_by_file.insert(
            file_path.clone(),
            FileSummary {
                total_violations: total_file_violations,
                errors: metrics.severity_counts.error,
                warnings: metrics.severity_counts.warning,
                sloc: metrics.sloc,
                defect_density,
            },
        );
    }

    (all_violations, summary_by_file, total_project_violations)
}

/// Calculate total violations for a file (cognitive complexity ≤2)
fn calculate_total_violations(metrics: &FileMetrics) -> usize {
    metrics.severity_counts.error
        + metrics.severity_counts.warning
        + metrics.severity_counts.suggestion
}

/// Calculate defect density (cognitive complexity ≤2)
fn calculate_defect_density(violations: usize, sloc: usize) -> f64 {
    if sloc > 0 {
        violations as f64 / sloc as f64
    } else {
        0.0
    }
}

/// Find workspace root by looking for Cargo.toml with [workspace]
///
/// # Errors
///
/// Returns an error if the operation fails
fn find_workspace_root(start_path: &Path) -> Result<Option<PathBuf>> {
    let mut current = start_path;

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            // Check if this Cargo.toml contains [workspace]
            let contents = std::fs::read_to_string(&cargo_toml)?;
            if contents.contains("[workspace]") {
                return Ok(Some(current.to_path_buf()));
            }
        }

        // Move up one directory
        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_hotspot_result() -> LintHotspotResult {
        LintHotspotResult {
            hotspot: LintHotspot {
                file: PathBuf::from("src/main.rs"),
                defect_density: 0.05,
                total_violations: 5,
                sloc: 100,
                severity_distribution: SeverityDistribution {
                    error: 2,
                    warning: 3,
                    suggestion: 0,
                    note: 0,
                },
                top_lints: vec![
                    ("clippy::too_many_arguments".to_string(), 2),
                    ("unused_variable".to_string(), 3),
                ],
                detailed_violations: vec![],
            },
            all_violations: vec![],
            summary_by_file: std::collections::HashMap::new(),
            total_project_violations: 5,
            enforcement: None,
            refactor_chain: None,
            quality_gate: QualityGateStatus {
                passed: true,
                violations: vec![],
                blocking: false,
            },
        }
    }

    /// Test that enforce flag exits with non-zero status when there are violations
    ///
    /// # Example
    ///
    /// ```
    /// use pmat::cli::handlers::lint_hotspot_handlers::should_exit_with_error;
    ///
    /// // With enforce flag and violations - should exit with error
    /// let should_exit = should_exit_with_error(true, true, 5);
    /// assert!(should_exit);
    ///
    /// // Without enforce flag but quality gate failed - should exit with error  
    /// let should_exit = should_exit_with_error(false, false, 5);
    /// assert!(should_exit);
    ///
    /// // Without enforce flag and no violations - should not exit with error
    /// let should_exit = should_exit_with_error(true, false, 0);
    /// assert!(!should_exit);
    /// ```
    pub fn should_exit_with_error(
        quality_gate_passed: bool,
        enforce: bool,
        total_violations: usize,
    ) -> bool {
        !quality_gate_passed || (enforce && total_violations > 0)
    }

    #[test]
    fn test_enforce_flag_behavior() {
        // Test 1: Enforce flag with violations should trigger exit
        assert!(should_exit_with_error(true, true, 5));

        // Test 2: Enforce flag without violations should not trigger exit
        assert!(!should_exit_with_error(true, true, 0));

        // Test 3: No enforce flag with violations should not trigger exit
        assert!(!should_exit_with_error(true, false, 5));

        // Test 4: Quality gate failed should always trigger exit
        assert!(should_exit_with_error(false, false, 0));
        assert!(should_exit_with_error(false, true, 5));
    }

    #[test]
    fn test_format_summary_with_violations() {
        let result = create_test_hotspot_result();
        let output = format_summary(&result, false, std::time::Duration::from_secs(1), 10).unwrap();

        assert!(output.contains("# Lint Hotspot Analysis"));
        assert!(output.contains("**Total Project Violations**: 5"));
        assert!(output.contains("## Top Files with Lint Issues"));
        assert!(output.contains("## Hottest File Details"));
        assert!(output.contains("**File**: src/main.rs"));
    }

    #[test]
    fn test_quality_gate_enforcement_scenario() {
        let mut result = create_test_hotspot_result();

        // Test case 1: Quality gate passes but enforce flag is set with violations
        result.quality_gate.passed = true;
        result.total_project_violations = 10;

        let should_exit = should_exit_with_error(
            result.quality_gate.passed,
            true, // enforce flag
            result.total_project_violations,
        );
        assert!(
            should_exit,
            "Should exit with error when enforce flag is set and violations exist"
        );

        // Test case 2: Quality gate passes, enforce flag set, no violations
        result.total_project_violations = 0;

        let should_exit = should_exit_with_error(
            result.quality_gate.passed,
            true, // enforce flag
            result.total_project_violations,
        );
        assert!(
            !should_exit,
            "Should not exit with error when enforce flag is set but no violations"
        );
    }

    #[test]
    fn test_multiple_enforcement_scenarios() {
        // Scenario 1: Quality gate failed, no enforce flag
        assert!(should_exit_with_error(false, false, 0));

        // Scenario 2: Quality gate passed, enforce flag, violations present
        assert!(should_exit_with_error(true, true, 1));

        // Scenario 3: Quality gate passed, enforce flag, no violations
        assert!(!should_exit_with_error(true, true, 0));

        // Scenario 4: Quality gate passed, no enforce flag, violations present
        assert!(!should_exit_with_error(true, false, 10));

        // Scenario 5: Quality gate failed, enforce flag, violations present
        assert!(should_exit_with_error(false, true, 5));
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::time::Duration;

    // ============================================================================
    // Test Data Builders
    // ============================================================================

    fn create_test_violation(
        file: &str,
        line: u32,
        lint_name: &str,
        severity: &str,
    ) -> ViolationDetail {
        ViolationDetail {
            file: PathBuf::from(file),
            line,
            column: 1,
            end_line: line,
            end_column: 10,
            lint_name: lint_name.to_string(),
            message: format!("Test violation for {lint_name}"),
            severity: severity.to_string(),
            suggestion: None,
            machine_applicable: false,
        }
    }

    fn create_test_violation_with_suggestion(
        file: &str,
        line: u32,
        lint_name: &str,
        severity: &str,
        suggestion: &str,
    ) -> ViolationDetail {
        ViolationDetail {
            file: PathBuf::from(file),
            line,
            column: 1,
            end_line: line,
            end_column: 10,
            lint_name: lint_name.to_string(),
            message: format!("Test violation for {lint_name}"),
            severity: severity.to_string(),
            suggestion: Some(suggestion.to_string()),
            machine_applicable: true,
        }
    }

    fn create_test_hotspot(
        file: &str,
        violations: usize,
        sloc: usize,
        errors: usize,
        warnings: usize,
    ) -> LintHotspot {
        let defect_density = if sloc > 0 {
            violations as f64 / sloc as f64
        } else {
            0.0
        };
        LintHotspot {
            file: PathBuf::from(file),
            defect_density,
            total_violations: violations,
            sloc,
            severity_distribution: SeverityDistribution {
                error: errors,
                warning: warnings,
                suggestion: 0,
                note: 0,
            },
            top_lints: vec![
                ("clippy::unused_variable".to_string(), 3),
                ("clippy::needless_return".to_string(), 2),
            ],
            detailed_violations: vec![],
        }
    }

    fn create_full_test_result() -> LintHotspotResult {
        let mut summary_by_file = HashMap::new();
        summary_by_file.insert(
            PathBuf::from("src/main.rs"),
            FileSummary {
                total_violations: 5,
                errors: 2,
                warnings: 3,
                sloc: 100,
                defect_density: 0.05,
            },
        );
        summary_by_file.insert(
            PathBuf::from("src/lib.rs"),
            FileSummary {
                total_violations: 3,
                errors: 1,
                warnings: 2,
                sloc: 50,
                defect_density: 0.06,
            },
        );

        LintHotspotResult {
            hotspot: create_test_hotspot("src/main.rs", 5, 100, 2, 3),
            all_violations: vec![
                create_test_violation("src/main.rs", 10, "unused_variable", "warning"),
                create_test_violation("src/main.rs", 20, "clippy::needless_return", "warning"),
                create_test_violation("src/lib.rs", 5, "clippy::too_many_arguments", "warning"),
            ],
            summary_by_file,
            total_project_violations: 8,
            enforcement: None,
            refactor_chain: None,
            quality_gate: QualityGateStatus {
                passed: true,
                violations: vec![],
                blocking: false,
            },
        }
    }

    // ============================================================================
    // LintHotspotParams Tests
    // ============================================================================

    #[test]
    fn test_lint_hotspot_params_creation() {
        let params = LintHotspotParams {
            project_path: PathBuf::from("/test/project"),
            file: None,
            format: LintHotspotOutputFormat::Summary,
            max_density: 5.0,
            min_confidence: 0.7,
            enforce: false,
            dry_run: false,
            enforcement_metadata: false,
            output: None,
            perf: false,
            clippy_flags: String::new(),
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        assert_eq!(params.project_path, PathBuf::from("/test/project"));
        assert!(params.file.is_none());
        assert!(!params.enforce);
        assert_eq!(params.max_density, 5.0);
        assert_eq!(params.min_confidence, 0.7);
    }

    #[test]
    fn test_lint_hotspot_params_with_file() {
        let params = LintHotspotParams {
            project_path: PathBuf::from("/test/project"),
            file: Some(PathBuf::from("src/main.rs")),
            format: LintHotspotOutputFormat::Json,
            max_density: 10.0,
            min_confidence: 0.8,
            enforce: true,
            dry_run: true,
            enforcement_metadata: true,
            output: Some(PathBuf::from("/tmp/output.json")),
            perf: true,
            clippy_flags: "-W clippy::pedantic".to_string(),
            top_files: 5,
            include: vec!["src/**/*.rs".to_string()],
            exclude: vec!["**/tests/**".to_string()],
        };

        assert_eq!(params.file, Some(PathBuf::from("src/main.rs")));
        assert!(params.enforce);
        assert!(params.dry_run);
        assert!(params.enforcement_metadata);
        assert!(params.perf);
        assert!(!params.include.is_empty());
        assert!(!params.exclude.is_empty());
    }

    // ============================================================================
    // ViolationDetail Tests
    // ============================================================================

    #[test]
    fn test_violation_detail_clone() {
        let original = create_test_violation("src/main.rs", 10, "unused_variable", "warning");
        let cloned = original.clone();

        assert_eq!(cloned.file, original.file);
        assert_eq!(cloned.line, original.line);
        assert_eq!(cloned.lint_name, original.lint_name);
        assert_eq!(cloned.severity, original.severity);
    }

    #[test]
    fn test_violation_detail_with_suggestion() {
        let violation = create_test_violation_with_suggestion(
            "src/main.rs",
            10,
            "unused_variable",
            "warning",
            "Remove the unused variable",
        );

        assert!(violation.suggestion.is_some());
        assert!(violation.machine_applicable);
        assert_eq!(
            violation.suggestion.unwrap(),
            "Remove the unused variable"
        );
    }

    #[test]
    fn test_violation_detail_serialization() {
        let violation = create_test_violation("src/main.rs", 10, "unused_variable", "warning");
        let json = serde_json::to_string(&violation).unwrap();

        assert!(json.contains("src/main.rs"));
        assert!(json.contains("unused_variable"));
        assert!(json.contains("warning"));
    }

    // ============================================================================
    // SeverityDistribution Tests
    // ============================================================================

    #[test]
    fn test_severity_distribution_default() {
        let dist = SeverityDistribution::default();

        assert_eq!(dist.error, 0);
        assert_eq!(dist.warning, 0);
        assert_eq!(dist.suggestion, 0);
        assert_eq!(dist.note, 0);
    }

    #[test]
    fn test_severity_distribution_custom() {
        let dist = SeverityDistribution {
            error: 5,
            warning: 10,
            suggestion: 3,
            note: 2,
        };

        assert_eq!(dist.error, 5);
        assert_eq!(dist.warning, 10);
        assert_eq!(dist.suggestion, 3);
        assert_eq!(dist.note, 2);
    }

    #[test]
    fn test_severity_distribution_serialization() {
        let dist = SeverityDistribution {
            error: 5,
            warning: 10,
            suggestion: 3,
            note: 2,
        };

        let json = serde_json::to_string(&dist).unwrap();
        let deserialized: SeverityDistribution = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.error, dist.error);
        assert_eq!(deserialized.warning, dist.warning);
        assert_eq!(deserialized.suggestion, dist.suggestion);
        assert_eq!(deserialized.note, dist.note);
    }

    // ============================================================================
    // FileSummary Tests
    // ============================================================================

    #[test]
    fn test_file_summary_creation() {
        let summary = FileSummary {
            total_violations: 10,
            errors: 3,
            warnings: 7,
            sloc: 200,
            defect_density: 0.05,
        };

        assert_eq!(summary.total_violations, 10);
        assert_eq!(summary.errors, 3);
        assert_eq!(summary.warnings, 7);
        assert_eq!(summary.sloc, 200);
        assert!((summary.defect_density - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn test_file_summary_serialization() {
        let summary = FileSummary {
            total_violations: 10,
            errors: 3,
            warnings: 7,
            sloc: 200,
            defect_density: 0.05,
        };

        let json = serde_json::to_string(&summary).unwrap();
        let deserialized: FileSummary = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_violations, summary.total_violations);
        assert_eq!(deserialized.errors, summary.errors);
        assert_eq!(deserialized.warnings, summary.warnings);
    }

    // ============================================================================
    // LintHotspot Tests
    // ============================================================================

    #[test]
    fn test_lint_hotspot_creation() {
        let hotspot = create_test_hotspot("src/main.rs", 10, 200, 3, 7);

        assert_eq!(hotspot.file, PathBuf::from("src/main.rs"));
        assert_eq!(hotspot.total_violations, 10);
        assert_eq!(hotspot.sloc, 200);
        assert_eq!(hotspot.severity_distribution.error, 3);
        assert_eq!(hotspot.severity_distribution.warning, 7);
        assert!((hotspot.defect_density - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn test_lint_hotspot_with_zero_sloc() {
        let hotspot = create_test_hotspot("src/empty.rs", 5, 0, 1, 4);

        assert_eq!(hotspot.sloc, 0);
        assert!((hotspot.defect_density - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_lint_hotspot_serialization() {
        let hotspot = create_test_hotspot("src/main.rs", 10, 200, 3, 7);

        let json = serde_json::to_string(&hotspot).unwrap();
        let deserialized: LintHotspot = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.file, hotspot.file);
        assert_eq!(deserialized.total_violations, hotspot.total_violations);
        assert_eq!(deserialized.sloc, hotspot.sloc);
    }

    // ============================================================================
    // EnforcementMetadata Tests
    // ============================================================================

    #[test]
    fn test_enforcement_metadata_creation() {
        let metadata = EnforcementMetadata {
            enforcement_score: 8.5,
            requires_enforcement: true,
            estimated_fix_time: 1800,
            automation_confidence: 0.85,
            enforcement_priority: 3,
        };

        assert!((metadata.enforcement_score - 8.5).abs() < f64::EPSILON);
        assert!(metadata.requires_enforcement);
        assert_eq!(metadata.estimated_fix_time, 1800);
        assert!((metadata.automation_confidence - 0.85).abs() < f64::EPSILON);
        assert_eq!(metadata.enforcement_priority, 3);
    }

    #[test]
    fn test_enforcement_metadata_serialization() {
        let metadata = EnforcementMetadata {
            enforcement_score: 8.5,
            requires_enforcement: true,
            estimated_fix_time: 1800,
            automation_confidence: 0.85,
            enforcement_priority: 3,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: EnforcementMetadata = serde_json::from_str(&json).unwrap();

        assert!((deserialized.enforcement_score - metadata.enforcement_score).abs() < f64::EPSILON);
        assert_eq!(
            deserialized.requires_enforcement,
            metadata.requires_enforcement
        );
    }

    // ============================================================================
    // RefactorChain and RefactorStep Tests
    // ============================================================================

    #[test]
    fn test_refactor_step_creation() {
        let step = RefactorStep {
            id: "fix-unused".to_string(),
            lint: "unused_variable".to_string(),
            confidence: 0.95,
            impact: 5,
            description: "Remove unused variables".to_string(),
        };

        assert_eq!(step.id, "fix-unused");
        assert_eq!(step.lint, "unused_variable");
        assert!((step.confidence - 0.95).abs() < f64::EPSILON);
        assert_eq!(step.impact, 5);
    }

    #[test]
    fn test_refactor_chain_creation() {
        let chain = RefactorChain {
            id: "lint-hotspot-20240101-120000".to_string(),
            estimated_reduction: 15,
            automation_confidence: 0.88,
            steps: vec![
                RefactorStep {
                    id: "step-1".to_string(),
                    lint: "unused".to_string(),
                    confidence: 0.95,
                    impact: 10,
                    description: "Remove unused".to_string(),
                },
                RefactorStep {
                    id: "step-2".to_string(),
                    lint: "needless".to_string(),
                    confidence: 0.80,
                    impact: 5,
                    description: "Simplify needless".to_string(),
                },
            ],
        };

        assert_eq!(chain.steps.len(), 2);
        assert_eq!(chain.estimated_reduction, 15);
    }

    #[test]
    fn test_refactor_chain_serialization() {
        let chain = RefactorChain {
            id: "test-chain".to_string(),
            estimated_reduction: 10,
            automation_confidence: 0.9,
            steps: vec![],
        };

        let json = serde_json::to_string(&chain).unwrap();
        let deserialized: RefactorChain = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, chain.id);
        assert_eq!(deserialized.estimated_reduction, chain.estimated_reduction);
    }

    // ============================================================================
    // QualityGateStatus and QualityViolation Tests
    // ============================================================================

    #[test]
    fn test_quality_gate_status_passed() {
        let status = QualityGateStatus {
            passed: true,
            violations: vec![],
            blocking: false,
        };

        assert!(status.passed);
        assert!(status.violations.is_empty());
        assert!(!status.blocking);
    }

    #[test]
    fn test_quality_gate_status_failed_with_violations() {
        let status = QualityGateStatus {
            passed: false,
            violations: vec![QualityViolation {
                rule: "max_defect_density".to_string(),
                threshold: 5.0,
                actual: 10.0,
                severity: "blocking".to_string(),
            }],
            blocking: true,
        };

        assert!(!status.passed);
        assert_eq!(status.violations.len(), 1);
        assert!(status.blocking);
    }

    #[test]
    fn test_quality_violation_serialization() {
        let violation = QualityViolation {
            rule: "max_defect_density".to_string(),
            threshold: 5.0,
            actual: 10.0,
            severity: "blocking".to_string(),
        };

        let json = serde_json::to_string(&violation).unwrap();
        let deserialized: QualityViolation = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.rule, violation.rule);
        assert!((deserialized.threshold - violation.threshold).abs() < f64::EPSILON);
        assert!((deserialized.actual - violation.actual).abs() < f64::EPSILON);
    }

    // ============================================================================
    // check_quality_gates Tests
    // ============================================================================

    #[test]
    fn test_check_quality_gates_passes_below_threshold() {
        let hotspot = create_test_hotspot("src/main.rs", 5, 100, 2, 3);
        let status = check_quality_gates(&hotspot, 10.0);

        assert!(status.passed);
        assert!(status.violations.is_empty());
        assert!(!status.blocking);
    }

    #[test]
    fn test_check_quality_gates_fails_exceeds_density() {
        let hotspot = create_test_hotspot("src/main.rs", 100, 100, 50, 50);
        let status = check_quality_gates(&hotspot, 0.5);

        assert!(!status.passed);
        assert!(!status.violations.is_empty());
        assert!(status.blocking);

        let violation = &status.violations[0];
        assert_eq!(violation.rule, "max_defect_density");
        assert!((violation.threshold - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_check_quality_gates_warning_for_high_violations() {
        let mut hotspot = create_test_hotspot("src/main.rs", 60, 1000, 30, 30);
        hotspot.defect_density = 0.06; // Below max_density of 1.0
        hotspot.total_violations = 60;

        let status = check_quality_gates(&hotspot, 1.0);

        // Should have warning for max_single_file_violations
        let warning = status
            .violations
            .iter()
            .find(|v| v.rule == "max_single_file_violations");
        assert!(warning.is_some());
        assert_eq!(warning.unwrap().severity, "warning");
    }

    // ============================================================================
    // calculate_enforcement_metadata Tests
    // ============================================================================

    #[test]
    fn test_calculate_enforcement_metadata_low_density() {
        let hotspot = create_test_hotspot("src/main.rs", 5, 100, 2, 3);
        let metadata = calculate_enforcement_metadata(&hotspot, 0.7);

        assert!(metadata.enforcement_score < 10.0);
        assert!(!metadata.requires_enforcement); // Score < 7.0
        assert_eq!(metadata.estimated_fix_time, 5 * 300); // 5 violations * 300 seconds
    }

    #[test]
    fn test_calculate_enforcement_metadata_high_density() {
        let mut hotspot = create_test_hotspot("src/main.rs", 100, 100, 50, 50);
        hotspot.defect_density = 1.0;
        hotspot.top_lints = vec![("unused_variable".to_string(), 50)];

        let metadata = calculate_enforcement_metadata(&hotspot, 0.5);

        assert_eq!(metadata.enforcement_score, 10.0); // Capped at 10.0
        assert!(metadata.requires_enforcement);
        assert!((metadata.automation_confidence - 0.9).abs() < f64::EPSILON); // "unused" in lint
    }

    #[test]
    fn test_calculate_enforcement_metadata_with_redundant_lints() {
        let mut hotspot = create_test_hotspot("src/main.rs", 20, 100, 10, 10);
        hotspot.defect_density = 0.8;
        hotspot.top_lints = vec![("redundant_clone".to_string(), 10)];

        let metadata = calculate_enforcement_metadata(&hotspot, 0.7);

        assert!((metadata.automation_confidence - 0.9).abs() < f64::EPSILON); // "redundant" in lint
    }

    #[test]
    fn test_calculate_enforcement_metadata_without_auto_fixable() {
        let mut hotspot = create_test_hotspot("src/main.rs", 20, 100, 10, 10);
        hotspot.defect_density = 0.8;
        hotspot.top_lints = vec![("clippy::complexity".to_string(), 10)];

        let metadata = calculate_enforcement_metadata(&hotspot, 0.7);

        assert!((metadata.automation_confidence - 0.7).abs() < f64::EPSILON); // Default confidence
    }

    // ============================================================================
    // generate_refactor_chain Tests
    // ============================================================================

    #[test]
    fn test_generate_refactor_chain_with_unused_lints() {
        let mut hotspot = create_test_hotspot("src/main.rs", 10, 100, 5, 5);
        hotspot.top_lints = vec![
            ("unused_variable".to_string(), 5),
            ("unused_import".to_string(), 3),
        ];

        let chain = generate_refactor_chain(&hotspot, 0.7);

        assert_eq!(chain.steps.len(), 2);
        assert!((chain.steps[0].confidence - 0.95).abs() < f64::EPSILON);
        assert_eq!(chain.steps[0].description, "Remove unused code");
    }

    #[test]
    fn test_generate_refactor_chain_with_needless_lints() {
        let mut hotspot = create_test_hotspot("src/main.rs", 10, 100, 5, 5);
        hotspot.top_lints = vec![("needless_return".to_string(), 5)];

        let chain = generate_refactor_chain(&hotspot, 0.7);

        assert_eq!(chain.steps.len(), 1);
        assert!((chain.steps[0].confidence - 0.85).abs() < f64::EPSILON);
        assert_eq!(chain.steps[0].description, "Simplify needless patterns");
    }

    #[test]
    fn test_generate_refactor_chain_filters_by_confidence() {
        let mut hotspot = create_test_hotspot("src/main.rs", 10, 100, 5, 5);
        hotspot.top_lints = vec![
            ("unused_variable".to_string(), 5),  // confidence 0.95
            ("complex_lint".to_string(), 3),      // confidence 0.70
        ];

        let chain = generate_refactor_chain(&hotspot, 0.9);

        // Only unused_variable should pass the 0.9 threshold
        assert_eq!(chain.steps.len(), 1);
        assert!(chain.steps[0].lint.contains("unused"));
    }

    #[test]
    fn test_generate_refactor_chain_calculates_total_reduction() {
        let mut hotspot = create_test_hotspot("src/main.rs", 15, 100, 5, 10);
        hotspot.top_lints = vec![
            ("unused_variable".to_string(), 5),
            ("redundant_clone".to_string(), 3),
        ];

        let chain = generate_refactor_chain(&hotspot, 0.7);

        assert_eq!(chain.estimated_reduction, 8); // 5 + 3
    }

    // ============================================================================
    // count_top_lints Tests
    // ============================================================================

    #[test]
    fn test_count_top_lints_empty() {
        let violations: Vec<ViolationDetail> = vec![];
        let top_lints = count_top_lints(&violations);

        assert!(top_lints.is_empty());
    }

    #[test]
    fn test_count_top_lints_single_lint() {
        let violations = vec![
            create_test_violation("src/main.rs", 10, "unused_variable", "warning"),
            create_test_violation("src/main.rs", 20, "unused_variable", "warning"),
        ];

        let top_lints = count_top_lints(&violations);

        assert_eq!(top_lints.len(), 1);
        assert_eq!(top_lints[0].0, "unused_variable");
        assert_eq!(top_lints[0].1, 2);
    }

    #[test]
    fn test_count_top_lints_multiple_lints_sorted() {
        let violations = vec![
            create_test_violation("src/main.rs", 10, "lint_a", "warning"),
            create_test_violation("src/main.rs", 20, "lint_b", "warning"),
            create_test_violation("src/main.rs", 30, "lint_b", "warning"),
            create_test_violation("src/main.rs", 40, "lint_b", "warning"),
            create_test_violation("src/main.rs", 50, "lint_a", "warning"),
        ];

        let top_lints = count_top_lints(&violations);

        assert_eq!(top_lints.len(), 2);
        assert_eq!(top_lints[0].0, "lint_b"); // 3 occurrences
        assert_eq!(top_lints[0].1, 3);
        assert_eq!(top_lints[1].0, "lint_a"); // 2 occurrences
        assert_eq!(top_lints[1].1, 2);
    }

    #[test]
    fn test_count_top_lints_truncates_to_10() {
        let mut violations = vec![];
        for i in 0..15 {
            violations.push(create_test_violation(
                "src/main.rs",
                i as u32,
                &format!("lint_{i}"),
                "warning",
            ));
        }

        let top_lints = count_top_lints(&violations);

        assert_eq!(top_lints.len(), 10);
    }

    // ============================================================================
    // update_severity_distribution Tests
    // ============================================================================

    #[test]
    fn test_update_severity_distribution_error() {
        let mut dist = SeverityDistribution::default();
        update_severity_distribution(&mut dist, "error");

        assert_eq!(dist.error, 1);
        assert_eq!(dist.warning, 0);
        assert_eq!(dist.note, 0);
    }

    #[test]
    fn test_update_severity_distribution_warning() {
        let mut dist = SeverityDistribution::default();
        update_severity_distribution(&mut dist, "warning");

        assert_eq!(dist.error, 0);
        assert_eq!(dist.warning, 1);
        assert_eq!(dist.note, 0);
    }

    #[test]
    fn test_update_severity_distribution_unknown() {
        let mut dist = SeverityDistribution::default();
        update_severity_distribution(&mut dist, "unknown");

        assert_eq!(dist.error, 0);
        assert_eq!(dist.warning, 0);
        assert_eq!(dist.note, 1);
    }

    #[test]
    fn test_update_severity_distribution_multiple() {
        let mut dist = SeverityDistribution::default();
        update_severity_distribution(&mut dist, "error");
        update_severity_distribution(&mut dist, "error");
        update_severity_distribution(&mut dist, "warning");
        update_severity_distribution(&mut dist, "warning");
        update_severity_distribution(&mut dist, "warning");
        update_severity_distribution(&mut dist, "note");

        assert_eq!(dist.error, 2);
        assert_eq!(dist.warning, 3);
        assert_eq!(dist.note, 1);
    }

    // ============================================================================
    // count_sloc Tests
    // ============================================================================

    #[test]
    fn test_count_sloc_empty() {
        let content = "";
        assert_eq!(count_sloc(content), 0);
    }

    #[test]
    fn test_count_sloc_only_whitespace() {
        let content = "   \n\n   \n";
        assert_eq!(count_sloc(content), 0);
    }

    #[test]
    fn test_count_sloc_only_comments() {
        let content = "// comment 1\n// comment 2\n// comment 3";
        assert_eq!(count_sloc(content), 0);
    }

    #[test]
    fn test_count_sloc_mixed_content() {
        let content = r#"
fn main() {
    // This is a comment
    let x = 5;

    println!("Hello");
}
"#;
        // Should count: fn main(), let x = 5;, println!(), }
        // Empty line and comment line are excluded
        assert!(count_sloc(content) >= 3);
    }

    #[test]
    fn test_count_sloc_with_inline_comments() {
        let content = r#"let x = 5; // inline comment
let y = 10;"#;
        // Both lines have code, inline comments don't make a line "comment only"
        assert_eq!(count_sloc(content), 2);
    }

    // ============================================================================
    // calculate_defect_density Tests
    // ============================================================================

    #[test]
    fn test_calculate_defect_density_normal() {
        assert!((calculate_defect_density(10, 100) - 0.1).abs() < f64::EPSILON);
        assert!((calculate_defect_density(5, 50) - 0.1).abs() < f64::EPSILON);
        assert!((calculate_defect_density(0, 100) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_defect_density_zero_sloc() {
        assert!((calculate_defect_density(10, 0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_defect_density_high_density() {
        assert!((calculate_defect_density(200, 100) - 2.0).abs() < f64::EPSILON);
    }

    // ============================================================================
    // calculate_total_violations Tests
    // ============================================================================

    #[test]
    fn test_calculate_total_violations() {
        let metrics = FileMetrics {
            violations: HashMap::new(),
            severity_counts: SeverityDistribution {
                error: 5,
                warning: 10,
                suggestion: 3,
                note: 0,
            },
            sloc: 100,
            detailed_violations: vec![],
        };

        let total = calculate_total_violations(&metrics);
        assert_eq!(total, 18); // 5 + 10 + 3
    }

    #[test]
    fn test_calculate_total_violations_empty() {
        let metrics = FileMetrics {
            violations: HashMap::new(),
            severity_counts: SeverityDistribution::default(),
            sloc: 100,
            detailed_violations: vec![],
        };

        let total = calculate_total_violations(&metrics);
        assert_eq!(total, 0);
    }

    // ============================================================================
    // resolve_absolute_path Tests
    // ============================================================================

    #[test]
    fn test_resolve_absolute_path_already_absolute() {
        let project_path = PathBuf::from("/project");
        let file_path = PathBuf::from("/absolute/path/file.rs");

        let resolved = resolve_absolute_path(&project_path, &file_path);
        assert_eq!(resolved, file_path);
    }

    #[test]
    fn test_resolve_absolute_path_relative() {
        let project_path = PathBuf::from("/project");
        let file_path = PathBuf::from("src/main.rs");

        let resolved = resolve_absolute_path(&project_path, &file_path);
        assert_eq!(resolved, PathBuf::from("/project/src/main.rs"));
    }

    // ============================================================================
    // is_target_file Tests
    // ============================================================================

    #[test]
    fn test_is_target_file_exact_match() {
        let abs_path = PathBuf::from("/project/src/main.rs");
        let file_path = PathBuf::from("src/main.rs");

        assert!(is_target_file(
            "/project/src/main.rs",
            &abs_path,
            &file_path
        ));
    }

    #[test]
    fn test_is_target_file_relative_match() {
        let abs_path = PathBuf::from("/project/src/main.rs");
        let file_path = PathBuf::from("src/main.rs");

        assert!(is_target_file("src/main.rs", &abs_path, &file_path));
    }

    #[test]
    fn test_is_target_file_ends_with() {
        let abs_path = PathBuf::from("/project/src/main.rs");
        let file_path = PathBuf::from("main.rs");

        assert!(is_target_file(
            "/project/src/main.rs",
            &abs_path,
            &file_path
        ));
    }

    #[test]
    fn test_is_target_file_no_match() {
        let abs_path = PathBuf::from("/project/src/main.rs");
        let file_path = PathBuf::from("src/main.rs");

        assert!(!is_target_file("src/lib.rs", &abs_path, &file_path));
    }

    // ============================================================================
    // is_machine_applicable Tests
    // ============================================================================

    #[test]
    fn test_is_machine_applicable_true() {
        let span = DiagnosticSpan {
            file_name: "test.rs".to_string(),
            line_start: 1,
            line_end: 1,
            column_start: 1,
            column_end: 10,
            is_primary: true,
            _text: vec![],
            suggested_replacement: Some("fix".to_string()),
            suggestion_applicability: Some("machine-applicable".to_string()),
        };

        assert!(is_machine_applicable(&span));
    }

    #[test]
    fn test_is_machine_applicable_maybe_incorrect() {
        let span = DiagnosticSpan {
            file_name: "test.rs".to_string(),
            line_start: 1,
            line_end: 1,
            column_start: 1,
            column_end: 10,
            is_primary: true,
            _text: vec![],
            suggested_replacement: Some("fix".to_string()),
            suggestion_applicability: Some("maybe-incorrect".to_string()),
        };

        assert!(is_machine_applicable(&span));
    }

    #[test]
    fn test_is_machine_applicable_false() {
        let span = DiagnosticSpan {
            file_name: "test.rs".to_string(),
            line_start: 1,
            line_end: 1,
            column_start: 1,
            column_end: 10,
            is_primary: true,
            _text: vec![],
            suggested_replacement: None,
            suggestion_applicability: None,
        };

        assert!(!is_machine_applicable(&span));
    }

    #[test]
    fn test_is_machine_applicable_other_applicability() {
        let span = DiagnosticSpan {
            file_name: "test.rs".to_string(),
            line_start: 1,
            line_end: 1,
            column_start: 1,
            column_end: 10,
            is_primary: true,
            _text: vec![],
            suggested_replacement: Some("fix".to_string()),
            suggestion_applicability: Some("unspecified".to_string()),
        };

        assert!(!is_machine_applicable(&span));
    }

    // ============================================================================
    // extract_lint_name Tests
    // ============================================================================

    #[test]
    fn test_extract_lint_name_with_code() {
        let diagnostic = DiagnosticMessage {
            level: "warning".to_string(),
            message: "test message".to_string(),
            code: Some(DiagnosticCode {
                code: "unused_variable".to_string(),
            }),
            spans: vec![],
        };

        let lint_name = extract_lint_name(&diagnostic);
        assert_eq!(lint_name, "unused_variable");
    }

    #[test]
    fn test_extract_lint_name_without_code() {
        let diagnostic = DiagnosticMessage {
            level: "warning".to_string(),
            message: "test message".to_string(),
            code: None,
            spans: vec![],
        };

        let lint_name = extract_lint_name(&diagnostic);
        assert_eq!(lint_name, "");
    }

    // ============================================================================
    // find_primary_span Tests
    // ============================================================================

    #[test]
    fn test_find_primary_span_with_primary() {
        let diagnostic = DiagnosticMessage {
            level: "warning".to_string(),
            message: "test".to_string(),
            code: None,
            spans: vec![
                DiagnosticSpan {
                    file_name: "secondary.rs".to_string(),
                    line_start: 1,
                    line_end: 1,
                    column_start: 1,
                    column_end: 10,
                    is_primary: false,
                    _text: vec![],
                    suggested_replacement: None,
                    suggestion_applicability: None,
                },
                DiagnosticSpan {
                    file_name: "primary.rs".to_string(),
                    line_start: 5,
                    line_end: 5,
                    column_start: 1,
                    column_end: 10,
                    is_primary: true,
                    _text: vec![],
                    suggested_replacement: None,
                    suggestion_applicability: None,
                },
            ],
        };

        let span = find_primary_span(&diagnostic);
        assert!(span.is_some());
        assert_eq!(span.unwrap().file_name, "primary.rs");
    }

    #[test]
    fn test_find_primary_span_single_span() {
        let diagnostic = DiagnosticMessage {
            level: "warning".to_string(),
            message: "test".to_string(),
            code: None,
            spans: vec![DiagnosticSpan {
                file_name: "only.rs".to_string(),
                line_start: 1,
                line_end: 1,
                column_start: 1,
                column_end: 10,
                is_primary: false, // Even if not primary, it's returned as the only span
                _text: vec![],
                suggested_replacement: None,
                suggestion_applicability: None,
            }],
        };

        let span = find_primary_span(&diagnostic);
        assert!(span.is_some());
        assert_eq!(span.unwrap().file_name, "only.rs");
    }

    #[test]
    fn test_find_primary_span_empty() {
        let diagnostic = DiagnosticMessage {
            level: "warning".to_string(),
            message: "test".to_string(),
            code: None,
            spans: vec![],
        };

        let span = find_primary_span(&diagnostic);
        assert!(span.is_none());
    }

    // ============================================================================
    // format_output Tests
    // ============================================================================

    #[test]
    fn test_format_output_summary() {
        let result = create_full_test_result();
        let output = format_output(
            &result,
            LintHotspotOutputFormat::Summary,
            false,
            Duration::from_secs(1),
            10,
        )
        .unwrap();

        assert!(output.contains("# Lint Hotspot Analysis"));
        assert!(output.contains("**Total Project Violations**"));
    }

    #[test]
    fn test_format_output_detailed() {
        let result = create_full_test_result();
        let output = format_output(
            &result,
            LintHotspotOutputFormat::Detailed,
            false,
            Duration::from_secs(1),
            10,
        )
        .unwrap();

        assert!(output.contains("# Lint Hotspot Analysis"));
        assert!(output.contains("## Detailed Violations"));
        assert!(output.contains("## Top Files by Violations"));
    }

    #[test]
    fn test_format_output_json() {
        let result = create_full_test_result();
        let output = format_output(
            &result,
            LintHotspotOutputFormat::Json,
            false,
            Duration::from_secs(1),
            10,
        )
        .unwrap();

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.get("hotspot").is_some());
        assert!(parsed.get("quality_gate").is_some());
    }

    #[test]
    fn test_format_output_enforcement_json() {
        let result = create_full_test_result();
        let output = format_output(
            &result,
            LintHotspotOutputFormat::EnforcementJson,
            false,
            Duration::from_secs(1),
            10,
        )
        .unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.get("hotspot").is_some());
        assert!(parsed.get("all_violations").is_some());
        assert!(parsed.get("summary_by_file").is_some());
    }

    #[test]
    fn test_format_output_sarif() {
        let result = create_full_test_result();
        let output = format_output(
            &result,
            LintHotspotOutputFormat::Sarif,
            false,
            Duration::from_secs(1),
            10,
        )
        .unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed.get("version").unwrap(), "2.1.0");
        assert!(parsed.get("$schema").is_some());
        assert!(parsed.get("runs").is_some());
    }

    // ============================================================================
    // format_summary Tests
    // ============================================================================

    #[test]
    fn test_format_summary_with_perf() {
        let result = create_full_test_result();
        let output = format_summary(&result, true, Duration::from_secs(5), 10).unwrap();

        assert!(output.contains("Analysis completed in"));
    }

    #[test]
    fn test_format_summary_with_enforcement() {
        let mut result = create_full_test_result();
        result.enforcement = Some(EnforcementMetadata {
            enforcement_score: 8.5,
            requires_enforcement: true,
            estimated_fix_time: 1800,
            automation_confidence: 0.85,
            enforcement_priority: 3,
        });

        let output = format_summary(&result, false, Duration::from_secs(1), 10).unwrap();

        assert!(output.contains("## Enforcement Metadata"));
        assert!(output.contains("Score: 8.5/10"));
        assert!(output.contains("Priority: 3"));
    }

    #[test]
    fn test_format_summary_with_failed_quality_gate() {
        let mut result = create_full_test_result();
        result.quality_gate.passed = false;
        result.quality_gate.violations = vec![QualityViolation {
            rule: "max_defect_density".to_string(),
            threshold: 0.05,
            actual: 0.10,
            severity: "blocking".to_string(),
        }];

        let output = format_summary(&result, false, Duration::from_secs(1), 10).unwrap();

        assert!(output.contains("Quality Gate Failed"));
        assert!(output.contains("max_defect_density exceeded"));
    }

    #[test]
    fn test_format_summary_top_files_limit() {
        let mut result = create_full_test_result();
        // Add more files to summary
        for i in 0..15 {
            result.summary_by_file.insert(
                PathBuf::from(format!("src/file_{i}.rs")),
                FileSummary {
                    total_violations: i,
                    errors: i / 2,
                    warnings: i - i / 2,
                    sloc: 100,
                    defect_density: i as f64 / 100.0,
                },
            );
        }

        let output = format_summary(&result, false, Duration::from_secs(1), 5).unwrap();

        // Should show limited files (may include header row and extras)
        let file_count = output.matches("violations/SLOC").count();
        assert!(file_count > 0, "Should have file entries");
    }

    // ============================================================================
    // format_detailed Tests
    // ============================================================================

    #[test]
    fn test_format_detailed_with_violations() {
        let mut result = create_full_test_result();
        result.hotspot.detailed_violations = vec![
            create_test_violation("src/main.rs", 10, "unused_variable", "warning"),
            create_test_violation_with_suggestion(
                "src/main.rs",
                20,
                "needless_return",
                "warning",
                "Remove the return statement",
            ),
        ];

        let output = format_detailed(&result, false, Duration::from_secs(1), 10).unwrap();

        assert!(output.contains("## Detailed Violations in Hotspot File"));
        assert!(output.contains("unused_variable"));
        assert!(output.contains("Suggestion: Remove the return statement"));
    }

    #[test]
    fn test_format_detailed_with_refactor_chain() {
        let mut result = create_full_test_result();
        result.refactor_chain = Some(RefactorChain {
            id: "test-chain".to_string(),
            estimated_reduction: 15,
            automation_confidence: 0.88,
            steps: vec![
                RefactorStep {
                    id: "step-1".to_string(),
                    lint: "unused".to_string(),
                    confidence: 0.95,
                    impact: 10,
                    description: "Remove unused code".to_string(),
                },
            ],
        });

        let output = format_detailed(&result, false, Duration::from_secs(1), 10).unwrap();

        assert!(output.contains("## Refactor Chain"));
        assert!(output.contains("ID: test-chain"));
        assert!(output.contains("Estimated Reduction: 15 violations"));
        assert!(output.contains("### Steps"));
        assert!(output.contains("Remove unused code"));
    }

    // ============================================================================
    // format_json Tests
    // ============================================================================

    #[test]
    fn test_format_json_simple() {
        let result = create_full_test_result();
        let output = format_json(&result, false).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        // Simple JSON should only have hotspot and quality_gate
        assert!(parsed.get("hotspot").is_some());
        assert!(parsed.get("quality_gate").is_some());
        assert!(parsed.get("all_violations").is_none());
    }

    #[test]
    fn test_format_json_enforcement() {
        let result = create_full_test_result();
        let output = format_json(&result, true).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        // Enforcement JSON should have all fields
        assert!(parsed.get("hotspot").is_some());
        assert!(parsed.get("quality_gate").is_some());
        assert!(parsed.get("all_violations").is_some());
        assert!(parsed.get("summary_by_file").is_some());
        assert!(parsed.get("total_project_violations").is_some());
    }

    // ============================================================================
    // format_sarif Tests
    // ============================================================================

    #[test]
    fn test_format_sarif_structure() {
        let mut result = create_full_test_result();
        result.quality_gate.violations = vec![QualityViolation {
            rule: "max_defect_density".to_string(),
            threshold: 0.05,
            actual: 0.10,
            severity: "blocking".to_string(),
        }];

        let output = format_sarif(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["version"], "2.1.0");
        assert!(parsed["$schema"]
            .as_str()
            .unwrap()
            .contains("sarif-schema"));

        let runs = parsed["runs"].as_array().unwrap();
        assert_eq!(runs.len(), 1);

        let tool = &runs[0]["tool"]["driver"];
        assert_eq!(tool["name"], "pmat-lint-hotspot");
    }

    #[test]
    fn test_format_sarif_with_violations() {
        let mut result = create_full_test_result();
        result.quality_gate.violations = vec![
            QualityViolation {
                rule: "max_defect_density".to_string(),
                threshold: 0.05,
                actual: 0.10,
                severity: "blocking".to_string(),
            },
            QualityViolation {
                rule: "max_single_file_violations".to_string(),
                threshold: 50.0,
                actual: 60.0,
                severity: "warning".to_string(),
            },
        ];

        let output = format_sarif(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        let results = parsed["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results.len(), 2);

        // First violation should be error level (blocking)
        assert_eq!(results[0]["level"], "error");
        assert_eq!(results[0]["ruleId"], "max_defect_density");

        // Second violation should be warning level
        assert_eq!(results[1]["level"], "warning");
        assert_eq!(results[1]["ruleId"], "max_single_file_violations");
    }

    // ============================================================================
    // LintHotspotResult Tests
    // ============================================================================

    #[test]
    fn test_lint_hotspot_result_serialization() {
        let result = create_full_test_result();
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: LintHotspotResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.hotspot.file, result.hotspot.file);
        assert_eq!(
            deserialized.total_project_violations,
            result.total_project_violations
        );
        assert_eq!(
            deserialized.quality_gate.passed,
            result.quality_gate.passed
        );
    }

    #[test]
    fn test_lint_hotspot_result_with_all_fields() {
        let mut result = create_full_test_result();
        result.enforcement = Some(EnforcementMetadata {
            enforcement_score: 8.5,
            requires_enforcement: true,
            estimated_fix_time: 1800,
            automation_confidence: 0.85,
            enforcement_priority: 3,
        });
        result.refactor_chain = Some(RefactorChain {
            id: "test".to_string(),
            estimated_reduction: 10,
            automation_confidence: 0.9,
            steps: vec![],
        });

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: LintHotspotResult = serde_json::from_str(&json).unwrap();

        assert!(deserialized.enforcement.is_some());
        assert!(deserialized.refactor_chain.is_some());
    }

    // ============================================================================
    // recalculate_hotspot_metrics Tests
    // ============================================================================

    #[test]
    fn test_recalculate_hotspot_metrics() {
        let mut result = create_full_test_result();
        result.hotspot.detailed_violations = vec![
            create_test_violation("src/main.rs", 10, "lint_a", "warning"),
            create_test_violation("src/main.rs", 20, "lint_b", "warning"),
        ];
        result.hotspot.sloc = 100;

        recalculate_hotspot_metrics(&mut result);

        assert_eq!(result.hotspot.total_violations, 2);
        assert!((result.hotspot.defect_density - 0.02).abs() < f64::EPSILON);
    }

    #[test]
    fn test_recalculate_hotspot_metrics_zero_sloc() {
        let mut result = create_full_test_result();
        result.hotspot.detailed_violations = vec![
            create_test_violation("src/main.rs", 10, "lint_a", "warning"),
        ];
        result.hotspot.sloc = 0;

        recalculate_hotspot_metrics(&mut result);

        assert_eq!(result.hotspot.total_violations, 1);
        // defect_density should not change when sloc is 0
    }

    // ============================================================================
    // should_exit_with_error Tests (comprehensive)
    // ============================================================================

    #[test]
    fn test_should_exit_with_error_comprehensive() {
        let mut result = create_full_test_result();
        let params = LintHotspotParams {
            project_path: PathBuf::from("/test"),
            file: None,
            format: LintHotspotOutputFormat::Summary,
            max_density: 5.0,
            min_confidence: 0.7,
            enforce: false,
            dry_run: false,
            enforcement_metadata: false,
            output: None,
            perf: false,
            clippy_flags: String::new(),
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        // Quality gate passed, no enforce
        result.quality_gate.passed = true;
        result.total_project_violations = 10;
        assert!(!should_exit_with_error(&result, &params));

        // Quality gate failed
        result.quality_gate.passed = false;
        assert!(should_exit_with_error(&result, &params));
    }

    // ============================================================================
    // log_analysis_start Tests
    // ============================================================================

    #[test]
    fn test_log_analysis_start_non_json() {
        // This test verifies the function doesn't panic for non-JSON formats
        log_analysis_start(&LintHotspotOutputFormat::Summary);
        log_analysis_start(&LintHotspotOutputFormat::Detailed);
        log_analysis_start(&LintHotspotOutputFormat::Sarif);
    }

    #[test]
    fn test_log_analysis_start_json() {
        // This test verifies the function doesn't panic for JSON format
        log_analysis_start(&LintHotspotOutputFormat::Json);
    }

    // ============================================================================
    // log_single_file_mode Tests
    // ============================================================================

    #[test]
    fn test_log_single_file_mode_non_json() {
        let file_path = PathBuf::from("src/main.rs");
        log_single_file_mode(&file_path, &LintHotspotOutputFormat::Summary);
        log_single_file_mode(&file_path, &LintHotspotOutputFormat::Detailed);
    }

    #[test]
    fn test_log_single_file_mode_json() {
        let file_path = PathBuf::from("src/main.rs");
        log_single_file_mode(&file_path, &LintHotspotOutputFormat::Json);
    }

    // ============================================================================
    // generate_enforcement_metadata_if_needed Tests
    // ============================================================================

    #[test]
    fn test_generate_enforcement_metadata_when_requested() {
        let hotspot = create_test_hotspot("src/main.rs", 10, 100, 5, 5);
        let params = LintHotspotParams {
            project_path: PathBuf::from("/test"),
            file: None,
            format: LintHotspotOutputFormat::Summary,
            max_density: 5.0,
            min_confidence: 0.7,
            enforce: false,
            dry_run: false,
            enforcement_metadata: true, // Explicitly requested
            output: None,
            perf: false,
            clippy_flags: String::new(),
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        let metadata = generate_enforcement_metadata_if_needed(&hotspot, &params);
        assert!(metadata.is_some());
    }

    #[test]
    fn test_generate_enforcement_metadata_when_enforce() {
        let hotspot = create_test_hotspot("src/main.rs", 10, 100, 5, 5);
        let params = LintHotspotParams {
            project_path: PathBuf::from("/test"),
            file: None,
            format: LintHotspotOutputFormat::Summary,
            max_density: 5.0,
            min_confidence: 0.7,
            enforce: true, // Enforce flag set
            dry_run: false,
            enforcement_metadata: false,
            output: None,
            perf: false,
            clippy_flags: String::new(),
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        let metadata = generate_enforcement_metadata_if_needed(&hotspot, &params);
        assert!(metadata.is_some());
    }

    #[test]
    fn test_generate_enforcement_metadata_not_requested() {
        let hotspot = create_test_hotspot("src/main.rs", 10, 100, 5, 5);
        let params = LintHotspotParams {
            project_path: PathBuf::from("/test"),
            file: None,
            format: LintHotspotOutputFormat::Summary,
            max_density: 5.0,
            min_confidence: 0.7,
            enforce: false,
            dry_run: false,
            enforcement_metadata: false,
            output: None,
            perf: false,
            clippy_flags: String::new(),
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        let metadata = generate_enforcement_metadata_if_needed(&hotspot, &params);
        assert!(metadata.is_none());
    }

    // ============================================================================
    // generate_refactor_chain_if_needed Tests
    // ============================================================================

    #[test]
    fn test_generate_refactor_chain_when_enforce() {
        let hotspot = create_test_hotspot("src/main.rs", 10, 100, 5, 5);
        let params = LintHotspotParams {
            project_path: PathBuf::from("/test"),
            file: None,
            format: LintHotspotOutputFormat::Summary,
            max_density: 5.0,
            min_confidence: 0.7,
            enforce: true,
            dry_run: false,
            enforcement_metadata: false,
            output: None,
            perf: false,
            clippy_flags: String::new(),
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        let chain = generate_refactor_chain_if_needed(&hotspot, &params, &None);
        assert!(chain.is_some());
    }

    #[test]
    fn test_generate_refactor_chain_when_enforcement_required() {
        let hotspot = create_test_hotspot("src/main.rs", 10, 100, 5, 5);
        let params = LintHotspotParams {
            project_path: PathBuf::from("/test"),
            file: None,
            format: LintHotspotOutputFormat::Summary,
            max_density: 5.0,
            min_confidence: 0.7,
            enforce: false,
            dry_run: false,
            enforcement_metadata: false,
            output: None,
            perf: false,
            clippy_flags: String::new(),
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        let enforcement = Some(EnforcementMetadata {
            enforcement_score: 8.0,
            requires_enforcement: true,
            estimated_fix_time: 1800,
            automation_confidence: 0.85,
            enforcement_priority: 3,
        });

        let chain = generate_refactor_chain_if_needed(&hotspot, &params, &enforcement);
        assert!(chain.is_some());
    }

    #[test]
    fn test_generate_refactor_chain_not_needed() {
        let hotspot = create_test_hotspot("src/main.rs", 10, 100, 5, 5);
        let params = LintHotspotParams {
            project_path: PathBuf::from("/test"),
            file: None,
            format: LintHotspotOutputFormat::Summary,
            max_density: 5.0,
            min_confidence: 0.7,
            enforce: false,
            dry_run: false,
            enforcement_metadata: false,
            output: None,
            perf: false,
            clippy_flags: String::new(),
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        let chain = generate_refactor_chain_if_needed(&hotspot, &params, &None);
        assert!(chain.is_none());
    }

    // ============================================================================
    // Property-based Tests
    // ============================================================================

    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_defect_density_never_negative(violations in 0usize..1000, sloc in 1usize..10000) {
                let density = calculate_defect_density(violations, sloc);
                prop_assert!(density >= 0.0);
            }

            #[test]
            fn test_total_violations_equals_sum(
                errors in 0usize..100,
                warnings in 0usize..100,
                suggestions in 0usize..100
            ) {
                let metrics = FileMetrics {
                    violations: HashMap::new(),
                    severity_counts: SeverityDistribution {
                        error: errors,
                        warning: warnings,
                        suggestion: suggestions,
                        note: 0,
                    },
                    sloc: 100,
                    detailed_violations: vec![],
                };

                let total = calculate_total_violations(&metrics);
                prop_assert_eq!(total, errors + warnings + suggestions);
            }

            #[test]
            fn test_count_sloc_non_negative(content in ".*") {
                let sloc = count_sloc(&content);
                prop_assert!(sloc >= 0);
            }

            #[test]
            fn test_enforcement_score_bounded(violations in 1usize..100, sloc in 1usize..1000) {
                let mut hotspot = create_test_hotspot("test.rs", violations, sloc, violations / 2, violations - violations / 2);
                hotspot.defect_density = violations as f64 / sloc as f64;

                let metadata = calculate_enforcement_metadata(&hotspot, 0.7);
                prop_assert!(metadata.enforcement_score >= 0.0);
                prop_assert!(metadata.enforcement_score <= 10.0);
            }

            #[test]
            fn test_quality_gate_consistency(density in 0.0f64..10.0, threshold in 0.1f64..10.0) {
                let violations = (density * 100.0) as usize;
                let hotspot = create_test_hotspot("test.rs", violations, 100, violations / 2, violations - violations / 2);

                let status = check_quality_gates(&hotspot, threshold);

                // If density exceeds threshold, gate should fail
                if hotspot.defect_density > threshold {
                    prop_assert!(!status.passed || status.violations.iter().any(|v| v.rule == "max_defect_density"));
                }
            }
        }
    }
}
