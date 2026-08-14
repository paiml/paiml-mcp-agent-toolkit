// Advanced Checks (3)
// ============================================================================

fn check_msrv_defined(project_path: &Path) -> DiagnosticCheck {
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
    let is_rs = path.extension().map(|e| e == "rs").unwrap_or(false);
    if !is_rs {
        return false;
    }
    std::fs::read_to_string(path)
        .map(|c| c.contains("#[test]") || c.contains("#[cfg(test)]"))
        .unwrap_or(false)
}

fn has_test_annotations(dir: &Path) -> bool {
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
    // GH #684: these four used to interpolate the raw `pub const` sequences,
    // which are `const` and so cannot consult `colors_enabled`. `pmat
    // project-diag --color never > out.txt` still wrote
    // `Overall: ^[[1;31mRED^[[0m` (and 30 such lines in the andon format), and
    // NO_COLOR=1 was ignored for the same reason. `colors::colored` is the
    // helper that honours the rule while keeping the colour SELECTION here.
    let status_icon = match report.overall_status {
        HealthStatus::Green => colors::colored(colors::BOLD_GREEN, "GREEN"),
        HealthStatus::Yellow => colors::colored(colors::BOLD_YELLOW, "YELLOW"),
        HealthStatus::Red => colors::colored(colors::BOLD_RED, "RED"),
        HealthStatus::Skip => colors::colored(colors::DIM, "SKIP"),
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

    // GH #684: every sequence below used to be a raw `pub const`, interpolated
    // with its opening half in one `format!` argument and its closing half in
    // another. `const`s cannot consult `colors_enabled`, so `pmat project-diag
    // --format andon --color never > out.txt` wrote 30 escape-bearing lines,
    // byte-identical to `--color auto`, and NO_COLOR=1 changed nothing.
    // `colors::seq` is the documented mechanical migration for exactly this
    // split-across-arguments shape: it is `""` when colour is off.
    // Gate explicitly on `colors_enabled` and take the bytes with `Sgr::raw`,
    // which is documented as ungated: `colors::seq` is now an identity on
    // `Sgr`, so interpolating its result emits the escape whatever `--color`
    // says.
    let on = colors::colors_enabled();
    let sgr = |s: colors::Sgr| if on { s.raw() } else { "" };
    let bold = sgr(colors::BOLD);
    let reset = sgr(colors::RESET);
    let dim = sgr(colors::DIM);
    let green = sgr(colors::GREEN);
    let yellow = sgr(colors::YELLOW);
    let red = sgr(colors::RED);
    let bold_red = sgr(colors::BOLD_RED);

    // Andon-style visualization (Toyota Way)
    output.push('\n');
    output.push_str(&format!("  {bold}\u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2557}{reset}\n"));
    output.push_str(&format!("  {bold}\u{2551}                    PROJECT DIAGNOSTICS                       \u{2551}{reset}\n"));
    output.push_str(&format!("  {bold}\u{2551}                      (Andon Board)                           \u{2551}{reset}\n"));
    output.push_str(&format!("  {bold}\u{2560}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2563}{reset}\n"));

    // Score display
    let bar_width = 40;
    let filled = ((report.percentage / 100.0) * bar_width as f64) as usize;
    let empty = bar_width - filled;
    let bar_color = sgr(colors::threshold_color(report.percentage, 85.0, 60.0));
    let progress_bar = format!(
        "{}{}{}{}",
        bar_color,
        "#".repeat(filled),
        dim,
        "-".repeat(empty)
    );

    output.push_str(&format!(
        "  {bold}\u{2551}{reset}  Score: [{progress_bar}{reset}] {}  {bold}\u{2551}{reset}\n",
        colors::pct(report.percentage, 85.0, 60.0)
    ));
    output.push_str(&format!("  {bold}\u{2560}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2563}{reset}\n"));

    // Category lights
    for cat in &report.categories {
        let light_color = if cat.failed > 0 {
            red
        } else if cat.warned > 0 {
            yellow
        } else {
            green
        };
        let light = format!("{light_color}\u{25cf}{reset} ");
        output.push_str(&format!(
            "  {bold}\u{2551}{reset}  {light} {:20} {}/{} checks passed          {bold}\u{2551}{reset}\n",
            cat.name, cat.passed, cat.total
        ));
    }

    output.push_str(&format!("  {bold}\u{2560}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2563}{reset}\n"));

    // Failed checks (Andon cord triggers)
    let failures: Vec<_> = report
        .checks
        .iter()
        .filter(|c| c.status == HealthStatus::Red)
        .collect();

    if failures.is_empty() {
        output.push_str(&format!(
            "  {bold}\u{2551}{reset}  {green}No critical issues - production ready{reset}                       {bold}\u{2551}{reset}\n"
        ));
    } else {
        output.push_str(&format!(
            "  {bold}\u{2551}{reset}  {bold_red}ANDON CORD TRIGGERED{reset} - Issues require attention:            {bold}\u{2551}{reset}\n"
        ));
        for check in failures.iter().take(5) {
            output.push_str(&format!(
                "  {bold}\u{2551}{reset}    {red}\u{25cf}{reset} {:<54} {bold}\u{2551}{reset}\n",
                check.name
            ));
        }
    }

    output.push_str(&format!("  {bold}\u{255a}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}{reset}\n"));
    output.push('\n');

    output
}

