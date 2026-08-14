//! Coverage Improvement Handler
//!
//! CLI handler for the `pmat coverage improve` command.
//! Autonomously improves test coverage using PMAT tools and Extreme TDD.

use crate::services::coverage_improvement::{
    CoverageImprovementConfig, CoverageImprovementReport, CoverageImprovementService,
};
use anyhow::Result;
use std::path::PathBuf;

/// Output format for coverage improvement reports
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum CoverageImproveOutputFormat {
    /// Human-readable text output
    Text,
    /// JSON format
    Json,
    /// Markdown report
    Markdown,
}

/// Handle the coverage improve command
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_coverage_improve(
    project_path: PathBuf,
    target: f64,
    max_iterations: usize,
    fast: bool,
    mutation_threshold: f64,
    focus: Vec<String>,
    exclude: Vec<String>,
    output: Option<PathBuf>,
    format: CoverageImproveOutputFormat,
) -> Result<()> {
    crate::status_eprintln!("📊 PMAT Coverage Improvement");
    crate::status_eprintln!("🎯 Target: {:.1}%", target);
    crate::status_eprintln!("📁 Project: {}", project_path.display());

    // Create configuration
    let config = CoverageImprovementConfig {
        project_path,
        target_coverage: target,
        max_iterations,
        fast_mode: fast,
        mutation_threshold,
        focus_patterns: focus,
        exclude_patterns: exclude,
    };

    // Create service and run improvement
    let service = CoverageImprovementService::new(config);
    let report = service.improve_coverage().await?;

    // Format and output results
    let formatted = match format {
        CoverageImproveOutputFormat::Text => format_text(&report),
        CoverageImproveOutputFormat::Json => format_json(&report)?,
        CoverageImproveOutputFormat::Markdown => format_markdown(&report),
    };

    if let Some(output_path) = output {
        std::fs::write(&output_path, &formatted)?;
        crate::status_eprintln!("📝 Report written to: {}", output_path.display());
    } else {
        println!("{}", formatted);
    }

    // Print summary
    print_summary(&report);

    if report.success {
        Ok(())
    } else {
        anyhow::bail!("Failed to reach target coverage");
    }
}

/// Format report as human-readable text
fn format_text(report: &CoverageImprovementReport) -> String {
    let mut output = String::new();
    output.push_str("Coverage Improvement Report\n");
    output.push_str("===========================\n\n");

    output.push_str(&format!("Baseline:  {:.2}%\n", report.baseline_coverage));
    output.push_str(&format!("Target:    {:.2}%\n", report.target_coverage));
    output.push_str(&format!("Final:     {:.2}%\n", report.final_coverage));
    output.push_str(&format!(
        "Gain:      +{:.2}%\n\n",
        report.final_coverage - report.baseline_coverage
    ));

    for iteration in &report.iterations {
        output.push_str(&format!("Iteration {}:\n", iteration.iteration));
        output.push_str(&format!(
            "  Files targeted: {}\n",
            iteration.files_targeted.len()
        ));
        output.push_str(&format!(
            "  Tests generated: {}\n",
            iteration.tests_generated
        ));
        output.push_str(&format!(
            "  Coverage gain: +{:.2}%\n",
            iteration.coverage_gain
        ));
        output.push_str(&format!(
            "  Mutation score: {:.1}%\n\n",
            iteration.mutation_score
        ));
    }

    output.push_str(&format!(
        "Status: {}\n",
        if report.success {
            "SUCCESS"
        } else {
            "INCOMPLETE"
        }
    ));
    output.push_str(&format!("Reason: {}\n", report.stop_reason));

    output
}

/// Format report as JSON
fn format_json(report: &CoverageImprovementReport) -> Result<String> {
    serde_json::to_string_pretty(report).map_err(Into::into)
}

/// Format report as Markdown
fn format_markdown(report: &CoverageImprovementReport) -> String {
    let mut output = String::new();
    output.push_str("# Coverage Improvement Report\n\n");

    output.push_str("## Summary\n\n");
    output.push_str(&format!(
        "- **Baseline Coverage**: {:.2}%\n",
        report.baseline_coverage
    ));
    output.push_str(&format!(
        "- **Target Coverage**: {:.2}%\n",
        report.target_coverage
    ));
    output.push_str(&format!(
        "- **Final Coverage**: {:.2}%\n",
        report.final_coverage
    ));
    output.push_str(&format!(
        "- **Total Gain**: +{:.2}%\n\n",
        report.final_coverage - report.baseline_coverage
    ));

    output.push_str("## Iterations\n\n");
    output.push_str("| Iteration | Files | Tests | Coverage Gain | Mutation Score |\n");
    output.push_str("|-----------|-------|-------|---------------|----------------|\n");

    for iteration in &report.iterations {
        output.push_str(&format!(
            "| {} | {} | {} | +{:.2}% | {:.1}% |\n",
            iteration.iteration,
            iteration.files_targeted.len(),
            iteration.tests_generated,
            iteration.coverage_gain,
            iteration.mutation_score
        ));
    }

    output.push_str(&format!(
        "\n## Result\n\n**Status**: {}\n\n",
        if report.success {
            "✅ SUCCESS"
        } else {
            "⚠️ INCOMPLETE"
        }
    ));
    output.push_str(&format!("**Reason**: {}\n", report.stop_reason));

    output
}

/// Print summary to stderr
fn print_summary(report: &CoverageImprovementReport) {
    crate::status_eprintln!("\n📊 Summary:");
    crate::status_eprintln!("   Baseline:  {:.2}%", report.baseline_coverage);
    crate::status_eprintln!("   Final:     {:.2}%", report.final_coverage);
    crate::status_eprintln!(
        "   Gain:      +{:.2}%",
        report.final_coverage - report.baseline_coverage
    );
    crate::status_eprintln!("   Iterations: {}", report.iterations.len());

    if report.success {
        crate::status_eprintln!("✅ Target coverage reached!");
    } else {
        eprintln!("⚠️  {}", report.stop_reason);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::coverage_improvement::IterationReport;

    #[test]
    fn test_format_text() {
        let report = CoverageImprovementReport {
            baseline_coverage: 50.0,
            target_coverage: 95.0,
            final_coverage: 65.0,
            iterations: vec![IterationReport {
                iteration: 1,
                files_targeted: vec![PathBuf::from("test.rs")],
                tests_generated: 5,
                coverage_gain: 15.0,
                mutation_score: 85.0,
            }],
            success: false,
            stop_reason: "Max iterations reached".to_string(),
        };

        let text = format_text(&report);
        assert!(text.contains("50.00%"));
        assert!(text.contains("65.00%"));
        assert!(text.contains("Iteration 1"));
    }

    #[test]
    fn test_format_json() {
        let report = CoverageImprovementReport {
            baseline_coverage: 50.0,
            target_coverage: 95.0,
            final_coverage: 95.0,
            iterations: vec![],
            success: true,
            stop_reason: "Target reached".to_string(),
        };

        let json = format_json(&report).unwrap();
        assert!(json.contains("baseline_coverage"));
        assert!(json.contains("95.0"));
    }
}
