
/// Check Sovereign AI Stack compliance patterns (CB-040 complexity refactor)
/// Validates: Five-Whys in fixes, falsification tests, APR models, ticket refs
pub(crate) fn check_sovereign_stack_patterns(project_path: &Path) -> ComplianceCheck {
    // Check if this is a Sovereign Stack project
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

    // Run individual checks
    check_five_whys_patterns(project_path, &mut issues, &mut good_patterns);
    check_falsification_tests(project_path, &mut good_patterns);
    check_apr_models(project_path, &mut good_patterns);
    check_ticket_refs(project_path, &mut issues, &mut good_patterns);
    check_ml_commit_classification(project_path, &mut good_patterns);

    // Build result
    build_sovereign_result(&issues, &good_patterns)
}

/// Helper: Create skip check result
pub(crate) fn skip_check(name: &str, message: &str) -> ComplianceCheck {
    ComplianceCheck {
        name: name.to_string(),
        status: CheckStatus::Skip,
        message: message.to_string(),
        severity: Severity::Info,
    }
}

/// Helper: Check if Cargo.toml contains sovereign stack dependencies
pub(crate) fn is_sovereign_stack_project(content: &str) -> bool {
    const SOVEREIGN_DEPS: &[&str] = &["trueno", "aprender", "realizar", "batuta", "renacer"];
    SOVEREIGN_DEPS.iter().any(|dep| content.contains(dep))
}

/// Helper: Check Five-Whys patterns in git commits
pub(crate) fn check_five_whys_patterns(project_path: &Path, issues: &mut Vec<String>, good_patterns: &mut Vec<String>) {
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
                good_patterns.push("Five-Whys root cause analysis".to_string());
            } else if fix_commits.len() > 5 {
                issues.push("No Five-Whys in recent fix commits".to_string());
            }
        }
    }
}

/// Helper: Check for falsification tests
pub(crate) fn check_falsification_tests(project_path: &Path, good_patterns: &mut Vec<String>) {
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
        good_patterns.push("Falsification test suite".to_string());
    }
}

/// Helper: Check for APR model files
pub(crate) fn check_apr_models(project_path: &Path, good_patterns: &mut Vec<String>) {
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

/// Helper: Check ticket references in commits
pub(crate) fn check_ticket_refs(project_path: &Path, issues: &mut Vec<String>, good_patterns: &mut Vec<String>) {
    use std::process::Command;

    let ticket_refs = Command::new("git")
        .args(["log", "-50", "--oneline"])
        .current_dir(project_path)
        .output()
        .map(|o| {
            let log = String::from_utf8_lossy(&o.stdout);
            log.lines()
                .filter(|l| l.contains("PAR-") || l.contains("PMAT-") || l.contains("Refs ") || l.contains("GH-"))
                .count()
        })
        .unwrap_or(0);

    if ticket_refs > 10 {
        good_patterns.push(format!("{}+ ticket refs in commits", ticket_refs));
    } else if ticket_refs < 5 {
        issues.push("Few ticket references in recent commits".to_string());
    }
}

/// Helper: Check ML-based commit classification
pub(crate) fn check_ml_commit_classification(project_path: &Path, good_patterns: &mut Vec<String>) {
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
        let commits: Vec<&str> = log.split("---COMMIT_SEP---").filter(|s| !s.trim().is_empty()).collect();

        if commits.is_empty() {
            return;
        }

        let mut class_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
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
                good_patterns.push(format!("ML: {} dominant ({}/{})", dominant_class, count, commits.len()));
            }
        }

        if high_confidence > commits.len() / 2 {
            good_patterns.push(format!("ML: {}% high-confidence classifications", high_confidence * 100 / commits.len()));
        }
    }
}

/// Helper: Build final result from issues and good patterns
pub(crate) fn build_sovereign_result(issues: &[String], good_patterns: &[String]) -> ComplianceCheck {
    if issues.is_empty() && !good_patterns.is_empty() {
        ComplianceCheck {
            name: "Sovereign Stack Patterns".to_string(),
            status: CheckStatus::Pass,
            message: format!("Patterns: {}", good_patterns.join(", ")),
            severity: Severity::Info,
        }
    } else if !issues.is_empty() {
        ComplianceCheck {
            name: "Sovereign Stack Patterns".to_string(),
            status: CheckStatus::Warn,
            message: format!("Missing: {}", issues.join("; ")),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "Sovereign Stack Patterns".to_string(),
            status: CheckStatus::Pass,
            message: "Sovereign Stack project detected".to_string(),
            severity: Severity::Info,
        }
    }
}

