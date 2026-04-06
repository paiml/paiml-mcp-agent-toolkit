// Sovereign stack patterns and PAIML dependency checks
//
// Originally from migrate_handlers.rs, these are check functions
// for sovereign stack compliance and PAIML dependency workspace state.

use crate::cli::colors as c;
use crate::services::commit_classifier::CommitClassifier;
use anyhow::Result;
use std::fs;
use std::path::Path;

use super::check_extended::{discover_source_files, estimate_avg_complexity, estimate_test_lines};
use super::types::*;
use crate::services::file_health::FileHealthMetrics;

/// Check Sovereign AI Stack compliance patterns (CB-040 complexity refactor)
pub(crate) fn check_sovereign_stack_patterns(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let cargo_toml = project_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return skip_check("Sovereign Stack Patterns", "No Cargo.toml found");
    }
    let content = fs::read_to_string(&cargo_toml).unwrap_or_default();
    if !is_sovereign_stack_project(&content) {
        return skip_check("Sovereign Stack Patterns", "Not a Sovereign Stack project");
    }
    let mut issues: Vec<String> = Vec::new();
    let mut good_patterns: Vec<String> = Vec::new();
    check_five_whys_patterns(project_path, &mut issues, &mut good_patterns);
    check_falsification_tests(project_path, &mut good_patterns);
    check_apr_models(project_path, &mut good_patterns);
    check_ticket_refs(project_path, &mut issues, &mut good_patterns);
    check_ml_commit_classification(project_path, &mut good_patterns);
    build_sovereign_result(&issues, &good_patterns)
}

pub(crate) fn is_sovereign_stack_project(content: &str) -> bool {
    debug_assert!(!content.is_empty(), "content must not be empty");
    const SOVEREIGN_DEPS: &[&str] = &["trueno", "aprender", "realizar", "batuta", "renacer"];
    SOVEREIGN_DEPS.iter().any(|dep| content.contains(dep))
}

pub(crate) fn check_five_whys_patterns(
    project_path: &Path,
    issues: &mut Vec<String>,
    good_patterns: &mut Vec<String>,
) {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    use std::process::Command;
    let git_log = Command::new("git")
        .args(["log", "--oneline", "-20", "--grep=fix"])
        .current_dir(project_path)
        .output();
    if let Ok(output) = git_log {
        let log = String::from_utf8_lossy(&output.stdout);
        let fix_commits: Vec<&str> = log.lines().collect();
        if !fix_commits.is_empty() {
            let has_five_whys = Command::new("git")
                .args(["log", "-20", "--grep=Five-Whys\\|ROOT CAUSE\\|Why 1:"])
                .current_dir(project_path)
                .output()
                .map(|o| !o.stdout.is_empty())
                .unwrap_or(false);
            if has_five_whys {
                good_patterns.push("Five-Whys root cause analysis".into());
            } else if fix_commits.len() > 5 {
                issues.push("No Five-Whys in recent fix commits".into());
            }
        }
    }
}

pub(crate) fn check_falsification_tests(project_path: &Path, good_patterns: &mut Vec<String>) {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let tests_dir = project_path.join("tests");
    if !tests_dir.exists() {
        return;
    }
    let has_falsification = walkdir::WalkDir::new(&tests_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .any(|e| {
            e.path().to_string_lossy().contains("falsification")
                || fs::read_to_string(e.path())
                    .map(|s| s.contains("F001") || (s.contains("F0") && s.contains("TEST")))
                    .unwrap_or(false)
        });
    if has_falsification {
        good_patterns.push("Falsification test suite".into());
    }
}

pub(crate) fn check_apr_models(project_path: &Path, good_patterns: &mut Vec<String>) {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let models_dir = project_path.join("models");
    if !models_dir.exists() {
        return;
    }
    let apr_count = walkdir::WalkDir::new(&models_dir)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "apr").unwrap_or(false))
        .count();
    if apr_count > 0 {
        good_patterns.push(format!("{} APR model(s)", apr_count));
    }
}

