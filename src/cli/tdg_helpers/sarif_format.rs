#![cfg_attr(coverage_nightly, coverage(off))]
//! TDG SARIF output formatting

use crate::models::tdg::TDGHotspot;
use anyhow::Result;
use std::path::Path;

/// Format TDG results as SARIF
pub fn format_tdg_sarif(hotspots: &[TDGHotspot], project_path: &Path) -> Result<String> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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
pub(super) fn generate_tdg_rules() -> Vec<serde_json::Value> {
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
