// Extended compliance checks - CB-300+, CB-081, file health, sovereign stack, PAIML deps
//
// Originally from check_handlers.rs and migrate_handlers.rs,
// contains the more specialized compliance check functions.

use crate::cli::handlers::comply_cb_detect::{
    detect_cb081_dependency_count, DependencyCountReport,
};
use crate::services::file_health::{
    scan_directory, FileHealthMetrics, FileHealthReport, DEFAULT_EXCLUDE_PATTERNS, RUST_EXTENSIONS,
};
use anyhow::Result;
use std::path::Path;

use super::types::*;

/// CB-300: Muda Waste Score (COMPLY-040)
pub(crate) fn check_muda_waste_score(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    use crate::cli::handlers::comply_handlers::muda_handlers;
    let report = muda_handlers::calculate_muda_score(project_path);
    let mut message = format!(
        "Muda Score: {:.1}/100 ({}) - Over:{:.0} Wait:{:.0} Inv:{:.0} Proc:{:.0} Def:{:.0}",
        report.total_score,
        report.grade,
        report.overproduction,
        report.waiting,
        report.inventory,
        report.over_processing,
        report.defects
    );

    // Append file-level details for non-zero categories
    for (category, files) in &report.file_details {
        if !files.is_empty() {
            message.push_str(&format!("\n    {}: {}", category, files.join(", ")));
        }
    }

    let (status, severity) = match report.grade {
        muda_handlers::MudaGrade::Lean | muda_handlers::MudaGrade::Efficient => {
            (CheckStatus::Pass, Severity::Info)
        }
        muda_handlers::MudaGrade::Moderate => (CheckStatus::Warn, Severity::Warning),
        muda_handlers::MudaGrade::High | muda_handlers::MudaGrade::Critical => {
            (CheckStatus::Fail, Severity::Error)
        }
    };
    ComplianceCheck {
        name: "CB-300: Muda Waste Score".into(),
        status,
        message,
        severity,
    }
}

/// CB-301: Reproducibility Level Check (COMPLY-041)
pub(crate) fn check_reproducibility_level(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    use crate::cli::handlers::comply_handlers::reproducibility_handlers;
    let report = reproducibility_handlers::check_reproducibility(project_path);
    let detail_summary: String = report
        .details
        .iter()
        .take(3)
        .cloned()
        .collect::<Vec<_>>()
        .join("; ");
    let message = format!("Reproducibility: {} - {}", report.level, detail_summary);
    let (status, severity) = match report.level {
        reproducibility_handlers::ReproducibilityLevel::Gold
        | reproducibility_handlers::ReproducibilityLevel::Silver => {
            (CheckStatus::Pass, Severity::Info)
        }
        reproducibility_handlers::ReproducibilityLevel::Bronze => {
            (CheckStatus::Warn, Severity::Warning)
        }
        reproducibility_handlers::ReproducibilityLevel::None => {
            (CheckStatus::Fail, Severity::Error)
        }
    };
    ComplianceCheck {
        name: "CB-301: Reproducibility Level".into(),
        status,
        message,
        severity,
    }
}

/// CB-302: Golden Trace Drift Detection (COMPLY-042)
pub(crate) fn check_golden_trace_drift(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    use crate::cli::handlers::comply_handlers::reproducibility_handlers;
    match reproducibility_handlers::check_golden_trace_drift(project_path) {
        None => ComplianceCheck {
            name: "CB-302: Golden Trace Drift".into(),
            status: CheckStatus::Skip,
            message: "No renacer.toml configured - golden tracing not enabled".into(),
            severity: Severity::Info,
        },
        Some(true) => ComplianceCheck {
            name: "CB-302: Golden Trace Drift".into(),
            status: CheckStatus::Pass,
            message: "Golden traces valid - no drift detected".into(),
            severity: Severity::Info,
        },
        Some(false) => ComplianceCheck {
            name: "CB-302: Golden Trace Drift".into(),
            status: CheckStatus::Fail,
            message: "Golden trace drift detected - run 'renacer validate' to investigate".into(),
            severity: Severity::Error,
        },
    }
}

