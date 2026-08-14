#![cfg_attr(coverage_nightly, coverage(off))]

use crate::cli::ComprehensiveOutputFormat;
use crate::services::facades::analysis_orchestrator::ComprehensiveAnalysisResult;
use anyhow::Result;
use std::path::PathBuf;

/// `--top-files N`, the one implementation, aliased for the six lists below.
///
/// Every one of them was a hardcoded `.iter().take(5)`, so `--top-files 1` and
/// `--top-files 50` printed the same five "Top Complexity Violations", five
/// "Dead Code Items" and five "SATD Violations" over a corpus with 87, 45 and
/// 63 of them respectively — the flag reached `DefectPredictionRequest` and
/// nothing else, and comprehensive discards those predictions.
use crate::cli::top_files_slice as top_slice;

/// Output results in the requested format
pub(super) async fn output_results(
    result: ComprehensiveAnalysisResult,
    format: ComprehensiveOutputFormat,
    executive_summary: bool,
    output: Option<PathBuf>,
    top_files: usize,
) -> Result<()> {
    let content = format_result(result, format, executive_summary, top_files)?;

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
    top_files: usize,
) -> Result<String> {
    match format {
        ComprehensiveOutputFormat::Json => format_as_json(&result),
        ComprehensiveOutputFormat::Markdown => {
            format_as_markdown(&result, executive_summary, top_files)
        }
        // SARIF stays complete: it is the format a code scanner ingests, and a
        // row limit there hides real findings from the tool meant to surface
        // them. JSON likewise carries the whole result set.
        ComprehensiveOutputFormat::Sarif => format_as_sarif(&result),
        ComprehensiveOutputFormat::Summary => format_as_text(&result, true, top_files),
        ComprehensiveOutputFormat::Detailed => format_as_text(&result, false, top_files),
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
    top_files: usize,
) -> Result<String> {
    use std::fmt::Write;

    let mut output = String::new();
    writeln!(&mut output, "# Comprehensive Code Analysis Report\n")?;

    if executive_summary {
        format_executive_summary(&mut output, &result.summary)?;
    }

    // Delegate each section to specialized functions
    if let Some(complexity) = &result.complexity {
        format_complexity_section(&mut output, complexity, top_files)?;
    }

    if let Some(dead_code) = &result.dead_code {
        format_dead_code_section(&mut output, dead_code, top_files)?;
    }

    if let Some(satd) = &result.satd {
        format_satd_section(&mut output, satd, top_files)?;
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
    top_files: usize,
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
        for (i, violation) in top_slice(&complexity.violations, top_files)
            .iter()
            .enumerate()
        {
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
    top_files: usize,
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
        for (i, item) in top_slice(&dead_code.dead_items, top_files)
            .iter()
            .enumerate()
        {
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
    top_files: usize,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "## Technical Debt (SATD) Analysis\n")?;
    writeln!(output, "- **Files Analyzed**: {}", satd.total_files)?;
    writeln!(output, "- **Violations**: {}", satd.violations.len())?;

    if !satd.violations.is_empty() {
        writeln!(output, "\n### SATD Violations\n")?;
        for (i, violation) in top_slice(&satd.violations, top_files).iter().enumerate() {
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

/// Format as colorized text for terminal output (Summary / Detailed)
pub(super) fn format_as_text(
    result: &ComprehensiveAnalysisResult,
    executive_summary: bool,
    top_files: usize,
) -> Result<String> {
    use crate::cli::colors as c;
    use std::fmt::Write;

    let mut output = String::new();
    writeln!(
        &mut output,
        "{}",
        c::header("Comprehensive Code Analysis Report")
    )?;
    writeln!(&mut output)?;

    if executive_summary {
        writeln!(&mut output, "{}\n", c::subheader("Executive Summary"))?;
        writeln!(
            &mut output,
            "  Project analysis completed with {} total files analyzed.\n",
            c::number(&result.summary.total_files.to_string())
        )?;
        writeln!(
            &mut output,
            "  Quality Score: {}",
            c::pct(result.summary.quality_score, 80.0, 50.0)
        )?;
        writeln!(
            &mut output,
            "  Total Files:   {}",
            c::number(&result.summary.total_files.to_string())
        )?;
        writeln!(
            &mut output,
            "  Total Issues:  {}",
            c::number(&result.summary.total_issues.to_string())
        )?;
        writeln!(
            &mut output,
            "  Critical:      {}",
            if result.summary.critical_issues > 0 {
                format!(
                    "{}{}{}",
                    c::BOLD_RED,
                    result.summary.critical_issues,
                    c::RESET
                )
            } else {
                c::number(&result.summary.critical_issues.to_string())
            }
        )?;
        writeln!(&mut output)?;
        if !result.summary.recommendations.is_empty() {
            writeln!(
                &mut output,
                "  {}Key Recommendations{}\n",
                c::BOLD,
                c::RESET
            )?;
            for rec in &result.summary.recommendations {
                writeln!(&mut output, "    {} {rec}", c::dim("-"))?;
            }
            writeln!(&mut output)?;
        }
    }

    if let Some(complexity) = &result.complexity {
        writeln!(&mut output, "{}\n", c::subheader("Complexity Analysis"))?;
        writeln!(
            &mut output,
            "  Files Analyzed:     {}",
            c::number(&complexity.total_files.to_string())
        )?;
        writeln!(
            &mut output,
            "  Average Complexity: {}",
            c::number(&format!("{:.1}", complexity.average_complexity))
        )?;
        writeln!(
            &mut output,
            "  Max Complexity:     {}",
            c::number(&complexity.max_complexity.to_string())
        )?;
        writeln!(
            &mut output,
            "  Violations:         {}",
            if complexity.violations.is_empty() {
                c::number("0")
            } else {
                format!("{}{}{}", c::YELLOW, complexity.violations.len(), c::RESET)
            }
        )?;
        if !complexity.violations.is_empty() {
            writeln!(
                &mut output,
                "\n  {}Top Complexity Violations{}\n",
                c::BOLD,
                c::RESET
            )?;
            for (i, v) in top_slice(&complexity.violations, top_files)
                .iter()
                .enumerate()
            {
                writeln!(
                    &mut output,
                    "    {}. {} - {} (complexity: {})",
                    c::number(&(i + 1).to_string()),
                    c::path(&v.file_path),
                    c::label(&v.function_name),
                    c::number(&v.complexity.to_string())
                )?;
            }
        }
        writeln!(&mut output)?;
    }

    if let Some(dead_code) = &result.dead_code {
        writeln!(&mut output, "{}\n", c::subheader("Dead Code Analysis"))?;
        writeln!(
            &mut output,
            "  Files Analyzed: {}",
            c::number(&dead_code.total_files.to_string())
        )?;
        writeln!(
            &mut output,
            "  Dead Items:     {}",
            c::number(&dead_code.dead_items.len().to_string())
        )?;
        writeln!(
            &mut output,
            "  Dead Code:      {}",
            c::pct(dead_code.dead_percentage, 5.0, 15.0)
        )?;
        if !dead_code.dead_items.is_empty() {
            writeln!(&mut output, "\n  {}Dead Code Items{}\n", c::BOLD, c::RESET)?;
            for (i, item) in top_slice(&dead_code.dead_items, top_files)
                .iter()
                .enumerate()
            {
                writeln!(
                    &mut output,
                    "    {}. {} - {} ({:?})",
                    c::number(&(i + 1).to_string()),
                    c::path(&item.file_path),
                    c::label(&item.item_name),
                    item.item_type
                )?;
            }
        }
        writeln!(&mut output)?;
    }

    if let Some(satd) = &result.satd {
        writeln!(
            &mut output,
            "{}\n",
            c::subheader("Technical Debt (SATD) Analysis")
        )?;
        writeln!(
            &mut output,
            "  Files Analyzed: {}",
            c::number(&satd.total_files.to_string())
        )?;
        writeln!(
            &mut output,
            "  Violations:     {}",
            if satd.violations.is_empty() {
                c::number("0")
            } else {
                format!("{}{}{}", c::YELLOW, satd.violations.len(), c::RESET)
            }
        )?;
        if !satd.violations.is_empty() {
            writeln!(&mut output, "\n  {}SATD Violations{}\n", c::BOLD, c::RESET)?;
            for (i, v) in top_slice(&satd.violations, top_files).iter().enumerate() {
                writeln!(
                    &mut output,
                    "    {}. {}:{} - {} ({:?})",
                    c::number(&(i + 1).to_string()),
                    c::path(&v.file_path),
                    c::number(&v.line_number.to_string()),
                    v.violation_type,
                    v.severity
                )?;
            }
        }
        writeln!(&mut output)?;
    }

    Ok(output)
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
