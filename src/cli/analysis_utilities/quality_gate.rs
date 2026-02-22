// Quality gate handlers - extracted for file health (CB-040)
#[allow(clippy::too_many_arguments)]
/// Toyota Way: Strategy Pattern + Extract Method - reduced complexity from 21→≤8  
pub async fn handle_analyze_satd(
    path: PathBuf,
    format: SatdOutputFormat,
    severity: Option<SatdSeverity>,
    critical_only: bool,
    include_tests: bool,
    evolution: bool,
    days: u32,
    metrics: bool,
    output: Option<PathBuf>,
) -> Result<()> {
    use crate::services::satd_detector::SATDDetector;
    eprintln!("🔍 Analyzing Self-Admitted Technical Debt (SATD)...");

    let detector = SATDDetector::new();
    let satd_items = analyze_satd_items(&detector, &path, include_tests).await?;
    let filtered_items = apply_satd_filters(satd_items, severity, critical_only);
    let output_content = generate_satd_output(format, &filtered_items, metrics, evolution, days);

    write_satd_output(output, &output_content).await?;

    if metrics {
        print_satd_metrics(&filtered_items);
    }

    Ok(())
}

/// Toyota Way: Extract Method - analyze SATD items (complexity ≤3)
async fn analyze_satd_items(
    detector: &crate::services::satd_detector::SATDDetector,
    path: &Path,
    include_tests: bool,
) -> Result<Vec<crate::services::satd_detector::TechnicalDebt>> {
    if include_tests {
        detector
            .analyze_directory_with_tests(path, true)
            .await
            .map_err(Into::into)
    } else {
        detector.analyze_directory(path).await.map_err(Into::into)
    }
}

/// Toyota Way: Extract Method - apply SATD filters (complexity ≤8)
pub fn apply_satd_filters(
    mut satd_items: Vec<crate::services::satd_detector::TechnicalDebt>,
    severity: Option<SatdSeverity>,
    critical_only: bool,
) -> Vec<crate::services::satd_detector::TechnicalDebt> {
    // Filter by severity if specified
    if let Some(min_severity) = severity {
        let min_sev = match min_severity {
            SatdSeverity::Critical => crate::services::satd_detector::Severity::Critical,
            SatdSeverity::High => crate::services::satd_detector::Severity::High,
            SatdSeverity::Medium => crate::services::satd_detector::Severity::Medium,
            SatdSeverity::Low => crate::services::satd_detector::Severity::Low,
        };
        // Severity enum: Low=0, Medium=1, High=2, Critical=3
        // Filter should keep items with severity >= min_sev (show critical and above)
        satd_items.retain(|item| item.severity as u8 >= min_sev as u8);
    }

    // Filter for critical items only if requested
    if critical_only {
        satd_items.retain(|item| {
            matches!(
                item.severity,
                crate::services::satd_detector::Severity::Critical
                    | crate::services::satd_detector::Severity::High
            )
        });
    }

    satd_items
}

/// Toyota Way: Strategy Pattern - generate output by format (complexity ≤4)
fn generate_satd_output(
    format: SatdOutputFormat,
    filtered_items: &[crate::services::satd_detector::TechnicalDebt],
    metrics: bool,
    evolution: bool,
    days: u32,
) -> String {
    match format {
        SatdOutputFormat::Summary => format_satd_summary(filtered_items),
        SatdOutputFormat::Json => format_satd_json(filtered_items, metrics, evolution),
        SatdOutputFormat::Sarif => format_satd_sarif(filtered_items),
        SatdOutputFormat::Markdown => format_satd_markdown(filtered_items, evolution, days),
    }
}

