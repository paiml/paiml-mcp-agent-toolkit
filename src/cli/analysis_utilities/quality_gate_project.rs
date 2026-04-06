/// Handles project-wide quality gate checks
#[allow(clippy::too_many_arguments)]
async fn handle_project_quality_gate(
    project_path: PathBuf,
    format: QualityGateOutputFormat,
    fail_on_violation: bool,
    checks: Vec<QualityCheckType>,
    max_dead_code: f64,
    min_entropy: f64,
    max_complexity_p99: u32,
    include_provability: bool,
    output: Option<PathBuf>,
    perf: bool,
    quiet: bool,
) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    use std::time::Instant;
    let mut violations = Vec::new();
    let mut results = QualityGateResults::default();

    // Run selected checks
    let checks_start = if perf { Some(Instant::now()) } else { None };

    run_project_checks(
        &project_path,
        &checks,
        max_dead_code,
        min_entropy,
        max_complexity_p99,
        &mut violations,
        &mut results,
        perf,
        quiet,
    )
    .await?;

    // Apply [exclude] paths from .pmat-metrics.toml to ALL violations (#196, #197)
    let exclude_paths = load_entropy_exclude_paths(&project_path);
    if !exclude_paths.is_empty() {
        let before = violations.len();
        filter_violations_by_exclude(&mut violations, &exclude_paths);
        let removed = before - violations.len();
        if removed > 0 {
            if !quiet {
                eprintln!("  📁 Excluded {removed} violations from excluded paths");
            }
            results.recalculate_from(&violations);
        }
    }

    // Add provability if requested
    if include_provability {
        let prov_start = if perf { Some(Instant::now()) } else { None };
        let provability_score = calculate_provability_score(&project_path).await?;
        results.provability_score = Some(provability_score);

        if !quiet {
            if let Some(start) = prov_start {
                eprintln!(
                    "  ⏱️  Provability analysis: {:.3}s",
                    start.elapsed().as_secs_f64()
                );
            }
        }
    }

    if !quiet {
        if let Some(start) = checks_start {
            let duration = start.elapsed();
            eprintln!(
                "\n⏱️  All checks completed in: {:.3}s",
                duration.as_secs_f64()
            );
        }
    }

    // Calculate overall pass/fail
    results.passed = violations.is_empty();
    results.total_violations = violations.len();

    // Persist violations to SQLite for `pmat sql` queryability
    persist_violations_to_sqlite(&project_path, &violations, quiet);

    // Persist per-function provability scores to specialized table (#231)
    persist_provability_to_sqlite(&project_path, quiet).await;

    // Format and output results
    output_project_results(&results, &violations, format, output).await?;

    // Print final status (suppress for JSON/machine-readable output, #230)
    if !quiet {
        print_quality_gate_final_status(&results, &violations);
    }

    // Handle exit status
    handle_quality_gate_exit_status(fail_on_violation, results.passed);

    Ok(())
}

/// Runs project-wide quality checks
#[allow(clippy::too_many_arguments)]
async fn run_project_checks(
    project_path: &Path,
    checks: &[QualityCheckType],
    max_dead_code: f64,
    min_entropy: f64,
    max_complexity_p99: u32,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
    perf: bool,
    quiet: bool,
) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    // If checks contains All, just run that single check which will run all checks
    if checks.contains(&QualityCheckType::All) {
        run_single_project_check(
            &QualityCheckType::All,
            project_path,
            max_dead_code,
            min_entropy,
            max_complexity_p99,
            violations,
            results,
            perf,
            quiet,
        )
        .await?;
    } else {
        // Otherwise run each specified check
        run_individual_project_checks(
            checks,
            project_path,
            max_dead_code,
            min_entropy,
            max_complexity_p99,
            violations,
            results,
            perf,
            quiet,
        )
        .await?;
    }
    Ok(())
}

/// Run individual quality checks with optional performance timing
#[allow(clippy::too_many_arguments)]
async fn run_individual_project_checks(
    checks: &[QualityCheckType],
    project_path: &Path,
    max_dead_code: f64,
    min_entropy: f64,
    max_complexity_p99: u32,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
    perf: bool,
    quiet: bool,
) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    use std::time::Instant;

    for check in checks {
        let check_start = if perf { Some(Instant::now()) } else { None };

        run_single_project_check(
            check,
            project_path,
            max_dead_code,
            min_entropy,
            max_complexity_p99,
            violations,
            results,
            perf,
            quiet,
        )
        .await?;

        if !quiet {
            if let Some(start) = check_start {
                print_check_performance(check, start.elapsed().as_secs_f64());
            }
        }
    }
    Ok(())
}

/// Print performance timing for a quality check
fn print_check_performance(check: &QualityCheckType, elapsed_secs: f64) {
    debug_assert!(true, "contract: print_check_performance");
    let check_name = get_check_display_name(check);
    eprintln!("    ⏱️  {check_name} check: {elapsed_secs:.3}s");
}

/// Get display name for a quality check type
fn get_check_display_name(check: &QualityCheckType) -> &'static str {
    debug_assert!(true, "contract: get_check_display_name");
    match check {
        QualityCheckType::Complexity => "Complexity",
        QualityCheckType::DeadCode => "Dead code",
        QualityCheckType::Satd => "SATD",
        QualityCheckType::Security => "Security",
        QualityCheckType::Entropy => "Entropy",
        QualityCheckType::Duplicates => "Duplicates",
        QualityCheckType::Coverage => "Coverage",
        QualityCheckType::Sections => "Sections",
        QualityCheckType::Provability => "Provability",
        QualityCheckType::All => "All",
    }
}