/// CB-303: Equation-Driven Development Compliance (COMPLY-043)
pub(crate) fn check_edd_compliance(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    use crate::cli::handlers::comply_handlers::edd_handlers;
    let report = edd_handlers::check_edd_compliance(project_path);
    if !report.is_simulation_project {
        return ComplianceCheck {
            name: "CB-303: EDD Compliance".into(),
            status: CheckStatus::Skip,
            message: "Not a simulation project (no simular/trueno-sim dependency)".into(),
            severity: Severity::Info,
        };
    }
    let violation_count = report.undocumented_fns.len();
    let message = format!(
        "EDD: {:.0}% ({}/{} pub fns documented with math)",
        report.compliance_pct, report.documented_fns, report.total_simulation_fns
    );
    if report.compliance_pct >= 80.0 {
        ComplianceCheck {
            name: "CB-303: EDD Compliance".into(),
            status: CheckStatus::Pass,
            message,
            severity: Severity::Info,
        }
    } else if violation_count > 0 {
        ComplianceCheck {
            name: "CB-303: EDD Compliance".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} - {} functions missing mathematical models",
                message, violation_count
            ),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "CB-303: EDD Compliance".into(),
            status: CheckStatus::Pass,
            message,
            severity: Severity::Info,
        }
    }
}

/// CB-304: Dead Code Percentage
pub(crate) fn check_dead_code_percentage(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let config = crate::models::deep_context_config::DeepContextConfig::default();
    let threshold_pct = config.dead_code_threshold * 100.0;
    let source_dirs: Vec<std::path::PathBuf> = ["src", "crates", "lean", "lib"]
        .iter()
        .map(|d| project_path.join(d))
        .filter(|d| d.exists() && d.is_dir())
        .collect();
    if source_dirs.is_empty() {
        return ComplianceCheck {
            name: "CB-304: Dead Code Percentage".into(),
            status: CheckStatus::Skip,
            message: "No source directory found (checked src/, crates/, lean/, lib/)".into(),
            severity: Severity::Info,
        };
    }
    let (mut total_items, mut dead_items, mut total_lines, mut dead_lines) = (0, 0, 0, 0);
    for src_dir in &source_dirs {
        let (ti, di, tl, dl) = super::check_dead_code::scan_dead_code_indicators(src_dir);
        total_items += ti;
        dead_items += di;
        total_lines += tl;
        dead_lines += dl;
    }
    if total_items == 0 {
        return ComplianceCheck {
            name: "CB-304: Dead Code Percentage".into(),
            status: CheckStatus::Pass,
            message: "No code items found to analyze".into(),
            severity: Severity::Info,
        };
    }
    let dead_pct = if total_lines > 0 && dead_lines > 0 {
        (dead_lines as f64 / total_lines as f64) * 100.0
    } else {
        (dead_items as f64 / total_items as f64) * 100.0
    };
    let message = format!(
        "Dead code: {:.1}% ({} dead items/{} total, ~{} dead lines/{} total) [threshold: {:.0}%]",
        dead_pct, dead_items, total_items, dead_lines, total_lines, threshold_pct
    );
    if dead_pct <= threshold_pct {
        ComplianceCheck {
            name: "CB-304: Dead Code Percentage".into(),
            status: CheckStatus::Pass,
            message,
            severity: Severity::Info,
        }
    } else if dead_pct <= threshold_pct * 2.0 {
        ComplianceCheck {
            name: "CB-304: Dead Code Percentage".into(),
            status: CheckStatus::Warn,
            message: format!("{message} - exceeds threshold"),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "CB-304: Dead Code Percentage".into(),
            status: CheckStatus::Fail,
            message: format!("{message} - significantly exceeds threshold"),
            severity: Severity::Error,
        }
    }
}

fn format_dependency_message(report: &DependencyCountReport) -> String {
    let transitive_display = if let Some(prod) = report.prod_transitive_count {
        format!(
            "{} prod transitive ({} total w/dev)",
            prod, report.transitive_count
        )
    } else {
        format!("{} transitive", report.transitive_count)
    };
    let mut details = vec![format!(
        "{} direct, {}",
        report.direct_count, transitive_display
    )];
    if let Some(ref trend) = report.trend {
        if trend.direct_delta != 0 || trend.transitive_delta != 0 {
            details.push(format!(
                "\u{0394} {:+}/{:+} since last",
                trend.direct_delta, trend.transitive_delta
            ));
        }
    }
    if !report.duplicate_crates.is_empty() {
        details.push(format!("{} duplicates", report.duplicate_crates.len()));
    }
    details.push(format!("{:.0}% feature-gated", report.feature_gated_pct));
    if report.sovereign_bonus > 0 {
        details.push(format!(
            "+{} sovereign ({})",
            report.sovereign_bonus,
            report.sovereign_crates.join(", ")
        ));
    }
    format!("Score: {}/5 | {}", report.score, details.join(" | "))
}

