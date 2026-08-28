#![cfg_attr(coverage_nightly, coverage(off))]

// ============================================================================
// Agent Context Tool Adapters (PMAT-470)
// ============================================================================
// These adapters bridge the agent context tools (crate::mcp::tools::agent_context_tools)
// to the mcp_integration tool system (mcp_integration::McpTool)

use crate::mcp::tools::agent_context_tools::{IndexManager, ToolError};
use crate::mcp_integration::{error_codes, McpError, McpTool, ToolMetadata};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

/// Map an inner tool failure onto its JSON-RPC code.
///
/// All four adapters used to hardcode `INTERNAL_ERROR` — a second copy of the
/// same confusion the pmcp adapter had, and a worse one: here even
/// `Missing required parameter` was reported as `-32603`, telling the caller
/// the server had faulted on an argument they had omitted. The classification
/// now travels with the error from the site that knows it.
fn adapt_tool_error(error: ToolError) -> McpError {
    let code = if error.is_invalid_params() {
        error_codes::INVALID_PARAMS
    } else {
        error_codes::INTERNAL_ERROR
    };
    McpError {
        code,
        message: error.message().to_string(),
        data: None,
    }
}

/// Adapter for pmat_query_code tool
pub struct QueryCodeToolAdapter {
    inner: crate::mcp::tools::agent_context_tools::QueryCodeTool,
}

impl QueryCodeToolAdapter {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(manager: Arc<IndexManager>) -> Self {
        Self {
            inner: crate::mcp::tools::agent_context_tools::QueryCodeTool::new(manager),
        }
    }
}

#[async_trait]
impl McpTool for QueryCodeToolAdapter {
    fn metadata(&self) -> ToolMetadata {
        use crate::mcp::tools::agent_context_tools::McpTool as SimpleMcpTool;
        let name = self.inner.name().to_string();
        let schema = self.inner.schema();

        let description = schema["description"]
            .as_str()
            .unwrap_or("Search code functions by natural language query")
            .to_string();

        let input_schema = schema["parameters"].clone();

        ToolMetadata {
            name,
            description,
            input_schema,
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        use crate::mcp::tools::agent_context_tools::McpTool as SimpleMcpTool;

        self.inner.execute(params).await.map_err(adapt_tool_error)
    }
}

/// Adapter for pmat_get_function tool
pub struct GetFunctionToolAdapter {
    inner: crate::mcp::tools::agent_context_tools::GetFunctionTool,
}

impl GetFunctionToolAdapter {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(manager: Arc<IndexManager>) -> Self {
        Self {
            inner: crate::mcp::tools::agent_context_tools::GetFunctionTool::new(manager),
        }
    }
}

#[async_trait]
impl McpTool for GetFunctionToolAdapter {
    fn metadata(&self) -> ToolMetadata {
        use crate::mcp::tools::agent_context_tools::McpTool as SimpleMcpTool;
        let name = self.inner.name().to_string();
        let schema = self.inner.schema();

        let description = schema["description"]
            .as_str()
            .unwrap_or("Get detailed information about a specific function")
            .to_string();

        let input_schema = schema["parameters"].clone();

        ToolMetadata {
            name,
            description,
            input_schema,
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        use crate::mcp::tools::agent_context_tools::McpTool as SimpleMcpTool;

        self.inner.execute(params).await.map_err(adapt_tool_error)
    }
}

/// Adapter for pmat_find_similar tool
pub struct FindSimilarToolAdapter {
    inner: crate::mcp::tools::agent_context_tools::FindSimilarTool,
}

impl FindSimilarToolAdapter {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(manager: Arc<IndexManager>) -> Self {
        Self {
            inner: crate::mcp::tools::agent_context_tools::FindSimilarTool::new(manager),
        }
    }
}

#[async_trait]
impl McpTool for FindSimilarToolAdapter {
    fn metadata(&self) -> ToolMetadata {
        use crate::mcp::tools::agent_context_tools::McpTool as SimpleMcpTool;
        let name = self.inner.name().to_string();
        let schema = self.inner.schema();

        let description = schema["description"]
            .as_str()
            .unwrap_or("Find functions similar to a reference function")
            .to_string();

        let input_schema = schema["parameters"].clone();

        ToolMetadata {
            name,
            description,
            input_schema,
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        use crate::mcp::tools::agent_context_tools::McpTool as SimpleMcpTool;

        self.inner.execute(params).await.map_err(adapt_tool_error)
    }
}

/// Adapter for pmat_index_stats tool
pub struct IndexStatsToolAdapter {
    inner: crate::mcp::tools::agent_context_tools::IndexStatsTool,
}

impl IndexStatsToolAdapter {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(manager: Arc<IndexManager>) -> Self {
        Self {
            inner: crate::mcp::tools::agent_context_tools::IndexStatsTool::new(manager),
        }
    }
}

#[async_trait]
impl McpTool for IndexStatsToolAdapter {
    fn metadata(&self) -> ToolMetadata {
        use crate::mcp::tools::agent_context_tools::McpTool as SimpleMcpTool;
        let name = self.inner.name().to_string();
        let schema = self.inner.schema();

        let description = schema["description"]
            .as_str()
            .unwrap_or("Get statistics about the code index")
            .to_string();

        let input_schema = schema["parameters"].clone();

        ToolMetadata {
            name,
            description,
            input_schema,
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        use crate::mcp::tools::agent_context_tools::McpTool as SimpleMcpTool;

        self.inner.execute(params).await.map_err(adapt_tool_error)
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    /// REGRESSION: this adapter tree hardcoded `INTERNAL_ERROR` for all four
    /// tools, so a caller-supplied value failing a documented bound was
    /// reported as a server fault — the same -32603/-32602 confusion the pmcp
    /// adapter had, in a second place.
    #[tokio::test]
    async fn documented_bounds_are_invalid_params_not_internal_errors() {
        let mgr = Arc::new(IndexManager::new(std::path::PathBuf::from(".")));
        let err = QueryCodeToolAdapter::new(mgr)
            .execute(serde_json::json!({"query": "x", "limit": 9999}))
            .await
            .expect_err("limit 9999 exceeds the schema's maximum of 100");
        assert_eq!(
            err.code,
            error_codes::INVALID_PARAMS,
            "a bad argument is -32602, not -32603: {}",
            err.message
        );
    }

    /// The other direction must still hold: our own failures stay -32603.
    #[test]
    fn our_own_failures_stay_internal() {
        let err = adapt_tool_error(ToolError::from("index build failed".to_string()));
        assert_eq!(err.code, error_codes::INTERNAL_ERROR);
        assert_eq!(err.message, "index build failed");
    }
}
