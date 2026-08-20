/// The coverage floor this gate enforces. It was written twice, as the literal
/// `80.0`, at the two call sites of [`check_coverage`].
const QUALITY_GATE_COVERAGE_MIN: f64 = 80.0;

/// The coverage check, with an absent report disclosed instead of counted as zero.
///
/// [`check_coverage`] does not measure coverage — it reads a report someone else
/// produced (`.pmat/coverage-cache.json`, else `.pmat-metrics/coverage.json`).
/// When neither file exists it returned an empty violation list, which is the
/// same value it returns for a project measured at 100%. `coverage_violations: 0`
/// therefore meant "clean" and "never looked" at once, and the second reading is
/// the common one: none of the three differential corpora carries a coverage
/// report, and neither does a freshly cloned repository.
///
/// A check that did not run has not passed, so the gap is reported as a coverage
/// finding of its own. `check_coverage` is left alone: it answers "how does the
/// measured coverage compare to the floor", which is a different question from
/// "was there a measurement", and only the caller composing the two can answer
/// the second without the first losing its meaning.
async fn run_coverage_check(project_path: &Path) -> Result<Vec<QualityViolation>> {
    if read_coverage_from_cache(project_path).is_none() {
        return Ok(vec![QualityViolation {
            check_type: "coverage".to_string(),
            severity: "error".to_string(),
            file: "project".to_string(),
            line: None,
            message: format!(
                "Code coverage was NOT measured (no coverage report at .pmat/coverage-cache.json \
                 or .pmat-metrics/coverage.json), so the {QUALITY_GATE_COVERAGE_MIN:.1}% minimum \
                 is unverified — this gate does not cover coverage"
            ),
            details: Some(ViolationDetails {
                affected_files: Vec::new(),
                example_code: None,
                fix_suggestion: Some(
                    "Produce a report first — `pmat query --coverage` runs cargo-llvm-cov and \
                     writes .pmat/coverage-cache.json; alternatively write \
                     {\"coverage\": <percent>} to .pmat-metrics/coverage.json — then re-run the \
                     gate, or deselect the check with `--checks` if coverage is gated elsewhere."
                        .to_string(),
                ),
                score_factors: vec!["coverage: not measured".to_string()],
            }),
        }]);
    }
    check_coverage(project_path, QUALITY_GATE_COVERAGE_MIN).await
}

/// Helper for provability check execution
async fn execute_provability_check(
    project_path: &Path,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    let threshold = load_provability_threshold(project_path);
    execute_quality_check_template(
        check_provability(project_path, threshold),
        |count| results.provability_violations = count,
        violations,
    )
    .await
}

/// Runs all project-wide checks.
///
/// THE list of checks behind the name "quality gate", called — not copied — by
/// every surface that claims to run them: `pmat quality-gate --checks all` here,
/// and the MCP `quality_gate` tool through [`run_gate_suite`]. That tool used to
/// carry its own two-check list, so a `coverage` gap the CLI reported was
/// invisible over MCP.
///
/// # Errors
/// Propagates any individual check's failure.
#[allow(clippy::too_many_arguments)]
pub async fn run_all_project_checks(
    project_path: &Path,
    max_dead_code: f64,
    min_entropy: Option<f64>,
    max_complexity_p99: u32,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
    perf: bool,
) -> Result<()> {
    use std::time::Instant;

    // Run all checks
    if !crate::cli::progress::quiet_mode_enabled() {
        eprint!("  🔍 Checking complexity...");
    }
    let start = if perf { Some(Instant::now()) } else { None };
    let complexity_violations = check_complexity(project_path, max_complexity_p99).await?;
    results.complexity_violations = complexity_violations.len();
    violations.extend(complexity_violations);
    if let Some(s) = start {
        crate::status_eprintln!(
            " {} violations found ({:.3}s)",
            results.complexity_violations,
            s.elapsed().as_secs_f64()
        );
    } else {
        crate::status_eprintln!(" {} violations found", results.complexity_violations);
    }

    // Macro to handle timing for each check (progress chatter: silent under --quiet)
    macro_rules! run_check {
        ($name:expr, $check_expr:expr, $result_field:ident) => {{
            if !$crate::cli::progress::quiet_mode_enabled() {
                eprint!("  🔍 Checking {}...", $name);
            }
            let start = if perf { Some(Instant::now()) } else { None };
            let check_violations = $check_expr.await?;
            results.$result_field = check_violations.len();
            violations.extend(check_violations);
            if let Some(s) = start {
                $crate::status_eprintln!(
                    " {} violations found ({:.3}s)",
                    results.$result_field,
                    s.elapsed().as_secs_f64()
                );
            } else {
                $crate::status_eprintln!(" {} violations found", results.$result_field);
            }
        }};
    }

    run_check!(
        "dead code",
        check_dead_code(project_path, max_dead_code),
        dead_code_violations
    );
    run_check!("technical debt", check_satd(project_path), satd_violations);
    run_entropy_check_gated(project_path, min_entropy, violations, results, perf).await?;
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
        run_coverage_check(project_path),
        coverage_violations
    );
    run_check!(
        "documentation sections",
        check_sections(project_path),
        section_violations
    );
    let provability_threshold = load_provability_threshold(project_path);
    run_check!(
        "provability",
        check_provability(project_path, provability_threshold),
        provability_violations
    );

    Ok(())
}

/// Run entropy check with gate config (#220): enabled, excludes, max_violations.
async fn run_entropy_check_gated(
    project_path: &Path,
    min_entropy: Option<f64>,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
    perf: bool,
) -> Result<()> {
    use std::time::Instant;

    let gate_config = load_entropy_gate_config(project_path);
    if !gate_config.enabled {
        crate::status_eprintln!(
            "  \u{23ed}\u{fe0f}  Skipping code entropy (disabled via .pmat-gates.toml)"
        );
        return Ok(());
    }

    let ent_threshold = load_entropy_threshold(project_path, min_entropy);
    let mut ent_excludes = load_entropy_exclude_paths(project_path);
    merge_excludes(&mut ent_excludes, &gate_config.exclude);

    if !crate::cli::progress::quiet_mode_enabled() {
        eprint!("  \u{1f50d} Checking code entropy...");
    }
    let start = if perf { Some(Instant::now()) } else { None };
    let ent_violations =
        check_entropy_with_excludes(project_path, ent_threshold, &ent_excludes).await?;
    results.entropy_violations = ent_violations.len();
    violations.extend(ent_violations);

    if let Some(s) = start {
        crate::status_eprintln!(
            " {} violations found ({:.3}s)",
            results.entropy_violations,
            s.elapsed().as_secs_f64()
        );
    } else {
        crate::status_eprintln!(" {} violations found", results.entropy_violations);
    }

    // Apply max_violations threshold (#220)
    if let Some(max) = gate_config.max_violations {
        if results.entropy_violations <= max {
            violations.retain(|v| v.check_type != "entropy");
            results.entropy_violations = 0;
        }
    }

    Ok(())
}

/// Merge exclude patterns, deduplicating.
fn merge_excludes(base: &mut Vec<String>, extra: &[String]) {
    for pattern in extra {
        if !base.contains(pattern) {
            base.push(pattern.clone());
        }
    }
}
