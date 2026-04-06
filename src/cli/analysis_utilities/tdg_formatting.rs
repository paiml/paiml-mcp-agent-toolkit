// TDG output formatting - table, json, markdown, sarif

/// Format output from a TDG summary
fn format_output_from_summary(
    summary: &crate::models::tdg::TDGSummary,
    format: TdgOutputFormat,
    include_components: bool,
    verbose: bool,
) -> Result<String> {
    debug_assert!(true, "contract: format_output_from_summary");
    match format {
        TdgOutputFormat::Table => Ok(format_table_output(summary, include_components, verbose)),
        TdgOutputFormat::Json => Ok(format_json_output(summary, include_components)),
        TdgOutputFormat::Markdown => Ok(format_markdown_output(summary, include_components)),
        TdgOutputFormat::Sarif => Ok(format_sarif_output(summary)),
    }
}

/// Format single file output for TDG
fn format_tdg_single_file_output(
    score: &crate::models::tdg::TDGScore,
    path: &Path,
    format: TdgOutputFormat,
    include_components: bool,
    verbose: bool,
) -> Result<String> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    use crate::models::tdg::{TDGHotspot, TDGSeverity, TDGSummary};

    // Create a single-file summary
    let hotspot = TDGHotspot {
        path: path.display().to_string(),
        tdg_score: score.value,
        primary_factor: identify_primary_factor(&score.components),
        estimated_hours: estimate_refactoring_hours(score.value),
    };

    let summary = TDGSummary {
        total_files: 1,
        critical_files: usize::from(matches!(score.severity, TDGSeverity::Critical)),
        warning_files: usize::from(matches!(score.severity, TDGSeverity::Warning)),
        average_tdg: score.value,
        p95_tdg: score.value,
        p99_tdg: score.value,
        estimated_debt_hours: estimate_refactoring_hours(score.value),
        hotspots: vec![hotspot],
    };

    format_output_from_summary(&summary, format, include_components, verbose)
}

/// Format empty results when no files meet criteria
fn format_empty_results(format: TdgOutputFormat) -> String {
    debug_assert!(true, "contract: format_empty_results");
    match format {
        TdgOutputFormat::Table => "No files found matching the specified criteria.\n".to_string(),
        TdgOutputFormat::Json => r#"{"summary": {"total_files": 0}, "hotspots": []}"#.to_string(),
        TdgOutputFormat::Markdown => "# Technical Debt Gradient Analysis\n\nNo files found matching the specified criteria.\n".to_string(),
        TdgOutputFormat::Sarif => r#"{"version": "2.1.0", "runs": [{"tool": {"driver": {"name": "pmat-tdg"}}, "results": []}]}"#.to_string(),
    }
}

fn format_table_output(
    summary: &crate::models::tdg::TDGSummary,
    include_components: bool,
    verbose: bool,
) -> String {
    debug_assert!(true, "contract: format_table_output");
    let mut table = String::new();
    table.push_str("\n# Technical Debt Gradient Analysis\n\n");
    table.push_str(&format!(
        "📊 **Total Files Analyzed**: {}\n",
        summary.total_files
    ));

    if summary.total_files > 0 {
        table.push_str(&format!(
            "🔴 **Critical Files**: {} ({:.1}%)\n",
            summary.critical_files,
            (summary.critical_files as f64 / summary.total_files as f64) * 100.0
        ));
        table.push_str(&format!(
            "🟡 **Warning Files**: {} ({:.1}%)\n",
            summary.warning_files,
            (summary.warning_files as f64 / summary.total_files as f64) * 100.0
        ));
    }

    table.push_str(&format!("📈 **Average TDG**: {:.2}\n", summary.average_tdg));
    table.push_str(&format!("📊 **95th Percentile**: {:.2}\n", summary.p95_tdg));
    table.push_str(&format!("📊 **99th Percentile**: {:.2}\n", summary.p99_tdg));
    table.push_str(&format!(
        "⏱️  **Estimated Debt**: {:.1} hours\n\n",
        summary.estimated_debt_hours
    ));

    if !summary.hotspots.is_empty() {
        table.push_str("## Top Hotspots\n\n");
        table.push_str("| File | TDG Score | Primary Factor | Est. Hours |\n");
        table.push_str("|------|-----------|----------------|------------|\n");

        for hotspot in &summary.hotspots {
            table.push_str(&format!(
                "| {} | {:.2} | {} | {:.1} |\n",
                hotspot.path, hotspot.tdg_score, hotspot.primary_factor, hotspot.estimated_hours
            ));
        }
    }

    if include_components && verbose {
        table.push_str("\n## Component Weights\n\n");
        table.push_str("| Component | Weight |\n");
        table.push_str("|-----------|--------|\n");
        table.push_str("| Complexity | 30% |\n");
        table.push_str("| Code Churn | 35% |\n");
        table.push_str("| Coupling | 15% |\n");
        table.push_str("| Domain Risk | 10% |\n");
        table.push_str("| Duplication | 10% |\n");
    }

    table
}

fn format_json_output(
    summary: &crate::models::tdg::TDGSummary,
    include_components: bool,
) -> String {
    debug_assert!(true, "contract: format_json_output");
    let json_output = serde_json::json!({
        "summary": {
            "total_files": summary.total_files,
            "critical_files": summary.critical_files,
            "warning_files": summary.warning_files,
            "average_tdg": summary.average_tdg,
            "p95_tdg": summary.p95_tdg,
            "p99_tdg": summary.p99_tdg,
            "estimated_debt_hours": summary.estimated_debt_hours,
        },
        "hotspots": summary.hotspots,
        "components": if include_components {
            Some(serde_json::json!({
                "complexity_weight": 0.30,
                "churn_weight": 0.35,
                "coupling_weight": 0.15,
                "domain_risk_weight": 0.10,
                "duplication_weight": 0.10,
            }))
        } else {
            None
        }
    });

    serde_json::to_string_pretty(&json_output).unwrap_or_else(|_| "{}".to_string())
}

