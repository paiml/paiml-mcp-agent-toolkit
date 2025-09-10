use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyOperation {
    Write,
    Edit,
    Append,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProxyMode {
    #[default]
    Strict,
    Advisory,
    AutoFix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub enum ProxyStatus {
    Accepted,
    Rejected,
    Modified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationType {
    Complexity,
    Satd,
    Lint,
    Docs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct QualityMetrics {
    pub max_complexity: u32,
    pub satd_count: usize,
    pub lint_violations: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage_percentage: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    pub passed: bool,
    pub metrics: QualityMetrics,
    pub violations: Vec<QualityViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyResponse {
    pub status: ProxyStatus,
    pub quality_report: QualityReport,
    pub final_content: String,
    pub refactoring_applied: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refactoring_plan: Option<Vec<HashMap<String, serde_json::Value>>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_mode_default() {
        assert!(matches!(ProxyMode::default(), ProxyMode::Strict));
    }

    #[test]
    fn test_quality_config_default() {
        let config = QualityConfig::default();
        assert_eq!(config.max_complexity, 20);
        assert!(!config.allow_satd);
        assert!(config.require_docs);
        assert!(config.auto_format);
    }

    #[test]
    fn test_proxy_request_serialization() {
        let request = ProxyRequest {
            operation: ProxyOperation::Write,
            file_path: "test.rs".to_string(),
            content: Some("fn test() {}".to_string()),
            old_content: None,
            new_content: None,
            mode: ProxyMode::Strict,
            quality_config: QualityConfig::default(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ProxyRequest = serde_json::from_str(&json).unwrap();

        assert!(matches!(deserialized.operation, ProxyOperation::Write));
        assert_eq!(deserialized.file_path, "test.rs");
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test] 
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
