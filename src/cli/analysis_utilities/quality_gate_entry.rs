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
/// ```
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_quality_gate(
    project_path: PathBuf,
    file: Option<PathBuf>,
    format: QualityGateOutputFormat,
    fail_on_violation: bool,
    checks: Vec<QualityCheckType>,
    max_dead_code: f64,
    min_entropy: Option<f64>,
    max_complexity_p99: u32,
    include_provability: bool,
    output: Option<PathBuf>,
    perf: bool,
) -> Result<()> {
    use std::time::Instant;

    if !project_path.exists() {
        return Err(anyhow::anyhow!(
            "Path not found: {}",
            project_path.display()
        ));
    }

    let start_time = if perf { Some(Instant::now()) } else { None };

    // Progress chatter is suppressed by the one rule — `--quiet`
    // (`crate::status_eprintln!`). This used to derive a second, private `quiet`
    // from the output format (#230) and thread it through six modules, so the
    // documented `--quiet` flag reached none of it. One rule, one implementation:
    // `--format json --quiet` is now the silent-progress recipe.
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

    // Show performance metrics if requested (chatter: silent under --quiet)
    if let Some(start) = start_time {
        let duration = start.elapsed();
        crate::status_eprintln!("\n⏱️  Performance Metrics:");
        crate::status_eprintln!("  Total execution time: {:.2}s", duration.as_secs_f64());
        crate::status_eprintln!("  Checks performed: {}", checks_to_run.len());
        crate::status_eprintln!(
            "  Average time per check: {:.2}s",
            duration.as_secs_f64() / checks_to_run.len() as f64
        );
    }

    result
}

/// Prints the initial quality gate status message
fn print_quality_gate_start_message(file: &Option<PathBuf>) {
    if let Some(single_file) = file {
        crate::status_eprintln!(
            "🔍 Running quality gate checks on file: {}...",
            single_file.display()
        );
    } else {
        crate::status_eprintln!("🔍 Running quality gate checks...");
    }
}

/// Prints which checks will be run
/// Toyota Way: Extract Method - Print checks to run (complexity ≤8)
fn print_checks_to_run(checks: &[QualityCheckType]) {
    crate::status_eprintln!("\n📋 Checks to run:");

    if checks.contains(&QualityCheckType::All) {
        print_all_checks();
    } else {
        print_selected_checks(checks);
    }
    crate::status_eprintln!();
}

/// Toyota Way: Extract Method - Print all quality checks (complexity ≤5)
fn print_all_checks() {
    crate::status_eprintln!("  ✓ Complexity analysis");
    crate::status_eprintln!("  ✓ Dead code detection");
    crate::status_eprintln!("  ✓ Self-admitted technical debt (SATD)");
    crate::status_eprintln!("  ✓ Security vulnerabilities");
    crate::status_eprintln!("  ✓ Code entropy");
    crate::status_eprintln!("  ✓ Duplicate code");
    crate::status_eprintln!("  ✓ Test coverage");
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
    crate::status_eprintln!("  ✓ {message}");
}
