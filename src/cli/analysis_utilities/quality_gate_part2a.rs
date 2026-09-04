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
    // A cache that exists but failed a guard is disclosed by name: the gate
    // used to trust any file it found (CRUX-02, #1153).
    if let CoverageCacheRead::Rejected(reason) = read_coverage_from_detail_cache(project_path) {
        if read_coverage_from_metrics(project_path).is_none() {
            return Ok(vec![QualityViolation {
                check_type: "coverage".to_string(),
                severity: "error".to_string(),
                file: "project".to_string(),
                line: None,
                message: format!(
                    "Code coverage report REJECTED ({reason}), so the \
                     {QUALITY_GATE_COVERAGE_MIN:.1}% minimum is unverified — this gate does not \
                     cover coverage until a report taken from this tree exists"
                ),
                details: Some(ViolationDetails {
                    affected_files: Vec::new(),
                    example_code: None,
                    fix_suggestion: Some(
                        "Regenerate the report from this tree — `pmat query --coverage` runs \
                         cargo-llvm-cov and rewrites .pmat/coverage-cache.json — then re-run the gate."
                            .to_string(),
                    ),
                    score_factors: vec![format!("coverage: report rejected — {reason}")],
                }),
            }]);
        }
    }
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
    thresholds: QualityThresholds,
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

    if !crate::cli::progress::quiet_mode_enabled() {
        eprint!("  🔍 Checking dead code...");
    }
    let dead_code_start = perf.then(Instant::now);
    run_dead_code_check(project_path, max_dead_code, violations, results).await?;
    report_dead_code_outcome(results, dead_code_start);
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
        identical_files
    );
    results.not_measured.push(duplicates_block_level_disclosure(project_path));
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
    // AD-05. `file-size` and `churn` join the suite; `lint` does NOT, and the
    // omission is deliberate rather than an oversight: clippy costs a full
    // compile of the analysed tree, so putting it here would make every
    // `pmat quality-gate` and every MCP `quality_gate` call build a crate. It
    // is reachable as `--checks lint`, and `default_checks()` — which is what
    // the suite ADVERTISES as run — does not name it, so the two stay in step.
    run_check!(
        "file size",
        check_file_size(project_path, thresholds.max_file_lines),
        file_size_violations
    );
    run_churn_check_counted(project_path, thresholds, violations, results, perf).await?;

    Ok(())
}

/// Churn, counted without its disclosure row.
///
/// Two things `run_check!` cannot do here. It sets the counter from the returned
/// list's LENGTH, and the churn check can return an advisory `scope` row when
/// there is no git history to read — through the macro that row would render as
/// `churn_violations: 1`, an unmeasured check reported as a breached one.
///
/// The row itself is then DROPPED from the suite's findings, which is the
/// convention `run_all_project_checks` already follows for the security check's
/// identical scope row: `--checks security` pushes
/// `security_scope_disclosure` and this suite does not. `--checks churn` keeps
/// the disclosure for the same reason. That asymmetry is inherited, not
/// invented here — see the receipt's open question.
async fn run_churn_check_counted(
    project_path: &Path,
    thresholds: QualityThresholds,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
    perf: bool,
) -> Result<()> {
    use std::time::Instant;

    if !crate::cli::progress::quiet_mode_enabled() {
        eprint!("  \u{1f50d} Checking churn...");
    }
    let start = if perf { Some(Instant::now()) } else { None };
    let mut found = check_churn(project_path, thresholds.max_churn_commits_90d).await?;
    found.retain(|v| v.check_type == "churn");
    results.churn_violations = found.len();
    violations.extend(found);
    if let Some(s) = start {
        crate::status_eprintln!(
            " {} violations found ({:.3}s)",
            results.churn_violations,
            s.elapsed().as_secs_f64()
        );
    } else {
        crate::status_eprintln!(" {} violations found", results.churn_violations);
    }
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

/// The progress line for the dead-code check, which unlike its neighbours has
/// three outcomes to report, not a count alone.
fn report_dead_code_outcome(results: &QualityGateResults, start: Option<std::time::Instant>) {
    let what = if let Some(u) = results.not_measured.iter().find(|u| u.check == DEAD_CODE_CHECK) {
        format!("NOT MEASURED ({})", u.reason)
    } else if results.not_applicable.iter().any(|u| u.check == DEAD_CODE_CHECK) {
        "not applicable (no Cargo.toml)".to_string()
    } else {
        format!("{} violations found", results.dead_code_violations)
    };
    match start {
        Some(s) => crate::status_eprintln!(" {what} ({:.3}s)", s.elapsed().as_secs_f64()),
        None => crate::status_eprintln!(" {what}"),
    }
}
