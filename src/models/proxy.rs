#![cfg_attr(coverage_nightly, coverage(off))]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Proxy operation.
pub enum ProxyOperation {
    Write,
    Edit,
    Append,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
/// Operational mode for proxy.
pub enum ProxyMode {
    #[default]
    Strict,
    Advisory,
    AutoFix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Configuration for quality.
pub struct QualityConfig {
    #[serde(default = "default_max_complexity")]
    pub max_complexity: u32,
    #[serde(default = "default_allow_satd")]
    pub allow_satd: bool,
    #[serde(default = "default_require_docs")]
    pub require_docs: bool,
    #[serde(default = "default_auto_format")]
    pub auto_format: bool,
}

fn default_max_complexity() -> u32 {
    20
}

fn default_allow_satd() -> bool {
    false
}

fn default_require_docs() -> bool {
    true
}

fn default_auto_format() -> bool {
    true
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            max_complexity: default_max_complexity(),
            allow_satd: default_allow_satd(),
            require_docs: default_require_docs(),
            auto_format: default_auto_format(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Request for proxy operation.
pub struct ProxyRequest {
    pub operation: ProxyOperation,
    pub file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_content: Option<String>,
    #[serde(default)]
    pub mode: ProxyMode,
    #[serde(default)]
    pub quality_config: QualityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Status of proxy operation.
pub enum ProxyStatus {
    Accepted,
    Rejected,
    Modified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Type classification for violation.
pub enum ViolationType {
    Complexity,
    Satd,
    Lint,
    Docs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Severity level classification for violation.
pub enum ViolationSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Violation record for quality.
pub struct QualityViolation {
    #[serde(rename = "type")]
    pub violation_type: ViolationType,
    pub severity: ViolationSeverity,
    pub location: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Quality metrics.
pub struct QualityMetrics {
    pub max_complexity: u32,
    pub satd_count: usize,
    pub lint_violations: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage_percentage: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Report containing quality data.
///
/// `language` and `gates_run` exist because a zero in `metrics` is not a
/// measurement. Every file whose extension was not the literal lowercase `rs`
/// used to come back `passed: true` with `max_complexity: 0, satd_count: 0,
/// lint_violations: 0` and an empty violation list — byte-identical to the
/// report a clean Rust file produces — so a consumer had no way to tell "this
/// content was judged and is clean" from "no gate ever looked at it". A Python
/// payload with two debt markers and five levels of nesting was accepted that
/// way in strict mode, which is the default mode.
///
/// The rule the two fields encode: a gate that is absent from `gates_run` did
/// NOT run, and every number it would have produced must be read as unknown
/// rather than as zero. Both carry `#[serde(default)]` so payloads recorded
/// before the fields existed still deserialize — as an empty `gates_run`, which
/// is exactly the honest reading of a report that claims nothing.
pub struct QualityReport {
    pub passed: bool,
    pub metrics: QualityMetrics,
    pub violations: Vec<QualityViolation>,
    /// The language the gates were selected for, lower-cased: `"rust"`,
    /// `"python"`, `"bash"`, `"unknown"`.
    #[serde(default)]
    pub language: String,
    /// The gates that actually ran and produced the numbers in `metrics`.
    /// Currently one of `satd`, `complexity`, `lint`, `docs`.
    #[serde(default)]
    pub gates_run: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Response from proxy operation.
pub struct ProxyResponse {
    pub status: ProxyStatus,
    pub quality_report: QualityReport,
    pub final_content: String,
    pub refactoring_applied: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refactoring_plan: Option<Vec<HashMap<String, serde_json::Value>>>,
}

// Unit tests for basic proxy types (ProxyOperation, ProxyMode, QualityConfig, ProxyRequest, ProxyStatus)
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    include!("proxy_tests.rs");
}

// Unit tests for quality/violation types (ViolationType, ViolationSeverity, QualityViolation,
// QualityMetrics, QualityReport, ProxyResponse) plus Clone and Debug trait verification
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod quality_tests {
    include!("proxy_quality_tests.rs");
}

// Property-based tests using proptest
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    include!("proxy_property_tests.rs");
}