/// Toyota Way: Extract Method - handle output writing (complexity ≤3)
async fn write_satd_output(output: Option<PathBuf>, content: &str) -> Result<()> {
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, content).await?;
        eprintln!("✅ SATD analysis written to: {}", output_path.display());
    } else {
        println!("{content}");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_dag(
    dag_type: DagType,
    project_path: PathBuf,
    output: Option<PathBuf>,
    max_depth: Option<usize>,
    filter_external: bool,
    show_complexity: bool,
    include_duplicates: bool,
    include_dead_code: bool,
    enhanced: bool,
) -> Result<()> {
    eprintln!("🔍 Analyzing Directed Acyclic Graph (DAG)...");
    eprintln!("📊 DAG Type: {dag_type:?}");
    eprintln!("📁 Project: {}", project_path.display());

    // Simple DAG analysis implementation
    let mut output_content = String::new();
    output_content.push_str(&format!("# {dag_type:?} DAG Analysis\n\n"));
    output_content.push_str(&format!("Project: {}\n", project_path.display()));

    if let Some(depth) = max_depth {
        output_content.push_str(&format!("Max depth: {depth}\n"));
    }

    output_content.push_str(&format!("Filter external: {filter_external}\n"));
    output_content.push_str(&format!("Show complexity: {show_complexity}\n"));
    output_content.push_str(&format!("Include duplicates: {include_duplicates}\n"));
    output_content.push_str(&format!("Include dead code: {include_dead_code}\n"));
    output_content.push_str(&format!("Enhanced mode: {enhanced}\n"));

    output_content.push_str("\n## Analysis Results\n");
    output_content.push_str(
        "DAG analysis functionality will be implemented with proper AST-based analysis.\n",
    );

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &output_content).await?;
        eprintln!("✅ DAG analysis written to: {}", output_path.display());
    } else {
        println!("{output_content}");
    }

    Ok(())
}

/// Handles quality gate checks for a project or single file
///
/// This function runs quality checks and displays which checks are being run,
/// addressing issue #30 where quality-gate didn't show checks.
/// With the --perf flag (issue #31), it also shows performance metrics.
///
/// # Examples
///
/// ```no_run
/// use pmat::cli::analysis_utilities::handle_quality_gate;
/// use pmat::cli::{QualityCheckType, QualityGateOutputFormat};
/// use std::path::{Path, PathBuf};
///
/// # async fn example() -> anyhow::Result<()> {
/// // Run with default checks (All)
/// handle_quality_gate(
///     PathBuf::from("."),
///     None,
///     QualityGateOutputFormat::Human,
///     false,
///     vec![], // Empty means run all checks
///     15.0,
///     0.5,
///     20,
///     false,
///     None,
///     false, // perf = false
/// ).await?;
/// // Will display:
/// // 📋 Checks to run:
/// //   ✓ Complexity analysis
/// //   ✓ Dead code detection
/// //   ✓ Self-admitted technical debt (SATD)
/// //   ✓ Security vulnerabilities
/// //   ✓ Code entropy
/// //   ✓ Duplicate code
/// //   ✓ Test coverage
///
/// // Run with performance metrics
/// handle_quality_gate(
///     PathBuf::from("."),
///     None,
///     QualityGateOutputFormat::Human,
///     false,
///     vec![QualityCheckType::Complexity, QualityCheckType::Security],
///     15.0,
///     0.5,
///     20,
///     false,
///     None,
///     true, // perf = true
/// ).await?;
/// // Will display:
/// // 📋 Checks to run:
/// //   ✓ Complexity analysis
/// //   ✓ Security vulnerabilities
/// //   🔍 Checking complexity... 2 violations found (0.123s)
/// //   🔍 Checking security... 0 violations found (0.045s)
/// //
/// // ⏱️  Performance Metrics:
/// //   Total execution time: 0.17s
/// //   Checks performed: 2
/// //   Average time per check: 0.08s
/// # Ok(())
/// # }
/// ```ignore
#[allow(clippy::too_many_arguments)]
pub async fn handle_quality_gate(
    project_path: PathBuf,
    file: Option<PathBuf>,
    format: QualityGateOutputFormat,
    fail_on_violation: bool,
    checks: Vec<QualityCheckType>,
    max_dead_code: f64,
    min_entropy: f64,
    max_complexity_p99: u32,
    include_provability: bool,
    output: Option<PathBuf>,
    perf: bool,
) -> Result<()> {
    use std::time::Instant;

    let start_time = if perf { Some(Instant::now()) } else { None };

    // Suppress progress output for machine-readable formats (#230)
    let quiet = matches!(format, QualityGateOutputFormat::Json | QualityGateOutputFormat::Junit);

    // Print initial status message
    if !quiet {
        print_quality_gate_start_message(&file);
    }

    // Show which checks will be run
    let checks_to_run = if checks.is_empty() {
        vec![QualityCheckType::All]
    } else {
        checks.clone()
    };
    if !quiet {
        print_checks_to_run(&checks_to_run);
    }

    // Handle single file or project-wide quality gate
    let result = if let Some(single_file) = file {
        handle_single_file_quality_gate(
            project_path,
            single_file,
            format,
            fail_on_violation,
            checks_to_run.clone(), // Use checks_to_run instead of checks
            max_complexity_p99,
            output,
            perf,
            quiet,
        )
        .await
    } else {
        handle_project_quality_gate(
            project_path,
            format,
            fail_on_violation,
            checks_to_run.clone(), // Use checks_to_run instead of checks
            max_dead_code,
            min_entropy,
            max_complexity_p99,
            include_provability,
            output,
            perf,
            quiet,
        )
        .await
    };

    // Show performance metrics if requested (suppress in quiet/JSON mode)
    if !quiet {
        if let Some(start) = start_time {
            let duration = start.elapsed();
            eprintln!("\n⏱️  Performance Metrics:");
            eprintln!("  Total execution time: {:.2}s", duration.as_secs_f64());
            eprintln!("  Checks performed: {}", checks_to_run.len());
            eprintln!(
                "  Average time per check: {:.2}s",
                duration.as_secs_f64() / checks_to_run.len() as f64
            );
        }
    }

    result
}

