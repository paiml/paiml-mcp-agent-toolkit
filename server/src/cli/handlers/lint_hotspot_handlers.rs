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
    #[serde(default)]
    #[allow(dead_code)]
    text: Vec<DiagnosticText>,
    #[serde(default)]
    suggested_replacement: Option<String>,
    #[serde(default)]
    suggestion_applicability: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DiagnosticText {
    text: String,
    highlight_start: u32,
    highlight_end: u32,
}

/// Handle analyze lint-hotspot command
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
) -> Result<()> {
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

    if params.format != LintHotspotOutputFormat::Json {
        eprintln!("🔍 Running Clippy analysis...");
    }

    // Run clippy and analyze output
    let result = if let Some(ref file_path) = params.file {
        // Single file mode - analyze only the specified file
        if params.format != LintHotspotOutputFormat::Json {
            eprintln!("📄 Analyzing single file: {}", file_path.display());
        }
        run_clippy_analysis_single_file(&params.project_path, file_path, &params.clippy_flags)
            .await?
    } else {
        // Normal mode - find the hotspot
        run_clippy_analysis(&params.project_path, &params.clippy_flags).await?
    };

    // Generate enforcement metadata if requested
    let enforcement = if params.enforcement_metadata || params.enforce {
        Some(calculate_enforcement_metadata(
            &result.hotspot,
            params.min_confidence,
        ))
    } else {
        None
    };

    // Generate refactor chain if enforcement is needed
    let refactor_chain =
        if params.enforce || (enforcement.as_ref().is_some_and(|e| e.requires_enforcement)) {
            Some(generate_refactor_chain(
                &result.hotspot,
                params.min_confidence,
            ))
        } else {
            None
        };

    // Check quality gates
    let quality_gate = check_quality_gates(&result.hotspot, params.max_density);

    let final_result = LintHotspotResult {
        hotspot: result.hotspot,
        all_violations: result.all_violations,
        summary_by_file: result.summary_by_file,
        total_project_violations: result.total_project_violations,
        enforcement,
        refactor_chain,
        quality_gate,
    };

    // Format and output results
    let output_content = format_output(
        &final_result,
        params.format,
        params.perf,
        start_time.elapsed(),
    )?;

    if let Some(output_path) = params.output {
        tokio::fs::write(output_path, &output_content).await?;
    } else {
        println!("{}", output_content);
    }

    // Execute enforcement if requested
    if params.enforce && !params.dry_run && final_result.quality_gate.blocking {
        eprintln!("🚨 Enforcement required - executing refactor chain...");
        // In a real implementation, this would execute the refactor chain
        eprintln!("⚠️  Enforcement execution not yet implemented");
    }

    // Exit with non-zero code if quality gate failed
    if !final_result.quality_gate.passed {
        std::process::exit(1);
    }

    Ok(())
}

