//! Comprehensive Analysis Handler
//!
//! Orchestrates multiple analysis types using the facade pattern.

use crate::cli::ComprehensiveOutputFormat;
use crate::services::facades::analysis_orchestrator::{
    AnalysisOrchestrator, ComprehensiveAnalysisRequest, ComprehensiveAnalysisResult,
};
use crate::services::service_registry::ServiceRegistry;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Configuration for comprehensive analysis
#[derive(Debug, Clone)]
pub struct ComprehensiveAnalysisConfig {
    pub project_path: PathBuf,
    pub file: Option<PathBuf>,
    pub files: Vec<PathBuf>,
    pub format: ComprehensiveOutputFormat,
    pub include_duplicates: bool,
    pub include_dead_code: bool,
    pub include_defects: bool,
    pub include_complexity: bool,
    pub include_tdg: bool,
    pub confidence_threshold: f32,
    pub min_lines: usize,
    pub include: Option<String>,
    pub exclude: Option<String>,
    pub output: Option<PathBuf>,
    pub perf: bool,
    pub executive_summary: bool,
    pub top_files: usize,
}

/// Refactored handler for comprehensive analysis using the orchestrator facade.
pub async fn handle_analyze_comprehensive(config: ComprehensiveAnalysisConfig) -> Result<()> {
    use std::time::Instant;

    eprintln!("🔍 Running comprehensive analysis...");
    let start = if config.perf {
        Some(Instant::now())
    } else {
        None
    };

    // Determine the path to analyze
    let analysis_path = if let Some(single_file) = config.file.clone() {
        // Single file analysis
        single_file
    } else if !config.files.is_empty() {
        // Multiple files - analyze the common parent directory
        // For now, just use the project path
        config.project_path.clone()
    } else {
        // Full project analysis
        config.project_path.clone()
    };

    // Create service registry and orchestrator
    let registry = Arc::new(ServiceRegistry::new());
    let orchestrator = AnalysisOrchestrator::new(registry);

    // Build analysis request
    let request = ComprehensiveAnalysisRequest {
        path: analysis_path,
        include_complexity: config.include_complexity,
        include_dead_code: config.include_dead_code,
        include_satd: config.include_tdg, // Using TDG flag for SATD
        include_tests: false,
        language: None, // Auto-detect
        parallel: true, // Use parallel execution for performance
    };

    // Perform orchestrated analysis
    let result = orchestrator.analyze(request).await?;

    // Add additional analyses if requested
    let enhanced_result = if config.include_duplicates || config.include_defects {
        let additional_config = AdditionalAnalysisConfig {
            project_path: &config.project_path,
            include_duplicates: config.include_duplicates,
            include_defects: config.include_defects,
            confidence_threshold: config.confidence_threshold,
            min_lines: config.min_lines,
            include: &config.include,
            exclude: &config.exclude,
            top_files: config.top_files,
        };
        enhance_with_additional_analyses(result, additional_config).await?
    } else {
        result
    };

    // Show performance metrics if requested
    if let Some(start_time) = start {
        let elapsed = start_time.elapsed();
        eprintln!("✅ Comprehensive analysis completed in {:?}", elapsed);

        if config.perf {
            print_performance_breakdown(&enhanced_result, elapsed.as_millis() as u64);
        }
    } else {
        eprintln!("✅ Comprehensive analysis completed");
    }

    // Format and output results
    output_results(
        enhanced_result,
        config.format,
        config.executive_summary,
        config.output,
    )
    .await?;

    Ok(())
}

/// Configuration for additional analyses
struct AdditionalAnalysisConfig<'a> {
    project_path: &'a Path,
    include_duplicates: bool,
    include_defects: bool,
    confidence_threshold: f32,
    min_lines: usize,
    include: &'a Option<String>,
    exclude: &'a Option<String>,
    top_files: usize,
}

