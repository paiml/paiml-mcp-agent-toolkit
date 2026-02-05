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

    // Print initial status message
    print_quality_gate_start_message(&file);

    // Show which checks will be run
    let checks_to_run = if checks.is_empty() {
        vec![QualityCheckType::All]
    } else {
        checks.clone()
    };
    print_checks_to_run(&checks_to_run);

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
        )
        .await
    };

    // Show performance metrics if requested
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
) -> Result<()> {
    use std::time::Instant;
    eprintln!("📄 Analyzing single file: {}", single_file.display());

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

    if let Some(start) = check_start {
        let duration = start.elapsed();
        eprintln!("\n⏱️  File analysis took: {:.3}s", duration.as_secs_f64());
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

    // Add provability if requested
    if include_provability {
        let prov_start = if perf { Some(Instant::now()) } else { None };
        let provability_score = calculate_provability_score(&project_path).await?;
        results.provability_score = Some(provability_score);

        if let Some(start) = prov_start {
            eprintln!(
                "  ⏱️  Provability analysis: {:.3}s",
                start.elapsed().as_secs_f64()
            );
        }
    }

    if let Some(start) = checks_start {
        let duration = start.elapsed();
        eprintln!(
            "\n⏱️  All checks completed in: {:.3}s",
            duration.as_secs_f64()
        );
    }

    // Calculate overall pass/fail
    results.passed = violations.is_empty();
    results.total_violations = violations.len();

    // Format and output results
    output_project_results(&results, &violations, format, output).await?;

    // Print final status
    print_quality_gate_final_status(&results, &violations);

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
    min_entropy: f64,
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

/// Helper for entropy check execution
async fn execute_entropy_check(
    project_path: &Path,
    min_entropy: f64,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    execute_quality_check_template(
        check_entropy(project_path, min_entropy),
        |count| results.entropy_violations = count,
        violations,
    )
    .await
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

/// Helper for provability check execution
async fn execute_provability_check(
    project_path: &Path,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    execute_quality_check_template(
        check_provability(project_path, 0.7),
        |count| results.provability_violations = count,
        violations,
    )
    .await
}

/// Runs all project-wide checks
async fn run_all_project_checks(
    project_path: &Path,
    max_dead_code: f64,
    min_entropy: f64,
    max_complexity_p99: u32,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
    perf: bool,
) -> Result<()> {
    use std::time::Instant;

    // Run all checks
    eprint!("  🔍 Checking complexity...");
    let start = if perf { Some(Instant::now()) } else { None };
    let complexity_violations = check_complexity(project_path, max_complexity_p99).await?;
    results.complexity_violations = complexity_violations.len();
    violations.extend(complexity_violations);
    if let Some(s) = start {
        eprintln!(
            " {} violations found ({:.3}s)",
            results.complexity_violations,
            s.elapsed().as_secs_f64()
        );
    } else {
        eprintln!(" {} violations found", results.complexity_violations);
    }

    // Macro to handle timing for each check
    macro_rules! run_check {
        ($name:expr, $check_expr:expr, $result_field:ident) => {{
            eprint!("  🔍 Checking {}...", $name);
            let start = if perf { Some(Instant::now()) } else { None };
            let check_violations = $check_expr.await?;
            results.$result_field = check_violations.len();
            violations.extend(check_violations);
            if let Some(s) = start {
                eprintln!(
                    " {} violations found ({:.3}s)",
                    results.$result_field,
                    s.elapsed().as_secs_f64()
                );
            } else {
                eprintln!(" {} violations found", results.$result_field);
            }
        }};
    }

    run_check!(
        "dead code",
        check_dead_code(project_path, max_dead_code),
        dead_code_violations
    );
    run_check!("technical debt", check_satd(project_path), satd_violations);
    run_check!(
        "code entropy",
        check_entropy(project_path, min_entropy),
        entropy_violations
    );
    run_check!(
        "security",
        check_security(project_path),
        security_violations
    );
    run_check!(
        "duplicates",
        check_duplicates(project_path),
        duplicate_violations
    );
    run_check!(
        "test coverage",
        check_coverage(project_path, 80.0),
        coverage_violations
    );
    run_check!(
        "documentation sections",
        check_sections(project_path),
        section_violations
    );
    run_check!(
        "provability",
        check_provability(project_path, 0.7),
        provability_violations
    );

    Ok(())
}

/// Formats and outputs project results
async fn output_project_results(
    results: &QualityGateResults,
    violations: &[QualityViolation],
    format: QualityGateOutputFormat,
    output: Option<PathBuf>,
) -> Result<()> {
    let content = format_quality_gate_output(results, violations, format)?;

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!(
            "✅ Quality gate report written to: {}",
            output_path.display()
        );
    } else {
        println!("{content}");
    }

    Ok(())
}

/// Prints the final quality gate status
fn print_quality_gate_final_status(results: &QualityGateResults, violations: &[QualityViolation]) {
    if results.passed {
        eprintln!("\n✅ Quality gate PASSED");
    } else {
        eprintln!("\n⚠️ Quality gate found {} violations", violations.len());
    }
}

/// Handles the exit status based on quality gate results
fn handle_quality_gate_exit_status(fail_on_violation: bool, passed: bool) {
    if fail_on_violation && !passed {
        eprintln!("\n❌ Quality gate FAILED");
        std::process::exit(1);
    }
}


// Single file quality check functions - extracted for file health (CB-040)
async fn check_single_file_complexity(
    project_path: &Path,
    file_path: &Path,
    max_complexity_p99: u32,
) -> Result<Vec<QualityViolation>> {
    let abs_file_path = resolve_absolute_file_path(project_path, file_path);
    validate_file_exists(&abs_file_path)?;

    let mut violations = Vec::new();
    analyze_file_complexity(
        &abs_file_path,
        file_path,
        max_complexity_p99,
        &mut violations,
    )
    .await?;

    Ok(violations)
}

/// Resolve file path to absolute path
fn resolve_absolute_file_path(project_path: &Path, file_path: &Path) -> PathBuf {
    if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        project_path.join(file_path)
    }
}

/// Validate that file exists
fn validate_file_exists(abs_file_path: &Path) -> Result<()> {
    if !abs_file_path.exists() {
        return Err(anyhow::anyhow!(
            "File not found: {}",
            abs_file_path.display()
        ));
    }
    Ok(())
}

/// Analyze file complexity based on file extension
async fn analyze_file_complexity(
    abs_file_path: &Path,
    original_path: &Path,
    max_complexity: u32,
    violations: &mut Vec<QualityViolation>,
) -> Result<()> {
    if let Some(ext) = abs_file_path.extension() {
        if ext == "rs" {
            analyze_rust_file_complexity(abs_file_path, original_path, max_complexity, violations)
                .await?;
        }
        // Add support for other languages as needed
    }
    Ok(())
}

/// Analyze Rust file complexity and generate violations
async fn analyze_rust_file_complexity(
    abs_file_path: &Path,
    original_path: &Path,
    max_complexity: u32,
    violations: &mut Vec<QualityViolation>,
) -> Result<()> {
    use crate::services::ast_rust::analyze_rust_file_with_complexity;

    let metrics = analyze_rust_file_with_complexity(abs_file_path).await?;

    for func in &metrics.functions {
        if function_exceeds_complexity_threshold(func, max_complexity) {
            violations.push(create_complexity_violation(
                func,
                original_path,
                max_complexity,
            ));
        }
    }

    Ok(())
}

/// Check if function exceeds complexity threshold
fn function_exceeds_complexity_threshold(
    func: &crate::services::complexity::FunctionComplexity,
    max_complexity: u32,
) -> bool {
    func.metrics.cyclomatic > max_complexity as u16
}

/// Create complexity violation for a function
fn create_complexity_violation(
    func: &crate::services::complexity::FunctionComplexity,
    file_path: &Path,
    max_complexity: u32,
) -> QualityViolation {
    QualityViolation {
        check_type: "complexity".to_string(),
        severity: "error".to_string(),
        file: file_path.to_string_lossy().to_string(),
        line: Some(func.line_start as usize),
        message: format!(
            "Function '{}' has cyclomatic complexity {} (max: {})",
            func.name, func.metrics.cyclomatic, max_complexity
        ),
    }
}

async fn check_single_file_dead_code(
    project_path: &Path,
    file_path: &Path,
) -> Result<Vec<QualityViolation>> {
    use regex::Regex;

    let mut violations = Vec::new();

    // Make file path absolute
    let abs_file_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        project_path.join(file_path)
    };

    if !abs_file_path.exists() {
        return Ok(violations); // No violations if file doesn't exist
    }

    // Read file content
    let content = tokio::fs::read_to_string(&abs_file_path).await?;

    // Check for common dead code patterns
    let dead_code_patterns = vec![
        (r"#\[allow\(dead_code\)\]", "Dead code attribute found"),
        (r"^\s*//\s*fn\s+\w+", "Commented out function"),
        (r"^\s*//\s*struct\s+\w+", "Commented out struct"),
        (r"^\s*//\s*impl\s+", "Commented out implementation"),
    ];

    for (pattern_str, message) in dead_code_patterns {
        let regex = Regex::new(pattern_str)?;
        for (line_no, line) in content.lines().enumerate() {
            if regex.is_match(line) {
                violations.push(QualityViolation {
                    check_type: "dead_code".to_string(),
                    severity: "warning".to_string(),
                    file: file_path.to_string_lossy().to_string(),
                    line: Some(line_no + 1),
                    message: message.to_string(),
                });
            }
        }
    }

    Ok(violations)
}