/// Run clippy and analyze the JSON output
///
/// # Errors
///
/// Returns an error if the operation fails
async fn run_clippy_analysis(project_path: &Path, clippy_flags: &str) -> Result<LintHotspotResult> {
    // Parse clippy flags
    let flags: Vec<&str> = clippy_flags.split_whitespace().collect();

    // Find workspace root if we're in a workspace
    let workspace_root = find_workspace_root(project_path)?;

    // Run clippy with JSON output and all targets (extreme quality)
    let mut cmd = Command::new("cargo");
    cmd.current_dir(project_path)
        .arg("clippy")
        .arg("--all-targets")
        .arg("--message-format=json");

    // Add clippy flags after -- separator
    if !flags.is_empty() {
        cmd.arg("--");
        cmd.args(&flags);
    }

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let output = cmd.output().await.context("Failed to run cargo clippy")?;

    // Check if clippy failed
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("⚠️  Clippy failed with status: {:?}", output.status);
        eprintln!("Stderr: {}", stderr);
        // Continue anyway - we might have partial output
    }

    // Parse JSON output line by line
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
    // Only print debug info if not in JSON mode
    if std::env::var("LINT_HOTSPOT_DEBUG").is_ok() {
        eprintln!("📊 Processed {} compiler messages", message_count);
        eprintln!("📁 Files with metrics: {}", file_metrics.len());
    }

    // Calculate SLOC for each file
    for (file_path, metrics) in file_metrics.iter_mut() {
        // Try multiple path resolutions to find the actual file
        let actual_path = if file_path.exists() {
            file_path.clone()
        } else if let Some(ws_root) = &workspace_root {
            // Try relative to workspace root
            let ws_relative = ws_root.join(file_path);
            if ws_relative.exists() {
                ws_relative
            } else {
                // Try with "server/" prefix if not present
                let with_server = ws_root.join("server").join(file_path);
                if with_server.exists() {
                    with_server
                } else {
                    // Try relative to project_path
                    let project_relative = project_path.join(file_path);
                    if project_relative.exists() {
                        project_relative
                    } else {
                        file_path.clone()
                    }
                }
            }
        } else {
            // No workspace, try relative to project_path
            let project_relative = project_path.join(file_path);
            if project_relative.exists() {
                project_relative
            } else {
                file_path.clone()
            }
        };

        if actual_path.exists() {
            let content = tokio::fs::read_to_string(&actual_path).await?;
            metrics.sloc = content
                .lines()
                .filter(|line| !line.trim().is_empty() && !line.trim().starts_with("//"))
                .count();
            if std::env::var("LINT_HOTSPOT_DEBUG").is_ok() && metrics.sloc > 0 {
                eprintln!("✓ File {} has {} SLOC", actual_path.display(), metrics.sloc);
            }
        } else if std::env::var("LINT_HOTSPOT_DEBUG").is_ok() {
            eprintln!("⚠️  Could not find file: {}", file_path.display());
            eprintln!("   Tried: {}", actual_path.display());
            if let Some(ws) = &workspace_root {
                eprintln!("   Workspace root: {}", ws.display());
            }
        }
    }

    // Collect all violations across the project
    let mut all_violations = Vec::new();
    let mut summary_by_file = HashMap::new();
    let mut total_project_violations = 0;

    for (file_path, metrics) in &file_metrics {
        all_violations.extend(metrics.detailed_violations.clone());

        let total_file_violations = metrics.severity_counts.error
            + metrics.severity_counts.warning
            + metrics.severity_counts.suggestion;

        total_project_violations += total_file_violations;

        let defect_density = if metrics.sloc > 0 {
            (total_file_violations as f64) / (metrics.sloc as f64)
        } else {
            0.0
        };

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

    // Find the file with highest defect density (including detailed violations)
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
    // Parse clippy flags
    let flags: Vec<&str> = clippy_flags.split_whitespace().collect();

    // Run clippy with JSON output
    let mut cmd = Command::new("cargo");
    cmd.current_dir(project_path)
        .arg("clippy")
        .arg("--message-format=json");

    // Add clippy flags after -- separator
    if !flags.is_empty() {
        cmd.arg("--");
        cmd.args(&flags);
    }

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let output = cmd.output().await.context("Failed to run cargo clippy")?;

    // Parse JSON output line by line
    let reader = BufReader::new(output.stdout.as_slice());
    let mut file_violations = Vec::new();
    let mut all_violations = Vec::new();
    let mut severity_dist = SeverityDistribution::default();

    // Convert file_path to absolute path for comparison
    let abs_file_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        project_path.join(file_path)
    };

    for line in std::io::BufRead::lines(reader) {
        let line = line?;
        if let Ok(msg) = serde_json::from_str::<ClippyMessage>(&line) {
            if let (Some("compiler-message"), Some(diagnostic)) =
                (msg.reason.as_deref(), &msg.message)
            {
                // Check if this diagnostic is for our target file
                if let Some(span) = diagnostic
                    .spans
                    .iter()
                    .find(|s| s.is_primary || diagnostic.spans.len() == 1)
                {
                    let diagnostic_path = PathBuf::from(&span.file_name);

                    // Check if this is our target file (handle both absolute and relative paths)
                    let matches = diagnostic_path == abs_file_path
                        || diagnostic_path == *file_path
                        || diagnostic_path.ends_with(file_path);

                    if matches {
                        // Create violation detail
                        let violation = ViolationDetail {
                            file: file_path.to_path_buf(),
                            line: span.line_start,
                            column: span.column_start,
                            end_line: span.line_end,
                            end_column: span.column_end,
                            lint_name: diagnostic
                                .code
                                .as_ref()
                                .map(|c| c.code.clone())
                                .unwrap_or_default(),
                            message: diagnostic.message.clone(),
                            severity: diagnostic.level.clone(),
                            suggestion: span.suggested_replacement.clone(),
                            machine_applicable: span
                                .suggestion_applicability
                                .as_ref()
                                .map(|a| a == "machine-applicable" || a == "maybe-incorrect")
                                .unwrap_or(false),
                        };

                        file_violations.push(violation.clone());
                        all_violations.push(violation);

                        // Update severity distribution
                        match diagnostic.level.as_str() {
                            "error" => severity_dist.error += 1,
                            "warning" => severity_dist.warning += 1,
                            _ => severity_dist.note += 1,
                        }
                    }
                }
            }
        }
    }

    // Count lines in the file
    let sloc = count_source_lines(project_path, file_path)
        .await
        .unwrap_or(100);
    let total_violations = file_violations.len();
    let defect_density = (total_violations as f64 / sloc as f64) * 100.0;

    // Create hotspot for the single file
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
            passed: defect_density <= 5.0, // Use default threshold
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

        // Handle workspace paths - if path starts with "server/", strip it
        if let Ok(stripped) = file_path.strip_prefix("server/") {
            file_path = PathBuf::from(stripped);
        }

        // Skip non-Rust files (config files, etc.)
        if !file_path
            .extension()
            .map(|ext| ext == "rs")
            .unwrap_or(false)
        {
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
            .map(|c| c.code.clone())
            .unwrap_or_else(|| "unknown".to_string());

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
                .map(|a| a == "MachineApplicable")
                .unwrap_or(false),
        };

        metrics.detailed_violations.push(violation);
    }
}

