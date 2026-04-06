#![cfg_attr(coverage_nightly, coverage(off))]

// ============================================================================
// Agent Context Tool Adapters (PMAT-470)
// ============================================================================
// These adapters bridge the agent context tools (crate::mcp::tools::agent_context_tools)
// to the mcp_integration tool system (mcp_integration::McpTool)

use crate::mcp::tools::agent_context_tools::IndexManager;
use crate::mcp_integration::{error_codes, McpError, McpTool, ToolMetadata};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

/// Adapter for pmat_query_code tool
pub struct QueryCodeToolAdapter {
    inner: crate::mcp::tools::agent_context_tools::QueryCodeTool,
}

impl QueryCodeToolAdapter {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

        self.inner
            .execute(params)
            .await
            .map_err(|err_msg| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: err_msg,
                data: None,
            })
    }
}

/// Adapter for pmat_get_function tool
pub struct GetFunctionToolAdapter {
    inner: crate::mcp::tools::agent_context_tools::GetFunctionTool,
}

impl GetFunctionToolAdapter {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

        self.inner
            .execute(params)
            .await
            .map_err(|err_msg| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: err_msg,
                data: None,
            })
    }
}

/// Adapter for pmat_find_similar tool
pub struct FindSimilarToolAdapter {
    inner: crate::mcp::tools::agent_context_tools::FindSimilarTool,
}

impl FindSimilarToolAdapter {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

        self.inner
            .execute(params)
            .await
            .map_err(|err_msg| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: err_msg,
                data: None,
            })
    }
}

/// Adapter for pmat_index_stats tool
pub struct IndexStatsToolAdapter {
    inner: crate::mcp::tools::agent_context_tools::IndexStatsTool,
}

impl IndexStatsToolAdapter {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

        self.inner
            .execute(params)
            .await
            .map_err(|err_msg| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: err_msg,
                data: None,
            })
    }
}
