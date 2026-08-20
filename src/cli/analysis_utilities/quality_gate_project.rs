/// Handles project-wide quality gate checks
#[allow(clippy::too_many_arguments)]
async fn handle_project_quality_gate(
    project_path: PathBuf,
    format: QualityGateOutputFormat,
    exit_on_violation: bool,
    checks: Vec<QualityCheckType>,
    max_dead_code: f64,
    min_entropy: Option<f64>,
    max_complexity_p99: u32,
    include_provability: bool,
    output: Option<PathBuf>,
    perf: bool,
) -> Result<()> {
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
    )
    .await?;

    // Apply [exclude] paths from .pmat-metrics.toml to ALL violations (#196, #197),
    // through the one function that owns the rule — the MCP gate applies the
    // same one to the same findings.
    let removed = apply_gate_exclude_paths(&project_path, &mut violations);
    if removed > 0 {
        crate::status_eprintln!("  📁 Excluded {removed} violations from excluded paths");
        results.recalculate_from(&violations);
    }

    // Add provability if requested
    if include_provability {
        let prov_start = if perf { Some(Instant::now()) } else { None };
        let provability_score = calculate_provability_score(&project_path).await?;
        results.provability_score = Some(provability_score);

        if let Some(start) = prov_start {
            crate::status_eprintln!(
                "  ⏱️  Provability analysis: {:.3}s",
                start.elapsed().as_secs_f64()
            );
        }
    }

    if let Some(start) = checks_start {
        let duration = start.elapsed();
        crate::status_eprintln!(
            "\n⏱️  All checks completed in: {:.3}s",
            duration.as_secs_f64()
        );
    }

    // Calculate overall pass/fail — THE rule, from the one place that owns it.
    // This was `violations.is_empty()`, i.e. any finding of any severity failed
    // the gate, while the MCP `quality_gate` tool over the SAME producer
    // (`check_satd`) ignored `severity:"info"`. One `// TODO` in one file was
    // `passed:false` here and `passed:true` there, with byte-identical findings.
    results.passed = violations_pass(&violations);
    results.total_violations = violations.len();
    results.blocking_violations = blocking_violation_count(&violations);
    // `results.violations` shipped as a permanently-empty array while
    // `results.total_violations` beside it said 3.
    results.set_violation_lines(&violations);

    // Persist violations to SQLite for `pmat sql` queryability
    persist_violations_to_sqlite(&project_path, &violations);

    // Persist per-function provability scores to specialized table (#231)
    persist_provability_to_sqlite(&project_path).await;

    // Format and output results
    output_project_results(&results, &violations, format, output).await?;

    // Print final status (chatter: the verdict is also in the report on stdout
    // and in the exit status, so --quiet suppresses this line only)
    print_quality_gate_final_status(&results, &violations);

    // Handle exit status
    handle_quality_gate_exit_status(exit_on_violation, results.passed);

    Ok(())
}

/// Runs project-wide quality checks
#[allow(clippy::too_many_arguments)]
async fn run_project_checks(
    project_path: &Path,
    checks: &[QualityCheckType],
    max_dead_code: f64,
    min_entropy: Option<f64>,
    max_complexity_p99: u32,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
    perf: bool,
) -> Result<()> {
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
    min_entropy: Option<f64>,
    max_complexity_p99: u32,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
    perf: bool,
) -> Result<()> {
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
        )
        .await?;

        if let Some(start) = check_start {
            print_check_performance(check, start.elapsed().as_secs_f64());
        }
    }
    Ok(())
}

/// Print performance timing for a quality check
fn print_check_performance(check: &QualityCheckType, elapsed_secs: f64) {
    let check_name = get_check_display_name(check);
    crate::status_eprintln!("    ⏱️  {check_name} check: {elapsed_secs:.3}s");
}

/// Get display name for a quality check type
fn get_check_display_name(check: &QualityCheckType) -> &'static str {
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
