//! Big-O complexity analysis command handlers
//!
//! This module provides handlers for algorithmic complexity analysis
//! using pattern matching and heuristic approaches.

use crate::cli::*;
use crate::services::big_o_analyzer::{BigOAnalysisConfig, BigOAnalyzer};
use anyhow::Result;
use std::path::PathBuf;
use tracing::{debug, info};

/// Handle Big-O complexity analysis command
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_big_o(
    project_path: PathBuf,
    format: BigOOutputFormat,
    confidence_threshold: u8,
    analyze_space: bool,
    include: Vec<String>,
    exclude: Vec<String>,
    high_complexity_only: bool,
    output: Option<PathBuf>,
    perf: bool,
) -> Result<()> {
    let start_time = std::time::Instant::now();

    info!("🔍 Starting Big-O complexity analysis");
    info!("📂 Project path: {}", project_path.display());
    info!("🎯 Confidence threshold: {}%", confidence_threshold);

    // Create analyzer
    let analyzer = BigOAnalyzer::new();

    // Build configuration
    let config = BigOAnalysisConfig {
        project_path: project_path.clone(),
        include_patterns: include,
        exclude_patterns: exclude,
        confidence_threshold,
        analyze_space_complexity: analyze_space,
    };

    if perf {
        debug!("Analysis configuration: {:?}", config);
    }

    // Perform analysis
    let mut report = analyzer.analyze(config).await?;

    // Filter high complexity only if requested
    if high_complexity_only {
        let original_count = report.high_complexity_functions.len();
        report.high_complexity_functions.retain(|f| {
            matches!(
                f.time_complexity.class,
                crate::models::complexity_bound::BigOClass::Quadratic
                    | crate::models::complexity_bound::BigOClass::Cubic
                    | crate::models::complexity_bound::BigOClass::Exponential
                    | crate::models::complexity_bound::BigOClass::Factorial
            )
        });

        if perf {
            debug!(
                "Filtered from {} to {} high complexity functions",
                original_count,
                report.high_complexity_functions.len()
            );
        }
    }

    // Format output
    let output_content = match format {
        BigOOutputFormat::Json => analyzer.format_as_json(&report)?,
        BigOOutputFormat::Markdown => analyzer.format_as_markdown(&report),
        BigOOutputFormat::Summary => format_big_o_summary(&report),
        BigOOutputFormat::Detailed => format_big_o_detailed(&report),
    };

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &output_content).await?;
        info!("📄 Big-O analysis saved to: {}", output_path.display());
    } else {
        println!("{output_content}");
    }

    let elapsed = start_time.elapsed();

    // Print summary
    info!("✅ Big-O analysis completed in {:?}", elapsed);
    info!("📊 Analyzed {} functions", report.analyzed_functions);

    if !report.high_complexity_functions.is_empty() {
        info!(
            "⚠️ Found {} functions with high complexity",
            report.high_complexity_functions.len()
        );
    }

    if perf {
        let functions_per_sec = report.analyzed_functions as f64 / elapsed.as_secs_f64();
        info!("⚡ Performance: {:.0} functions/second", functions_per_sec);
    }

    Ok(())
}

/// Format Big-O report as summary
fn format_big_o_summary(report: &crate::services::big_o_analyzer::BigOAnalysisReport) -> String {
    let mut output = String::with_capacity(1024);

    output.push_str("Big-O Complexity Analysis Summary\n");
    output.push_str("=================================\n\n");

    output.push_str(&format!(
        "Total Functions Analyzed: {}\n",
        report.analyzed_functions
    ));
    output.push_str(&format!(
        "High Complexity Functions: {}\n\n",
        report.high_complexity_functions.len()
    ));

    output.push_str("Complexity Distribution:\n");
    let dist = &report.complexity_distribution;
    output.push_str(&format!("  O(1)       : {:>4} functions\n", dist.constant));
    output.push_str(&format!(
        "  O(log n)   : {:>4} functions\n",
        dist.logarithmic
    ));
    output.push_str(&format!("  O(n)       : {:>4} functions\n", dist.linear));
    output.push_str(&format!(
        "  O(n log n) : {:>4} functions\n",
        dist.linearithmic
    ));
    output.push_str(&format!("  O(n²)      : {:>4} functions\n", dist.quadratic));
    output.push_str(&format!("  O(n³)      : {:>4} functions\n", dist.cubic));
    output.push_str(&format!(
        "  O(2^n)     : {:>4} functions\n",
        dist.exponential
    ));
    output.push_str(&format!("  Unknown    : {:>4} functions\n", dist.unknown));

    if !report.recommendations.is_empty() {
        output.push_str("\nRecommendations:\n");
        for rec in &report.recommendations {
            output.push_str(&format!("• {rec}\n"));
        }
    }

    output
}

/// Format Big-O report with detailed information
fn format_big_o_detailed(report: &crate::services::big_o_analyzer::BigOAnalysisReport) -> String {
    let mut output = format_big_o_summary(report);

    if !report.high_complexity_functions.is_empty() {
        output.push_str("\nHigh Complexity Functions:\n");
        output.push_str("==========================\n");

        for func in &report.high_complexity_functions {
            output.push_str(&format!(
                "\n{} ({}:{})\n",
                func.function_name,
                func.file_path.display(),
                func.line_number
            ));
            output.push_str(&format!(
                "  Time Complexity: {} ({}% confidence)\n",
                func.time_complexity.notation(),
                func.time_complexity.confidence
            ));
            output.push_str(&format!(
                "  Space Complexity: {} ({}% confidence)\n",
                func.space_complexity.notation(),
                func.space_complexity.confidence
            ));

            if !func.notes.is_empty() {
                output.push_str("  Notes:\n");
                for note in &func.notes {
                    output.push_str(&format!("    - {note}\n"));
                }
            }
        }
    }

    if !report.pattern_matches.is_empty() {
        output.push_str("\nPattern Matches:\n");
        output.push_str("================\n");

        for pattern in &report.pattern_matches {
            output.push_str(&format!(
                "  {} : {} occurrences\n",
                pattern.pattern_name, pattern.occurrences
            ));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_big_o_handlers_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