pub(crate) fn check_ticket_refs(
    project_path: &Path,
    issues: &mut Vec<String>,
    good_patterns: &mut Vec<String>,
) {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    use std::process::Command;
    let ticket_refs = Command::new("git")
        .args(["log", "-50", "--oneline"])
        .current_dir(project_path)
        .output()
        .map(|o| {
            let log = String::from_utf8_lossy(&o.stdout);
            log.lines()
                .filter(|l| {
                    l.contains("PAR-")
                        || l.contains("PMAT-")
                        || l.contains("Refs ")
                        || l.contains("GH-")
                })
                .count()
        })
        .unwrap_or(0);
    if ticket_refs > 10 {
        good_patterns.push(format!("{}+ ticket refs in commits", ticket_refs));
    } else if ticket_refs < 5 {
        issues.push("Few ticket references in recent commits".into());
    }
}

pub(crate) fn check_ml_commit_classification(project_path: &Path, good_patterns: &mut Vec<String>) {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    use std::process::Command;
    let classifier = match CommitClassifier::load_sovereign_stack() {
        Ok(c) => c,
        Err(_) => return,
    };
    let git_log_full = Command::new("git")
        .args(["log", "-10", "--format=%B---COMMIT_SEP---"])
        .current_dir(project_path)
        .output();
    if let Ok(output) = git_log_full {
        let log = String::from_utf8_lossy(&output.stdout);
        let commits: Vec<&str> = log
            .split("---COMMIT_SEP---")
            .filter(|s| !s.trim().is_empty())
            .collect();
        if commits.is_empty() {
            return;
        }
        let mut class_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut high_confidence = 0;
        for commit in &commits {
            let result = classifier.classify(commit);
            *class_counts.entry(result.class).or_insert(0) += 1;
            if result.confidence > 0.6 {
                high_confidence += 1;
            }
        }
        if let Some((dominant_class, count)) = class_counts.iter().max_by_key(|(_, c)| *c) {
            if *count >= commits.len() / 2 {
                good_patterns.push(format!(
                    "ML: {} dominant ({}/{})",
                    dominant_class,
                    count,
                    commits.len()
                ));
            }
        }
        if high_confidence > commits.len() / 2 {
            good_patterns.push(format!(
                "ML: {}% high-confidence classifications",
                high_confidence * 100 / commits.len()
            ));
        }
    }
}

pub(crate) fn build_sovereign_result(
    issues: &[String],
    good_patterns: &[String],
) -> ComplianceCheck {
    debug_assert!(!issues.is_empty(), "issues must not be empty");
    if issues.is_empty() && !good_patterns.is_empty() {
        ComplianceCheck {
            name: "Sovereign Stack Patterns".into(),
            status: CheckStatus::Pass,
            message: format!("Patterns: {}", good_patterns.join(", ")),
            severity: Severity::Info,
        }
    } else if !issues.is_empty() {
        ComplianceCheck {
            name: "Sovereign Stack Patterns".into(),
            status: CheckStatus::Warn,
            message: format!("Missing: {}", issues.join("; ")),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "Sovereign Stack Patterns".into(),
            status: CheckStatus::Pass,
            message: "Sovereign Stack project detected".into(),
            severity: Severity::Info,
        }
    }
}

fn make_paiml_check(status: CheckStatus, message: String, severity: Severity) -> ComplianceCheck {
    ComplianceCheck {
        name: "PAIML Deps Workspace".into(),
        status,
        message,
        severity,
    }
}

fn classify_local_deps(src_dir: &Path, paiml_deps: &[&str]) -> (Vec<String>, Vec<String>) {
    debug_assert!(
        src_dir.exists(),
        "src_dir must exist: {}",
        src_dir.display()
    );
    use std::process::Command;
    let mut dirty = Vec::new();
    let mut clean = Vec::new();
    for dep in paiml_deps {
        let dep_path = src_dir.join(dep);
        if !dep_path.exists() {
            continue;
        }
        let Ok(out) = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&dep_path)
            .output()
        else {
            continue;
        };
        let status = String::from_utf8_lossy(&out.stdout);
        if status.trim().is_empty() {
            clean.push(dep.to_string());
        } else {
            dirty.push(dep.to_string());
        }
    }
    (dirty, clean)
}