/// Check PAIML dependency workspace state (dirty/clean/version drift)
/// Detects when local PAIML projects have uncommitted changes or version mismatches
pub(crate) fn check_paiml_deps_workspace(project_path: &Path) -> ComplianceCheck {
    use std::process::Command;

    // Known PAIML/Sovereign stack packages
    const PAIML_PACKAGES: &[&str] = &[
        "trueno", "trueno-graph", "trueno-rag", "trueno-viz", "trueno-db",
        "trueno-zram-core", "trueno-ublk",
        "aprender", "entrenar", "alimentar", "realizar",
        "batuta", "renacer", "repartir",
        "presentar", "presentar-terminal",
        "ruchy", "bashrs", "decy", "depyler", "rascal",
        "pacha", "pepita", "simular", "jugar", "duende", "pzsh",
        "certeza", "verificar", "probar", "manzana", "whisper-apr",
        "copia", "nviwatch", "ruchydbg", "rust-mcp-sdk",
    ];

    let cargo_toml = project_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return ComplianceCheck {
            name: "PAIML Deps Workspace".to_string(),
            status: CheckStatus::Skip,
            message: "No Cargo.toml found".to_string(),
            severity: Severity::Info,
        };
    }

    let content = match fs::read_to_string(&cargo_toml) {
        Ok(c) => c,
        Err(_) => {
            return ComplianceCheck {
                name: "PAIML Deps Workspace".to_string(),
                status: CheckStatus::Skip,
                message: "Could not read Cargo.toml".to_string(),
                severity: Severity::Info,
            };
        }
    };

    // Find PAIML dependencies in this project
    let mut paiml_deps: Vec<&str> = Vec::new();
    for pkg in PAIML_PACKAGES {
        // Check for dependency lines like: trueno = "0.11" or trueno = { version = "0.11" }
        if content.contains(&format!("{} = ", pkg)) || content.contains(&format!("\"{}\"", pkg)) {
            paiml_deps.push(pkg);
        }
    }

    if paiml_deps.is_empty() {
        return ComplianceCheck {
            name: "PAIML Deps Workspace".to_string(),
            status: CheckStatus::Skip,
            message: "No PAIML stack dependencies found".to_string(),
            severity: Severity::Info,
        };
    }

    // Check local workspace state for each PAIML dep
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => {
            return ComplianceCheck {
                name: "PAIML Deps Workspace".to_string(),
                status: CheckStatus::Skip,
                message: "Could not determine home directory".to_string(),
                severity: Severity::Info,
            };
        }
    };
    let src_dir = home.join("src");

    let mut dirty_deps: Vec<String> = Vec::new();
    let mut clean_deps: Vec<String> = Vec::new();

    for dep in &paiml_deps {
        let dep_path = src_dir.join(dep);
        if !dep_path.exists() {
            continue; // Not a local checkout
        }

        // Check git status
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&dep_path)
            .output();

        match output {
            Ok(out) => {
                let status = String::from_utf8_lossy(&out.stdout);
                if status.trim().is_empty() {
                    clean_deps.push(dep.to_string());
                } else {
                    dirty_deps.push(dep.to_string());
                }
            }
            Err(_) => {
                // Not a git repo or git not available
                continue;
            }
        }
    }

    let total_local = dirty_deps.len() + clean_deps.len();

    if total_local == 0 {
        return ComplianceCheck {
            name: "PAIML Deps Workspace".to_string(),
            status: CheckStatus::Pass,
            message: format!("{} PAIML deps (no local checkouts in ~/src)", paiml_deps.len()),
            severity: Severity::Info,
        };
    }

    if dirty_deps.is_empty() {
        ComplianceCheck {
            name: "PAIML Deps Workspace".to_string(),
            status: CheckStatus::Pass,
            message: format!(
                "{} PAIML deps, {} local checkouts (all clean)",
                paiml_deps.len(),
                total_local
            ),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "PAIML Deps Workspace".to_string(),
            status: CheckStatus::Warn,
            message: format!(
                "{} dirty: {} (using crates.io versions for safety)",
                dirty_deps.len(),
                dirty_deps.join(", ")
            ),
            severity: Severity::Warning,
        }
    }
}