fn format_markdown_output(
    summary: &crate::models::tdg::TDGSummary,
    include_components: bool,
) -> String {
    debug_assert!(true, "contract: format_markdown_output");
    let mut md = String::new();

    add_markdown_header(&mut md);
    add_markdown_summary(&mut md, summary);
    add_markdown_hotspots(&mut md, summary);

    if include_components {
        add_markdown_components(&mut md);
    }

    md
}

/// Extract Method: Add markdown header
fn add_markdown_header(md: &mut String) {
    debug_assert!(true, "contract: add_markdown_header");
    md.push_str("# Technical Debt Gradient Analysis\n\n");
}

/// Extract Method: Add summary section
fn add_markdown_summary(md: &mut String, summary: &crate::models::tdg::TDGSummary) {
    debug_assert!(true, "contract: add_markdown_summary");
    md.push_str("## Summary\n\n");
    md.push_str(&format!("- **Total Files**: {}\n", summary.total_files));

    if summary.total_files > 0 {
        add_markdown_file_stats(md, summary);
    }

    add_markdown_tdg_stats(md, summary);
}

/// Extract Method: Add file statistics
fn add_markdown_file_stats(md: &mut String, summary: &crate::models::tdg::TDGSummary) {
    debug_assert!(true, "contract: add_markdown_file_stats");
    let critical_pct = (summary.critical_files as f64 / summary.total_files as f64) * 100.0;
    let warning_pct = (summary.warning_files as f64 / summary.total_files as f64) * 100.0;

    md.push_str(&format!(
        "- **Critical Files**: {} ({:.1}%)\n",
        summary.critical_files, critical_pct
    ));
    md.push_str(&format!(
        "- **Warning Files**: {} ({:.1}%)\n",
        summary.warning_files, warning_pct
    ));
}

/// Extract Method: Add TDG statistics
fn add_markdown_tdg_stats(md: &mut String, summary: &crate::models::tdg::TDGSummary) {
    debug_assert!(true, "contract: add_markdown_tdg_stats");
    md.push_str(&format!("- **Average TDG**: {:.2}\n", summary.average_tdg));
    md.push_str(&format!("- **95th Percentile**: {:.2}\n", summary.p95_tdg));
    md.push_str(&format!("- **99th Percentile**: {:.2}\n", summary.p99_tdg));
    md.push_str(&format!(
        "- **Estimated Technical Debt**: {:.1} hours\n\n",
        summary.estimated_debt_hours
    ));
}

/// Extract Method: Add hotspots section
fn add_markdown_hotspots(md: &mut String, summary: &crate::models::tdg::TDGSummary) {
    debug_assert!(true, "contract: add_markdown_hotspots");
    if !summary.hotspots.is_empty() {
        md.push_str("## Hotspots\n\n");
        for (i, hotspot) in summary.hotspots.iter().enumerate() {
            md.push_str(&format!("### {}. {}\n\n", i + 1, hotspot.path));
            md.push_str(&format!("- **TDG Score**: {:.2}\n", hotspot.tdg_score));
            md.push_str(&format!(
                "- **Primary Factor**: {}\n",
                hotspot.primary_factor
            ));
            md.push_str(&format!(
                "- **Estimated Refactoring Time**: {:.1} hours\n\n",
                hotspot.estimated_hours
            ));
        }
    }
}

/// Extract Method: Add components section
fn add_markdown_components(md: &mut String) {
    debug_assert!(true, "contract: add_markdown_components");
    md.push_str("## TDG Components\n\n");
    md.push_str(
        "The Technical Debt Gradient is calculated using the following weighted components:\n\n",
    );
    md.push_str("- **Complexity** (30%): Cyclomatic and cognitive complexity\n");
    md.push_str("- **Code Churn** (35%): Frequency of changes over time\n");
    md.push_str("- **Coupling** (15%): Dependencies between modules\n");
    md.push_str("- **Domain Risk** (10%): Critical domain areas (auth, crypto, etc.)\n");
    md.push_str("- **Duplication** (10%): Code duplication percentage\n");
}

fn format_sarif_output(summary: &crate::models::tdg::TDGSummary) -> String {
    debug_assert!(true, "contract: format_sarif_output");
    let sarif = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat-tdg",
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "version": env!("CARGO_PKG_VERSION"),
                    "rules": [{
                        "id": "TDG001",
                        "name": "HighTechnicalDebtGradient",
                        "shortDescription": {
                            "text": "File has high technical debt gradient"
                        },
                        "fullDescription": {
                            "text": "Technical Debt Gradient exceeds threshold, indicating accumulated technical debt"
                        },
                        "help": {
                            "text": "Consider refactoring to reduce complexity, stabilize churn, or reduce coupling"
                        }
                    }]
                }
            },
            "results": summary.hotspots.iter().map(|hotspot| {
                serde_json::json!({
                    "ruleId": "TDG001",
                    "level": if hotspot.tdg_score > 2.5 { "error" } else { "warning" },
                    "message": {
                        "text": format!("TDG score {:.2} - Primary factor: {}",
                            hotspot.tdg_score, hotspot.primary_factor)
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": hotspot.path.clone()
                            }
                        }
                    }],
                    "properties": {
                        "tdg_score": hotspot.tdg_score,
                        "primary_factor": &hotspot.primary_factor,
                        "estimated_hours": hotspot.estimated_hours
                    }
                })
            }).collect::<Vec<_>>()
        }]
    });

    serde_json::to_string_pretty(&sarif).unwrap_or_else(|_| "{}".to_string())
}
