use crate::mcp_pmcp::tool_functions;
use crate::mcp_pmcp::tool_schemas::{build_tool_info, paths_object_schema};
use async_trait::async_trait;
use pmcp::types::ToolInfo;
use pmcp::{Error, RequestHandlerExtra, Result, ToolHandler};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use tracing::debug;

// Quality Gate Check Tool

/// MCP args for quality.gate: paths to check, optional strict mode and single-file override.
#[derive(Debug, Deserialize)]
struct QualityGateArgs {
    paths: Vec<String>,
    #[serde(default)]
    strict: bool,
    #[serde(default)]
    file: Option<String>,
}

/// Quality gate tool.
pub struct QualityGateTool;

impl QualityGateTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for QualityGateTool {
    fn default() -> Self {
        Self::new()
    }
}

// Quality Gate Summary Tool

/// MCP args for quality.gate_summary: paths to summarize with optional output format.
#[derive(Debug, Deserialize)]
struct QualityGateSummaryArgs {
    paths: Vec<String>,
    #[serde(default)]
    format: Option<String>,
}

/// Quality gate summary tool.
pub struct QualityGateSummaryTool;

impl QualityGateSummaryTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for QualityGateSummaryTool {
    fn default() -> Self {
        Self::new()
    }
}

// Quality Gate Baseline Tool

/// MCP args for quality.gate_baseline: paths to snapshot with optional output file path.
#[derive(Debug, Deserialize)]
struct QualityGateBaselineArgs {
    paths: Vec<String>,
    #[serde(default)]
    output: Option<String>,
}

/// Quality gate baseline tool.
pub struct QualityGateBaselineTool;

impl QualityGateBaselineTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for QualityGateBaselineTool {
    fn default() -> Self {
        Self::new()
    }
}

// Quality Gate Compare Tool

/// MCP args for quality.gate_compare: baseline file path and current paths to diff against it.
#[derive(Debug, Deserialize)]
struct QualityGateCompareArgs {
    baseline: String,
    paths: Vec<String>,
}

/// Quality gate compare tool.
pub struct QualityGateCompareTool;

impl QualityGateCompareTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for QualityGateCompareTool {
    fn default() -> Self {
        Self::new()
    }
}

// --- Submodule includes ---

include!("quality_handlers_impl.rs");
include!("quality_handlers_tests.rs");
