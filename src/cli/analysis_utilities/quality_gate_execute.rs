/// Runs a single project-wide check
#[allow(clippy::too_many_arguments)]
/// Toyota Way: Data-Driven Design - eliminated 41→≤8 complexity
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn run_single_project_check(
    check: &QualityCheckType,
    project_path: &Path,
    max_dead_code: f64,
    min_entropy: Option<f64>,
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
    min_entropy: Option<f64>,
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
    min_entropy: Option<f64>,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    let gate_config = load_entropy_gate_config(project_path);
    if !gate_config.enabled {
        crate::status_eprintln!("  ⏭️  Entropy check disabled via .pmat-gates.toml");
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
    // `--checks coverage` and `--checks all` run the same composition, so the
    // one surface cannot report a gap the other silently counts as zero.
    execute_quality_check_template(
        run_coverage_check(project_path),
        |count| results.coverage_violations = count,
        violations,
    )
    .await
}

/// `results.coverage_violations` had two meanings and one value.
///
/// These run the surface a user runs — `pmat quality-gate --checks coverage` is
/// exactly [`execute_coverage_check`] — rather than the helper underneath it, so
/// they fail if the composition is ever unwired again.
#[cfg(test)]
mod coverage_is_measured_or_disclosed_tests {
    use super::*;

    async fn coverage_violations_for(project_path: &Path) -> (usize, Vec<QualityViolation>) {
        let mut violations = Vec::new();
        let mut results = QualityGateResults::default();
        execute_coverage_check(project_path, &mut violations, &mut results)
            .await
            .expect("the coverage check reports");
        (results.coverage_violations, violations)
    }

    fn write_metrics(dir: &Path, body: &str) {
        std::fs::create_dir_all(dir.join(".pmat-metrics")).expect("mkdir .pmat-metrics");
        std::fs::write(dir.join(".pmat-metrics/coverage.json"), body).expect("write coverage.json");
    }

    #[tokio::test]
    async fn an_absent_coverage_report_is_disclosed_not_counted_as_zero() {
        let dir = tempfile::tempdir().expect("tempdir");
        let (count, violations) = coverage_violations_for(dir.path()).await;

        // It used to be 0 here — the same 0 a project measured at 100% gets.
        assert_eq!(
            count, 1,
            "an unmeasured check has not passed: {violations:?}"
        );
        let disclosure = &violations[0];
        assert_eq!(disclosure.check_type, "coverage");
        assert_eq!(disclosure.severity, "error", "an unmeasured check blocks");
        assert!(
            disclosure.message.contains("NOT measured"),
            "the row must say which of the two zeros this is: {}",
            disclosure.message
        );
    }

    #[tokio::test]
    async fn a_measured_project_above_the_floor_reports_no_violation() {
        // The disclosure must not become a second constant: a project that DID
        // measure, and passed, still reports 0.
        let dir = tempfile::tempdir().expect("tempdir");
        write_metrics(dir.path(), "{\"coverage\": 95.0}");
        let (count, violations) = coverage_violations_for(dir.path()).await;
        assert_eq!(count, 0, "95% clears the 80% floor: {violations:?}");
    }

    #[tokio::test]
    async fn a_measured_project_below_the_floor_reports_the_measurement() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_metrics(dir.path(), "{\"coverage\": 42.5}");
        let (count, violations) = coverage_violations_for(dir.path()).await;
        assert_eq!(count, 1);
        assert!(
            violations[0].message.contains("42.5"),
            "a real breach names the measured value, not the gap: {}",
            violations[0].message
        );
        assert!(
            !violations[0].message.contains("NOT measured"),
            "and is not the disclosure: {}",
            violations[0].message
        );
    }
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