fn append_violation_details(msg: &mut String, report: &DependencyCountReport, limit: usize) {
    for v in report.violations.iter().take(limit) {
        let icon = if v.severity == crate::cli::handlers::comply_cb_detect::Severity::Error {
            "\u{2717}"
        } else {
            "\u{26a0}"
        };
        msg.push_str(&format!("\n    {} {}", icon, v.description));
    }
}

/// CB-081: Dependency Count Check
pub(crate) fn check_dependency_count(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    if !project_path.join("Cargo.toml").exists() {
        return ComplianceCheck {
            name: "CB-081: Dependency Health".into(),
            status: CheckStatus::Skip,
            message: "Not a Rust project (no Cargo.toml)".into(),
            severity: Severity::Info,
        };
    }
    let report = detect_cb081_dependency_count(project_path);
    let message = format_dependency_message(&report);
    let has_critical = report
        .violations
        .iter()
        .any(|v| v.severity == crate::cli::handlers::comply_cb_detect::Severity::Error);
    if report.score >= 4 && !has_critical {
        ComplianceCheck {
            name: "CB-081: Dependency Health".into(),
            status: CheckStatus::Pass,
            message,
            severity: Severity::Info,
        }
    } else if report.score >= 2 && !has_critical {
        let mut msg = message;
        append_violation_details(&mut msg, &report, 1);
        ComplianceCheck {
            name: "CB-081: Dependency Health".into(),
            status: CheckStatus::Warn,
            message: msg,
            severity: Severity::Warning,
        }
    } else {
        let mut msg = message;
        append_violation_details(&mut msg, &report, 3);
        ComplianceCheck {
            name: "CB-081: Dependency Health".into(),
            status: CheckStatus::Fail,
            message: msg,
            severity: Severity::Error,
        }
    }
}

/// Detect project type and discover source files across all source directories.
pub(crate) fn discover_source_files(
    project_path: &Path,
) -> Result<Vec<std::path::PathBuf>, String> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let is_rust = project_path.join("Cargo.toml").exists();
    let is_lean = project_path.join("lakefile.lean").exists()
        || project_path.join("lean-toolchain").exists()
        || project_path.join("lean").join("lakefile.lean").exists()
        || project_path.join("lean").join("lean-toolchain").exists();
    if !is_rust && !is_lean {
        return Err("No Cargo.toml or lakefile.lean found".into());
    }
    let source_dirs: Vec<std::path::PathBuf> = ["src", "crates", "lean", "lib"]
        .iter()
        .map(|d| project_path.join(d))
        .filter(|d| d.exists() && d.is_dir())
        .collect();
    if source_dirs.is_empty() {
        return Err("No source directory found (checked src/, crates/, lean/, lib/)".into());
    }
    let extensions: &[&str] = if is_lean && !is_rust {
        &["lean"]
    } else if is_rust && is_lean {
        &["rs", "lean"]
    } else {
        RUST_EXTENSIONS
    };
    let mut files = Vec::new();
    for src_dir in &source_dirs {
        files.extend(scan_directory(
            src_dir,
            extensions,
            DEFAULT_EXCLUDE_PATTERNS,
        ));
    }
    Ok(files)
}

/// Check file health across the project (CB-040)
/// Analyze a single file for health metrics, returning (critical, problem, over_500) increments
fn classify_file_health(file_path: &std::path::PathBuf, content: &str) -> (usize, usize, usize) {
    let lines = content.lines().count();
    let is_test_file = file_path.to_string_lossy().contains("/tests/")
        || file_path
            .file_name()
            .map(|f| {
                let name = f.to_string_lossy();
                name.starts_with("test") || name.ends_with("_tests.rs")
            })
            .unwrap_or(false);
    let critical_threshold = if is_test_file { 4000 } else { 2000 };
    let problem_threshold = if is_test_file { 2000 } else { 1000 };
    let critical = usize::from(lines > critical_threshold);
    let problem = usize::from(lines <= critical_threshold && lines > problem_threshold);
    let over_500 = usize::from(lines > 500);
    (critical, problem, over_500)
}

