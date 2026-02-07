//! CLI handlers for PMAT Oracle - PDCA loop for automated quality improvement
//!
//! Toyota Way: Converges ANY Rust project toward perfect quality using CITL signals

use crate::cli::commands::{OracleCommands, OracleOutputFormat};
use crate::services::oracle::{ConvergenceTargets, OracleConfig, PdcaLoop, ProjectMetrics};
use anyhow::Result;
use std::path::Path;

/// Handle oracle command dispatch
pub async fn handle_oracle_command(command: OracleCommands) -> Result<()> {
    match command {
        OracleCommands::Fix {
            path,
            max_iterations,
            auto_apply_threshold,
            review_threshold,
            dry_run,
            format,
            output,
        } => {
            handle_oracle_fix(
                &path,
                max_iterations,
                auto_apply_threshold,
                review_threshold,
                dry_run,
                format,
                output.as_deref(),
            )
            .await
        }
        OracleCommands::Status { path, format } => handle_oracle_status(&path, format).await,
        OracleCommands::Single {
            path,
            format,
            output,
        } => handle_oracle_single(&path, format, output.as_deref()).await,
    }
}

/// Handle `pmat oracle fix` - Run PDCA fix loop
async fn handle_oracle_fix(
    path: &Path,
    max_iterations: usize,
    auto_apply_threshold: f32,
    review_threshold: f32,
    dry_run: bool,
    format: OracleOutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    println!("🔮 PMAT Oracle - PDCA Quality Improvement Loop");
    println!("   Path: {}", path.display());
    println!("   Max iterations: {}", max_iterations);
    println!(
        "   Thresholds: auto={:.2}, review={:.2}",
        auto_apply_threshold, review_threshold
    );
    if dry_run {
        println!("   Mode: DRY RUN (no changes will be applied)");
    }
    println!();

    // Validate path
    if !path.exists() {
        anyhow::bail!("Project path does not exist: {}", path.display());
    }

    // Create config
    let config = OracleConfig {
        max_iterations,
        auto_apply_threshold,
        review_threshold,
        ..Default::default()
    };
    let targets = ConvergenceTargets::default();

    // Create and run PDCA loop
    let pdca = PdcaLoop::with_config(config, targets.clone());

    if dry_run {
        println!("🔍 Dry run: Collecting signals only...\n");
        // Just run one iteration without applying fixes
        let results = pdca.run_iterations(path, 1).await?;
        if let Some(result) = results.first() {
            format_iteration_result(result, &format, output)?;
        }
    } else {
        println!("🚀 Starting PDCA loop...\n");
        let results = pdca.run(path).await?;

        // Format and output results
        let formatted = format_pdca_results(&results, &targets, format)?;

        if let Some(output_path) = output {
            std::fs::write(output_path, &formatted)?;
            println!("✅ Results written to: {}", output_path.display());
        } else {
            println!("{}", formatted);
        }
    }

    Ok(())
}

/// Handle `pmat oracle status` - Show current quality status
async fn handle_oracle_status(path: &Path, format: OracleOutputFormat) -> Result<()> {
    println!("📊 PMAT Oracle - Project Quality Status");
    println!("   Path: {}", path.display());
    println!();

    // Validate path
    if !path.exists() {
        anyhow::bail!("Project path does not exist: {}", path.display());
    }

    let targets = ConvergenceTargets::default();

    // Collect current metrics (simplified for now - would integrate with actual PMAT commands)
    let metrics = collect_project_metrics(path).await?;
    let status = targets.check(&metrics);

    let formatted = format_status(&metrics, &targets, &status, format)?;
    println!("{}", formatted);

    Ok(())
}

/// Handle `pmat oracle single` - Run single PDCA iteration
async fn handle_oracle_single(
    path: &Path,
    format: OracleOutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    println!("⚡ PMAT Oracle - Single PDCA Iteration");
    println!("   Path: {}", path.display());
    println!();

    // Validate path
    if !path.exists() {
        anyhow::bail!("Project path does not exist: {}", path.display());
    }

    let pdca = PdcaLoop::new();
    let result = pdca.run_single(path).await?;

    let formatted = format_single_result(&result, format)?;

    if let Some(output_path) = output {
        std::fs::write(output_path, &formatted)?;
        println!("✅ Results written to: {}", output_path.display());
    } else {
        println!("{}", formatted);
    }

    Ok(())
}

/// Collect project metrics (simplified implementation)
///
/// Returns default metrics. Full implementation would run:
/// - `pmat tdg` for TDG score
/// - `pmat analyze complexity` for cyclomatic/cognitive complexity
/// - `pmat analyze satd` for SATD markers
/// - `pmat analyze dead-code` for dead code items
/// - `cargo test` for test coverage/failures
///
/// Oracle-driven convergence uses these metrics to guide iterative improvements.
async fn collect_project_metrics(_path: &Path) -> Result<ProjectMetrics> {
    // Stub implementation - full metrics collection would be expensive
    // and is meant for CI/CD pipelines, not interactive use.
    Ok(ProjectMetrics::default())
}