pub(crate) fn check_paiml_deps_workspace(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    const PAIML_PACKAGES: &[&str] = &[
        "trueno",
        "trueno-graph",
        "trueno-rag",
        "trueno-viz",
        "trueno-db",
        "trueno-zram-core",
        "trueno-ublk",
        "aprender",
        "entrenar",
        "alimentar",
        "realizar",
        "batuta",
        "renacer",
        "repartir",
        "presentar",
        "presentar-terminal",
        "ruchy",
        "bashrs",
        "decy",
        "depyler",
        "rascal",
        "pacha",
        "pepita",
        "simular",
        "jugar",
        "duende",
        "pzsh",
        "certeza",
        "verificar",
        "probar",
        "manzana",
        "whisper-apr",
        "copia",
        "nviwatch",
        "ruchydbg",
        "rust-mcp-sdk",
    ];
    let cargo_toml = project_path.join("Cargo.toml");
    let content = match fs::read_to_string(&cargo_toml) {
        Ok(c) => c,
        Err(_) => {
            return make_paiml_check(
                CheckStatus::Skip,
                "No Cargo.toml found".into(),
                Severity::Info,
            )
        }
    };
    let paiml_deps: Vec<&str> = PAIML_PACKAGES
        .iter()
        .filter(|pkg| {
            content.contains(&format!("{} = ", pkg)) || content.contains(&format!("\"{}\"", pkg))
        })
        .copied()
        .collect();
    if paiml_deps.is_empty() {
        return make_paiml_check(
            CheckStatus::Skip,
            "No PAIML stack dependencies found".into(),
            Severity::Info,
        );
    }
    let Some(home) = dirs::home_dir() else {
        return make_paiml_check(
            CheckStatus::Skip,
            "Could not determine home directory".into(),
            Severity::Info,
        );
    };
    let (dirty_deps, clean_deps) = classify_local_deps(&home.join("src"), &paiml_deps);
    let total_local = dirty_deps.len() + clean_deps.len();
    if total_local == 0 {
        return make_paiml_check(
            CheckStatus::Pass,
            format!(
                "{} PAIML deps (no local checkouts in ~/src)",
                paiml_deps.len()
            ),
            Severity::Info,
        );
    }
    if dirty_deps.is_empty() {
        make_paiml_check(
            CheckStatus::Pass,
            format!(
                "{} PAIML deps, {} local checkouts (all clean)",
                paiml_deps.len(),
                total_local
            ),
            Severity::Info,
        )
    } else {
        make_paiml_check(
            CheckStatus::Warn,
            format!(
                "{} dirty: {} (using crates.io versions for safety)",
                dirty_deps.len(),
                dirty_deps.join(", ")
            ),
            Severity::Warning,
        )
    }
}

/// Generate file health baseline for ratchet enforcement.
pub(crate) fn generate_file_health_baseline(project_path: &Path, dry_run: bool) -> Result<()> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    use crate::services::file_health::{FileHealthBaseline, FileHealthMetrics};
    let files = match discover_source_files(project_path) {
        Ok(f) => f,
        Err(msg) => {
            println!("{}", c::skip(&format!("Skipping baseline: {}", msg)));
            return Ok(());
        }
    };
    let mut baseline = FileHealthBaseline::new();
    for file_path in &files {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines = content.lines().count();
        let test_lines = estimate_test_lines(&content);
        let avg_complexity = estimate_avg_complexity(&content);
        let rel_path = file_path.strip_prefix(project_path).unwrap_or(file_path);
        let metrics = FileHealthMetrics::calculate(
            rel_path.to_path_buf(),
            lines,
            test_lines,
            avg_complexity,
            0,
        );
        baseline.add_file(&metrics);
    }
    if dry_run {
        println!(
            "{} would save file health baseline ({} files) to {}",
            c::dim("Dry run:"),
            c::number(&baseline.files.len().to_string()),
            c::path(".pmat/file-health-baseline.json")
        );
        return Ok(());
    }
    let pmat_dir = project_path.join(".pmat");
    fs::create_dir_all(&pmat_dir)?;
    let baseline_path = pmat_dir.join("file-health-baseline.json");
    baseline.save(&baseline_path)?;
    println!(
        "{} {} ({} files)",
        c::pass("File health baseline saved:"),
        c::path(&baseline_path.display().to_string()),
        c::number(&baseline.files.len().to_string())
    );
    Ok(())
}