/// Prints the initial quality gate status message
fn print_quality_gate_start_message(file: &Option<PathBuf>) {
    if let Some(single_file) = file {
        eprintln!(
            "🔍 Running quality gate checks on file: {}...",
            single_file.display()
        );
    } else {
        eprintln!("🔍 Running quality gate checks...");
    }
}

/// Prints which checks will be run
/// Toyota Way: Extract Method - Print checks to run (complexity ≤8)
fn print_checks_to_run(checks: &[QualityCheckType]) {
    eprintln!("\n📋 Checks to run:");

    if checks.contains(&QualityCheckType::All) {
        print_all_checks();
    } else {
        print_selected_checks(checks);
    }
    eprintln!();
}

/// Toyota Way: Extract Method - Print all quality checks (complexity ≤5)
fn print_all_checks() {
    eprintln!("  ✓ Complexity analysis");
    eprintln!("  ✓ Dead code detection");
    eprintln!("  ✓ Self-admitted technical debt (SATD)");
    eprintln!("  ✓ Security vulnerabilities");
    eprintln!("  ✓ Code entropy");
    eprintln!("  ✓ Duplicate code");
    eprintln!("  ✓ Test coverage");
}

/// Toyota Way: Extract Method - Print selected checks (complexity ≤8)
fn print_selected_checks(checks: &[QualityCheckType]) {
    for check in checks {
        print_single_check(check);
    }
}

/// Toyota Way: Extract Method - Print single check description (complexity ≤7)
fn print_single_check(check: &QualityCheckType) {
    if let Some(message) = get_check_message(check) {
        print_check_success(message);
    }
}

/// Get the success message for a specific quality check type
fn get_check_message(check: &QualityCheckType) -> Option<&'static str> {
    match check {
        QualityCheckType::Complexity => Some("Complexity analysis"),
        QualityCheckType::DeadCode => Some("Dead code detection"),
        QualityCheckType::Satd => Some("Self-admitted technical debt (SATD)"),
        QualityCheckType::Security => Some("Security vulnerabilities"),
        QualityCheckType::Entropy => Some("Code entropy"),
        QualityCheckType::Duplicates => Some("Duplicate code"),
        QualityCheckType::Coverage => Some("Test coverage"),
        _ => None,
    }
}

/// Print a check success message with consistent formatting
fn print_check_success(message: &str) {
    eprintln!("  ✓ {message}");
}

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
    let check_name = get_check_display_name(check);
    eprintln!("    ⏱️  {check_name} check: {elapsed_secs:.3}s");
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

/// Runs a single project-wide check
#[allow(clippy::too_many_arguments)]
/// Toyota Way: Data-Driven Design - eliminated 41→≤8 complexity
pub async fn run_single_project_check(
    check: &QualityCheckType,
    project_path: &Path,
    max_dead_code: f64,
    min_entropy: f64,
    max_complexity_p99: u32,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
    perf: bool,
    quiet: bool,
) -> Result<()> {
    match check {
        QualityCheckType::All => {
            run_all_project_checks(
                project_path,
                max_dead_code,
                min_entropy,
                max_complexity_p99,
                violations,
                results,
                perf,
                quiet,
            )
            .await
        }
        _ => {
            execute_specific_quality_check(
                check,
                project_path,
                max_dead_code,
                min_entropy,
                max_complexity_p99,
                violations,
                results,
            )
            .await
        }
    }
}

