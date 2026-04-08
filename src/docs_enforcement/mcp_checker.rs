//! MCP Documentation Checker
//!
//! TICKET: PMAT-7001 Phase 2 (GREEN)
//!
//! This module validates that MCP tools have complete, accurate documentation
//! including tool descriptions, parameter schemas, and non-generic text.

#![cfg_attr(coverage_nightly, coverage(off))]
use crate::docs_enforcement::generic_detector::is_generic_description;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// MCP tool definition (simplified for validation)
#[derive(Debug, Clone)]
pub struct McpToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// MCP documentation validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpDocumentationReport {
    pub tool_name: String,
    pub has_description: bool,
    pub description_length: usize,
    pub description_is_generic: bool,
    pub has_input_schema: bool,
    pub parameters: Vec<ParameterReport>,
    pub issues: Vec<String>,
}

/// Parameter documentation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterReport {
    pub name: String,
    pub has_description: bool,
    pub description: String,
    pub description_is_generic: bool,
    pub has_type: bool,
    pub param_type: String,
    pub is_required: bool,
    pub has_default: bool,
    pub issues: Vec<String>,
}

impl McpDocumentationReport {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Is valid.
    pub fn is_valid(&self) -> bool {
        self.has_description
            && self.description_length > 20
            && !self.description_is_generic
            && self.has_input_schema
            && self.parameters.iter().all(|p| p.is_valid())
            && self.issues.is_empty()
    }
}

impl ParameterReport {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Is valid.
    pub fn is_valid(&self) -> bool {
        self.has_description
            && !self.description_is_generic
            && self.has_type
            && self.issues.is_empty()
    }
}

include!("mcp_checker_validation.rs");
include!("mcp_checker_definitions.rs");
include!("mcp_checker_tests.rs");
