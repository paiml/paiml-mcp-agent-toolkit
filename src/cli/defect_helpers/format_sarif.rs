#![cfg_attr(coverage_nightly, coverage(off))]
//! SARIF output formatting for defect predictions

use crate::services::defect_probability::DefectScore;
use anyhow::Result;
use std::path::Path;

/// Format defect predictions as SARIF
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn format_defect_sarif(
    predictions: &[(String, DefectScore)],
    _project_path: &Path,
) -> Result<String> {
    let mut results = Vec::new();

    for (file, score) in predictions {
        let level = if score.probability > 0.7 {
            "error"
        } else if score.probability > 0.4 {
            "warning"
        } else {
            "note"
        };

        let rule_id = if score.probability > 0.7 {
            "high-defect-probability"
        } else if score.probability > 0.4 {
            "medium-defect-probability"
        } else {
            "low-defect-probability"
        };

        results.push(serde_json::json!({
            "ruleId": rule_id,
            "level": level,
            "message": {
                "text": format!(
                    "File has {:.1}% defect probability with {:.1}% confidence. Risk factors: {:?}",
                    score.probability * 100.0,
                    score.confidence * 100.0,
                    score.contributing_factors
                )
            },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": file
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
                    "name": "paiml-defect-predictor",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": generate_defect_rules(),
                }
            },
            "results": results
        }]
    });

    serde_json::to_string_pretty(&sarif).map_err(Into::into)
}

/// Generate SARIF rules for defect prediction
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn generate_defect_rules() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "id": "high-defect-probability",
            "name": "High Defect Probability",
            "shortDescription": {
                "text": "File has high probability of containing defects"
            },
            "fullDescription": {
                "text": "Files with >70% defect probability require immediate review"
            },
            "defaultConfiguration": {
                "level": "error"
            }
        }),
        serde_json::json!({
            "id": "medium-defect-probability",
            "name": "Medium Defect Probability",
            "shortDescription": {
                "text": "File has medium probability of containing defects"
            },
            "fullDescription": {
                "text": "Files with 40-70% defect probability should be reviewed"
            },
            "defaultConfiguration": {
                "level": "warning"
            }
        }),
        serde_json::json!({
            "id": "low-defect-probability",
            "name": "Low Defect Probability",
            "shortDescription": {
                "text": "File has low probability of containing defects"
            },
            "fullDescription": {
                "text": "Files with <40% defect probability are lower risk"
            },
            "defaultConfiguration": {
                "level": "note"
            }
        }),
    ]
}
