#![cfg_attr(coverage_nightly, coverage(off))]
// MCP Agent Context Tools
// PMAT-470: RAG-powered semantic code search for agents
//
// These tools expose the agent context index via MCP protocol,
// enabling AI agents to search code with quality-aware filtering.

use crate::services::agent_context::{AgentContextIndex, QueryOptions};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// MCP Tool trait (same as semantic_search_tools)
#[async_trait]
pub trait McpTool: Send + Sync {
    fn name(&self) -> &str;
    fn schema(&self) -> Value;
    async fn execute(&self, params: Value) -> Result<Value, String>;
}

// ============================================================================
// Struct Definitions
// ============================================================================

/// Manages the agent context index lifecycle
pub struct IndexManager {
    index: RwLock<Option<AgentContextIndex>>,
    project_path: PathBuf,
}

/// Search functions by natural language query with quality filtering
pub struct QueryCodeTool {
    manager: Arc<IndexManager>,
}

/// Get details for a specific function by ID
pub struct GetFunctionTool {
    manager: Arc<IndexManager>,
}

/// Find functions similar to a reference function
pub struct FindSimilarTool {
    manager: Arc<IndexManager>,
}

/// Get index statistics and health
pub struct IndexStatsTool {
    manager: Arc<IndexManager>,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse function ID in format "file_path::function_name"
fn parse_function_id(function_id: &str) -> Result<(String, String), String> {
    debug_assert!(!function_id.is_empty(), "function_id must not be empty");
    // Find the last "::" separator
    if let Some(pos) = function_id.rfind("::") {
        let file_path = function_id.get(..pos).unwrap_or_default();
        let function_name = function_id.get(pos + 2..).unwrap_or_default();
        if file_path.is_empty() || function_name.is_empty() {
            return Err(format!(
                "Invalid function_id format. Expected 'file_path::function_name', got: {}",
                function_id
            ));
        }
        Ok((file_path.to_string(), function_name.to_string()))
    } else {
        Err(format!(
            "Invalid function_id format. Expected 'file_path::function_name', got: {}",
            function_id
        ))
    }
}

// ============================================================================
// Implementations (split into include files)
// ============================================================================

include!("agent_context_tools_index_manager.rs");
include!("agent_context_tools_query_tool.rs");
include!("agent_context_tools_lookup_tools.rs");
include!("agent_context_tools_tests.rs");
