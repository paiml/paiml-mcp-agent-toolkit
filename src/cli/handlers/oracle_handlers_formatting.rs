/// Format PDCA results based on output format
fn format_pdca_results(
    results: &[crate::services::oracle::PdcaIterationResult],
    targets: &ConvergenceTargets,
    format: OracleOutputFormat,
) -> Result<String> {
    debug_assert!(!results.is_empty(), "results must not be empty");
    match format {
        OracleOutputFormat::Text => format_pdca_text(results, targets),
        OracleOutputFormat::Json => format_pdca_json(results),
        OracleOutputFormat::Markdown => format_pdca_markdown(results, targets),
    }
}

fn format_pdca_text(
    results: &[crate::services::oracle::PdcaIterationResult],
    targets: &ConvergenceTargets,
) -> Result<String> {
    debug_assert!(!results.is_empty(), "results must not be empty");
    let mut output = String::new();
    output.push_str("=== PDCA Loop Results ===\n\n");

    for result in results {
        output.push_str(&format!("Iteration {}\n", result.iteration));
        output.push_str(&format!("  Defects found: {}\n", result.defects_found));
        output.push_str(&format!("  Defects fixed: {}\n", result.defects_fixed));
        output.push_str(&format!("  Defects skipped: {}\n", result.defects_skipped));
        output.push_str(&format!(
            "  Converged: {}\n\n",
            if result.converged { "YES" } else { "NO" }
        ));
    }

    // Summary
    if let Some(last) = results.last() {
        output.push_str("=== Summary ===\n");
        output.push_str(&format!("Total iterations: {}\n", results.len()));
        output.push_str(&format!(
            "Final status: {}\n",
            if last.converged {
                "CONVERGED"
            } else {
                "NOT CONVERGED"
            }
        ));

        // Show targets
        output.push_str("\nConvergence Targets:\n");
        output.push_str(&format!(
            "  Coverage: ≥{:.0}%\n",
            targets.test_coverage * 100.0
        ));
        output.push_str(&format!(
            "  Mutation score: ≥{:.0}%\n",
            targets.mutation_score * 100.0
        ));
        output.push_str(&format!(
            "  Compiler errors: ≤{}\n",
            targets.max_compiler_errors
        ));
        output.push_str(&format!(
            "  Clippy warnings: ≤{}\n",
            targets.max_clippy_warnings
        ));
        output.push_str(&format!(
            "  Test failures: ≤{}\n",
            targets.max_test_failures
        ));
    }

    Ok(output)
}

fn format_pdca_json(results: &[crate::services::oracle::PdcaIterationResult]) -> Result<String> {
    debug_assert!(!results.is_empty(), "results must not be empty");
    let json_results: Vec<_> = results
        .iter()
        .map(|r| {
            serde_json::json!({
                "iteration": r.iteration,
                "defects_found": r.defects_found,
                "defects_fixed": r.defects_fixed,
                "defects_skipped": r.defects_skipped,
                "converged": r.converged
            })
        })
        .collect();

    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "iterations": json_results,
        "total_iterations": results.len(),
        "converged": results.last().map(|r| r.converged).unwrap_or(false)
    }))?)
}

fn format_pdca_markdown(
    results: &[crate::services::oracle::PdcaIterationResult],
    targets: &ConvergenceTargets,
) -> Result<String> {
    debug_assert!(!results.is_empty(), "results must not be empty");
    let mut output = String::new();
    output.push_str("# PMAT Oracle - PDCA Loop Results\n\n");

    output.push_str("## Iterations\n\n");
    output.push_str("| Iteration | Defects Found | Fixed | Skipped | Converged |\n");
    output.push_str("|-----------|---------------|-------|---------|----------|\n");

    for result in results {
        output.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            result.iteration,
            result.defects_found,
            result.defects_fixed,
            result.defects_skipped,
            if result.converged { "✅" } else { "❌" }
        ));
    }

    output.push_str("\n## Convergence Targets\n\n");
    output.push_str(&format!(
        "- **Test Coverage**: ≥{:.0}%\n",
        targets.test_coverage * 100.0
    ));
    output.push_str(&format!(
        "- **Mutation Score**: ≥{:.0}%\n",
        targets.mutation_score * 100.0
    ));
    output.push_str(&format!(
        "- **Compiler Errors**: ≤{}\n",
        targets.max_compiler_errors
    ));
    output.push_str(&format!(
        "- **Clippy Warnings**: ≤{}\n",
        targets.max_clippy_warnings
    ));

    Ok(output)
}

fn format_iteration_result(
    result: &crate::services::oracle::PdcaIterationResult,
    format: &OracleOutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    debug_assert!(true, "contract: format_iteration_result");
    let formatted = match format {
        OracleOutputFormat::Text => format!(
            "Defects found: {}\nDefects that would be fixed: {}\nSkipped: {}\n",
            result.defects_found, result.defects_fixed, result.defects_skipped
        ),
        OracleOutputFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
            "defects_found": result.defects_found,
            "defects_fixed": result.defects_fixed,
            "defects_skipped": result.defects_skipped,
            "dry_run": true
        }))?,
        OracleOutputFormat::Markdown => format!(
            "## Dry Run Results\n\n- Defects found: {}\n- Would fix: {}\n- Skipped: {}\n",
            result.defects_found, result.defects_fixed, result.defects_skipped
        ),
    };

    if let Some(path) = output {
        std::fs::write(path, &formatted)?;
    } else {
        println!("{}", formatted);
    }

    Ok(())
}

