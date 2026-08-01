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

/// Runs all project-wide checks
#[allow(clippy::too_many_arguments)]
async fn run_all_project_checks(
    project_path: &Path,
    max_dead_code: f64,
    min_entropy: Option<f64>,
    max_complexity_p99: u32,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
    perf: bool,
    quiet: bool,
) -> Result<()> {
    use std::time::Instant;

    // Run all checks
    if !quiet {
        eprint!("  🔍 Checking complexity...");
    }
    let start = if perf { Some(Instant::now()) } else { None };
    let complexity_violations = check_complexity(project_path, max_complexity_p99).await?;
    results.complexity_violations = complexity_violations.len();
    violations.extend(complexity_violations);
    if !quiet {
        if let Some(s) = start {
            eprintln!(
                " {} violations found ({:.3}s)",
                results.complexity_violations,
                s.elapsed().as_secs_f64()
            );
        } else {
            eprintln!(" {} violations found", results.complexity_violations);
        }
    }

    // Macro to handle timing for each check (#230: suppress progress in JSON mode)
    macro_rules! run_check {
        ($name:expr, $check_expr:expr, $result_field:ident) => {{
            if !quiet {
                eprint!("  🔍 Checking {}...", $name);
            }
            let start = if perf { Some(Instant::now()) } else { None };
            let check_violations = $check_expr.await?;
            results.$result_field = check_violations.len();
            violations.extend(check_violations);
            if !quiet {
                if let Some(s) = start {
                    eprintln!(
                        " {} violations found ({:.3}s)",
                        results.$result_field,
                        s.elapsed().as_secs_f64()
                    );
                } else {
                    eprintln!(" {} violations found", results.$result_field);
                }
            }
        }};
    }

    run_check!(
        "dead code",
        check_dead_code(project_path, max_dead_code),
        dead_code_violations
    );
    run_check!("technical debt", check_satd(project_path), satd_violations);
    run_entropy_check_gated(project_path, min_entropy, violations, results, perf, quiet).await?;
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
    quiet: bool,
) -> Result<()> {
    use std::time::Instant;

    let gate_config = load_entropy_gate_config(project_path);
    if !gate_config.enabled {
        if !quiet {
            eprintln!("  \u{23ed}\u{fe0f}  Skipping code entropy (disabled via .pmat-gates.toml)");
        }
        return Ok(());
    }

    let ent_threshold = load_entropy_threshold(project_path, min_entropy)?;
    let mut ent_excludes = load_entropy_exclude_paths(project_path);
    merge_excludes(&mut ent_excludes, &gate_config.exclude);

    if !quiet {
        eprint!("  \u{1f50d} Checking code entropy...");
    }
    let start = if perf { Some(Instant::now()) } else { None };
    let ent_violations =
        check_entropy_with_excludes(project_path, ent_threshold, &ent_excludes).await?;
    results.entropy_violations = ent_violations.len();
    violations.extend(ent_violations);

    if !quiet {
        if let Some(s) = start {
            eprintln!(
                " {} violations found ({:.3}s)",
                results.entropy_violations,
                s.elapsed().as_secs_f64()
            );
        } else {
            eprintln!(" {} violations found", results.entropy_violations);
        }
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
