/// Toyota Way: Extract Method - Print checks to run (complexity ≤8)
/// Console output utility for displaying which quality checks will be executed
pub fn print_checks_to_run(checks: &[QualityCheckType]) {
    eprintln!("\n📋 Checks to run:");

    if checks.contains(&QualityCheckType::All) {
        print_all_checks();
    } else {
        print_specific_checks(checks);
    }
    eprintln!();
}

/// Toyota Way: Extract Method - Print all check types (complexity ≤3)
fn print_all_checks() {
    eprintln!("  ✓ Complexity analysis");
    eprintln!("  ✓ Dead code detection");
    eprintln!("  ✓ Self-admitted technical debt (SATD)");
    eprintln!("  ✓ Security vulnerabilities");
    eprintln!("  ✓ Code entropy");
    eprintln!("  ✓ Duplicate code");
    eprintln!("  ✓ Test coverage");
}

/// Toyota Way: Extract Method - Print specific check types (complexity ≤8)
fn print_specific_checks(checks: &[QualityCheckType]) {
    for check in checks {
        let check_name = match check {
            QualityCheckType::Complexity => "✓ Complexity analysis",
            QualityCheckType::DeadCode => "✓ Dead code detection",
            QualityCheckType::Satd => "✓ Self-admitted technical debt (SATD)",
            QualityCheckType::Security => "✓ Security vulnerabilities",
            QualityCheckType::Entropy => "✓ Code entropy",
            QualityCheckType::Duplicates => "✓ Duplicate code",
            QualityCheckType::Coverage => "✓ Test coverage",
            _ => continue, // Skip other types
        };
        eprintln!("  {check_name}");
    }
}

/// Configuration for quality checks (SPRINT-23)
#[derive(Debug, Clone)]
pub struct QualityCheckConfig<'a> {
    pub project_path: &'a Path,
    pub checks: &'a [QualityCheckType],
    pub max_dead_code: f64,
    pub min_entropy: f64,
    pub max_complexity_p99: u32,
    pub perf: bool,
}

/// Toyota Way: Extract Method - Run project quality checks (complexity ≤8)
/// Orchestrates the execution of quality checks based on the specified types
pub async fn run_project_checks(
    config: QualityCheckConfig<'_>,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    // If checks contains All, run the comprehensive check
    if config.checks.contains(&QualityCheckType::All) {
        run_all_checks(
            config.project_path,
            config.max_dead_code,
            config.min_entropy,
            config.max_complexity_p99,
            violations,
            results,
            config.perf,
        )
        .await?;
    } else {
        // Run individual checks with performance timing
        let checks_config = IndividualChecksConfig {
            checks: config.checks,
            project_path: config.project_path,
            max_dead_code: config.max_dead_code,
            min_entropy: config.min_entropy,
            max_complexity_p99: config.max_complexity_p99,
            perf: config.perf,
        };
        run_individual_checks(checks_config, violations, results).await?;
    }
    Ok(())
}

/// Toyota Way: Extract Method - Run all quality checks (complexity ≤3)
async fn run_all_checks(
    project_path: &Path,
    max_dead_code: f64,
    min_entropy: f64,
    max_complexity_p99: u32,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
    perf: bool,
) -> Result<()> {
    crate::cli::analysis_utilities::run_single_project_check(
        &QualityCheckType::All,
        project_path,
        max_dead_code,
        min_entropy,
        max_complexity_p99,
        violations,
        results,
        perf,
        false, // quiet=false for human-readable output
    )
    .await
}

/// Configuration for running individual checks
struct IndividualChecksConfig<'a> {
    checks: &'a [QualityCheckType],
    project_path: &'a Path,
    max_dead_code: f64,
    min_entropy: f64,
    max_complexity_p99: u32,
    perf: bool,
}

/// Toyota Way: Extract Method - Run individual checks with timing (complexity ≤8)
async fn run_individual_checks(
    config: IndividualChecksConfig<'_>,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    use std::time::Instant;

    for check in config.checks {
        let check_start = if config.perf {
            Some(Instant::now())
        } else {
            None
        };

        crate::cli::analysis_utilities::run_single_project_check(
            check,
            config.project_path,
            config.max_dead_code,
            config.min_entropy,
            config.max_complexity_p99,
            violations,
            results,
            config.perf,
            false, // quiet=false for human-readable output
        )
        .await?;

        // Print timing if performance monitoring is enabled
        if let Some(start) = check_start {
            print_check_timing(check, start.elapsed().as_secs_f64());
        }
    }
    Ok(())
}

/// Toyota Way: Extract Method - Print check timing (complexity ≤8)
fn print_check_timing(check: &QualityCheckType, elapsed_secs: f64) {
    let check_name = match check {
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
    };
    eprintln!("    ⏱️  {check_name} check: {elapsed_secs:.3}s");
}
