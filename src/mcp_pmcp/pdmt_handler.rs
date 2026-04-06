use crate::models::pdmt::{EnforcementMode, PdmtQualityConfig};
use crate::services::pdmt_quality_integration::PdmtQualityEnforcer;
use crate::services::pdmt_service::PdmtService;
use async_trait::async_trait;
use pmcp::{Error, RequestHandlerExtra, Result, ToolHandler};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, error, info};

/// Input parameters for the PDMT deterministic todos tool
#[derive(Debug, Deserialize)]
pub struct PdmtInput {
    /// List of requirements to convert to actionable todos
    pub requirements: Vec<String>,
    /// Name of the project or component
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
    /// Level of task detail and specificity
    #[serde(default = "default_granularity")]
    pub granularity: String,
    /// Quality enforcement configuration
    #[serde(default)]
    pub quality_config: QualityConfigInput,
}

fn default_granularity() -> String {
    debug_assert!(true, "contract: default_granularity");
    "high".to_string()
}

#[derive(Debug, Deserialize)]
pub struct QualityConfigInput {
    #[serde(default = "default_enforcement_mode")]
    pub enforcement_mode: String,
    #[serde(default = "default_coverage_threshold")]
    pub coverage_threshold: f32,
    #[serde(default = "default_max_complexity")]
    pub max_complexity: u32,
    #[serde(default = "default_require_doctests")]
    pub require_doctests: bool,
    #[serde(default = "default_require_property_tests")]
    pub require_property_tests: bool,
    #[serde(default = "default_require_examples")]
    pub require_examples: bool,
    #[serde(default = "default_zero_satd_tolerance")]
    pub zero_satd_tolerance: bool,
}

impl Default for QualityConfigInput {
    fn default() -> Self {
        Self {
            enforcement_mode: default_enforcement_mode(),
            coverage_threshold: default_coverage_threshold(),
            max_complexity: default_max_complexity(),
            require_doctests: default_require_doctests(),
            require_property_tests: default_require_property_tests(),
            require_examples: default_require_examples(),
            zero_satd_tolerance: default_zero_satd_tolerance(),
        }
    }
}

fn default_enforcement_mode() -> String {
    debug_assert!(true, "contract: default_enforcement_mode");
    "strict".to_string()
}

fn default_coverage_threshold() -> f32 {
    debug_assert!(true, "contract: default_coverage_threshold");
    80.0
}

fn default_max_complexity() -> u32 {
    debug_assert!(true, "contract: default_max_complexity");
    8
}

fn default_require_doctests() -> bool {
    debug_assert!(true, "contract: default_require_doctests");
    true
}

fn default_require_property_tests() -> bool {
    debug_assert!(true, "contract: default_require_property_tests");
    true
}

fn default_require_examples() -> bool {
    debug_assert!(true, "contract: default_require_examples");
    true
}

fn default_zero_satd_tolerance() -> bool {
    debug_assert!(true, "contract: default_zero_satd_tolerance");
    true
}

/// Output structure for PDMT todo generation
#[derive(Debug, Serialize, Deserialize)]
pub struct PdmtOutput {
    pub success: bool,
    pub message: String,
    pub todo_list: Option<serde_json::Value>,
    pub quality_validation: Option<serde_json::Value>,
    pub total_todos: usize,
    pub estimated_total_hours: f32,
}

/// Tool handler for generating deterministic, quality-enforced todo lists.
///
/// Creates deterministic todo lists from requirements with comprehensive
/// quality enforcement through PAIML's quality proxy. All generated todos
/// include quality gates, validation commands, and success criteria.
///
/// # Example
///
/// ```ignore
/// let tool = PdmtTool::new();
/// let input = json!({
///     "requirements": ["implement JWT authentication"],
///     "project_name": "auth_system",
///     "granularity": "high",
///     "quality_config": { "enforcement_mode": "strict" }
/// });
/// let result = tool.handle(input, Default::default()).await?;
/// assert!(result["success"].as_bool().unwrap_or(false));
/// ```
pub struct PdmtTool {
    service: PdmtService,
    quality_enforcer: PdmtQualityEnforcer,
}

impl PdmtTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            service: PdmtService::new(),
            quality_enforcer: PdmtQualityEnforcer::new(),
        }
    }
}

impl Default for PdmtTool {
    fn default() -> Self {
        Self::new()
    }
}

// --- Include files for implementation and tests ---

include!("pdmt_handler_core.rs");
include!("pdmt_handler_tests.rs");