/// Toyota Way: Extract Method - handle specific quality checks (complexity ≤5)
/// Toyota Way: Template Method pattern - reduced complexity from 23→≤3
async fn execute_specific_quality_check(
    check: &QualityCheckType,
    project_path: &Path,
    max_dead_code: f64,
    min_entropy: f64,
    max_complexity_p99: u32,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    use QualityCheckType::{
        All, Complexity, Coverage, DeadCode, Duplicates, Entropy, Provability, Satd, Sections,
        Security,
    };

    match check {
        Complexity => {
            execute_complexity_check(project_path, max_complexity_p99, violations, results).await
        }
        DeadCode => execute_dead_code_check(project_path, max_dead_code, violations, results).await,
        Satd => execute_satd_check(project_path, violations, results).await,
        Entropy => execute_entropy_check(project_path, min_entropy, violations, results).await,
        Security => execute_security_check(project_path, violations, results).await,
        Duplicates => execute_duplicates_check(project_path, violations, results).await,
        Coverage => execute_coverage_check(project_path, violations, results).await,
        Sections => execute_sections_check(project_path, violations, results).await,
        Provability => execute_provability_check(project_path, violations, results).await,
        All => unreachable!("All case handled in parent function"),
    }
}

/// Toyota Way: Template Method - extracts common quality check pattern
async fn execute_quality_check_template<Fut, S>(
    check_future: Fut,
    set_result: S,
    violations: &mut Vec<QualityViolation>,
) -> Result<()>
where
    Fut: std::future::Future<Output = Result<Vec<QualityViolation>>>,
    S: FnOnce(usize),
{
    let violations_found = check_future.await?;
    set_result(violations_found.len());
    violations.extend(violations_found);
    Ok(())
}

/// Helper for complexity check execution
async fn execute_complexity_check(
    project_path: &Path,
    max_complexity_p99: u32,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    execute_quality_check_template(
        check_complexity(project_path, max_complexity_p99),
        |count| results.complexity_violations = count,
        violations,
    )
    .await
}

/// Helper for dead code check execution
async fn execute_dead_code_check(
    project_path: &Path,
    max_dead_code: f64,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    execute_quality_check_template(
        check_dead_code(project_path, max_dead_code),
        |count| results.dead_code_violations = count,
        violations,
    )
    .await
}

/// Helper for SATD check execution
async fn execute_satd_check(
    project_path: &Path,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    execute_quality_check_template(
        check_satd(project_path),
        |count| results.satd_violations = count,
        violations,
    )
    .await
}

/// Helper for entropy check execution (loads config from .pmat-gates.toml, #220)
async fn execute_entropy_check(
    project_path: &Path,
    min_entropy: f64,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    let gate_config = load_entropy_gate_config(project_path);
    if !gate_config.enabled {
        eprintln!("  ⏭️  Entropy check disabled via .pmat-gates.toml");
        return Ok(());
    }
    let threshold = load_entropy_threshold(project_path, min_entropy);
    let mut exclude_paths = load_entropy_exclude_paths(project_path);
    // Merge per-check excludes from [entropy] section (#220)
    for pattern in &gate_config.exclude {
        if !exclude_paths.contains(pattern) {
            exclude_paths.push(pattern.clone());
        }
    }
    let mut entropy_violations =
        check_entropy_with_excludes(project_path, threshold, &exclude_paths).await?;
    // Apply max_violations threshold (#220)
    if let Some(max) = gate_config.max_violations {
        if entropy_violations.len() <= max {
            entropy_violations.clear();
        }
    }
    results.entropy_violations = entropy_violations.len();
    violations.extend(entropy_violations);
    Ok(())
}

/// Helper for security check execution
async fn execute_security_check(
    project_path: &Path,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    execute_quality_check_template(
        check_security(project_path),
        |count| results.security_violations = count,
        violations,
    )
    .await
}

/// Helper for duplicates check execution
async fn execute_duplicates_check(
    project_path: &Path,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    execute_quality_check_template(
        check_duplicates(project_path),
        |count| results.duplicate_violations = count,
        violations,
    )
    .await
}

