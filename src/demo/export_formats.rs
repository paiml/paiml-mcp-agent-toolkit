// Exporter implementations for Markdown, JSON, and SARIF formats.
// Included from export.rs — no `use` imports or inner attributes allowed.

impl Exporter for MarkdownExporter {
    fn export(&self, report: &ExportReport) -> Result<String> {
        let mut output = String::with_capacity(1024);

        // Header
        output.push_str(&format!(
            "# Analysis: {}\n\nGenerated: {}\n\n",
            report.repository,
            report.timestamp.format("%Y-%m-%d %H:%M UTC")
        ));

        // Summary section
        output.push_str("## Summary\n\n");
        output.push_str(&format!(
            "- **Analyzed Files**: {}\n",
            report.summary.analyzed_files
        ));
        output.push_str(&format!(
            "- **Average Complexity**: {:.2}\n",
            report.complexity_analysis.average_complexity
        ));
        output.push_str(&format!(
            "- **Technical Debt**: {} hours\n",
            report.complexity_analysis.technical_debt_hours
        ));
        output.push_str(&format!(
            "- **Analysis Time**: {}ms\n\n",
            report.summary.analysis_time_ms
        ));

        // Dependency Graph
        output.push_str("## Dependency Graph\n\n```mermaid\n");
        if let Some(mermaid) = report.mermaid_graphs.get("main") {
            output.push_str(mermaid);
        } else {
            output.push_str("graph TD\n    NoData[No dependency graph available]");
        }
        output.push_str("\n```\n\n");

        // Complexity Hotspots
        output.push_str("## Complexity Hotspots\n\n");
        output.push_str("| File | Complexity | Cognitive Load |\n");
        output.push_str("|------|------------|----------------|\n");

        for hotspot in &report.complexity_analysis.hotspots {
            output.push_str(&format!(
                "| {} | {} | {} |\n",
                hotspot.file,
                hotspot.complexity,
                hotspot.churn_score // Using churn_score as cognitive load
            ));
        }
        output.push('\n');

        // Churn Analysis (if available)
        if let Some(churn) = &report.churn_analysis {
            output.push_str(&format!(
                "## Code Churn Analysis (Last {} Days)\n\n",
                churn.period_days
            ));
            output.push_str("| File | Churn Score | Commits |\n");
            output.push_str("|------|-------------|----------|\n");

            for file in churn.files.iter().take(10) {
                output.push_str(&format!(
                    "| {} | {:.2} | {} |\n",
                    file.relative_path, file.churn_score, file.commit_count
                ));
            }
            output.push('\n');
        }

        // Raw Data Section
        output.push_str("<details>\n<summary>Raw Data</summary>\n\n```json\n");
        output.push_str(&serde_json::to_string_pretty(report)?);
        output.push_str("\n```\n</details>\n");

        Ok(output)
    }

    fn file_extension(&self) -> &'static str {
        "md"
    }
}

impl JsonExporter {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(pretty: bool) -> Self {
        Self { pretty }
    }
}

impl Exporter for JsonExporter {
    fn export(&self, report: &ExportReport) -> Result<String> {
        if self.pretty {
            Ok(serde_json::to_string_pretty(report)?)
        } else {
            Ok(serde_json::to_string(report)?)
        }
    }

    fn file_extension(&self) -> &'static str {
        "json"
    }
}

impl Exporter for SarifExporter {
    fn export(&self, report: &ExportReport) -> Result<String> {
        let sarif = serde_json::json!({
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "pmat",
                        "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                        "version": env!("CARGO_PKG_VERSION"),
                        "rules": [{
                            "id": "COMPLEXITY001",
                            "name": "HighComplexity",
                            "shortDescription": {
                                "text": "Function has high cyclomatic complexity"
                            },
                            "fullDescription": {
                                "text": "Functions with high cyclomatic complexity are harder to understand, test, and maintain."
                            },
                            "defaultConfiguration": {
                                "level": "warning"
                            }
                        }]
                    }
                },
                "results": report.complexity_analysis.hotspots.iter().filter_map(|hotspot| {
                    if hotspot.complexity > 10 {
                        Some(serde_json::json!({
                            "ruleId": "COMPLEXITY001",
                            "ruleIndex": 0,
                            "level": if hotspot.complexity > 20 { "error" } else { "warning" },
                            "message": {
                                "text": format!("Function '{}' has cyclomatic complexity of {}",
                                    hotspot.file, hotspot.complexity)
                            },
                            "locations": [{
                                "physicalLocation": {
                                    "artifactLocation": {
                                        "uri": hotspot.file.split("::").next().unwrap_or(&hotspot.file)
                                    }
                                }
                            }],
                            "properties": {
                                "complexity": hotspot.complexity,
                                "cognitiveComplexity": hotspot.churn_score
                            }
                        }))
                    } else {
                        None
                    }
                }).collect::<Vec<_>>(),
                "invocations": [{
                    "executionSuccessful": true,
                    "endTimeUtc": report.timestamp.to_rfc3339()
                }]
            }]
        });

        Ok(serde_json::to_string_pretty(&sarif)?)
    }

    fn file_extension(&self) -> &'static str {
        "sarif"
    }
}