/// Find the file with highest defect density
///
/// # Errors
///
/// Returns an error if the operation fails
#[allow(dead_code)]
fn find_hotspot(file_metrics: HashMap<PathBuf, FileMetrics>) -> Result<LintHotspot> {
    let mut hotspot_file = None;
    let mut max_density = 0.0;

    for (file_path, metrics) in file_metrics {
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
                detailed_violations: vec![], // Old function doesn't collect detailed violations
            });
        }
    }

    hotspot_file.ok_or_else(|| anyhow::anyhow!("No lint violations found"))
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
                id: format!("fix-{}", lint_code),
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
) -> Result<String> {
    match format {
        LintHotspotOutputFormat::Summary => format_summary(result, perf, elapsed),
        LintHotspotOutputFormat::Detailed => format_detailed(result, perf, elapsed),
        LintHotspotOutputFormat::Json => format_json(result, false),
        LintHotspotOutputFormat::EnforcementJson => format_json(result, true),
        LintHotspotOutputFormat::Sarif => format_sarif(result),
    }
}

/// Format summary output
fn format_summary(
    result: &LintHotspotResult,
    perf: bool,
    elapsed: std::time::Duration,
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

    output.push_str("## Hottest File\n");
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
        output.push_str(&format!("- {}: {} occurrences\n", lint, count));
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
) -> Result<String> {
    let mut output = format_summary(result, perf, elapsed)?;

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
            output.push_str(&format!("  Suggestion: {}\n", suggestion));
        }
    }

    // Add top files by violation count
    output.push_str("\n## Top Files by Violations\n");
    let mut sorted_files: Vec<_> = result.summary_by_file.iter().collect();
    sorted_files.sort_by(|a, b| b.1.total_violations.cmp(&a.1.total_violations));

    for (file, summary) in sorted_files.iter().take(10) {
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