/// Helper for coverage check execution
async fn execute_coverage_check(
    project_path: &Path,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    execute_quality_check_template(
        check_coverage(project_path, 80.0),
        |count| results.coverage_violations = count,
        violations,
    )
    .await
}

/// Helper for sections check execution
async fn execute_sections_check(
    project_path: &Path,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    execute_quality_check_template(
        check_sections(project_path),
        |count| results.section_violations = count,
        violations,
    )
    .await
}

/// Default provability threshold when no config is found
const DEFAULT_PROVABILITY_THRESHOLD: f64 = 0.70;

/// Load the provability threshold from `.pmat-metrics.toml`.
///
/// Looks for `provability_min` under the `[thresholds]` section.
/// Falls back to `DEFAULT_PROVABILITY_THRESHOLD` (0.70) if the file
/// is missing, unreadable, or does not contain the key.
fn load_provability_threshold(project_path: &Path) -> f64 {
    let config_path = project_path.join(".pmat-metrics.toml");
    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return DEFAULT_PROVABILITY_THRESHOLD,
    };
    let table: toml::Table = match content.parse() {
        Ok(t) => t,
        Err(_) => return DEFAULT_PROVABILITY_THRESHOLD,
    };
    table
        .get("thresholds")
        .and_then(|t| t.get("provability_min"))
        .and_then(|v| v.as_float())
        .unwrap_or(DEFAULT_PROVABILITY_THRESHOLD)
}

/// Load entropy min_pattern_diversity from config files (#194, #219, #227).
///
/// Priority: `.pmat-gates.toml` > `.pmat-metrics.toml` > `pmat.toml` > CLI default.
/// Reads from `[entropy] min_pattern_diversity`, `[thresholds] entropy_min_diversity`,
/// or `[quality] min_pattern_diversity`.
/// Clamps result to 0.0-1.0 range to prevent unreachable thresholds.
fn load_entropy_threshold(project_path: &Path, cli_value: f64) -> f64 {
    let mut result = cli_value;

    // Load from pmat.toml [quality] (lowest config priority, #227)
    if let Some(val) = read_entropy_threshold_from_pmat_toml(project_path) {
        result = val;
    }

    // Load from .pmat-metrics.toml (medium priority)
    if let Some(val) = read_entropy_threshold_from_file(
        &project_path.join(".pmat-metrics.toml"),
    ) {
        result = val;
    }

    // Load from .pmat-gates.toml (highest priority, #219)
    if let Some(val) = read_entropy_threshold_from_file(
        &project_path.join(".pmat-gates.toml"),
    ) {
        result = val;
    }

    // Clamp to valid range (#219: prevent 200% unreachable thresholds)
    result.clamp(0.0, 1.0)
}

/// Read entropy threshold from `pmat.toml [quality] min_pattern_diversity` (#227).
fn read_entropy_threshold_from_pmat_toml(project_path: &Path) -> Option<f64> {
    let content = std::fs::read_to_string(project_path.join("pmat.toml")).ok()?;
    let table: toml::Table = content.parse().ok()?;
    table
        .get("quality")
        .and_then(|t| t.get("min_pattern_diversity"))
        .and_then(|v| v.as_float())
}

/// Read entropy threshold from a single TOML file.
/// Checks `[entropy] min_pattern_diversity` and `[thresholds] entropy_min_diversity`.
fn read_entropy_threshold_from_file(path: &Path) -> Option<f64> {
    let content = std::fs::read_to_string(path).ok()?;
    let table: toml::Table = content.parse().ok()?;

    // Check [entropy] min_pattern_diversity first (preferred key)
    if let Some(val) = table
        .get("entropy")
        .and_then(|t| t.get("min_pattern_diversity"))
        .and_then(|v| v.as_float())
    {
        return Some(val);
    }

    // Fallback: [thresholds] entropy_min_diversity (legacy key)
    table
        .get("thresholds")
        .and_then(|t| t.get("entropy_min_diversity"))
        .and_then(|v| v.as_float())
}

/// Entropy gate configuration loaded from `.pmat-gates.toml` (#220).
struct EntropyGateConfig {
    enabled: bool,
    max_violations: Option<usize>,
    exclude: Vec<String>,
}

