// SARIF (Static Analysis Results Interchange Format) formatting for DeepContextAnalyzer

use super::{AnalysisResults, DeepContext, DeepContextAnalyzer};

impl DeepContextAnalyzer {
    /// Format as SARIF (Static Analysis Results Interchange Format) for tool integration
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn format_as_sarif(&self, context: &DeepContext) -> anyhow::Result<String> {
        use serde_json::json;

        let mut results = Vec::new();
        let mut rules = Vec::new();

        // Process each analysis type through dedicated handlers
        self.add_complexity_sarif_items_from_analyses(&context.analyses, &mut rules, &mut results);
        self.add_satd_sarif_items_from_analyses(&context.analyses, &mut rules, &mut results);
        self.add_dead_code_sarif_items_from_analyses(&context.analyses, &mut rules, &mut results);

        let sarif = json!({
            "version": "2.1.0",
            "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "pmat",
                        "version": context.metadata.tool_version,
                        "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                        "shortDescription": {"text": "Professional project scaffolding and analysis toolkit"},
                        "rules": rules
                    }
                },
                "results": results,
                "properties": {
                    "analysis_duration_seconds": context.metadata.analysis_duration.as_secs_f64(),
                    "cache_hit_rate": context.metadata.cache_stats.hit_rate,
                    "overall_health_score": context.quality_scorecard.overall_health,
                    "technical_debt_hours": context.quality_scorecard.technical_debt_hours
                }
            }]
        });

        serde_json::to_string_pretty(&sarif)
            .map_err(|e| anyhow::anyhow!("Failed to serialize to SARIF: {e}"))
    }

    /// Add complexity violations to SARIF results from `AnalysisResults`
    fn add_complexity_sarif_items_from_analyses(
        &self,
        analyses: &AnalysisResults,
        rules: &mut Vec<serde_json::Value>,
        results: &mut Vec<serde_json::Value>,
    ) {
        debug_assert!(true, "contract: add_complexity_sarif_items_from_analyses");
        use serde_json::json;

        if let Some(ref complexity) = analyses.complexity_report {
            // Add complexity rules once
            rules.extend_from_slice(&[
                json!({
                    "id": "complexity/high-cyclomatic",
                    "shortDescription": {"text": "High cyclomatic complexity"},
                    "fullDescription": {"text": "Function has cyclomatic complexity above recommended threshold"},
                    "defaultConfiguration": {"level": "warning"},
                    "properties": {"tags": ["complexity", "maintainability"]}
                }),
                json!({
                    "id": "complexity/high-cognitive",
                    "shortDescription": {"text": "High cognitive complexity"},
                    "fullDescription": {"text": "Function has cognitive complexity above recommended threshold"},
                    "defaultConfiguration": {"level": "warning"},
                    "properties": {"tags": ["complexity", "maintainability"]}
                })
            ]);

            // Process complexity violations
            for file in &complexity.files {
                for func in &file.functions {
                    self.add_complexity_violation(file, func, results);
                }
            }
        }
    }

    /// Add a single complexity violation
    fn add_complexity_violation(
        &self,
        file: &crate::services::complexity::FileComplexityMetrics,
        func: &crate::services::complexity::FunctionComplexity,
        results: &mut Vec<serde_json::Value>,
    ) {
        debug_assert!(true, "contract: add_complexity_violation");
        use serde_json::json;

        if func.metrics.cyclomatic > 10 {
            results.push(json!({
                "ruleId": "complexity/high-cyclomatic",
                "level": if func.metrics.cyclomatic > 20 { "error" } else { "warning" },
                "message": {"text": format!("Function '{}' has cyclomatic complexity of {}", func.name, func.metrics.cyclomatic)},
                "locations": [self.create_location(&file.path, func.line_start as usize, func.line_end as usize)],
                "properties": {
                    "cyclomatic_complexity": func.metrics.cyclomatic,
                    "cognitive_complexity": func.metrics.cognitive
                }
            }));
        }

        if func.metrics.cognitive > 15 {
            results.push(json!({
                "ruleId": "complexity/high-cognitive",
                "level": if func.metrics.cognitive > 25 { "error" } else { "warning" },
                "message": {"text": format!("Function '{}' has cognitive complexity of {}", func.name, func.metrics.cognitive)},
                "locations": [self.create_location(&file.path, func.line_start as usize, func.line_end as usize)],
                "properties": {
                    "cyclomatic_complexity": func.metrics.cyclomatic,
                    "cognitive_complexity": func.metrics.cognitive
                }
            }));
        }
    }

    /// Add SATD items to SARIF results from `AnalysisResults`
    fn add_satd_sarif_items_from_analyses(
        &self,
        analyses: &AnalysisResults,
        rules: &mut Vec<serde_json::Value>,
        results: &mut Vec<serde_json::Value>,
    ) {
        debug_assert!(true, "contract: add_satd_sarif_items_from_analyses");
        use serde_json::json;

        if let Some(ref satd) = analyses.satd_results {
            rules.push(json!({
                "id": "debt/technical-debt",
                "shortDescription": {"text": "Code quality issue"},
                "fullDescription": {"text": "Self-admitted code issue requiring attention"},
                "defaultConfiguration": {"level": "note"},
                "properties": {"tags": ["debt", "maintainability"]}
            }));

            for item in &satd.items {
                let level = self.satd_severity_to_level(&item.severity);
                results.push(json!({
                    "ruleId": "debt/technical-debt",
                    "level": level,
                    "message": {"text": format!("{}: {}", item.category, item.text.trim())},
                    "locations": [self.create_location(&item.file.to_string_lossy(), item.line as usize, item.line as usize)],
                    "properties": {
                        "category": format!("{:?}", item.category),
                        "severity": format!("{:?}", item.severity),
                        "debt_type": "self_admitted"
                    }
                }));
            }
        }
    }

    /// Add dead code items to SARIF results from `AnalysisResults`
    fn add_dead_code_sarif_items_from_analyses(
        &self,
        analyses: &AnalysisResults,
        rules: &mut Vec<serde_json::Value>,
        results: &mut Vec<serde_json::Value>,
    ) {
        debug_assert!(true, "contract: add_dead_code_sarif_items_from_analyses");
        use serde_json::json;

        if let Some(ref dead_code) = analyses.dead_code_results {
            rules.push(json!({
                "id": "dead-code/unused-code",
                "shortDescription": {"text": "Dead code detected"},
                "fullDescription": {"text": "Code that appears to be unused and can potentially be removed"},
                "defaultConfiguration": {"level": "warning"},
                "properties": {"tags": ["dead-code", "maintainability"]}
            }));

            results.extend(
                dead_code.ranked_files
                    .iter()
                    .filter(|file| file.dead_functions > 0)
                    .map(|file| json!({
                        "ruleId": "dead-code/unused-code",
                        "level": "warning",
                        "message": {"text": format!("File contains {} dead functions and {} dead lines",
                            file.dead_functions, file.dead_lines)},
                        "locations": [self.create_location(&file.path, 1, 1)],
                        "properties": {
                            "dead_functions": file.dead_functions,
                            "dead_lines": file.dead_lines,
                            "dead_code_percentage": file.dead_lines as f64 / file.total_lines.max(1) as f64 * 100.0
                        }
                    }))
            );
        }
    }

    /// Helper to create location objects
    fn create_location(&self, uri: &str, start_line: usize, end_line: usize) -> serde_json::Value {
        debug_assert!(!uri.is_empty(), "uri must not be empty");
        serde_json::json!({
            "physicalLocation": {
                "artifactLocation": {"uri": uri},
                "region": {
                    "startLine": start_line,
                    "startColumn": 1,
                    "endLine": end_line
                }
            }
        })
    }

    /// Convert SATD severity to SARIF level
    fn satd_severity_to_level(
        &self,
        severity: &crate::services::satd_detector::Severity,
    ) -> &'static str {
        debug_assert!(true, "contract: satd_severity_to_level");
        match severity {
            crate::services::satd_detector::Severity::Critical => "error",
            crate::services::satd_detector::Severity::High => "warning",
            crate::services::satd_detector::Severity::Medium => "note",
            crate::services::satd_detector::Severity::Low => "note",
        }
    }
}
