#![cfg_attr(coverage_nightly, coverage(off))]
//! Report formatting (JSON and Markdown) for SimpleDeepContext analysis reports.

use anyhow::Result;

use super::{SimpleAnalysisReport, SimpleDeepContext};

impl SimpleDeepContext {
    /// Format report as JSON
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn format_as_json(&self, report: &SimpleAnalysisReport) -> Result<String> {
        let json_report = serde_json::json!({
            "summary": {
                "file_count": report.file_count,
                "analysis_duration_ms": report.analysis_duration.as_millis(),
                "total_functions": report.complexity_metrics.total_functions,
                "high_complexity_functions": report.complexity_metrics.high_complexity_count,
                "avg_complexity": report.complexity_metrics.avg_complexity
            },
            "files": report.file_complexity_details.iter().map(|file| {
                serde_json::json!({
                    "path": file.file_path.to_string_lossy(),
                    "function_count": file.function_count,
                    "high_complexity_functions": file.high_complexity_functions,
                    "avg_complexity": file.avg_complexity,
                    "complexity_score": file.complexity_score
                })
            }).collect::<Vec<_>>(),
            "recommendations": report.recommendations
        });

        Ok(serde_json::to_string_pretty(&json_report)?)
    }

    /// Format report as Markdown
    ///
    /// # Example
    ///
    /// ```rust
    /// use pmat::services::simple_deep_context::{SimpleDeepContext, SimpleAnalysisReport, ComplexityMetrics, FileComplexityDetail};
    /// use std::path::PathBuf;
    /// use std::time::Duration;
    ///
    /// let analyzer = SimpleDeepContext::new();
    /// let report = SimpleAnalysisReport {
    ///     file_count: 5,
    ///     analysis_duration: Duration::from_millis(500),
    ///     complexity_metrics: ComplexityMetrics {
    ///         total_functions: 25,
    ///         high_complexity_count: 3,
    ///         avg_complexity: 4.2,
    ///     },
    ///     recommendations: vec!["Consider refactoring 3 high-complexity functions".to_string()],
    ///     file_complexity_details: vec![
    ///         FileComplexityDetail {
    ///             file_path: PathBuf::from("src/main.rs"),
    ///             function_count: 10,
    ///             high_complexity_functions: 2,
    ///             avg_complexity: 5.5,
    ///             complexity_score: 8.5,
    ///             function_names: vec![],
    ///         },
    ///         FileComplexityDetail {
    ///             file_path: PathBuf::from("src/lib.rs"),
    ///             function_count: 15,
    ///             high_complexity_functions: 1,
    ///             avg_complexity: 3.8,
    ///             complexity_score: 7.2,
    ///             function_names: vec![],
    ///         },
    ///     ],
    /// };
    ///
    /// let output = analyzer.format_as_markdown(&report, 10);
    ///
    /// assert!(output.contains("# Deep Context Analysis Report"));
    /// assert!(output.contains("**Files Analyzed**: 5"));
    /// assert!(output.contains("## Top Files by Complexity"));
    /// assert!(output.contains("1. `main.rs` - 5.5 avg complexity"));
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn format_as_markdown(&self, report: &SimpleAnalysisReport, top_files: usize) -> String {
        let mut markdown = String::new();

        markdown.push_str("# Deep Context Analysis Report\n\n");

        markdown.push_str("## Summary\n\n");
        markdown.push_str(&format!("- **Files Analyzed**: {}\n", report.file_count));
        markdown.push_str(&format!(
            "- **Analysis Duration**: {:?}\n",
            report.analysis_duration
        ));
        markdown.push_str(&format!(
            "- **Total Functions**: {}\n",
            report.complexity_metrics.total_functions
        ));
        markdown.push_str(&format!(
            "- **High Complexity Functions**: {}\n",
            report.complexity_metrics.high_complexity_count
        ));
        markdown.push_str(&format!(
            "- **Average Complexity**: {:.1}\n\n",
            report.complexity_metrics.avg_complexity
        ));

        // Show top files by complexity
        if !report.file_complexity_details.is_empty() {
            markdown.push_str("## Top Files by Complexity\n\n");

            // Sort files by complexity score (descending)
            let mut sorted_files = report.file_complexity_details.clone();
            sorted_files.sort_by(|a, b| {
                b.complexity_score
                    .partial_cmp(&a.complexity_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let files_to_show = if top_files == 0 { 10 } else { top_files };
            for (i, file_detail) in sorted_files.iter().take(files_to_show).enumerate() {
                let filename = file_detail
                    .file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map_or_else(
                        || file_detail.file_path.to_string_lossy().to_string(),
                        std::string::ToString::to_string,
                    );
                markdown.push_str(&format!(
                    "{}. `{}` - {:.1} avg complexity ({} functions, {} high complexity)\n",
                    i + 1,
                    filename,
                    file_detail.avg_complexity,
                    file_detail.function_count,
                    file_detail.high_complexity_functions
                ));
            }
            markdown.push('\n');
        }

        markdown.push_str("## Recommendations\n\n");
        for (i, rec) in report.recommendations.iter().enumerate() {
            markdown.push_str(&format!("{}. {}\n", i + 1, rec));
        }

        markdown
    }
}
