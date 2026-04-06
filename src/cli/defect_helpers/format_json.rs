#![cfg_attr(coverage_nightly, coverage(off))]
//! JSON output formatting for defect predictions

use crate::services::defect_probability::DefectScore;
use anyhow::Result;

/// Format defect predictions as JSON
/// Formats defect predictions as JSON
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::cli::defect_helpers::format_defect_json;
/// use pmat::services::defect_probability::{DefectScore, RiskLevel};
///
/// let predictions = vec![
///     ("src/main.rs".to_string(), DefectScore {
///         probability: 0.8,
///         confidence: 0.9,
///         contributing_factors: vec![("complexity".to_string(), 0.5)],
///         risk_level: RiskLevel::High,
///         recommendations: vec!["Reduce complexity".to_string()],
///     })
/// ];
///
/// let json = format_defect_json(&predictions).expect("internal error");
/// assert!(json.contains("defect_predictions"));
/// assert!(json.contains("src/main.rs"));
/// ```
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_defect_json(predictions: &[(String, DefectScore)]) -> Result<String> {
    debug_assert!(!predictions.is_empty(), "predictions must not be empty");
    let json_data = serde_json::json!({
        "defect_predictions": predictions.iter().map(|(file, score)| {
            serde_json::json!({
                "file": file,
                "probability": score.probability,
                "confidence": score.confidence,
                "risk_factors": score.contributing_factors,
            })
        }).collect::<Vec<_>>(),
        "summary": {
            "total_files": predictions.len(),
            "high_risk_files": predictions.iter().filter(|(_, s)| s.probability > 0.7).count(),
            "medium_risk_files": predictions.iter().filter(|(_, s)| s.probability > 0.4 && s.probability <= 0.7).count(),
            "low_risk_files": predictions.iter().filter(|(_, s)| s.probability <= 0.4).count(),
        }
    });

    serde_json::to_string_pretty(&json_data).map_err(Into::into)
}