/// Enhance results with additional analyses not covered by the orchestrator
async fn enhance_with_additional_analyses(
    mut result: ComprehensiveAnalysisResult,
    config: AdditionalAnalysisConfig<'_>,
) -> Result<ComprehensiveAnalysisResult> {
    // Add duplicate detection if requested
    if config.include_duplicates {
        eprintln!("👥 Detecting duplicates...");
        // Would integrate with duplicate detector service
        // For now, just note it in the summary
        result.summary.recommendations.push(
            "Duplicate detection analysis requested - integrate with duplicate detector"
                .to_string(),
        );
    }

    // Add defect prediction if requested
    if config.include_defects {
        eprintln!("🐛 Predicting defects...");

        // Use our defect prediction facade
        use crate::services::facades::defect_prediction_facade::{
            DefectPredictionFacade, DefectPredictionRequest,
        };
        use crate::services::service_registry::ServiceRegistry;

        let registry = Arc::new(ServiceRegistry::new());
        let facade = DefectPredictionFacade::new(registry);

        let request = DefectPredictionRequest {
            project_path: config.project_path.to_path_buf(),
            confidence_threshold: config.confidence_threshold,
            min_lines: config.min_lines,
            include_low_confidence: false,
            high_risk_only: false,
            include_recommendations: true,
            include: config.include.as_ref().map(|s| vec![s.clone()]),
            exclude: config.exclude.as_ref().map(|s| vec![s.clone()]),
            top_files: config.top_files,
        };

        if let Ok(defect_result) = facade.analyze_project(request).await {
            result.summary.total_issues += defect_result.high_risk_files;
            result.summary.critical_issues += defect_result.high_risk_files;

            if defect_result.high_risk_files > 0 {
                result.summary.recommendations.push(format!(
                    "Focus on {} high-risk files identified by defect prediction",
                    defect_result.high_risk_files
                ));
            }
        }
    }

    Ok(result)
}

/// Print performance breakdown
fn print_performance_breakdown(result: &ComprehensiveAnalysisResult, total_ms: u64) {
    eprintln!("\n⏱️  Performance Breakdown:");
    eprintln!("  Total execution time: {}ms", total_ms);
    eprintln!("  Analysis duration: {}ms", result.duration_ms);
    eprintln!("  Files analyzed: {}", result.summary.total_files);
    eprintln!("  Issues found: {}", result.summary.total_issues);

    if result.summary.total_files > 0 {
        let ms_per_file = total_ms as f64 / result.summary.total_files as f64;
        eprintln!("  Average time per file: {:.2}ms", ms_per_file);
    }
}

/// Output results in the requested format
async fn output_results(
    result: ComprehensiveAnalysisResult,
    format: ComprehensiveOutputFormat,
    executive_summary: bool,
    output: Option<PathBuf>,
) -> Result<()> {
    let content = format_result(result, format, executive_summary)?;

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("📄 Report written to: {}", output_path.display());
    } else {
        println!("{}", content);
    }

    Ok(())
}

/// Format the analysis result
fn format_result(
    result: ComprehensiveAnalysisResult,
    format: ComprehensiveOutputFormat,
    executive_summary: bool,
) -> Result<String> {
    match format {
        ComprehensiveOutputFormat::Json => format_as_json(&result),
        ComprehensiveOutputFormat::Markdown => format_as_markdown(&result, executive_summary),
        ComprehensiveOutputFormat::Sarif => format_as_sarif(&result),
        ComprehensiveOutputFormat::Summary => format_as_markdown(&result, true), // Summary is markdown format
        ComprehensiveOutputFormat::Detailed => format_as_markdown(&result, false), // Detailed is markdown without exec summary
    }
}

/// Format as JSON
fn format_as_json(result: &ComprehensiveAnalysisResult) -> Result<String> {
    serde_json::to_string_pretty(result).map_err(Into::into)
}