/// Load entropy gate configuration from `.pmat-gates.toml`, with `pmat.toml` fallback (#220, #227).
///
/// Priority: `.pmat-gates.toml [entropy]` > `pmat.toml [quality]` > defaults.
/// Reads `enabled`, `max_violations`, `exclude` from `[entropy]` section.
fn load_entropy_gate_config(project_path: &Path) -> EntropyGateConfig {
    // Start with pmat.toml [quality] max_entropy_violations as lowest priority (#227)
    let mut max_violations_fallback: Option<usize> = None;
    if let Ok(content) = std::fs::read_to_string(project_path.join("pmat.toml")) {
        if let Ok(table) = content.parse::<toml::Table>() {
            max_violations_fallback = table
                .get("quality")
                .and_then(|t| t.get("max_entropy_violations"))
                .and_then(|v| v.as_integer())
                .map(|v| v.max(0) as usize);
        }
    }

    let path = project_path.join(".pmat-gates.toml");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => {
            return EntropyGateConfig {
                enabled: true,
                max_violations: max_violations_fallback,
                exclude: Vec::new(),
            }
        }
    };
    let table: toml::Table = match content.parse() {
        Ok(t) => t,
        Err(_) => {
            return EntropyGateConfig {
                enabled: true,
                max_violations: max_violations_fallback,
                exclude: Vec::new(),
            }
        }
    };

    let entropy = table.get("entropy");

    let enabled = entropy
        .and_then(|t| t.get("enabled"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // .pmat-gates.toml overrides pmat.toml if present
    let max_violations = entropy
        .and_then(|t| t.get("max_violations"))
        .and_then(|v| v.as_integer())
        .map(|v| v.max(0) as usize)
        .or(max_violations_fallback);

    let exclude = entropy
        .and_then(|t| t.get("exclude"))
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    EntropyGateConfig {
        enabled,
        max_violations,
        exclude,
    }
}

/// Extract exclude paths from a parsed TOML table.
///
/// Checks multiple patterns:
/// - `[exclude] paths = [...]`
/// - `exclude_paths = [...]`
/// - `[quality-gates] exclude = [...]`
fn extract_excludes_from_table(table: &toml::Table) -> Vec<String> {
    let arr = table
        .get("exclude")
        .and_then(|t| t.get("paths"))
        .and_then(|v| v.as_array())
        .or_else(|| table.get("exclude_paths").and_then(|v| v.as_array()))
        .or_else(|| {
            table
                .get("quality-gates")
                .and_then(|t| t.get("exclude"))
                .and_then(|v| v.as_array())
        });
    arr.map(|a| {
        a.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()
    })
    .unwrap_or_default()
}

/// Load exclude paths from `.pmat-metrics.toml` and `.pmat-gates.toml` (#195, #217).
///
/// Checks both config files and merges exclude patterns.
/// Returns an empty vec if neither file exists or no exclude config exists.
fn load_entropy_exclude_paths(project_path: &Path) -> Vec<String> {
    let mut excludes = Vec::new();

    // Load from .pmat-metrics.toml
    if let Ok(content) = std::fs::read_to_string(project_path.join(".pmat-metrics.toml")) {
        if let Ok(table) = content.parse::<toml::Table>() {
            excludes.extend(extract_excludes_from_table(&table));
        }
    }

    // Load from .pmat-gates.toml (#217)
    if let Ok(content) = std::fs::read_to_string(project_path.join(".pmat-gates.toml")) {
        if let Ok(table) = content.parse::<toml::Table>() {
            for pattern in extract_excludes_from_table(&table) {
                if !excludes.contains(&pattern) {
                    excludes.push(pattern);
                }
            }
        }
    }

    excludes
}

/// Filter violations whose file path matches any exclude path (#196).
///
/// Matches both exact prefix and glob patterns. Violations with `file = "project"`
/// or other non-path values are kept (project-level metrics).
fn filter_violations_by_exclude(violations: &mut Vec<QualityViolation>, exclude_paths: &[String]) {
    violations.retain(|v| {
        // Keep project-level violations (no file path)
        if v.file == "project" || v.file.is_empty() {
            return true;
        }
        // Check if the violation's file matches any exclude path
        !exclude_paths.iter().any(|excl| {
            let normalized = excl.trim_end_matches('/');
            v.file.starts_with(normalized)
                || v.file.starts_with(&format!("{normalized}/"))
                || v.file.starts_with(&format!("./{normalized}"))
                || glob::Pattern::new(excl).is_ok_and(|p| p.matches(&v.file))
        })
    });
}

