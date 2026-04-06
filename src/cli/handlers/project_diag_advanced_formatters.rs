// Advanced Checks (3)
// ============================================================================

fn check_msrv_defined(project_path: &Path) -> DiagnosticCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let cargo_toml = project_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();

    let (status, score, message) = if content.contains("rust-version") {
        (
            HealthStatus::Green,
            5.0,
            "MSRV defined (rust-version field)".to_string(),
        )
    } else {
        (
            HealthStatus::Yellow,
            0.0,
            "No MSRV - add rust-version to Cargo.toml".to_string(),
        )
    };

    DiagnosticCheck {
        name: "MSRV Defined".to_string(),
        category: "Advanced".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn check_benchmarks(project_path: &Path) -> DiagnosticCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let benches_dir = project_path.join("benches");
    let cargo_toml = project_path.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();

    let has_bench_dir = benches_dir.exists() && benches_dir.is_dir();
    let has_criterion = content.contains("criterion");

    let (status, score, message) = if has_bench_dir && has_criterion {
        (
            HealthStatus::Green,
            5.0,
            "Criterion benchmarks configured".to_string(),
        )
    } else if has_bench_dir {
        (
            HealthStatus::Yellow,
            3.0,
            "Benchmarks present (consider Criterion)".to_string(),
        )
    } else {
        (
            HealthStatus::Yellow,
            0.0,
            "No benchmarks - add benches/ directory".to_string(),
        )
    };

    DiagnosticCheck {
        name: "Benchmarks".to_string(),
        category: "Advanced".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

fn score_github_workflows(workflows_dir: &Path) -> (HealthStatus, f64, String) {
    debug_assert!(workflows_dir.exists(), "workflows_dir must exist: {}", workflows_dir.display());
    let workflow_count = std::fs::read_dir(workflows_dir)
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0);
    match workflow_count {
        3.. => (HealthStatus::Green, 5.0, format!("{workflow_count} GitHub Actions workflows")),
        1..=2 => (HealthStatus::Yellow, 3.0, format!("{workflow_count} GitHub Actions workflow(s)")),
        _ => (HealthStatus::Yellow, 1.0, "Empty .github/workflows directory".to_string()),
    }
}

fn detect_ci_system(project_path: &Path) -> (HealthStatus, f64, String) {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let github_workflows = project_path.join(".github").join("workflows");
    if github_workflows.exists() && github_workflows.is_dir() {
        return score_github_workflows(&github_workflows);
    }
    if project_path.join(".gitlab-ci.yml").exists() {
        return (HealthStatus::Green, 5.0, "GitLab CI configured".to_string());
    }
    if project_path.join("Jenkinsfile").exists() {
        return (HealthStatus::Green, 5.0, "Jenkins pipeline configured".to_string());
    }
    (HealthStatus::Red, 0.0, "No CI configured - add .github/workflows/".to_string())
}

fn check_ci_configured(project_path: &Path) -> DiagnosticCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let (status, score, message) = detect_ci_system(project_path);
    DiagnosticCheck {
        name: "CI Configured".to_string(),
        category: "Advanced".to_string(),
        status,
        message,
        score,
        max_score: 5.0,
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn dir_size(path: &Path) -> std::io::Result<u64> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let mut total = 0u64;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                total += dir_size(&path).unwrap_or(0);
            } else {
                total += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    Ok(total)
}

fn file_has_test_markers(path: &Path) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let is_rs = path.extension().map(|e| e == "rs").unwrap_or(false);
    if !is_rs {
        return false;
    }
    std::fs::read_to_string(path)
        .map(|c| c.contains("#[test]") || c.contains("#[cfg(test)]"))
        .unwrap_or(false)
}

fn has_test_annotations(dir: &Path) -> bool {
    debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return false,
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && file_has_test_markers(&path) {
            return true;
        }
        if path.is_dir() && has_test_annotations(&path) {
            return true;
        }
    }
    false
}

