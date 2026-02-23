#![cfg_attr(coverage_nightly, coverage(off))]
//! Clippy integration, lint parsing, and diagnostic processing

use super::metrics::build_lint_hotspot_result;
use super::types::*;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

/// Run clippy and analyze the JSON output
///
/// # Errors
///
/// Returns an error if the operation fails
pub(crate) async fn run_clippy_analysis(
    project_path: &Path,
    clippy_flags: &str,
) -> Result<LintHotspotResult> {
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
pub(crate) async fn run_clippy_analysis_single_file(
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
pub(crate) fn count_top_lints(violations: &[ViolationDetail]) -> Vec<(String, usize)> {
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
pub(crate) async fn count_source_lines(project_path: &Path, file_path: &Path) -> Result<usize> {
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
pub(crate) fn process_diagnostic(
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

/// Execute clippy command with given flags (cognitive complexity <=3)
pub(crate) async fn execute_clippy_command(
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

/// Check clippy output status (cognitive complexity <=5)
pub(crate) fn check_clippy_output(output: &std::process::Output) -> Result<()> {
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

/// Parse clippy JSON output into file metrics (cognitive complexity <=8)
pub(crate) fn parse_clippy_json_output(
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

/// Calculate SLOC for each file in metrics
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

/// Resolve actual file path trying various locations
fn resolve_file_path(
    file_path: &Path,
    project_path: &Path,
    workspace_root: Option<&PathBuf>,
) -> PathBuf {
    if file_path.exists() {
        return file_path.to_path_buf();
    }

    if let Some(ws_root) = workspace_root {
        let ws_relative = ws_root.join(file_path);
        if ws_relative.exists() {
            return ws_relative;
        }

        let with_server = ws_root.join("server").join(file_path);
        if with_server.exists() {
            return with_server;
        }
    }

    let project_relative = project_path.join(file_path);
    if project_relative.exists() {
        project_relative
    } else {
        file_path.to_path_buf()
    }
}

/// Count source lines of code (non-empty, non-comment lines)
fn count_sloc(content: &str) -> usize {
    content
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with("//"))
        .count()
}

/// Log SLOC debug info if enabled
fn log_sloc_debug(path: &Path, sloc: usize) {
    if std::env::var("LINT_HOTSPOT_DEBUG").is_ok() && sloc > 0 {
        eprintln!("✓ File {} has {} SLOC", path.display(), sloc);
    }
}

/// Log file not found debug info if enabled
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

/// Find workspace root by looking for Cargo.toml with [workspace]
///
/// # Errors
///
/// Returns an error if the operation fails
pub(crate) fn find_workspace_root(start_path: &Path) -> Result<Option<PathBuf>> {
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
