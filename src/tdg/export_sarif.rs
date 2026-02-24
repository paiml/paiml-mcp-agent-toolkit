// SARIF (Static Analysis Results Interchange Format) export implementations
// Included by export.rs - shares parent module scope

impl TdgExporter {
    fn score_to_sarif(score: &TdgScore, _options: &ExportOptions) -> Result<String> {
        let sarif = json!({
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "TDG Analyzer",
                        "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                        "version": env!("CARGO_PKG_VERSION"),
                        "rules": Self::get_sarif_rules(),
                    }
                },
                "results": Self::score_to_sarif_results(score),
            }]
        });

        Ok(serde_json::to_string_pretty(&sarif)?)
    }

    fn project_to_sarif(project: &ProjectScore, _options: &ExportOptions) -> Result<String> {
        let mut all_results = Vec::new();

        for score in &project.files {
            all_results.extend(Self::score_to_sarif_results(score));
        }

        let sarif = json!({
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "TDG Analyzer",
                        "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                        "version": env!("CARGO_PKG_VERSION"),
                        "rules": Self::get_sarif_rules(),
                    }
                },
                "results": all_results,
                "properties": {
                    "total_files": project.total_files,
                    "average_score": project.average_score,
                    "average_grade": project.average_grade.to_string(),
                }
            }]
        });

        Ok(serde_json::to_string_pretty(&sarif)?)
    }

    fn get_sarif_rules() -> Vec<serde_json::Value> {
        vec![
            json!({
                "id": "TDG001",
                "name": "HighComplexity",
                "shortDescription": {
                    "text": "High code complexity detected"
                },
                "fullDescription": {
                    "text": "The code has high structural or semantic complexity that may impact maintainability"
                },
                "defaultConfiguration": {
                    "level": "warning"
                }
            }),
            json!({
                "id": "TDG002",
                "name": "CodeDuplication",
                "shortDescription": {
                    "text": "Code duplication detected"
                },
                "fullDescription": {
                    "text": "Duplicated code patterns detected that should be refactored"
                },
                "defaultConfiguration": {
                    "level": "note"
                }
            }),
            json!({
                "id": "TDG003",
                "name": "LowDocumentation",
                "shortDescription": {
                    "text": "Insufficient documentation"
                },
                "fullDescription": {
                    "text": "Code lacks adequate documentation for public APIs"
                },
                "defaultConfiguration": {
                    "level": "note"
                }
            }),
        ]
    }

    fn score_to_sarif_results(score: &TdgScore) -> Vec<serde_json::Value> {
        let mut results = Vec::new();

        if score.structural_complexity < 15.0 || score.semantic_complexity < 15.0 {
            results.push(json!({
                "ruleId": "TDG001",
                "level": if score.total < 50.0 { "error" } else if score.total < 70.0 { "warning" } else { "note" },
                "message": {
                    "text": format!("Code complexity issues detected. Structural: {:.1}, Semantic: {:.1}",
                        score.structural_complexity, score.semantic_complexity)
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": score.file_path.as_ref().map_or_else(|| "unknown".to_string(), |p| p.display().to_string())
                        }
                    }
                }],
                "properties": {
                    "tdg_score": score.total,
                    "grade": score.grade.to_string(),
                }
            }));
        }

        if score.duplication_ratio < 15.0 {
            results.push(json!({
                "ruleId": "TDG002",
                "level": "note",
                "message": {
                    "text": format!("Code duplication detected: {:.1}% duplication ratio",
                        (20.0 - score.duplication_ratio) * 5.0)
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": score.file_path.as_ref().map_or_else(|| "unknown".to_string(), |p| p.display().to_string())
                        }
                    }
                }],
            }));
        }

        if score.doc_coverage < 8.0 {
            results.push(json!({
                "ruleId": "TDG003",
                "level": "note",
                "message": {
                    "text": format!("Low documentation coverage: {:.1}%",
                        score.doc_coverage * 10.0)
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": score.file_path.as_ref().map_or_else(|| "unknown".to_string(), |p| p.display().to_string())
                        }
                    }
                }],
            }));
        }

        results
    }
}