// ============================================================================
// Output Formatters
// ============================================================================

fn format_summary(report: &DiagnosticReport, failures_only: bool) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&format!(
        "\n  {}\n",
        colors::header(&format!("Project Diagnostics: {}", colors::path(&report.project_path)))
    ));
    output.push_str(&format!("  {}\n\n", colors::rule()));

    // Overall score
    let status_icon = match report.overall_status {
        HealthStatus::Green => format!("{}GREEN{}", colors::BOLD_GREEN, colors::RESET),
        HealthStatus::Yellow => format!("{}YELLOW{}", colors::BOLD_YELLOW, colors::RESET),
        HealthStatus::Red => format!("{}RED{}", colors::BOLD_RED, colors::RESET),
        HealthStatus::Skip => format!("{}SKIP{}", colors::DIM, colors::RESET),
    };
    output.push_str(&format!(
        "  Overall: {} {} ({})\n\n",
        status_icon,
        colors::score(report.total_score, report.max_score, 85.0, 60.0),
        colors::pct(report.percentage, 85.0, 60.0)
    ));

    // Category summaries
    for cat in &report.categories {
        output.push_str(&format!(
            "  {} [{}/{}]\n",
            colors::label(&cat.name),
            colors::number(&cat.passed.to_string()),
            cat.total
        ));
    }
    output.push('\n');

    // Individual checks
    output.push_str(&format!("  {}\n", colors::subheader("Checks:")));
    output.push_str(&format!("  {}\n", colors::separator()));

    for check in &report.checks {
        if failures_only && check.status == HealthStatus::Green {
            continue;
        }

        let line = format!("{} - {}", check.name, check.message);
        let formatted = match check.status {
            HealthStatus::Green => colors::pass(&line),
            HealthStatus::Yellow => colors::warn(&line),
            HealthStatus::Red => colors::fail(&line),
            HealthStatus::Skip => colors::skip(&line),
        };

        output.push_str(&format!("  {}\n", formatted));
    }

    output.push('\n');
    output
}

fn format_json(report: &DiagnosticReport) -> Result<String> {
    serde_json::to_string_pretty(report).map_err(|e| anyhow::anyhow!(e))
}

fn format_markdown(report: &DiagnosticReport, failures_only: bool) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "# Project Diagnostics: {}\n\n",
        report.project_path
    ));

    // Overall status
    let badge = match report.overall_status {
        HealthStatus::Green => "![Status](https://img.shields.io/badge/status-healthy-green)",
        HealthStatus::Yellow => "![Status](https://img.shields.io/badge/status-warning-yellow)",
        HealthStatus::Red => "![Status](https://img.shields.io/badge/status-critical-red)",
        HealthStatus::Skip => "![Status](https://img.shields.io/badge/status-skipped-gray)",
    };
    output.push_str(&format!("{}\n\n", badge));
    output.push_str(&format!(
        "**Score:** {:.1}/{:.1} ({:.1}%)\n\n",
        report.total_score, report.max_score, report.percentage
    ));

    // Category table
    output.push_str("## Categories\n\n");
    output.push_str("| Category | Passed | Warned | Failed | Score |\n");
    output.push_str("|----------|--------|--------|--------|-------|\n");
    for cat in &report.categories {
        output.push_str(&format!(
            "| {} | {} | {} | {} | {:.1}/{:.1} |\n",
            cat.name, cat.passed, cat.warned, cat.failed, cat.score, cat.max_score
        ));
    }
    output.push('\n');

    // Checks table
    output.push_str("## Checks\n\n");
    output.push_str("| Status | Check | Message |\n");
    output.push_str("|--------|-------|--------|\n");
    for check in &report.checks {
        if failures_only && check.status == HealthStatus::Green {
            continue;
        }
        let emoji = match check.status {
            HealthStatus::Green => "✅",
            HealthStatus::Yellow => "⚠️",
            HealthStatus::Red => "❌",
            HealthStatus::Skip => "⏭️",
        };
        output.push_str(&format!(
            "| {} | {} | {} |\n",
            emoji, check.name, check.message
        ));
    }

    output
}