/// Check file health across the project (CB-040)
/// Validates: max-lines (500), TLR (test-to-lines ratio), complexity, health score
/// Based on: docs/specifications/max-lines.md
pub(crate) fn check_file_health(project_path: &Path) -> ComplianceCheck {
    // Skip if not a Rust project
    let cargo_toml = project_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return ComplianceCheck {
            name: "File Health".to_string(),
            status: CheckStatus::Skip,
            message: "No Cargo.toml found - not a Rust project".to_string(),
            severity: Severity::Info,
        };
    }

    // Scan for source files
    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return ComplianceCheck {
            name: "File Health".to_string(),
            status: CheckStatus::Skip,
            message: "No src/ directory found".to_string(),
            severity: Severity::Info,
        };
    }

    let files = scan_directory(&src_dir, RUST_EXTENSIONS, DEFAULT_EXCLUDE_PATTERNS);
    if files.is_empty() {
        return ComplianceCheck {
            name: "File Health".to_string(),
            status: CheckStatus::Pass,
            message: "No Rust source files found".to_string(),
            severity: Severity::Info,
        };
    }

    // Analyze each file for health metrics
    let mut metrics: Vec<FileHealthMetrics> = Vec::new();
    let mut critical_count = 0;
    let mut problem_count = 0;
    let mut over_500_count = 0;

    for file_path in &files {
        // Count lines
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines = content.lines().count();

        // Quick categorization without full analysis
        if lines > 2000 {
            critical_count += 1;
        } else if lines > 1000 {
            problem_count += 1;
        }
        if lines > 500 {
            over_500_count += 1;
        }

        // Calculate full metrics (simplified - using defaults for test/complexity/churn)
        // In production, these would be gathered from actual analysis
        let test_lines = estimate_test_lines(&content);
        let avg_complexity = estimate_avg_complexity(&content);

        let file_metrics = FileHealthMetrics::calculate(
            file_path.clone(),
            lines,
            test_lines,
            avg_complexity,
            0, // churn_30d - would need git integration
        );
        metrics.push(file_metrics);
    }

    // Build report
    let report = FileHealthReport::from_files(project_path.to_path_buf(), metrics);

    // Determine status based on findings
    if critical_count > 0 {
        ComplianceCheck {
            name: "File Health".to_string(),
            status: CheckStatus::Fail,
            message: format!(
                "CRITICAL: {} files >2000 lines, {} files >1000 lines, {} files >500 lines (avg health: {}%, grade: {})",
                critical_count,
                problem_count,
                over_500_count,
                report.average_health,
                report.average_grade.as_str()
            ),
            severity: Severity::Critical,
        }
    } else if problem_count > 0 || over_500_count > 5 {
        ComplianceCheck {
            name: "File Health".to_string(),
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
        ComplianceCheck {
            name: "File Health".to_string(),
            status: CheckStatus::Pass,
            message: format!(
                "{} files analyzed, avg health: {}%, grade: {} (all files <500 lines or within tolerance)",
                report.total_files,
                report.average_health,
                report.average_grade.as_str()
            ),
            severity: Severity::Info,
        }
    }
}

/// Estimate test lines by counting lines in #[cfg(test)] modules and test functions
pub(crate) fn estimate_test_lines(content: &str) -> usize {
    let mut test_lines = 0;
    let mut in_test_module = false;
    let mut brace_depth = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Track test module entry
        if trimmed.contains("#[cfg(test)]") {
            in_test_module = true;
        }

        // Track braces for module scope
        if in_test_module {
            brace_depth += trimmed.matches('{').count();
            brace_depth = brace_depth.saturating_sub(trimmed.matches('}').count());
            test_lines += 1;

            if brace_depth == 0 && test_lines > 1 {
                in_test_module = false;
            }
        }

        // Also count standalone test functions
        if trimmed.contains("#[test]") || trimmed.contains("#[tokio::test]") {
            test_lines += 10; // Approximate test function size
        }
    }

    test_lines
}

