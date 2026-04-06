#![cfg_attr(coverage_nightly, coverage(off))]
use super::super::TdgScore;

/// Format TDG score as JSON output.
///
/// Serializes the TDG score to a JSON string for programmatic consumption
/// or integration with other tools and systems.
///
/// # Arguments  
/// * `score` - The TDG score to serialize
///
/// # Returns
/// A JSON string representation of the TDG score
///
/// # Example
/// ```ignore
/// use pmat::tdg::{TdgScore, Grade};
/// let score = TdgScore::new(85.5, Grade::A, 0.95);
/// let json = format_json(&score);
/// assert!(json.contains("85.5"));
/// ```ignore
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_json(score: &TdgScore) -> String {
    serde_json::to_string_pretty(score).unwrap_or_else(|_| "{}".to_string())
}
