//! Helper functions for TDG analysis to reduce complexity

use crate::models::tdg::{TDGHotspot, TDGSummary};
use anyhow::Result;
use std::fmt::Write;
use std::path::Path;

/// Filter TDG hotspots based on criteria
pub fn filter_tdg_hotspots(
    mut hotspots: Vec<TDGHotspot>,
    threshold: f64,
    top: usize,
    critical_only: bool,
) -> Vec<TDGHotspot> {
    // Apply threshold filter
    if threshold > 0.0 {
        hotspots.retain(|h| h.tdg_score >= threshold);
    }

    // Apply critical filter
    if critical_only {
        hotspots.retain(|h| h.tdg_score > 2.5);
    }

    // Apply top limit
    if top > 0 && hotspots.len() > top {
        hotspots.truncate(top);
    }

    hotspots
}

/// Format TDG results as JSON
pub fn format_tdg_json(
    summary: &TDGSummary,
    hotspots: &[TDGHotspot],
    include_components: bool,
) -> Result<String> {
    let mut json_data = serde_json::json!({
        "summary": {
            "total_files": summary.total_files,
            "critical_files": summary.critical_files,
            "warning_files": summary.warning_files,
            "average_tdg": summary.average_tdg,
            "p95_tdg": summary.p95_tdg,
            "p99_tdg": summary.p99_tdg,
            "estimated_debt_hours": summary.estimated_debt_hours,
        },
        "hotspots": hotspots,
    });

    if include_components {
        // Add component breakdown if requested
        json_data["components"] = serde_json::json!({
            "complexity_weight": 0.4,
            "churn_weight": 0.3,
            "duplication_weight": 0.2,
            "coupling_weight": 0.1,
        });
    }

    serde_json::to_string_pretty(&json_data).map_err(Into::into)
}

/// Format TDG results as table
pub fn format_tdg_table(hotspots: &[TDGHotspot], verbose: bool) -> Result<String> {
    let mut output = String::new();

    writeln!(
        &mut output,
        "| File | TDG Score | Primary Factor | Est. Hours |"
    )?;
    writeln!(
        &mut output,
        "|------|-----------|----------------|-----------|"
    )?;

    for hotspot in hotspots {
        writeln!(
            &mut output,
            "| {} | {:.2} | {} | {:.1} |",
            std::path::Path::new(&hotspot.path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            hotspot.tdg_score,
            hotspot.primary_factor,
            hotspot.estimated_hours
        )?;

        if verbose {
            writeln!(
                &mut output,
                "|      | Components: C={:.2} Ch={:.2} D={:.2} Co={:.2} |",
                hotspot.tdg_score * 0.4, // Complexity component
                hotspot.tdg_score * 0.3, // Churn component
                hotspot.tdg_score * 0.2, // Duplication component
                hotspot.tdg_score * 0.1, // Coupling component
            )?;
        }
    }

    Ok(output)
}

/// Format TDG results as markdown
pub fn format_tdg_markdown(
    summary: &TDGSummary,
    hotspots: &[TDGHotspot],
    include_components: bool,
) -> Result<String> {
    let mut output = String::new();

    writeln!(&mut output, "# Technical Debt Gradient Analysis\n")?;

    // Summary section
    writeln!(&mut output, "## Summary\n")?;
    writeln!(&mut output, "- **Total Files**: {}", summary.total_files)?;
    writeln!(
        &mut output,
        "- **Critical Files**: {} (TDG > 2.5)",
        summary.critical_files
    )?;
    writeln!(
        &mut output,
        "- **Warning Files**: {} (TDG > 1.5)",
        summary.warning_files
    )?;
    writeln!(&mut output, "- **Average TDG**: {:.3}", summary.average_tdg)?;
    writeln!(&mut output, "- **95th Percentile**: {:.3}", summary.p95_tdg)?;
    writeln!(
        &mut output,
        "- **Estimated Debt**: {:.1} hours\n",
        summary.estimated_debt_hours
    )?;

    // Hotspots section
    if !hotspots.is_empty() {
        writeln!(&mut output, "## Top Hotspots\n")?;

        for (i, hotspot) in hotspots.iter().enumerate() {
            writeln!(&mut output, "### {}. {}\n", i + 1, hotspot.path)?;
            writeln!(&mut output, "- **TDG Score**: {:.3}", hotspot.tdg_score)?;
            writeln!(
                &mut output,
                "- **Primary Factor**: {}",
                hotspot.primary_factor
            )?;
            writeln!(
                &mut output,
                "- **Estimated Hours**: {:.1}\n",
                hotspot.estimated_hours
            )?;

            if include_components {
                writeln!(&mut output, "#### Component Breakdown:")?;
                writeln!(&mut output, "- Complexity: {:.3}", hotspot.tdg_score * 0.4)?;
                writeln!(&mut output, "- Churn: {:.3}", hotspot.tdg_score * 0.3)?;
                writeln!(&mut output, "- Duplication: {:.3}", hotspot.tdg_score * 0.2)?;
                writeln!(&mut output, "- Coupling: {:.3}\n", hotspot.tdg_score * 0.1)?;
            }
        }
    }

    Ok(output)
}

/// Format TDG results as SARIF
pub fn format_tdg_sarif(hotspots: &[TDGHotspot], project_path: &Path) -> Result<String> {
    let mut results = Vec::new();

    for hotspot in hotspots {
        let level = if hotspot.tdg_score > 2.5 {
            "error"
        } else if hotspot.tdg_score > 1.5 {
            "warning"
        } else {
            "note"
        };

        let rule_id = if hotspot.tdg_score > 2.5 {
            "critical-tdg"
        } else if hotspot.tdg_score > 1.5 {
            "high-tdg"
        } else {
            "moderate-tdg"
        };

        results.push(serde_json::json!({
            "ruleId": rule_id,
            "level": level,
            "message": {
                "text": format!(
                    "File has TDG score of {:.2} ({}). Estimated refactoring time: {:.1} hours",
                    hotspot.tdg_score,
                    hotspot.primary_factor,
                    hotspot.estimated_hours
                )
            },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": std::path::Path::new(&hotspot.path)
                            .strip_prefix(project_path)
                            .unwrap_or(std::path::Path::new(&hotspot.path))
                            .to_string_lossy()
                    }
                }
            }]
        }));
    }

    let sarif = serde_json::json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "paiml-tdg-analyzer",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": generate_tdg_rules(),
                }
            },
            "results": results
        }]
    });

    serde_json::to_string_pretty(&sarif).map_err(Into::into)
}

/// Generate SARIF rules for TDG
fn generate_tdg_rules() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "id": "critical-tdg",
            "name": "Critical Technical Debt",
            "shortDescription": {
                "text": "File has critical technical debt gradient"
            },
            "fullDescription": {
                "text": "Files with TDG > 2.5 require immediate refactoring"
            },
            "defaultConfiguration": {
                "level": "error"
            }
        }),
        serde_json::json!({
            "id": "high-tdg",
            "name": "High Technical Debt",
            "shortDescription": {
                "text": "File has high technical debt gradient"
            },
            "fullDescription": {
                "text": "Files with TDG > 1.5 should be refactored soon"
            },
            "defaultConfiguration": {
                "level": "warning"
            }
        }),
        serde_json::json!({
            "id": "moderate-tdg",
            "name": "Moderate Technical Debt",
            "shortDescription": {
                "text": "File has moderate technical debt gradient"
            },
            "fullDescription": {
                "text": "Files with TDG > 1.0 should be monitored"
            },
            "defaultConfiguration": {
                "level": "note"
            }
        }),
    ]
}