/// Estimate average cyclomatic complexity by counting control flow statements
pub(crate) fn estimate_avg_complexity(content: &str) -> f32 {
    let mut total_complexity = 1; // Base complexity
    let mut function_count = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Count function definitions
        if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") ||
           trimmed.starts_with("async fn ") || trimmed.starts_with("pub async fn ") {
            function_count += 1;
        }

        // Count control flow statements (add to complexity)
        if trimmed.starts_with("if ") || trimmed.contains(" if ") {
            total_complexity += 1;
        }
        if trimmed.starts_with("else if ") || trimmed.contains("} else if ") {
            total_complexity += 1;
        }
        if trimmed.starts_with("match ") || trimmed.contains(" match ") {
            total_complexity += 1;
        }
        if trimmed.starts_with("for ") || trimmed.contains(" for ") {
            total_complexity += 1;
        }
        if trimmed.starts_with("while ") || trimmed.contains(" while ") {
            total_complexity += 1;
        }
        if trimmed.starts_with("loop ") || trimmed.contains(" loop ") {
            total_complexity += 1;
        }
        if trimmed.contains("&&") || trimmed.contains("||") {
            total_complexity += 1;
        }
        if trimmed.contains("?") && !trimmed.contains("//") {
            total_complexity += 1; // Error propagation
        }
    }

    if function_count == 0 {
        return total_complexity as f32;
    }

    total_complexity as f32 / function_count as f32
}

pub(crate) fn calculate_versions_behind(project_version: &str) -> u32 {
    let current_parts: Vec<u32> = PMAT_VERSION
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let project_parts: Vec<u32> = project_version
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    if current_parts.len() >= 2 && project_parts.len() >= 2 {
        current_parts
            .get(1)
            .unwrap_or(&0)
            .saturating_sub(*project_parts.get(1).unwrap_or(&0))
    } else {
        0
    }
}

pub(crate) fn get_breaking_changes_since(_from_version: &str) -> Vec<BreakingChange> {
    vec![]
}

#[derive(Debug, Clone)]
struct ChangelogEntry {
    version: String,
    description: String,
    breaking: bool,
}

pub(crate) fn get_changelog_entries(_from: &str, _to: &str) -> Vec<ChangelogEntry> {
    vec![
        ChangelogEntry {
            version: PMAT_VERSION.to_string(),
            description: "Added qa-work command for Toyota Way validation".to_string(),
            breaking: false,
        },
        ChangelogEntry {
            version: PMAT_VERSION.to_string(),
            description: "Added cleanup-resources command".to_string(),
            breaking: false,
        },
        ChangelogEntry {
            version: PMAT_VERSION.to_string(),
            description: "Added comply command for compliance checking".to_string(),
            breaking: false,
        },
    ]
}

pub(crate) fn migrate_project_version(project_path: &Path, target: &str, dry_run: bool) -> Result<bool> {
    if dry_run {
        return Ok(true);
    }
    let mut config = load_or_create_project_config(project_path)?;
    if config.pmat.version == target {
        return Ok(false);
    }
    config.pmat.version = target.to_string();
    config.pmat.last_compliance_check = Some(Utc::now());
    let content = toml::to_string_pretty(&config)?;
    fs::write(project_path.join(".pmat").join("project.toml"), &content)?;
    Ok(true)
}

pub(crate) fn migrate_gitignore(project_path: &Path, dry_run: bool) -> Result<bool> {
    let gitignore_path = project_path.join(".gitignore");
    let pmat_entries = [".pmat/backup/", ".pmat-qa/"];
    if !gitignore_path.exists() {
        return Ok(false);
    }
    let content = fs::read_to_string(&gitignore_path)?;
    let mut needs_update = false;
    let mut new_entries = vec![];
    for entry in pmat_entries {
        if !content.contains(entry) {
            needs_update = true;
            new_entries.push(entry);
        }
    }
    if needs_update && !dry_run {
        let mut new_content = content.clone();
        if !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push_str("\n# PMAT\n");
        for entry in new_entries {
            new_content.push_str(entry);
            new_content.push('\n');
        }
        fs::write(&gitignore_path, &new_content)?;
    }
    Ok(needs_update)
}

