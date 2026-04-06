/// Format provability results as SARIF
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_provability_sarif(
    function_ids: &[FunctionId],
    summaries: &[ProofSummary],
) -> Result<String> {
    let mut results = Vec::new();

    for (func_id, summary) in function_ids.iter().zip(summaries.iter()) {
        let rule_id = if summary.provability_score < 0.5 {
            "low-provability"
        } else if summary.provability_score < 0.8 {
            "medium-provability"
        } else {
            "high-provability"
        };

        let level = if summary.provability_score < 0.5 {
            "warning"
        } else if summary.provability_score < 0.8 {
            "note"
        } else {
            "none"
        };

        results.push(serde_json::json!({
            "ruleId": rule_id,
            "level": level,
            "message": {
                "text": format!(
                    "Function '{}' has {:.0}% provability score",
                    func_id.function_name,
                    summary.provability_score * 100.0
                )
            },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": func_id.file_path
                    },
                    "region": {
                        "startLine": func_id.line_number
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
                    "name": "paiml-provability-analyzer",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": generate_provability_rules(),
                }
            },
            "results": results
        }]
    });

    serde_json::to_string_pretty(&sarif).map_err(Into::into)
}

/// Generate SARIF rules for provability analysis
fn generate_provability_rules() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "id": "low-provability",
            "name": "Low Provability Score",
            "shortDescription": {
                "text": "Function has low formal verification confidence"
            },
            "fullDescription": {
                "text": "Functions with low provability scores may contain complex control flow or lack sufficient type information for verification"
            },
            "defaultConfiguration": {
                "level": "warning"
            }
        }),
        serde_json::json!({
            "id": "medium-provability",
            "name": "Medium Provability Score",
            "shortDescription": {
                "text": "Function has medium formal verification confidence"
            },
            "fullDescription": {
                "text": "Functions with medium provability scores can be partially verified but may have some unverifiable properties"
            },
            "defaultConfiguration": {
                "level": "note"
            }
        }),
        serde_json::json!({
            "id": "high-provability",
            "name": "High Provability Score",
            "shortDescription": {
                "text": "Function has high formal verification confidence"
            },
            "fullDescription": {
                "text": "Functions with high provability scores can be formally verified with high confidence"
            },
            "defaultConfiguration": {
                "level": "none"
            }
        }),
    ]
}
