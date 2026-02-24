// Output formatting for refactoring results (JSON, Markdown, text).
// Included via include!() — shares parent module scope.

/// Format and output refactoring results (Phase 5: Extract Output Formatting)
///
/// Generates final output in the requested format with comprehensive results.
/// This function has complexity <5 and follows Toyota Way principles.
async fn format_and_output_results(
    iteration_results: &[IterationResult],
    final_validation: &ValidationResult,
    context: &RefactorContext,
) -> Result<()> {
    eprintln!("📋 Formatting and outputting refactoring results...");

    match &context.config.output.format {
        RefactorAutoOutputFormat::Json => {
            output_json_results(iteration_results, final_validation, context).await?;
        }
        RefactorAutoOutputFormat::Detailed => {
            output_markdown_results(iteration_results, final_validation, context).await?;
        }
        RefactorAutoOutputFormat::Summary => {
            output_text_results(iteration_results, final_validation, context).await?;
        }
    }

    eprintln!("✅ Results output completed");
    Ok(())
}

/// Output results in JSON format
///
/// This function has complexity <3 and follows Toyota Way principles.
async fn output_json_results(
    iteration_results: &[IterationResult],
    final_validation: &ValidationResult,
    context: &RefactorContext,
) -> Result<()> {
    let summary = create_refactoring_summary(iteration_results, final_validation, context).await?;

    let json_output = serde_json::json!({
        "refactoring_session": {
            "project_path": context.config.project_path,
            "start_time": context.start_time.elapsed().as_secs(),
            "total_iterations": iteration_results.len(),
            "final_validation": {
                "overall_success": final_validation.overall_success,
                "compilation_passed": final_validation.compilation_passed,
                "tests_passed": final_validation.tests_passed,
                "quality_improved": final_validation.quality_improved
            },
            "summary": summary,
            "iterations": iteration_results.iter().map(|result| {
                serde_json::json!({
                    "iteration_number": result.iteration_number,
                    "successful_requests": result.successful_requests.len(),
                    "failed_requests": result.failed_requests.len(),
                    "duration_seconds": result.iteration_duration.as_secs(),
                    "quality_improvement": {
                        "complexity_reduced": result.quality_improvement.complexity_reduced,
                        "violations_fixed": result.quality_improvement.violations_fixed,
                        "satd_resolved": result.quality_improvement.satd_resolved,
                        "coverage_increased": result.quality_improvement.coverage_increased
                    }
                })
            }).collect::<Vec<_>>()
        }
    });

    println!("{}", serde_json::to_string_pretty(&json_output)?);
    Ok(())
}

/// Output results in Markdown format
///
/// This function has complexity <3 and follows Toyota Way principles.
async fn output_markdown_results(
    iteration_results: &[IterationResult],
    final_validation: &ValidationResult,
    context: &RefactorContext,
) -> Result<()> {
    let summary = create_refactoring_summary(iteration_results, final_validation, context).await?;

    println!("# Automated Refactoring Report\n");

    println!("## Project Information");
    println!(
        "- **Project Path**: `{}`",
        context.config.project_path.display()
    );
    println!(
        "- **Execution Time**: {:.2}s",
        context.start_time.elapsed().as_secs_f64()
    );
    println!("- **Total Iterations**: {}\n", iteration_results.len());

    println!("## Summary");
    println!(
        "- **Overall Success**: {}",
        if final_validation.overall_success {
            "✅ YES"
        } else {
            "❌ NO"
        }
    );
    println!(
        "- **Compilation**: {}",
        if final_validation.compilation_passed {
            "✅ PASSED"
        } else {
            "❌ FAILED"
        }
    );
    println!(
        "- **Tests**: {}",
        if final_validation.tests_passed {
            "✅ PASSED"
        } else {
            "❌ FAILED"
        }
    );
    println!(
        "- **Quality Improved**: {}",
        if final_validation.quality_improved {
            "✅ YES"
        } else {
            "❌ NO"
        }
    );
    println!(
        "- **Total Refactorings**: {}",
        summary.total_successful_requests
    );
    println!("- **Quality Score**: {:.1}\n", summary.total_quality_score);

    println!("## Iteration Details\n");
    for result in iteration_results {
        println!("### Iteration #{}", result.iteration_number);
        println!("- **Duration**: {:?}", result.iteration_duration);
        println!(
            "- **Successful**: {} requests",
            result.successful_requests.len()
        );
        println!("- **Failed**: {} requests", result.failed_requests.len());
        println!("- **Quality Improvement**:");
        println!(
            "  - Complexity reduced: {}",
            result.quality_improvement.complexity_reduced
        );
        println!(
            "  - Violations fixed: {}",
            result.quality_improvement.violations_fixed
        );
        println!(
            "  - SATD resolved: {}",
            result.quality_improvement.satd_resolved
        );
        println!(
            "  - Coverage increased: {:.1}%",
            result.quality_improvement.coverage_increased
        );
        println!();
    }

    if !final_validation.issues_found.is_empty() {
        println!("## Issues Found\n");
        for issue in &final_validation.issues_found {
            println!("- ❌ {issue}");
        }
    }

    Ok(())
}