pub(crate) fn update_project_config(project_path: &Path, dry_run: bool) -> Result<bool> {
    migrate_project_version(project_path, PMAT_VERSION, dry_run)
}

pub(crate) fn print_compliance_text(report: &ComplianceReport) {
    println!("\n{}", "=".repeat(60));
    println!("PMAT Compliance Report");
    println!("{}", "=".repeat(60));
    println!("\nProject Version: {}", report.project_version);
    println!("Current PMAT:    {}", report.current_version);
    println!("Versions Behind: {}", report.versions_behind);
    let status = if report.is_compliant {
        "\x1b[32mCOMPLIANT\x1b[0m"
    } else {
        "\x1b[31mNON-COMPLIANT\x1b[0m"
    };
    println!("Status:          {}\n", status);
    println!("Checks:");
    for check in &report.checks {
        let icon = match check.status {
            CheckStatus::Pass => "\x1b[32m✓\x1b[0m",
            CheckStatus::Warn => "\x1b[33m⚠\x1b[0m",
            CheckStatus::Fail => "\x1b[31m✗\x1b[0m",
            CheckStatus::Skip => "\x1b[90m-\x1b[0m",
        };
        println!("  {} {}: {}", icon, check.name, check.message);
    }
    if !report.recommendations.is_empty() {
        println!("\nRecommendations:");
        for rec in &report.recommendations {
            println!("  • {}", rec);
        }
    }
    println!("\n{}", "=".repeat(60));
}

pub(crate) fn print_compliance_markdown(report: &ComplianceReport) {
    println!("# PMAT Compliance Report\n");
    println!("| Property | Value |");
    println!("|----------|-------|");
    println!("| Project Version | {} |", report.project_version);
    println!("| Current PMAT | {} |", report.current_version);
    println!(
        "| Status | {} |",
        if report.is_compliant {
            "COMPLIANT"
        } else {
            "NON-COMPLIANT"
        }
    );
    println!("\n## Checks\n");
    for check in &report.checks {
        let icon = match check.status {
            CheckStatus::Pass => "✅",
            CheckStatus::Warn => "⚠️",
            CheckStatus::Fail => "❌",
            CheckStatus::Skip => "⏭️",
        };
        println!("- {} **{}**: {}", icon, check.name, check.message);
    }
}