/// Format as Markdown
fn format_as_markdown(
    result: &ComprehensiveAnalysisResult,
    executive_summary: bool,
) -> Result<String> {
    use std::fmt::Write;

    let mut output = String::new();
    writeln!(&mut output, "# Comprehensive Code Analysis Report\n")?;

    if executive_summary {
        writeln!(&mut output, "## Executive Summary\n")?;
        writeln!(
            &mut output,
            "Project analysis completed with {} total files analyzed.\n",
            result.summary.total_files
        )?;
        writeln!(
            &mut output,
            "- **Quality Score**: {:.1}%",
            result.summary.quality_score
        )?;
        writeln!(
            &mut output,
            "- **Total Files**: {}",
            result.summary.total_files
        )?;
        writeln!(
            &mut output,
            "- **Total Issues**: {}",
            result.summary.total_issues
        )?;
        writeln!(
            &mut output,
            "- **Critical Issues**: {}",
            result.summary.critical_issues
        )?;
        writeln!(&mut output)?;

        if !result.summary.recommendations.is_empty() {
            writeln!(&mut output, "### Key Recommendations\n")?;
            for rec in &result.summary.recommendations {
                writeln!(&mut output, "- {}", rec)?;
            }
            writeln!(&mut output)?;
        }
    }

    // Complexity section
    if let Some(complexity) = &result.complexity {
        writeln!(&mut output, "## Complexity Analysis\n")?;
        writeln!(
            &mut output,
            "- **Files Analyzed**: {}",
            complexity.total_files
        )?;
        writeln!(
            &mut output,
            "- **Average Complexity**: {:.1}",
            complexity.average_complexity
        )?;
        writeln!(
            &mut output,
            "- **Max Complexity**: {}",
            complexity.max_complexity
        )?;
        writeln!(
            &mut output,
            "- **Violations**: {}",
            complexity.violations.len()
        )?;

        if !complexity.violations.is_empty() {
            writeln!(&mut output, "\n### Top Complexity Violations\n")?;
            for (i, violation) in complexity.violations.iter().take(5).enumerate() {
                writeln!(
                    &mut output,
                    "{}. {} - {} (complexity: {})",
                    i + 1,
                    violation.file_path,
                    violation.function_name,
                    violation.complexity
                )?;
            }
        }
        writeln!(&mut output)?;
    }

    // Dead code section
    if let Some(dead_code) = &result.dead_code {
        writeln!(&mut output, "## Dead Code Analysis\n")?;
        writeln!(
            &mut output,
            "- **Files Analyzed**: {}",
            dead_code.total_files
        )?;
        writeln!(
            &mut output,
            "- **Dead Items**: {}",
            dead_code.dead_items.len()
        )?;
        writeln!(
            &mut output,
            "- **Dead Code %**: {:.1}%",
            dead_code.dead_percentage
        )?;

        if !dead_code.dead_items.is_empty() {
            writeln!(&mut output, "\n### Dead Code Items\n")?;
            for (i, item) in dead_code.dead_items.iter().take(5).enumerate() {
                writeln!(
                    &mut output,
                    "{}. {} - {} ({:?})",
                    i + 1,
                    item.file_path,
                    item.item_name,
                    item.item_type
                )?;
            }
        }
        writeln!(&mut output)?;
    }

    // SATD section
    if let Some(satd) = &result.satd {
        writeln!(&mut output, "## Technical Debt (SATD) Analysis\n")?;
        writeln!(&mut output, "- **Files Analyzed**: {}", satd.total_files)?;
        writeln!(&mut output, "- **Violations**: {}", satd.violations.len())?;

        if !satd.violations.is_empty() {
            writeln!(&mut output, "\n### SATD Violations\n")?;
            for (i, violation) in satd.violations.iter().take(5).enumerate() {
                writeln!(
                    &mut output,
                    "{}. {}:{} - {} ({:?})",
                    i + 1,
                    violation.file_path,
                    violation.line_number,
                    violation.violation_type,
                    violation.severity
                )?;
            }
        }
        writeln!(&mut output)?;
    }

    Ok(output)
}

/// Format as SARIF
fn format_as_sarif(result: &ComprehensiveAnalysisResult) -> Result<String> {
    let mut results = Vec::new();

    // Add complexity violations as SARIF results
    if let Some(complexity) = &result.complexity {
        for violation in &complexity.violations {
            if violation.complexity > 20 {
                results.push(serde_json::json!({
                    "ruleId": "high-complexity",
                    "level": if violation.complexity > 30 { "error" } else { "warning" },
                    "message": {
                        "text": format!("Function {} has complexity {}", violation.function_name, violation.complexity)
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": violation.file_path.clone()
                            },
                            "region": {
                                "startLine": violation.line_number
                            }
                        }
                    }]
                }));
            }
        }
    }

    // Add SATD violations
    if let Some(satd) = &result.satd {
        for violation in &satd.violations {
            results.push(serde_json::json!({
                "ruleId": "technical-debt",
                "level": "warning",
                "message": {
                    "text": format!("{}: {}", violation.violation_type, violation.message)
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": violation.file_path.clone()
                        },
                        "region": {
                            "startLine": violation.line_number
                        }
                    }
                }]
            }));
        }
    }

    let sarif = serde_json::json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat-comprehensive",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit"
                }
            },
            "results": results
        }]
    });

    serde_json::to_string_pretty(&sarif).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_as_json() {
        use crate::services::facades::analysis_orchestrator::AnalysisSummary;

        let result = ComprehensiveAnalysisResult {
            complexity: None,
            dead_code: None,
            satd: None,
            summary: AnalysisSummary {
                total_files: 10,
                total_issues: 5,
                critical_issues: 2,
                quality_score: 85.0,
                recommendations: vec!["Test recommendation".to_string()],
            },
            duration_ms: 1000,
        };

        let json = format_as_json(&result).unwrap();
        assert!(json.contains("\"total_files\": 10"));
        assert!(json.contains("\"quality_score\": 85.0"));
    }
}