async fn check_single_file_satd(
    project_path: &Path,
    file_path: &Path,
) -> Result<Vec<QualityViolation>> {
    use regex::Regex;

    let mut violations = Vec::new();
    let satd_pattern = Regex::new(r"(?i)\b(TODO|FIXME|HACK|XXX|BUG|REFACTOR):\s*(.+)")?;

    // Make file path absolute
    let abs_file_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        project_path.join(file_path)
    };

    if !abs_file_path.exists() {
        return Ok(violations);
    }

    let content = tokio::fs::read_to_string(&abs_file_path).await?;

    for (line_no, line) in content.lines().enumerate() {
        if let Some(captures) = satd_pattern.captures(line) {
            let satd_type = captures
                .get(1)
                .expect("Match group 1 exists for successful regex match")
                .as_str();
            let text = captures
                .get(2)
                .expect("Match group 2 exists for successful regex match")
                .as_str();

            violations.push(QualityViolation {
                check_type: "satd".to_string(),
                severity: "warning".to_string(),
                file: file_path.to_string_lossy().to_string(),
                line: Some(line_no + 1),
                message: format!("Self-admitted technical debt: {satd_type} - {text}"),
            });
        }
    }

    Ok(violations)
}

async fn check_single_file_security(
    project_path: &Path,
    file_path: &Path,
) -> Result<Vec<QualityViolation>> {
    use regex::Regex;

    let mut violations = Vec::new();

    // Security patterns to check
    let security_patterns = vec![
        (
            r#"(?i)password\s*=\s*["'][^"']+["']"#,
            "Hardcoded password detected",
        ),
        (
            r#"(?i)api_key\s*=\s*["'][^"']+["']"#,
            "Hardcoded API key detected",
        ),
        (
            r#"(?i)secret\s*=\s*["'][^"']+["']"#,
            "Hardcoded secret detected",
        ),
        (
            r#"(?i)token\s*=\s*["'][^"']+["']"#,
            "Hardcoded token detected",
        ),
        (r"(?i)unsafe\s*\{", "Unsafe code block detected"),
        (
            r"std::env::var\(.*\)\.unwrap\(\)",
            "Unsafe environment variable access",
        ),
    ];

    // Make file path absolute
    let abs_file_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        project_path.join(file_path)
    };

    if !abs_file_path.exists() {
        return Ok(violations);
    }

    let content = tokio::fs::read_to_string(&abs_file_path).await?;

    for (pattern_str, message) in security_patterns {
        let regex = Regex::new(pattern_str)?;
        for (line_no, line) in content.lines().enumerate() {
            if regex.is_match(line) {
                violations.push(QualityViolation {
                    check_type: "security".to_string(),
                    severity: "error".to_string(),
                    file: file_path.to_string_lossy().to_string(),
                    line: Some(line_no + 1),
                    message: message.to_string(),
                });
            }
        }
    }

    Ok(violations)
}

