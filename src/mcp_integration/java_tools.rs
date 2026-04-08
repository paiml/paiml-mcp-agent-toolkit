use crate::mcp_integration::ast_item_helpers::{extract_complexity, extract_kind, extract_name};
use crate::mcp_integration::{McpError, McpTool, ToolMetadata};
use crate::services::languages::java::JavaAstVisitor;
use crate::utils::path_validator::PathValidator;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tracing::{info, warn};

/// Analyzes Java source code for complexity and structure
pub struct JavaAnalysisTool {
    agent_registry: Arc<crate::agents::registry::AgentRegistry>,
}

impl JavaAnalysisTool {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(agent_registry: Arc<crate::agents::registry::AgentRegistry>) -> Self {
        Self { agent_registry }
    }
}

/// Java mutation testing tool
pub struct JavaMutationTool {
    agent_registry: Arc<crate::agents::registry::AgentRegistry>,
}

impl JavaMutationTool {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(agent_registry: Arc<crate::agents::registry::AgentRegistry>) -> Self {
        Self { agent_registry }
    }
}

// JavaAnalysisTool McpTool impl, analyze_java_file, analyze_java_directory, find_java_files
include!("java_tools_analysis.rs");

// JavaMutationTool McpTool impl
include!("java_tools_mutation.rs");

// Tests
include!("java_tools_tests.rs");
