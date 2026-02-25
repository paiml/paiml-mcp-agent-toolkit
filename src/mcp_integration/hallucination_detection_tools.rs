//! MCP tools for hallucination detection and documentation validation
//!
//! Exposes Sprint 37's hallucination detection system via MCP to enable
//! AI agents to validate documentation claims against the actual codebase.
//!
//! Based on peer-reviewed research:
//! - Semantic Entropy (Farquhar et al., Nature 2024)
//! - MIND framework (IJCAI 2025)
//! - Unified Detection Framework (Complex & Intelligent Systems 2025)

use super::*;
use crate::agents::registry::AgentRegistry;
use crate::services::hallucination_detector::{
    ClaimExtractor, CodeFactDatabase, HallucinationDetector, ValidationStatus,
};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

/// Validate documentation tool - checks documentation claims against codebase
pub struct ValidateDocumentationTool {
    _registry: Arc<AgentRegistry>,
}

impl ValidateDocumentationTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }
}

/// Check single claim tool - validates a single claim against codebase
pub struct CheckClaimTool {
    _registry: Arc<AgentRegistry>,
}

impl CheckClaimTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }
}

include!("hallucination_detection_tools_validate.rs");
include!("hallucination_detection_tools_check.rs");
include!("hallucination_detection_tools_tests.rs");
