//! MCP tools for TDG (Technical Debt Gradient) analysis
//!
//! Exposes PMAT's TDG quality analysis system via MCP to enable
//! AI agents to assess code quality and get actionable recommendations.

use super::*;
use crate::agents::registry::AgentRegistry;
use crate::tdg::analyzer_simple::TdgAnalyzer;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

/// Analyze technical debt tool - comprehensive quality analysis
pub struct AnalyzeTechnicalDebtTool {
    _registry: Arc<AgentRegistry>,
}

impl AnalyzeTechnicalDebtTool {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }
}

/// Get quality recommendations tool - actionable improvement suggestions
pub struct GetQualityRecommendationsTool {
    _registry: Arc<AgentRegistry>,
}

impl GetQualityRecommendationsTool {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }
}

// --- Implementation split across include files ---

include!("tdg_tools_handlers.rs");
include!("tdg_tools_helpers.rs");
include!("tdg_tools_tests.rs");