/// Output results in plain text format
///
/// This function has complexity <3 and follows Toyota Way principles.
async fn output_text_results(
    iteration_results: &[IterationResult],
    final_validation: &ValidationResult,
    context: &RefactorContext,
) -> Result<()> {
    let summary = create_refactoring_summary(iteration_results, final_validation, context).await?;

    println!("🚀 AUTOMATED REFACTORING REPORT");
    println!("=====================================");
    println!("📁 Project: {}", context.config.project_path.display());
    println!(
        "⏱️  Total Time: {:.2}s",
        context.start_time.elapsed().as_secs_f64()
    );
    println!("🔄 Iterations: {}", iteration_results.len());
    println!();

    println!("📊 FINAL RESULTS");
    println!("=====================================");
    println!(
        "Overall Success:    {}",
        if final_validation.overall_success {
            "✅ YES"
        } else {
            "❌ NO"
        }
    );
    println!(
        "Compilation:        {}",
        if final_validation.compilation_passed {
            "✅ PASSED"
        } else {
            "❌ FAILED"
        }
    );
    println!(
        "Tests:              {}",
        if final_validation.tests_passed {
            "✅ PASSED"
        } else {
            "❌ FAILED"
        }
    );
    println!(
        "Quality Improved:   {}",
        if final_validation.quality_improved {
            "✅ YES"
        } else {
            "❌ NO"
        }
    );
    println!("Total Refactorings: {}", summary.total_successful_requests);
    println!("Quality Score:      {:.1}", summary.total_quality_score);
    println!();

    if !iteration_results.is_empty() {
        println!("🔄 ITERATION BREAKDOWN");
        println!("=====================================");
        for result in iteration_results {
            println!(
                "Iteration #{}: {} successful, {} failed ({:?})",
                result.iteration_number,
                result.successful_requests.len(),
                result.failed_requests.len(),
                result.iteration_duration
            );
        }
    }

    if !final_validation.issues_found.is_empty() {
        println!();
        println!("❌ ISSUES FOUND");
        println!("=====================================");
        for issue in &final_validation.issues_found {
            println!("• {issue}");
        }
    }

    Ok(())
}

/// Create comprehensive refactoring summary
///
/// This function has complexity <3 and follows Toyota Way principles.
async fn create_refactoring_summary(
    iteration_results: &[IterationResult],
    _final_validation: &ValidationResult,
    _context: &RefactorContext,
) -> Result<RefactoringSummary> {
    let total_successful_requests = iteration_results
        .iter()
        .map(|r| r.successful_requests.len())
        .sum::<usize>();

    let total_failed_requests = iteration_results
        .iter()
        .map(|r| r.failed_requests.len())
        .sum::<usize>();

    let total_quality_score = iteration_results
        .iter()
        .map(|r| r.quality_improvement.overall_score)
        .sum::<f64>();

    let total_complexity_reduced = iteration_results
        .iter()
        .map(|r| r.quality_improvement.complexity_reduced)
        .sum::<u32>();

    let total_violations_fixed = iteration_results
        .iter()
        .map(|r| r.quality_improvement.violations_fixed)
        .sum::<u32>();

    let total_satd_resolved = iteration_results
        .iter()
        .map(|r| r.quality_improvement.satd_resolved)
        .sum::<u32>();

    let total_coverage_increased = iteration_results
        .iter()
        .map(|r| r.quality_improvement.coverage_increased)
        .sum::<f64>();

    Ok(RefactoringSummary {
        total_successful_requests,
        total_failed_requests,
        total_quality_score,
        total_complexity_reduced,
        total_violations_fixed,
        total_satd_resolved,
        total_coverage_increased,
    })
}