/// Cross-stack file health check across multiple projects.
pub(crate) fn check_file_health_multi(
    primary_path: &Path,
    include_projects: &[std::path::PathBuf],
) -> Result<()> {
    debug_assert!(
        primary_path.exists(),
        "primary_path must exist: {}",
        primary_path.display()
    );
    use crate::services::file_health::{FileHealthReport, StackHealthReport};
    let mut project_reports: Vec<(String, FileHealthReport)> = Vec::new();
    let primary_name: String = primary_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("primary")
        .to_string();
    if let Ok(report) = analyze_project_health(primary_path) {
        project_reports.push((primary_name, report));
    }
    for project_path in include_projects {
        let project_name: String = project_path
            .file_name()
            .and_then(|n: &std::ffi::OsStr| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        match analyze_project_health(project_path) {
            Ok(report) => {
                project_reports.push((project_name, report));
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to analyze {}: {}",
                    project_path.display(),
                    e
                );
            }
        }
    }
    if project_reports.is_empty() {
        println!(
            "{}",
            c::warn("No projects could be analyzed for file health.")
        );
        return Ok(());
    }
    let stack_report = StackHealthReport::from_projects(project_reports);
    println!("\n{}", c::header("Stack File Health"));
    println!(
        "{} {:?} (avg health: {})",
        c::label("Stack Grade:"),
        stack_report.stack_grade,
        c::number(&stack_report.stack_average_health.to_string())
    );
    println!(
        "{} {}",
        c::label("Projects analyzed:"),
        c::number(&stack_report.projects.len().to_string())
    );
    for (name, report) in &stack_report.projects {
        println!(
            "  {} \u{2014} {:?} ({} files, avg health: {})",
            c::label(name),
            report.average_grade,
            c::number(&report.total_files.to_string()),
            c::number(&report.average_health.to_string())
        );
    }
    if !stack_report.stack_worst_files.is_empty() {
        println!("\n{}", c::label("Worst files across stack:"));
        for (project, metrics) in &stack_report.stack_worst_files {
            println!(
                "  [{}] {} \u{2014} {} lines, health: {}",
                c::label(project),
                c::path(&metrics.path.display().to_string()),
                c::number(&metrics.lines.to_string()),
                c::number(&metrics.health_score.to_string())
            );
        }
    }
    Ok(())
}

fn analyze_project_health(
    project_path: &Path,
) -> Result<crate::services::file_health::FileHealthReport> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    use crate::services::file_health::FileHealthReport;
    let files = discover_source_files(project_path)
        .map_err(|e| anyhow::anyhow!("Failed to discover source files: {}", e))?;
    let mut all_metrics = Vec::new();
    for file_path in &files {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines = content.lines().count();
        let test_lines = estimate_test_lines(&content);
        let avg_complexity = estimate_avg_complexity(&content);
        let rel_path = file_path.strip_prefix(project_path).unwrap_or(file_path);
        let metrics = FileHealthMetrics::calculate(
            rel_path.to_path_buf(),
            lines,
            test_lines,
            avg_complexity,
            0,
        );
        all_metrics.push(metrics);
    }
    Ok(FileHealthReport::from_files(
        project_path.to_path_buf(),
        all_metrics,
    ))
}