fn format_single_file_summary(
    file_path: &Path,
    results: &QualityGateResults,
    violations: &[QualityViolation],
) -> String {
    let mut output = String::new();

    format_report_header(&mut output, file_path, results.passed);
    format_results_summary(&mut output, results);

    if !violations.is_empty() {
        format_violations_section(&mut output, violations);
    }

    output
}

/// Format the report header with title and pass/fail status
fn format_report_header(output: &mut String, file_path: &Path, passed: bool) {
    output.push_str(&format!(
        "# Quality Gate Report: {}\n\n",
        file_path.display()
    ));

    if passed {
        output.push_str("✅ **Quality Gate: PASSED**\n\n");
    } else {
        output.push_str("❌ **Quality Gate: FAILED**\n\n");
    }
}

/// Format the summary section with violation counts
fn format_results_summary(output: &mut String, results: &QualityGateResults) {
    output.push_str("## Summary\n\n");
    output.push_str(&format!(
        "- Total Violations: {}\n",
        results.total_violations
    ));
    output.push_str(&format!(
        "- Complexity Issues: {}\n",
        results.complexity_violations
    ));
    output.push_str(&format!("- Dead Code: {}\n", results.dead_code_violations));
    output.push_str(&format!(
        "- Technical Debt (SATD): {}\n",
        results.satd_violations
    ));
    output.push_str(&format!(
        "- Security Issues: {}\n",
        results.security_violations
    ));
}