fn format_andon(report: &DiagnosticReport) -> String {
    let mut output = String::new();

    // Andon-style visualization (Toyota Way)
    output.push('\n');
    output.push_str(&format!("  {}╔══════════════════════════════════════════════════════════════╗{}\n", colors::BOLD, colors::RESET));
    output.push_str(&format!("  {}║                    PROJECT DIAGNOSTICS                       ║{}\n", colors::BOLD, colors::RESET));
    output.push_str(&format!("  {}║                      (Andon Board)                           ║{}\n", colors::BOLD, colors::RESET));
    output.push_str(&format!("  {}╠══════════════════════════════════════════════════════════════╣{}\n", colors::BOLD, colors::RESET));

    // Score display
    let bar_width = 40;
    let filled = ((report.percentage / 100.0) * bar_width as f64) as usize;
    let empty = bar_width - filled;
    let bar_color = if report.percentage >= 85.0 {
        colors::GREEN
    } else if report.percentage >= 60.0 {
        colors::YELLOW
    } else {
        colors::RED
    };
    let progress_bar = format!(
        "{}{}{}{}",
        bar_color,
        "#".repeat(filled),
        colors::DIM,
        "-".repeat(empty)
    );

    output.push_str(&format!(
        "  {}║{}  Score: [{progress_bar}{}] {}  {}║{}\n",
        colors::BOLD, colors::RESET,
        colors::RESET,
        colors::pct(report.percentage, 85.0, 60.0),
        colors::BOLD, colors::RESET
    ));
    output.push_str(&format!("  {}╠══════════════════════════════════════════════════════════════╣{}\n", colors::BOLD, colors::RESET));

    // Category lights
    for cat in &report.categories {
        let light = if cat.failed > 0 {
            format!("{}●{} ", colors::RED, colors::RESET)
        } else if cat.warned > 0 {
            format!("{}●{} ", colors::YELLOW, colors::RESET)
        } else {
            format!("{}●{} ", colors::GREEN, colors::RESET)
        };
        output.push_str(&format!(
            "  {}║{}  {} {:20} {}/{} checks passed          {}║{}\n",
            colors::BOLD, colors::RESET,
            light, cat.name, cat.passed, cat.total,
            colors::BOLD, colors::RESET
        ));
    }

    output.push_str(&format!("  {}╠══════════════════════════════════════════════════════════════╣{}\n", colors::BOLD, colors::RESET));

    // Failed checks (Andon cord triggers)
    let failures: Vec<_> = report
        .checks
        .iter()
        .filter(|c| c.status == HealthStatus::Red)
        .collect();

    if failures.is_empty() {
        output.push_str(&format!(
            "  {}║{}  {}No critical issues - production ready{}                       {}║{}\n",
            colors::BOLD, colors::RESET,
            colors::GREEN, colors::RESET,
            colors::BOLD, colors::RESET
        ));
    } else {
        output.push_str(&format!(
            "  {}║{}  {}ANDON CORD TRIGGERED{} - Issues require attention:            {}║{}\n",
            colors::BOLD, colors::RESET,
            colors::BOLD_RED, colors::RESET,
            colors::BOLD, colors::RESET
        ));
        for check in failures.iter().take(5) {
            output.push_str(&format!(
                "  {}║{}    {}●{} {:<54} {}║{}\n",
                colors::BOLD, colors::RESET,
                colors::RED, colors::RESET,
                check.name,
                colors::BOLD, colors::RESET
            ));
        }
    }

    output.push_str(&format!("  {}╚══════════════════════════════════════════════════════════════╝{}\n", colors::BOLD, colors::RESET));
    output.push('\n');

    output
}