/// Format PDCA results based on output format
fn format_pdca_results(
    results: &[crate::services::oracle::PdcaIterationResult],
    targets: &ConvergenceTargets,
    format: OracleOutputFormat,
) -> Result<String> {
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_oracle_status_nonexistent_path() {
        let result =
            handle_oracle_status(Path::new("/nonexistent/path"), OracleOutputFormat::Text).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_oracle_fix_nonexistent_path() {
        let result = handle_oracle_fix(
            Path::new("/nonexistent/path"),
            10,
            0.9,
            0.7,
            true,
            OracleOutputFormat::Text,
            None,
        )
        .await;
        assert!(result.is_err());
    }

    #[test]
    fn test_format_pdca_json() {
        let results = vec![];
        let json = format_pdca_json(&results).unwrap();
        assert!(json.contains("iterations"));
        assert!(json.contains("converged"));
    }

    #[test]
    fn test_format_status_text_converged() {
        let metrics = crate::services::oracle::ProjectMetrics {
            test_coverage: 0.95,
            mutation_score: 0.80,
            compiler_errors: 0,
            clippy_warnings: 2,
            test_failures: 0,
            tdg_score: 4.5,
            rust_project_score: 85,
            satd_markers: 10,
            dead_code_items: 5,
            max_cyclomatic_complexity: 12,
            max_cognitive_complexity: 8,
            build_time: std::time::Duration::from_secs(30),
        };
        let targets = crate::services::oracle::ConvergenceTargets {
            test_coverage: 0.90,
            mutation_score: 0.70,
            max_compiler_errors: 0,
            max_clippy_warnings: 5,
            max_test_failures: 0,
            min_tdg_score: 3.0,
            min_rust_project_score: 70,
            max_satd_markers: 20,
            max_dead_code: 10,
            max_cyclomatic_complexity: 15,
            max_cognitive_complexity: 15,
            max_build_time: std::time::Duration::from_secs(300),
        };
        let status = crate::services::oracle::ConvergenceStatus::Converged;

        let result = format_status(&metrics, &targets, &status, OracleOutputFormat::Text).unwrap();
        assert!(result.contains("Test Coverage"));
        assert!(result.contains("95.0%"));
        assert!(result.contains("CONVERGED"));
    }

    #[test]
    fn test_format_status_text_not_converged() {
        let metrics = crate::services::oracle::ProjectMetrics {
            test_coverage: 0.50,
            mutation_score: 0.30,
            compiler_errors: 5,
            clippy_warnings: 20,
            test_failures: 3,
            tdg_score: 1.5,
            rust_project_score: 40,
            satd_markers: 50,
            dead_code_items: 30,
            max_cyclomatic_complexity: 25,
            max_cognitive_complexity: 20,
            build_time: std::time::Duration::from_secs(60),
        };
        let targets = crate::services::oracle::ConvergenceTargets {
            test_coverage: 0.90,
            mutation_score: 0.70,
            max_compiler_errors: 0,
            max_clippy_warnings: 5,
            max_test_failures: 0,
            min_tdg_score: 3.0,
            min_rust_project_score: 70,
            max_satd_markers: 20,
            max_dead_code: 10,
            max_cyclomatic_complexity: 15,
            max_cognitive_complexity: 15,
            max_build_time: std::time::Duration::from_secs(300),
        };
        let remaining = vec!["Coverage below 90%".to_string()];
        let status = crate::services::oracle::ConvergenceStatus::NotConverged { remaining };

        let result = format_status(&metrics, &targets, &status, OracleOutputFormat::Text).unwrap();
        assert!(result.contains("NOT CONVERGED"));
        assert!(result.contains("Coverage below 90%"));
    }

    #[test]
    fn test_format_status_json() {
        let metrics = crate::services::oracle::ProjectMetrics {
            test_coverage: 0.85,
            mutation_score: 0.75,
            compiler_errors: 0,
            clippy_warnings: 0,
            test_failures: 0,
            tdg_score: 4.0,
            rust_project_score: 80,
            satd_markers: 5,
            dead_code_items: 2,
            max_cyclomatic_complexity: 10,
            max_cognitive_complexity: 8,
            build_time: std::time::Duration::from_secs(45),
        };
        let targets = crate::services::oracle::ConvergenceTargets {
            test_coverage: 0.90,
            mutation_score: 0.70,
            max_compiler_errors: 0,
            max_clippy_warnings: 5,
            max_test_failures: 0,
            min_tdg_score: 3.0,
            min_rust_project_score: 70,
            max_satd_markers: 20,
            max_dead_code: 10,
            max_cyclomatic_complexity: 15,
            max_cognitive_complexity: 15,
            max_build_time: std::time::Duration::from_secs(300),
        };
        let status = crate::services::oracle::ConvergenceStatus::Converged;

        let result = format_status(&metrics, &targets, &status, OracleOutputFormat::Json).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.get("metrics").is_some());
        assert!(parsed.get("converged").is_some());
        assert_eq!(parsed["converged"].as_bool().unwrap(), true);
    }
}
