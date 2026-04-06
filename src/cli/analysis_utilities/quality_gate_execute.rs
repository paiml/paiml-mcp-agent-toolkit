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
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    execute_quality_check_template(
        check_sections(project_path),
        |count| results.section_violations = count,
        violations,
    )
    .await
}