/// Format the violations section grouped by type
fn format_violations_section(output: &mut String, violations: &[QualityViolation]) {
    use std::collections::HashMap;

    output.push_str("\n## Violations\n\n");

    // Group violations by type
    let mut by_type: HashMap<String, Vec<&QualityViolation>> = HashMap::new();
    for violation in violations {
        by_type
            .entry(violation.check_type.clone())
            .or_default()
            .push(violation);
    }

    for (check_type, type_violations) in by_type {
        format_violation_type_group(output, &check_type, &type_violations);
    }
}

/// Format a single violation type group
fn format_violation_type_group(
    output: &mut String,
    check_type: &str,
    violations: &[&QualityViolation],
) {
    output.push_str(&format!(
        "### {} ({})\n\n",
        check_type.to_uppercase(),
        violations.len()
    ));

    for violation in violations {
        format_single_violation(output, violation);
    }
    output.push('\n');
}

/// Format a single violation with severity icon, file path, and location
fn format_single_violation(output: &mut String, violation: &QualityViolation) {
    let severity_icon = get_severity_icon(&violation.severity);

    // Format file path - use short relative path if possible
    let file_display = if violation.file.is_empty() {
        String::new()
    } else {
        // Extract just the filename or short path for display
        let path = std::path::Path::new(&violation.file);
        let short_path = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| violation.file.clone());
        format!(" {}", short_path)
    };

    if let Some(line) = violation.line {
        output.push_str(&format!(
            "- {}{}:{}: {}\n",
            severity_icon, file_display, line, violation.message
        ));
    } else if !violation.file.is_empty() {
        output.push_str(&format!(
            "- {}{}: {}\n",
            severity_icon, file_display, violation.message
        ));
    } else {
        output.push_str(&format!("- {} {}\n", severity_icon, violation.message));
    }
}

