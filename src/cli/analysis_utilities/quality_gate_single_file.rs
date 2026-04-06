/// Handles quality gate checks for a single file
#[allow(clippy::too_many_arguments)]
async fn handle_single_file_quality_gate(
    project_path: PathBuf,
    single_file: PathBuf,
    format: QualityGateOutputFormat,
    fail_on_violation: bool,
    checks: Vec<QualityCheckType>,
    max_complexity_p99: u32,
    output: Option<PathBuf>,
    perf: bool,
    quiet: bool,
) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    debug_assert!(single_file.exists(), "single_file must exist: {}", single_file.display());
    use std::time::Instant;
    if !quiet {
        eprintln!("📄 Analyzing single file: {}", single_file.display());
    }

    let mut violations = Vec::new();
    let mut results = QualityGateResults::default();

    // Determine which checks to run (default to All if none specified)
    let checks_to_run = if checks.is_empty() {
        vec![QualityCheckType::All]
    } else {
        checks
    };

    // Run checks on the single file
    let check_start = if perf { Some(Instant::now()) } else { None };

    run_single_file_checks(
        &project_path,
        &single_file,
        &checks_to_run,
        max_complexity_p99,
        &mut violations,
        &mut results,
    )
    .await?;

    if !quiet {
        if let Some(start) = check_start {
            let duration = start.elapsed();
            eprintln!("\n⏱️  File analysis took: {:.3}s", duration.as_secs_f64());
        }
    }

    // Calculate overall status
    results.passed = violations.is_empty();
    results.total_violations = violations.len();

    // Format and output results
    output_single_file_results(&single_file, &results, &violations, format, output).await?;

    // Handle exit status
    handle_quality_gate_exit_status(fail_on_violation, results.passed);

    Ok(())
}

/// Runs quality checks on a single file
async fn run_single_file_checks(
    project_path: &Path,
    single_file: &Path,
    checks_to_run: &[QualityCheckType],
    max_complexity_p99: u32,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    debug_assert!(single_file.exists(), "single_file must exist: {}", single_file.display());
    for check in checks_to_run {
        execute_single_file_check(
            check,
            project_path,
            single_file,
            max_complexity_p99,
            violations,
            results,
        )
        .await?;
    }
    Ok(())
}

/// Extract Method: Execute a specific single file check
async fn execute_single_file_check(
    check: &QualityCheckType,
    project_path: &Path,
    single_file: &Path,
    max_complexity_p99: u32,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    debug_assert!(single_file.exists(), "single_file must exist: {}", single_file.display());
    match check {
        QualityCheckType::Complexity => {
            run_single_file_complexity_check(
                project_path,
                single_file,
                max_complexity_p99,
                violations,
                results,
            )
            .await
        }
        QualityCheckType::DeadCode => {
            run_single_file_dead_code_check(project_path, single_file, violations, results).await
        }
        QualityCheckType::Satd => {
            run_single_file_satd_check(project_path, single_file, violations, results).await
        }
        QualityCheckType::Security => {
            run_single_file_security_check(project_path, single_file, violations, results).await
        }
        QualityCheckType::All => {
            run_all_single_file_checks(
                project_path,
                single_file,
                max_complexity_p99,
                violations,
                results,
            )
            .await
        }
        _ => {
            handle_unsupported_single_file_check(check);
            Ok(())
        }
    }
}

/// Extract Method: Handle unsupported single file check types
fn handle_unsupported_single_file_check(check: &QualityCheckType) {
    debug_assert!(true, "contract: handle_unsupported_single_file_check");
    eprintln!("⚠️  Skipping {check} check - not applicable to single file");
}

/// Runs all single file checks
async fn run_all_single_file_checks(
    project_path: &Path,
    single_file: &Path,
    max_complexity_p99: u32,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    debug_assert!(single_file.exists(), "single_file must exist: {}", single_file.display());
    run_single_file_complexity_check(
        project_path,
        single_file,
        max_complexity_p99,
        violations,
        results,
    )
    .await?;
    run_single_file_dead_code_check(project_path, single_file, violations, results).await?;
    run_single_file_satd_check(project_path, single_file, violations, results).await?;
    run_single_file_security_check(project_path, single_file, violations, results).await?;
    Ok(())
}

/// Runs complexity check on a single file
async fn run_single_file_complexity_check(
    project_path: &Path,
    single_file: &Path,
    max_complexity_p99: u32,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    debug_assert!(single_file.exists(), "single_file must exist: {}", single_file.display());
    eprint!("  🔍 Checking complexity...");
    let violations_found =
        check_single_file_complexity(project_path, single_file, max_complexity_p99).await?;
    results.complexity_violations = violations_found.len();
    eprintln!(" {} violations found", results.complexity_violations);
    violations.extend(violations_found);
    Ok(())
}

/// Runs dead code check on a single file
async fn run_single_file_dead_code_check(
    project_path: &Path,
    single_file: &Path,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    debug_assert!(single_file.exists(), "single_file must exist: {}", single_file.display());
    eprint!("  🔍 Checking dead code...");
    let violations_found = check_single_file_dead_code(project_path, single_file).await?;
    results.dead_code_violations = violations_found.len();
    eprintln!(" {} violations found", results.dead_code_violations);
    violations.extend(violations_found);
    Ok(())
}

/// Runs SATD check on a single file
async fn run_single_file_satd_check(
    project_path: &Path,
    single_file: &Path,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    debug_assert!(single_file.exists(), "single_file must exist: {}", single_file.display());
    eprint!("  🔍 Checking SATD...");
    let violations_found = check_single_file_satd(project_path, single_file).await?;
    results.satd_violations = violations_found.len();
    eprintln!(" {} violations found", results.satd_violations);
    violations.extend(violations_found);
    Ok(())
}

/// Runs security check on a single file
async fn run_single_file_security_check(
    project_path: &Path,
    single_file: &Path,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    debug_assert!(single_file.exists(), "single_file must exist: {}", single_file.display());
    eprint!("  🔍 Checking security...");
    let violations_found = check_single_file_security(project_path, single_file).await?;
    results.security_violations = violations_found.len();
    eprintln!(" {} violations found", results.security_violations);
    violations.extend(violations_found);
    Ok(())
}

/// Formats and outputs single file results
async fn output_single_file_results(
    single_file: &Path,
    results: &QualityGateResults,
    violations: &[QualityViolation],
    format: QualityGateOutputFormat,
    output: Option<PathBuf>,
) -> Result<()> {
    debug_assert!(single_file.exists(), "single_file must exist: {}", single_file.display());
    let output_content = format_single_file_output(single_file, results, violations, format)?;

    if let Some(output_path) = output {
        std::fs::write(output_path, &output_content)?;
    } else {
        println!("{output_content}");
    }

    Ok(())
}

/// Formats single file output based on the requested format
fn format_single_file_output(
    single_file: &Path,
    results: &QualityGateResults,
    violations: &[QualityViolation],
    format: QualityGateOutputFormat,
) -> Result<String> {
    debug_assert!(single_file.exists(), "single_file must exist: {}", single_file.display());
    match format {
        QualityGateOutputFormat::Json => Ok(serde_json::to_string_pretty(&json!({
            "file": single_file,
            "passed": results.passed,
            "results": results,
            "violations": violations,
        }))?),
        QualityGateOutputFormat::Summary
        | QualityGateOutputFormat::Markdown
        | QualityGateOutputFormat::Detailed
        | QualityGateOutputFormat::Human
        | QualityGateOutputFormat::Junit => {
            Ok(format_single_file_summary(single_file, results, violations))
        }
    }
}