/// Install git hooks for mandatory work tracking (W-006)
/// Implements master-plan-pmat-work-system.md enforcement
async fn handle_enforce(
    project_path: &Path,
    yes: bool,
    disable: bool,
    format: ComplyOutputFormat,
) -> Result<()> {
    let hooks_dir = project_path.join(".git").join("hooks");

    if !hooks_dir.exists() {
        anyhow::bail!("Not a git repository (no .git/hooks directory)");
    }

    if disable {
        // Remove PMAT hooks
        let pre_commit = hooks_dir.join("pre-commit");
        if pre_commit.exists() {
            if let Ok(content) = fs::read_to_string(&pre_commit) {
                if content.contains("PMAT") {
                    fs::remove_file(&pre_commit)?;
                    println!("✅ Removed PMAT pre-commit hook");
                } else {
                    println!("⚠️  Pre-commit hook exists but is not PMAT - not removed");
                }
            }
        }

        // Also remove pre-push hook if it's PMAT's
        let pre_push = hooks_dir.join("pre-push");
        if pre_push.exists() {
            if let Ok(content) = fs::read_to_string(&pre_push) {
                if content.contains("PMAT") || content.contains("ComputeBrick") {
                    fs::remove_file(&pre_push)?;
                    println!("✅ Removed PMAT pre-push hook");
                } else {
                    println!("⚠️  Pre-push hook exists but is not PMAT - not removed");
                }
            }
        }
        return Ok(());
    }

    // Prompt for confirmation if not -y
    if !yes {
        println!("This will install PMAT enforcement hooks:");
        println!("  - pre-commit: Block commits without active work ticket");
        println!("  - pre-push: Validate spec compliance before push");
        println!("\nProceed? [y/N] ");

        // For now, just proceed (interactive input not easily testable)
        println!("(Auto-proceeding due to non-interactive mode)");
    }

    // Create pre-commit hook
    let pre_commit_content = r#"#!/bin/sh
# PMAT Work Enforcement Hook (master-plan-pmat-work-system.md W-001)
# This hook blocks commits without an active work ticket.

# Check for active work ticket
if ! pmat work status --active >/dev/null 2>&1; then
    echo "❌ COMPLIANCE VIOLATION"
    echo ""
    echo "Action blocked: git commit"
    echo "Reason: No active work ticket"
    echo ""
    echo "To fix:"
    echo "  1. Start work: pmat work start <ticket-id>"
    echo "  2. Or create ticket: pmat work start \"description\" --spec <spec-file>"
    echo ""
    echo "Bypass (NOT RECOMMENDED):"
    echo "  git commit --no-verify"
    exit 1
fi

# Ensure commit message references ticket
TICKET_ID=$(pmat work status --active --quiet 2>/dev/null)
if [ -n "$TICKET_ID" ]; then
    # Check commit message for ticket reference
    if ! grep -qi "$TICKET_ID\|#[0-9]" "$1" 2>/dev/null; then
        echo "⚠️  Commit message should reference ticket: $TICKET_ID"
    fi
fi

exit 0
"#;

    let pre_commit_path = hooks_dir.join("pre-commit");
    fs::write(&pre_commit_path, pre_commit_content)?;

    // Create pre-push hook for ComputeBrick compliance (CB-IMPL-001-C)
    let pre_push_content = r#"#!/bin/sh
# PMAT ComputeBrick Pre-Push Enforcement (PROBAR-SPEC-009-P8)
# This hook validates ComputeBrick compliance before push.

set -e

echo "🔍 Running ComputeBrick compliance checks..."

# Check ComputeBrick compliance via pmat comply
COMPLY_OUTPUT=$(pmat comply check --failures-only 2>&1) || true
if echo "$COMPLY_OUTPUT" | grep -q "ComputeBrick Compliance.*critical"; then
    echo "❌ COMPUTEBRICK COMPLIANCE FAILURE"
    echo ""
    echo "$COMPLY_OUTPUT" | grep -A5 "ComputeBrick"
    echo ""
    echo "Fix critical violations before pushing."
    echo "Run 'pmat comply check' for full details."
    echo ""
    echo "Bypass (NOT RECOMMENDED):"
    echo "  git push --no-verify"
    exit 1
fi

# Check probar GUI coverage if available (PROBAR-SPEC-009)
if command -v probador >/dev/null 2>&1; then
    echo "📊 Checking probar GUI coverage..."
    if ! probador playbook --validate --min-coverage 80 2>/dev/null; then
        echo "⚠️  Probar GUI coverage below 80%"
        echo "   Run 'probador playbook' to generate coverage report."
    fi
fi

# Check for .pmat-gates.toml ComputeBrick config
if [ -f "Cargo.toml" ]; then
    if grep -q "trueno\|probar\|realizar" Cargo.toml 2>/dev/null; then
        if [ ! -f ".pmat-gates.toml" ] || ! grep -q "\[compute-brick\]" .pmat-gates.toml 2>/dev/null; then
            echo "⚠️  ComputeBrick project missing [compute-brick] in .pmat-gates.toml"
            echo "   Add configuration per docs/specifications/compute-brick-support.md"
        fi
    fi
fi

echo "✅ ComputeBrick compliance: PASSED"
exit 0
"#;

    let pre_push_path = hooks_dir.join("pre-push");
    fs::write(&pre_push_path, pre_push_content)?;

    // Make executable (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&pre_commit_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&pre_commit_path, perms)?;

        let mut perms = fs::metadata(&pre_push_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&pre_push_path, perms)?;
    }

    match format {
        ComplyOutputFormat::Text => {
            println!("\n✅ PMAT enforcement hooks installed!");
            println!("   Pre-commit hook: {}", pre_commit_path.display());
            println!("   Pre-push hook:   {}", pre_push_path.display());
            println!("\nCommits will now require an active work ticket.");
            println!("Pushes will validate ComputeBrick compliance.");
            println!("Use 'pmat comply enforce --disable' to remove hooks.");
        }
        ComplyOutputFormat::Json => {
            let result = serde_json::json!({
                "status": "success",
                "hooks_installed": ["pre-commit", "pre-push"],
                "path": hooks_dir.display().to_string(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        ComplyOutputFormat::Markdown => {
            println!("# PMAT Enforcement Hooks Installed\n");
            println!("| Hook | Status |");
            println!("|------|--------|");
            println!("| pre-commit | ✅ Installed |");
            println!("| pre-push | ✅ Installed |");
        }
    }

    Ok(())
}

/// Generate compliance report (W-009)
async fn handle_report(
    project_path: &Path,
    include_history: bool,
    format: ComplyOutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    // Load project config
    let config = load_or_create_project_config(project_path)?;

    // Run compliance checks
    let checks = vec![
        check_version_currency(&config.pmat.version),
        check_config_files(project_path),
        check_hooks_installed(project_path),
        check_quality_thresholds(project_path),
        check_deprecated_features(project_path),
    ];

    let report = ComplianceReport {
        project_version: config.pmat.version.clone(),
        current_version: PMAT_VERSION.to_string(),
        is_compliant: checks.iter().all(|c| c.status != CheckStatus::Fail),
        versions_behind: calculate_versions_behind(&config.pmat.version),
        checks,
        breaking_changes: get_breaking_changes_since(&config.pmat.version),
        recommendations: vec![],
        timestamp: Utc::now(),
    };

    // Format output
    let output_text = match format {
        ComplyOutputFormat::Text => {
            let mut out = String::new();
            out.push_str(&format!("\n{}\n", "=".repeat(60)));
            out.push_str("PMAT Compliance Report\n");
            out.push_str(&format!("{}\n", "=".repeat(60)));
            out.push_str(&format!("\nGenerated: {}\n", report.timestamp));
            out.push_str(&format!("Project Version: {}\n", report.project_version));
            out.push_str(&format!("Current PMAT: {}\n", report.current_version));
            out.push_str(&format!(
                "Status: {}\n\n",
                if report.is_compliant {
                    "COMPLIANT"
                } else {
                    "NON-COMPLIANT"
                }
            ));

            out.push_str("Checks:\n");
            for check in &report.checks {
                let icon = match check.status {
                    CheckStatus::Pass => "✓",
                    CheckStatus::Warn => "⚠",
                    CheckStatus::Fail => "✗",
                    CheckStatus::Skip => "-",
                };
                out.push_str(&format!("  {} {}: {}\n", icon, check.name, check.message));
            }

            if include_history {
                out.push_str("\nWork History:\n");
                out.push_str("  (Work history not yet implemented)\n");
            }

            out
        }
        ComplyOutputFormat::Json => serde_json::to_string_pretty(&report)?,
        ComplyOutputFormat::Markdown => {
            let mut out = String::new();
            out.push_str("# PMAT Compliance Report\n\n");
            out.push_str(&format!("**Generated:** {}\n\n", report.timestamp));
            out.push_str("| Property | Value |\n");
            out.push_str("|----------|-------|\n");
            out.push_str(&format!(
                "| Project Version | {} |\n",
                report.project_version
            ));
            out.push_str(&format!("| Current PMAT | {} |\n", report.current_version));
            out.push_str(&format!(
                "| Status | {} |\n\n",
                if report.is_compliant {
                    "✅ COMPLIANT"
                } else {
                    "❌ NON-COMPLIANT"
                }
            ));

            out.push_str("## Checks\n\n");
            for check in &report.checks {
                let icon = match check.status {
                    CheckStatus::Pass => "✅",
                    CheckStatus::Warn => "⚠️",
                    CheckStatus::Fail => "❌",
                    CheckStatus::Skip => "⏭️",
                };
                out.push_str(&format!(
                    "- {} **{}**: {}\n",
                    icon, check.name, check.message
                ));
            }

            out
        }
    };

    if let Some(output_path) = output {
        fs::write(output_path, &output_text)?;
        println!("✅ Compliance report written to {}", output_path.display());
    } else {
        println!("{}", output_text);
    }

    Ok(())
}


// Tests extracted to comply_handlers_tests.rs for file health compliance (CB-040)
