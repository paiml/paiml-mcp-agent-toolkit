#![cfg_attr(coverage_nightly, coverage(off))]

use crate::cli::ComprehensiveOutputFormat;
use crate::services::facades::analysis_orchestrator::ComprehensiveAnalysisResult;
use anyhow::Result;
use std::path::PathBuf;

/// Output results in the requested format
pub(super) async fn output_results(
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
        println!("{content}");
    }

    Ok(())
}

/// Format the analysis result
pub(super) fn format_result(
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
pub(super) fn format_as_json(result: &ComprehensiveAnalysisResult) -> Result<String> {
    serde_json::to_string_pretty(result).map_err(Into::into)
}

/// Format as Markdown
pub(super) fn format_as_markdown(
    result: &ComprehensiveAnalysisResult,
    executive_summary: bool,
) -> Result<String> {
    use std::fmt::Write;

    let mut output = String::new();
    writeln!(&mut output, "# Comprehensive Code Analysis Report\n")?;

    if executive_summary {
        format_executive_summary(&mut output, &result.summary)?;
    }

    // Delegate each section to specialized functions
    if let Some(complexity) = &result.complexity {
        format_complexity_section(&mut output, complexity)?;
    }

    if let Some(dead_code) = &result.dead_code {
        format_dead_code_section(&mut output, dead_code)?;
    }

    if let Some(satd) = &result.satd {
        format_satd_section(&mut output, satd)?;
    }

    Ok(output)
}

// Helper functions to reduce complexity below 20

pub(super) fn format_executive_summary(
    output: &mut String,
    summary: &crate::services::facades::analysis_orchestrator::AnalysisSummary,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "## Executive Summary\n")?;
    writeln!(
        output,
        "Project analysis completed with {} total files analyzed.\n",
        summary.total_files
    )?;

    writeln!(output, "- **Quality Score**: {:.1}%", summary.quality_score)?;
    writeln!(output, "- **Total Files**: {}", summary.total_files)?;
    writeln!(output, "- **Total Issues**: {}", summary.total_issues)?;
    writeln!(output, "- **Critical Issues**: {}", summary.critical_issues)?;
    writeln!(output)?;

    if !summary.recommendations.is_empty() {
        writeln!(output, "### Key Recommendations\n")?;
        for rec in &summary.recommendations {
            writeln!(output, "- {rec}")?;
        }
        writeln!(output)?;
    }

    Ok(())
}

pub(super) fn format_complexity_section(
    output: &mut String,
    complexity: &crate::services::facades::complexity_facade::ComplexityAnalysisResult,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "## Complexity Analysis\n")?;
    writeln!(output, "- **Files Analyzed**: {}", complexity.total_files)?;
    writeln!(
        output,
        "- **Average Complexity**: {:.1}",
        complexity.average_complexity
    )?;
    writeln!(
        output,
        "- **Max Complexity**: {}",
        complexity.max_complexity
    )?;
    writeln!(output, "- **Violations**: {}", complexity.violations.len())?;

    if !complexity.violations.is_empty() {
        writeln!(output, "\n### Top Complexity Violations\n")?;
        for (i, violation) in complexity.violations.iter().take(5).enumerate() {
            writeln!(
                output,
                "{}. {} - {} (complexity: {})",
                i + 1,
                violation.file_path,
                violation.function_name,
                violation.complexity
            )?;
        }
    }
    writeln!(output)?;
    Ok(())
}

pub(super) fn format_dead_code_section(
    output: &mut String,
    dead_code: &crate::services::facades::dead_code_facade::DeadCodeAnalysisResult,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "## Dead Code Analysis\n")?;
    writeln!(output, "- **Files Analyzed**: {}", dead_code.total_files)?;
    writeln!(output, "- **Dead Items**: {}", dead_code.dead_items.len())?;
    writeln!(
        output,
        "- **Dead Code %**: {:.1}%",
        dead_code.dead_percentage
    )?;

    if !dead_code.dead_items.is_empty() {
        writeln!(output, "\n### Dead Code Items\n")?;
        for (i, item) in dead_code.dead_items.iter().take(5).enumerate() {
            writeln!(
                output,
                "{}. {} - {} ({:?})",
                i + 1,
                item.file_path,
                item.item_name,
                item.item_type
            )?;
        }
    }
    writeln!(output)?;
    Ok(())
}

pub(super) fn format_satd_section(
    output: &mut String,
    satd: &crate::services::facades::satd_facade::SatdAnalysisResult,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "## Technical Debt (SATD) Analysis\n")?;
    writeln!(output, "- **Files Analyzed**: {}", satd.total_files)?;
    writeln!(output, "- **Violations**: {}", satd.violations.len())?;

    if !satd.violations.is_empty() {
        writeln!(output, "\n### SATD Violations\n")?;
        for (i, violation) in satd.violations.iter().take(5).enumerate() {
            writeln!(
                output,
                "{}. {}:{} - {} ({:?})",
                i + 1,
                violation.file_path,
                violation.line_number,
                violation.violation_type,
                violation.severity
            )?;
        }
    }
    writeln!(output)?;
    Ok(())
}

/// Format as SARIF
pub(super) fn format_as_sarif(result: &ComprehensiveAnalysisResult) -> Result<String> {
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
