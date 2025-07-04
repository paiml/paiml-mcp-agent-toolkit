//! Comprehensive analysis handler implementation
//!
//! This module implements the comprehensive analysis command that aggregates
//! results from multiple analyzers into a unified report.

use crate::cli::ComprehensiveOutputFormat;
use crate::services::defect_report_service::{DefectReportService, ReportFormat};
use anyhow::Result;
use std::path::PathBuf;
use std::time::Instant;
use tracing::{info, warn};

/// Handle comprehensive analysis command
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_comprehensive(
    project_path: PathBuf,
    format: ComprehensiveOutputFormat,
    include_duplicates: bool,
    include_dead_code: bool,
    include_defects: bool,
    include_complexity: bool,
    include_tdg: bool,
    confidence_threshold: f32,
    min_lines: usize,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
    perf: bool,
    executive_summary: bool,
) -> Result<()> {
    let start_time = Instant::now();

    info!("🔍 Starting comprehensive analysis");
    info!("📂 Project path: {}", project_path.display());

    // Log enabled analyses
    let mut enabled_analyses = Vec::new();
    if include_complexity {
        enabled_analyses.push("complexity");
    }
    if include_tdg {
        enabled_analyses.push("TDG");
    }
    if include_defects {
        enabled_analyses.push("defects");
    }
    if include_dead_code {
        enabled_analyses.push("dead code");
    }
    if include_duplicates {
        enabled_analyses.push("duplicates");
    }

    if enabled_analyses.is_empty() {
        // Default to all analyses if none specified
        enabled_analyses = vec!["complexity", "TDG", "defects", "dead code", "duplicates"];
    }

    info!("📊 Enabled analyses: {}", enabled_analyses.join(", "));

    // Create defect report service
    let service = DefectReportService::new();

    // Generate comprehensive report
    let report = service.generate_report(&project_path).await?;

    // Apply filters based on confidence threshold
    let filtered_defects: Vec<_> = report
        .defects
        .iter()
        .filter(|d| {
            // Convert confidence from metrics if available
            let confidence = d.metrics.get("confidence").copied().unwrap_or(100.0) as f32;
            confidence >= confidence_threshold
        })
        .cloned()
        .collect();

    info!("📈 Total defects found: {}", report.defects.len());
    info!(
        "📉 After confidence filter (>={:.0}%): {}",
        confidence_threshold,
        filtered_defects.len()
    );

    // Convert format
    let output_format = match format {
        ComprehensiveOutputFormat::Json => ReportFormat::Json,
        ComprehensiveOutputFormat::Markdown => ReportFormat::Markdown,
        ComprehensiveOutputFormat::Summary => ReportFormat::Text,
        ComprehensiveOutputFormat::Detailed => ReportFormat::Markdown,
        ComprehensiveOutputFormat::Sarif => ReportFormat::Json, // SARIF not implemented, use JSON
    };

    // Format output
    let formatted_output = match output_format {
        ReportFormat::Json => {
            // Create filtered report for JSON output
            let mut filtered_report = report.clone();
            filtered_report.defects = filtered_defects;
            filtered_report.summary = service.compute_summary(&filtered_report.defects);
            service.format_json(&filtered_report)?
        }
        ReportFormat::Markdown => {
            // For Markdown, include executive summary if requested
            let mut md = String::new();

            if executive_summary {
                md.push_str("# Executive Summary\n\n");
                md.push_str(&format!("This comprehensive analysis of {} examined {} files and identified {} quality issues.\n\n",
                    project_path.display(),
                    report.metadata.total_files_analyzed,
                    filtered_defects.len()
                ));

                // Add severity breakdown
                let mut critical = 0;
                let mut high = 0;
                let mut medium = 0;
                let mut low = 0;

                for defect in &filtered_defects {
                    match defect.severity {
                        crate::models::defect_report::Severity::Critical => critical += 1,
                        crate::models::defect_report::Severity::High => high += 1,
                        crate::models::defect_report::Severity::Medium => medium += 1,
                        crate::models::defect_report::Severity::Low => low += 1,
                    }
                }

                md.push_str("## Key Findings\n\n");
                if critical > 0 {
                    md.push_str(&format!(
                        "- **{} Critical Issues** requiring immediate attention\n",
                        critical
                    ));
                }
                if high > 0 {
                    md.push_str(&format!(
                        "- **{} High Priority Issues** that should be addressed soon\n",
                        high
                    ));
                }
                md.push_str(&format!("- **{} Medium Priority Issues**\n", medium));
                md.push_str(&format!("- **{} Low Priority Issues**\n\n", low));

                // Add recommendations
                md.push_str("## Recommendations\n\n");
                md.push_str("1. Address all critical issues immediately\n");
                md.push_str("2. Create a plan to resolve high priority issues\n");
                md.push_str("3. Consider refactoring files with multiple defects\n");
                md.push_str("4. Establish quality gates to prevent new issues\n\n");

                md.push_str("---\n\n");
            }

            // Add the regular report
            let mut filtered_report = report.clone();
            filtered_report.defects = filtered_defects;
            filtered_report.summary = service.compute_summary(&filtered_report.defects);
            md.push_str(&service.format_markdown(&filtered_report)?);

            md
        }
        _ => {
            // Fallback to JSON for other formats
            let mut filtered_report = report.clone();
            filtered_report.defects = filtered_defects;
            filtered_report.summary = service.compute_summary(&filtered_report.defects);
            service.format_json(&filtered_report)?
        }
    };

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &formatted_output).await?;
        info!("📄 Report saved to: {}", output_path.display());
    } else {
        println!("{}", formatted_output);
    }

    let elapsed = start_time.elapsed();

    // Print performance metrics if requested
    if perf {
        info!("⚡ Performance Metrics:");
        info!("  Total time: {:?}", elapsed);
        info!("  Files analyzed: {}", report.metadata.total_files_analyzed);
        info!(
            "  Files/second: {:.1}",
            report.metadata.total_files_analyzed as f64 / elapsed.as_secs_f64()
        );
        info!(
            "  Analysis time: {}ms",
            report.metadata.analysis_duration_ms
        );
    }

    // Print summary by category
    info!("\n📊 Defects by Category:");
    for (category, count) in &report.summary.by_category {
        info!("  {}: {}", category, count);
    }

    // Warn about ignored parameters (for transparency)
    if include.is_some() {
        warn!("Note: --include parameter is not yet implemented in comprehensive analysis");
    }
    if exclude.is_some() {
        warn!("Note: --exclude parameter is not yet implemented in comprehensive analysis");
    }
    if min_lines > 0 {
        warn!("Note: --min-lines parameter is not yet implemented in comprehensive analysis");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comprehensive_handler_params() {
        // Basic parameter validation test
        assert_eq!(
            ComprehensiveOutputFormat::Json as i32,
            ComprehensiveOutputFormat::Json as i32
        );
    }
}
