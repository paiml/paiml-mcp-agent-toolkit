#![cfg_attr(coverage_nightly, coverage(off))]
//! Output formatting functions for Big-O complexity analysis

use crate::cli::BigOOutputFormat;
use crate::models::complexity_bound::BigOClass;
use crate::services::big_o_analyzer::{BigOAnalysisReport, BigOAnalyzer};
use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

/// Format analysis output
pub(super) fn format_analysis_output(
    analyzer: &BigOAnalyzer,
    report: &BigOAnalysisReport,
    format: BigOOutputFormat,
) -> Result<String> {
    match format {
        BigOOutputFormat::Json => analyzer.format_as_json(report),
        BigOOutputFormat::Markdown => Ok(analyzer.format_as_markdown(report)),
        BigOOutputFormat::Summary => Ok(format_big_o_summary(report)),
        BigOOutputFormat::Detailed => Ok(format_big_o_detailed(report)),
    }
}

/// Write analysis output to file or stdout
pub(super) async fn write_analysis_output(content: &str, output: Option<PathBuf>) -> Result<()> {
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, content).await?;
        info!("📄 Big-O analysis saved to: {}", output_path.display());
    } else {
        println!("{content}");
    }
    Ok(())
}

/// Format Big-O report as summary with top files
///
/// # Examples
///
/// ```no_run
/// use pmat::cli::handlers::big_o_handlers::format_big_o_summary;
/// use pmat::services::big_o_analyzer::{BigOAnalysisReport, FunctionComplexity};
/// use pmat::models::complexity_bound::{ComplexityBound, BigOClass};
/// use std::path::PathBuf;
///
/// let report = BigOAnalysisReport {
///     analyzed_functions: 100,
///     high_complexity_functions: vec![
///         FunctionComplexity {
///             function_name: "sort_data".to_string(),
///             file_path: PathBuf::from("src/utils.rs"),
///             line_number: 42,
///             time_complexity: ComplexityBound::quadratic().with_confidence(90),
///             space_complexity: ComplexityBound::linear().with_confidence(85),
///             confidence: 90,
///             notes: vec![],
///         },
///     ],
///     complexity_distribution: pmat::services::big_o_analyzer::ComplexityDistribution {
///         constant: 20,
///         logarithmic: 10,
///         linear: 50,
///         linearithmic: 5,
///         quadratic: 10,
///         cubic: 2,
///         exponential: 1,
///         unknown: 2,
///     },
///     pattern_matches: vec![],
///     recommendations: vec!["Consider optimizing quadratic algorithms".to_string()],
/// };
///
/// let output = format_big_o_summary(&report);
/// assert!(output.contains("Top Files by Complexity"));
/// assert!(output.contains("utils.rs"));
/// ```ignore
#[must_use]
pub fn format_big_o_summary(report: &BigOAnalysisReport) -> String {
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

    // Show top files by complexity
    if !report.high_complexity_functions.is_empty() {
        output.push_str("\nTop Files by Complexity:\n");

        // Group functions by file
        use std::collections::HashMap;
        let mut file_scores: HashMap<&std::path::Path, f64> = HashMap::new();
        let mut file_function_counts: HashMap<&std::path::Path, usize> = HashMap::new();

        for func in &report.high_complexity_functions {
            let score = match func.time_complexity.class {
                BigOClass::Constant => 1.0,
                BigOClass::Logarithmic => 2.0,
                BigOClass::Linear => 3.0,
                BigOClass::Linearithmic => 4.0,
                BigOClass::Quadratic => 5.0,
                BigOClass::Cubic => 6.0,
                BigOClass::Exponential => 7.0,
                BigOClass::Factorial => 8.0,
                BigOClass::Unknown => 3.0,
            };
            *file_scores.entry(&func.file_path).or_insert(0.0) += score;
            *file_function_counts.entry(&func.file_path).or_insert(0) += 1;
        }

        // Sort files by total complexity score
        let mut sorted_files: Vec<_> = file_scores.into_iter().collect();
        sorted_files.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Display top 10 files
        for (i, (file_path, score)) in sorted_files.iter().take(10).enumerate() {
            let filename = file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file_path.to_str().unwrap_or("unknown"));
            let function_count = file_function_counts.get(file_path).unwrap_or(&0);
            output.push_str(&format!(
                "  {}. {} - score: {:.1}, {} functions\n",
                i + 1,
                filename,
                score,
                function_count
            ));
        }
    }

    output
}

/// Format Big-O report with detailed information
pub(super) fn format_big_o_detailed(report: &BigOAnalysisReport) -> String {
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
