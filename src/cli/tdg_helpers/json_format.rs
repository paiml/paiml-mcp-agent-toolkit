#![cfg_attr(coverage_nightly, coverage(off))]
//! TDG JSON output formatting

use crate::models::tdg::{TDGHotspot, TDGSummary};
use anyhow::Result;

/// Format TDG results as JSON
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
