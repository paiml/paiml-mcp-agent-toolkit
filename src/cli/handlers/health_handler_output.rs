/// Parse coverage percentage from llvm-cov output
fn parse_coverage_percentage(output: &str) -> f64 {
    for line in output.lines() {
        if line.contains("TOTAL") {
            // Expected format: "TOTAL   1234   1000   80.0%"
            if let Some(pct_str) = line.split_whitespace().last() {
                if let Some(num_str) = pct_str.strip_suffix('%') {
                    if let Ok(pct) = num_str.parse::<f64>() {
                        return pct;
                    }
                }
            }
        }
    }
    0.0
}

/// Calculate summary from checks
fn calculate_summary(checks: &[HealthCheck]) -> HealthSummary {
    let mut summary = HealthSummary {
        total_checks: checks.len(),
        passed: 0,
        warned: 0,
        failed: 0,
        skipped: 0,
    };

    for check in checks {
        match check.status {
            CheckStatus::Pass => summary.passed += 1,
            CheckStatus::Warn => summary.warned += 1,
            CheckStatus::Fail => summary.failed += 1,
            CheckStatus::Skip => summary.skipped += 1,
        }
    }

    summary
}

/// Print health report
fn print_health_report(report: &HealthReport, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(report)?);
        }
        OutputFormat::Yaml => {
            print_health_yaml(report);
        }
        OutputFormat::Table => {
            print_health_table(report);
        }
        _ => {
            print_health_table(report);
        }
    }
    Ok(())
}

/// Print health report as table
fn print_health_table(report: &HealthReport) {
    let overall_icon = if report.healthy { "✅" } else { "❌" };
    eprintln!("{} {}\n", overall_icon, colors::header("Project Health Report"));

    for check in &report.checks {
        let (icon, status_color) = match check.status {
            CheckStatus::Pass => ("✅", colors::GREEN),
            CheckStatus::Warn => ("⚠️ ", colors::YELLOW),
            CheckStatus::Fail => ("❌", colors::RED),
            CheckStatus::Skip => ("⏭️ ", colors::DIM),
        };

        eprintln!(
            "{} {}{}{}: {}",
            icon, status_color, check.name, colors::RESET, check.message
        );
        if let Some(details) = &check.details {
            eprintln!("   {}", colors::path(details));
        }
    }

    eprintln!("\n📊 {}", colors::subheader("Summary:"));
    eprintln!(
        "   Total:   {}",
        colors::number(&report.summary.total_checks.to_string())
    );
    eprintln!(
        "   Passed:  {}{}{}",
        colors::GREEN,
        report.summary.passed,
        colors::RESET
    );
    eprintln!(
        "   Warned:  {}{}{}",
        colors::YELLOW,
        report.summary.warned,
        colors::RESET
    );
    eprintln!(
        "   Failed:  {}{}{}",
        colors::RED,
        report.summary.failed,
        colors::RESET
    );
    eprintln!(
        "   Skipped: {}{}{}",
        colors::DIM,
        report.summary.skipped,
        colors::RESET
    );

    if report.healthy {
        eprintln!(
            "\n✨ {}Project is healthy!{}",
            colors::BOLD_GREEN,
            colors::RESET
        );
    } else {
        eprintln!(
            "\n⚠️  {}Project has {} issue(s){}",
            colors::BOLD_RED,
            report.summary.failed,
            colors::RESET
        );
    }
}

/// Print health report as YAML
fn print_health_yaml(report: &HealthReport) {
    println!("healthy: {}", report.healthy);
    println!("checks:");
    for check in &report.checks {
        println!("  - name: {}", check.name);
        println!("    status: {:?}", check.status);
        println!("    message: {}", check.message);
        if let Some(details) = &check.details {
            println!("    details: {}", details);
        }
    }
    println!("summary:");
    println!("  total_checks: {}", report.summary.total_checks);
    println!("  passed: {}", report.summary.passed);
    println!("  warned: {}", report.summary.warned);
    println!("  failed: {}", report.summary.failed);
    println!("  skipped: {}", report.summary.skipped);
}

/// Checks configuration (TICKET-PMAT-6001)
struct ChecksToRun {
    build: bool,
    tests: bool,
    coverage: bool,
    complexity: bool,
    satd: bool,
}

/// Determine which checks to run based on flags (TICKET-PMAT-6001)
///
/// # Logic
/// - quick: only build
/// - all: enable everything
/// - no flags: only build (default)
/// - specific flags: only those checks
///
/// # Complexity
/// - Cyclomatic: 7
fn determine_checks_to_run(
    quick: bool,
    all: bool,
    check_build: bool,
    check_tests: bool,
    check_coverage: bool,
    check_complexity: bool,
    check_satd: bool,
) -> ChecksToRun {
    // Quick mode: only build
    if quick {
        return ChecksToRun {
            build: true,
            tests: false,
            coverage: false,
            complexity: false,
            satd: false,
        };
    }

    // All mode: enable everything
    if all {
        return ChecksToRun {
            build: true,
            tests: true,
            coverage: true,
            complexity: true,
            satd: true,
        };
    }

    // Check if any specific flags are set
    let has_specific_flags =
        check_build || check_tests || check_coverage || check_complexity || check_satd;

    // If no flags specified, default to build only
    if !has_specific_flags {
        return ChecksToRun {
            build: true,
            tests: false,
            coverage: false,
            complexity: false,
            satd: false,
        };
    }

    // Use specified flags
    ChecksToRun {
        build: check_build,
        tests: check_tests,
        coverage: check_coverage,
        complexity: check_complexity,
        satd: check_satd,
    }
}