/// Get the appropriate icon for violation severity
pub fn get_severity_icon(severity: &str) -> &'static str {
    match severity {
        "error" => "🔴",
        "warning" => "🟡",
        _ => "🟢",
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod quality_gate_unit_tests {
    use super::*;

    // ===================
    // get_severity_icon Tests
    // ===================

    #[test]
    fn test_get_severity_icon_error() {
        assert_eq!(get_severity_icon("error"), "🔴");
    }

    #[test]
    fn test_get_severity_icon_warning() {
        assert_eq!(get_severity_icon("warning"), "🟡");
    }

    #[test]
    fn test_get_severity_icon_other() {
        assert_eq!(get_severity_icon("info"), "🟢");
        assert_eq!(get_severity_icon("note"), "🟢");
        assert_eq!(get_severity_icon("suggestion"), "🟢");
        assert_eq!(get_severity_icon(""), "🟢");
    }

    // ===================
    // get_check_message Tests
    // ===================

    #[test]
    fn test_get_check_message_complexity() {
        let result = get_check_message(&QualityCheckType::Complexity);
        assert_eq!(result, Some("Complexity analysis"));
    }

    #[test]
    fn test_get_check_message_dead_code() {
        let result = get_check_message(&QualityCheckType::DeadCode);
        assert_eq!(result, Some("Dead code detection"));
    }

    #[test]
    fn test_get_check_message_satd() {
        let result = get_check_message(&QualityCheckType::Satd);
        assert_eq!(result, Some("Self-admitted technical debt (SATD)"));
    }

    #[test]
    fn test_get_check_message_security() {
        let result = get_check_message(&QualityCheckType::Security);
        assert_eq!(result, Some("Security vulnerabilities"));
    }

    #[test]
    fn test_get_check_message_entropy() {
        let result = get_check_message(&QualityCheckType::Entropy);
        assert_eq!(result, Some("Code entropy"));
    }

    #[test]
    fn test_get_check_message_duplicates() {
        let result = get_check_message(&QualityCheckType::Duplicates);
        assert_eq!(result, Some("Duplicate code"));
    }

    #[test]
    fn test_get_check_message_coverage() {
        let result = get_check_message(&QualityCheckType::Coverage);
        assert_eq!(result, Some("Test coverage"));
    }

    #[test]
    fn test_get_check_message_all() {
        let result = get_check_message(&QualityCheckType::All);
        assert!(result.is_none());
    }

    // ===================
    // format_report_header Tests
    // ===================

    #[test]
    fn test_format_report_header_passed() {
        let mut output = String::new();
        format_report_header(&mut output, Path::new("src/test.rs"), true);
        assert!(output.contains("Quality Gate Report: src/test.rs"));
        assert!(output.contains("PASSED"));
        assert!(output.contains("✅"));
    }

    #[test]
    fn test_format_report_header_failed() {
        let mut output = String::new();
        format_report_header(&mut output, Path::new("src/main.rs"), false);
        assert!(output.contains("Quality Gate Report: src/main.rs"));
        assert!(output.contains("FAILED"));
        assert!(output.contains("❌"));
    }

    // ===================
    // format_results_summary Tests
    // ===================

    #[test]
    fn test_format_results_summary_zeros() {
        let results = QualityGateResults::default();

        let mut output = String::new();
        format_results_summary(&mut output, &results);

        assert!(output.contains("## Summary"));
        assert!(output.contains("Total Violations: 0"));
        assert!(output.contains("Complexity Issues: 0"));
        assert!(output.contains("Dead Code: 0"));
        assert!(output.contains("Technical Debt (SATD): 0"));
        assert!(output.contains("Security Issues: 0"));
    }

    #[test]
    fn test_format_results_summary_with_violations() {
        let mut results = QualityGateResults::default();
        results.passed = false;
        results.total_violations = 10;
        results.complexity_violations = 3;
        results.dead_code_violations = 2;
        results.satd_violations = 4;
        results.security_violations = 1;

        let mut output = String::new();
        format_results_summary(&mut output, &results);

        assert!(output.contains("Total Violations: 10"));
        assert!(output.contains("Complexity Issues: 3"));
        assert!(output.contains("Dead Code: 2"));
        assert!(output.contains("Technical Debt (SATD): 4"));
        assert!(output.contains("Security Issues: 1"));
    }

    // ===================
    // QualityViolation Tests
    // ===================

    #[test]
    fn test_quality_violation_struct() {
        let violation = QualityViolation {
            check_type: "complexity".to_string(),
            severity: "error".to_string(),
            file: "src/main.rs".to_string(),
            line: Some(42),
            message: "Function too complex".to_string(),
        };

        assert_eq!(violation.check_type, "complexity");
        assert_eq!(violation.severity, "error");
        assert_eq!(violation.file, "src/main.rs");
        assert_eq!(violation.line, Some(42));
    }

    #[test]
    fn test_quality_violation_no_line() {
        let violation = QualityViolation {
            check_type: "dead_code".to_string(),
            severity: "warning".to_string(),
            file: "src/lib.rs".to_string(),
            line: None,
            message: "Unused function".to_string(),
        };

        assert!(violation.line.is_none());
    }

    // ===================
    // QualityGateResults Tests
    // ===================

    #[test]
    fn test_quality_gate_results_default() {
        let results = QualityGateResults::default();
        // Default is passed: true when no violations
        assert!(results.passed);
        assert_eq!(results.total_violations, 0);
        assert_eq!(results.complexity_violations, 0);
        assert_eq!(results.dead_code_violations, 0);
        assert_eq!(results.satd_violations, 0);
        assert_eq!(results.security_violations, 0);
        assert!(results.violations.is_empty());
    }

    #[test]
    fn test_quality_gate_results_with_values() {
        let mut results = QualityGateResults::default();
        results.passed = true;
        results.total_violations = 5;
        results.complexity_violations = 2;
        results.dead_code_violations = 1;
        results.satd_violations = 1;
        results.security_violations = 1;

        assert!(results.passed);
        assert_eq!(results.total_violations, 5);
    }

    // ===================
    // format_violations_section Tests
    // ===================

    #[test]
    fn test_format_violations_section_empty() {
        let violations: Vec<QualityViolation> = vec![];
        let mut output = String::new();
        format_violations_section(&mut output, &violations);
        assert!(output.contains("## Violations"));
    }

    #[test]
    fn test_format_violations_section_single() {
        let violations = vec![QualityViolation {
            check_type: "complexity".to_string(),
            severity: "error".to_string(),
            file: "src/main.rs".to_string(),
            line: Some(10),
            message: "Too complex".to_string(),
        }];

        let mut output = String::new();
        format_violations_section(&mut output, &violations);

        assert!(output.contains("## Violations"));
        assert!(output.contains("COMPLEXITY"));
        assert!(output.contains("main.rs"));
    }

    #[test]
    fn test_format_violations_section_multiple_types() {
        let violations = vec![
            QualityViolation {
                check_type: "complexity".to_string(),
                severity: "error".to_string(),
                file: "src/a.rs".to_string(),
                line: Some(10),
                message: "Complex".to_string(),
            },
            QualityViolation {
                check_type: "security".to_string(),
                severity: "error".to_string(),
                file: "src/b.rs".to_string(),
                line: Some(20),
                message: "Unsafe".to_string(),
            },
        ];

        let mut output = String::new();
        format_violations_section(&mut output, &violations);

        assert!(output.contains("COMPLEXITY"));
        assert!(output.contains("SECURITY"));
    }

    // ===================
    // format_single_violation Tests
    // ===================

    #[test]
    fn test_format_single_violation_with_line() {
        let violation = QualityViolation {
            check_type: "complexity".to_string(),
            severity: "error".to_string(),
            file: "src/main.rs".to_string(),
            line: Some(42),
            message: "Function too complex".to_string(),
        };

        let mut output = String::new();
        format_single_violation(&mut output, &violation);

        assert!(output.contains("🔴")); // error icon
        assert!(output.contains("main.rs"));
        assert!(output.contains("42"));
        assert!(output.contains("Function too complex"));
    }

    #[test]
    fn test_format_single_violation_without_line() {
        let violation = QualityViolation {
            check_type: "dead_code".to_string(),
            severity: "warning".to_string(),
            file: "src/lib.rs".to_string(),
            line: None,
            message: "Unused code".to_string(),
        };

        let mut output = String::new();
        format_single_violation(&mut output, &violation);

        assert!(output.contains("🟡")); // warning icon
        assert!(output.contains("lib.rs"));
        assert!(output.contains("Unused code"));
    }

    #[test]
    fn test_format_single_violation_no_file() {
        let violation = QualityViolation {
            check_type: "satd".to_string(),
            severity: "info".to_string(),
            file: String::new(),
            line: None,
            message: "Technical debt found".to_string(),
        };

        let mut output = String::new();
        format_single_violation(&mut output, &violation);

        assert!(output.contains("🟢")); // other/info icon
        assert!(output.contains("Technical debt found"));
    }

    // ===================
    // resolve_absolute_file_path Tests
    // ===================

    #[test]
    fn test_resolve_absolute_file_path_already_absolute() {
        let project = Path::new("/home/user/project");
        let file = Path::new("/home/user/project/src/main.rs");
        let result = resolve_absolute_file_path(project, file);
        assert_eq!(result, PathBuf::from("/home/user/project/src/main.rs"));
    }

    #[test]
    fn test_resolve_absolute_file_path_relative() {
        let project = Path::new("/home/user/project");
        let file = Path::new("src/main.rs");
        let result = resolve_absolute_file_path(project, file);
        assert_eq!(result, PathBuf::from("/home/user/project/src/main.rs"));
    }
}