fn format_status(
    metrics: &ProjectMetrics,
    targets: &ConvergenceTargets,
    status: &crate::services::oracle::ConvergenceStatus,
    format: OracleOutputFormat,
) -> Result<String> {
    debug_assert!(true, "contract: format_status");
    match format {
        OracleOutputFormat::Text => {
            let mut output = String::new();
            output.push_str("=== Project Quality Status ===\n\n");

            output.push_str(&format!(
                "Test Coverage:     {:.1}% (target: ≥{:.1}%)\n",
                metrics.test_coverage * 100.0,
                targets.test_coverage * 100.0
            ));
            output.push_str(&format!(
                "Mutation Score:    {:.1}% (target: ≥{:.1}%)\n",
                metrics.mutation_score * 100.0,
                targets.mutation_score * 100.0
            ));
            output.push_str(&format!(
                "Compiler Errors:   {} (target: ≤{})\n",
                metrics.compiler_errors, targets.max_compiler_errors
            ));
            output.push_str(&format!(
                "Clippy Warnings:   {} (target: ≤{})\n",
                metrics.clippy_warnings, targets.max_clippy_warnings
            ));
            output.push_str(&format!(
                "Test Failures:     {} (target: ≤{})\n",
                metrics.test_failures, targets.max_test_failures
            ));
            output.push_str(&format!(
                "TDG Score:         {:.1} (target: ≥{:.1})\n",
                metrics.tdg_score, targets.min_tdg_score
            ));
            output.push_str(&format!(
                "Rust Project Score: {} (target: ≥{})\n",
                metrics.rust_project_score, targets.min_rust_project_score
            ));

            output.push('\n');
            match status {
                crate::services::oracle::ConvergenceStatus::Converged => {
                    output.push_str("✅ CONVERGED - Project meets all quality targets!\n");
                }
                crate::services::oracle::ConvergenceStatus::NotConverged { remaining } => {
                    output.push_str("❌ NOT CONVERGED - Remaining issues:\n");
                    for issue in remaining {
                        output.push_str(&format!("   - {}\n", issue));
                    }
                }
            }

            Ok(output)
        }
        OracleOutputFormat::Json => Ok(serde_json::to_string_pretty(&serde_json::json!({
            "metrics": {
                "test_coverage": metrics.test_coverage,
                "mutation_score": metrics.mutation_score,
                "compiler_errors": metrics.compiler_errors,
                "clippy_warnings": metrics.clippy_warnings,
                "test_failures": metrics.test_failures,
                "tdg_score": metrics.tdg_score,
                "rust_project_score": metrics.rust_project_score
            },
            "targets": {
                "test_coverage": targets.test_coverage,
                "mutation_score": targets.mutation_score,
                "max_compiler_errors": targets.max_compiler_errors,
                "max_clippy_warnings": targets.max_clippy_warnings,
                "max_test_failures": targets.max_test_failures,
                "min_tdg_score": targets.min_tdg_score,
                "min_rust_project_score": targets.min_rust_project_score
            },
            "converged": matches!(status, crate::services::oracle::ConvergenceStatus::Converged)
        }))?),
        OracleOutputFormat::Markdown => {
            let mut output = String::new();
            output.push_str("# Project Quality Status\n\n");
            output.push_str("| Metric | Current | Target | Status |\n");
            output.push_str("|--------|---------|--------|--------|\n");

            let coverage_ok = metrics.test_coverage >= targets.test_coverage;
            output.push_str(&format!(
                "| Test Coverage | {:.1}% | ≥{:.1}% | {} |\n",
                metrics.test_coverage * 100.0,
                targets.test_coverage * 100.0,
                if coverage_ok { "✅" } else { "❌" }
            ));

            let mutation_ok = metrics.mutation_score >= targets.mutation_score;
            output.push_str(&format!(
                "| Mutation Score | {:.1}% | ≥{:.1}% | {} |\n",
                metrics.mutation_score * 100.0,
                targets.mutation_score * 100.0,
                if mutation_ok { "✅" } else { "❌" }
            ));

            let errors_ok = metrics.compiler_errors <= targets.max_compiler_errors;
            output.push_str(&format!(
                "| Compiler Errors | {} | ≤{} | {} |\n",
                metrics.compiler_errors,
                targets.max_compiler_errors,
                if errors_ok { "✅" } else { "❌" }
            ));

            Ok(output)
        }
    }
}

fn format_single_result(
    result: &crate::services::oracle::PdcaIterationResult,
    format: OracleOutputFormat,
) -> Result<String> {
    debug_assert!(true, "contract: format_single_result");
    match format {
        OracleOutputFormat::Text => Ok(format!(
            "=== Single PDCA Iteration ===\n\n\
             Defects found: {}\n\
             Defects fixed: {}\n\
             Defects skipped: {}\n\
             Converged: {}\n",
            result.defects_found,
            result.defects_fixed,
            result.defects_skipped,
            if result.converged { "YES" } else { "NO" }
        )),
        OracleOutputFormat::Json => Ok(serde_json::to_string_pretty(&serde_json::json!({
            "iteration": result.iteration,
            "defects_found": result.defects_found,
            "defects_fixed": result.defects_fixed,
            "defects_skipped": result.defects_skipped,
            "converged": result.converged
        }))?),
        OracleOutputFormat::Markdown => Ok(format!(
            "# Single PDCA Iteration\n\n\
             | Metric | Value |\n\
             |--------|-------|\n\
             | Defects found | {} |\n\
             | Defects fixed | {} |\n\
             | Defects skipped | {} |\n\
             | Converged | {} |\n",
            result.defects_found,
            result.defects_fixed,
            result.defects_skipped,
            if result.converged { "✅" } else { "❌" }
        )),
    }
}