/// Build the final ComplianceCheck from file health counts
fn build_file_health_check(
    report: &FileHealthReport,
    critical_count: usize,
    problem_count: usize,
    over_500_count: usize,
) -> ComplianceCheck {
    if critical_count > 0 {
        ComplianceCheck { name: "File Health".into(), status: CheckStatus::Fail, message: format!("CRITICAL: {} files >2000 lines, {} files >1000 lines, {} files >500 lines (avg health: {}%, grade: {})", critical_count, problem_count, over_500_count, report.average_health, report.average_grade.as_str()), severity: Severity::Critical }
    } else if problem_count > 0 || over_500_count > 5 {
        ComplianceCheck {
            name: "File Health".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} files >1000 lines, {} files >500 lines (avg health: {}%, grade: {})",
                problem_count,
                over_500_count,
                report.average_health,
                report.average_grade.as_str()
            ),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck { name: "File Health".into(), status: CheckStatus::Pass, message: format!("{} files analyzed, avg health: {}%, grade: {} (all files <500 lines or within tolerance)", report.total_files, report.average_health, report.average_grade.as_str()), severity: Severity::Info }
    }
}

pub(crate) fn check_file_health(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let files = match discover_source_files(project_path) {
        Ok(f) => f,
        Err(msg) => {
            return ComplianceCheck {
                name: "File Health".into(),
                status: CheckStatus::Skip,
                message: msg,
                severity: Severity::Info,
            }
        }
    };
    if files.is_empty() {
        return ComplianceCheck {
            name: "File Health".into(),
            status: CheckStatus::Pass,
            message: "No source files found".into(),
            severity: Severity::Info,
        };
    }
    let mut metrics: Vec<FileHealthMetrics> = Vec::new();
    let (mut critical_count, mut problem_count, mut over_500_count) = (0, 0, 0);
    for file_path in &files {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let (c, p, o) = classify_file_health(file_path, &content);
        critical_count += c;
        problem_count += p;
        over_500_count += o;
        let lines = content.lines().count();
        let test_lines = estimate_test_lines(&content);
        let avg_complexity = estimate_avg_complexity(&content);
        metrics.push(FileHealthMetrics::calculate(
            file_path.clone(),
            lines,
            test_lines,
            avg_complexity,
            0,
        ));
    }
    let report = FileHealthReport::from_files(project_path.to_path_buf(), metrics);
    build_file_health_check(&report, critical_count, problem_count, over_500_count)
}

pub(crate) fn estimate_test_lines(content: &str) -> usize {
    debug_assert!(!content.is_empty(), "content must not be empty");
    let mut test_lines = 0;
    let mut in_test_module = false;
    let mut brace_depth = 0;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains("#[cfg(test)]") {
            in_test_module = true;
        }
        if in_test_module {
            brace_depth += trimmed.matches('{').count();
            brace_depth = brace_depth.saturating_sub(trimmed.matches('}').count());
            test_lines += 1;
            if brace_depth == 0 && test_lines > 1 {
                in_test_module = false;
            }
        }
        if trimmed.contains("#[test]") || trimmed.contains("#[tokio::test]") {
            test_lines += 10;
        }
    }
    test_lines
}

fn count_line_complexity(trimmed: &str) -> u32 {
    let flow_keywords: &[(&str, &str)] = &[
        ("if ", " if "),
        ("else if ", "} else if "),
        ("match ", " match "),
        ("for ", " for "),
        ("while ", " while "),
        ("loop ", " loop "),
    ];
    let mut count = 0u32;
    for &(prefix, infix) in flow_keywords {
        if trimmed.starts_with(prefix) || trimmed.contains(infix) {
            count += 1;
        }
    }
    if trimmed.contains("&&") || trimmed.contains("||") {
        count += 1;
    }
    if trimmed.contains('?') && !trimmed.contains("//") {
        count += 1;
    }
    count
}

pub(crate) fn estimate_avg_complexity(content: &str) -> f32 {
    debug_assert!(!content.is_empty(), "content must not be empty");
    let mut total_complexity = 1u32;
    let mut function_count = 0u32;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("fn ")
            || trimmed.starts_with("pub fn ")
            || trimmed.starts_with("async fn ")
            || trimmed.starts_with("pub async fn ")
        {
            function_count += 1;
        }
        total_complexity += count_line_complexity(trimmed);
    }
    if function_count == 0 {
        return total_complexity as f32;
    }
    total_complexity as f32 / function_count as f32
}

// Sovereign stack patterns and PAIML deps checks (from migrate_handlers.rs)
pub(crate) use super::check_sovereign::*;
