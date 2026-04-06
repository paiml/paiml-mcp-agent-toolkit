#![cfg_attr(coverage_nightly, coverage(off))]

// ============================================================================
// Semantic Search Tool Adapters (Sprint 33: PMAT-SEARCH-012)
// ============================================================================
// These adapters bridge the semantic search tools (crate::mcp::McpTool)
// to the mcp_integration tool system (mcp_integration::McpTool)

use crate::mcp_integration::{error_codes, McpError, McpTool, ToolMetadata};
use crate::services::semantic::HybridSearchEngine;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

/// Adapter for semantic_search tool
pub struct SemanticSearchToolAdapter {
    inner: crate::mcp::tools::semantic_search_tools::SemanticSearchTool,
}

impl SemanticSearchToolAdapter {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(engine: Arc<HybridSearchEngine>) -> Self {
        Self {
            inner: crate::mcp::tools::semantic_search_tools::SemanticSearchTool::new(engine),
        }
    }
}

#[async_trait]
impl McpTool for SemanticSearchToolAdapter {
    fn metadata(&self) -> ToolMetadata {
        use crate::mcp::tools::semantic_search_tools::McpTool as SimpleMcpTool;
        let name = self.inner.name().to_string();
        let schema = self.inner.schema();

        // Extract description from schema
        let description = schema["description"]
            .as_str()
            .unwrap_or("Search code by natural language query")
            .to_string();

        // Extract input_schema from schema parameters
        let input_schema = schema["parameters"].clone();

        ToolMetadata {
            name,
            description,
            input_schema,
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        use crate::mcp::tools::semantic_search_tools::McpTool as SimpleMcpTool;

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

/// Adapter for find_similar_code tool
pub struct FindSimilarCodeToolAdapter {
    inner: crate::mcp::tools::semantic_search_tools::FindSimilarCodeTool,
}

impl FindSimilarCodeToolAdapter {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(engine: Arc<HybridSearchEngine>) -> Self {
        Self {
            inner: crate::mcp::tools::semantic_search_tools::FindSimilarCodeTool::new(engine),
        }
    }
}

#[async_trait]
impl McpTool for FindSimilarCodeToolAdapter {
    fn metadata(&self) -> ToolMetadata {
        use crate::mcp::tools::semantic_search_tools::McpTool as SimpleMcpTool;
        let name = self.inner.name().to_string();
        let schema = self.inner.schema();

        let description = schema["description"]
            .as_str()
            .unwrap_or("Find similar code files using vector similarity")
            .to_string();

        let input_schema = schema["parameters"].clone();

        ToolMetadata {
            name,
            description,
            input_schema,
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        use crate::mcp::tools::semantic_search_tools::McpTool as SimpleMcpTool;

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

/// Adapter for cluster_code tool
pub struct ClusterCodeToolAdapter {
    inner: crate::mcp::tools::semantic_search_tools::ClusterCodeTool,
}

impl ClusterCodeToolAdapter {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(engine: Arc<HybridSearchEngine>) -> Self {
        Self {
            inner: crate::mcp::tools::semantic_search_tools::ClusterCodeTool::new(engine),
        }
    }
}

#[async_trait]
impl McpTool for ClusterCodeToolAdapter {
    fn metadata(&self) -> ToolMetadata {
        use crate::mcp::tools::semantic_search_tools::McpTool as SimpleMcpTool;
        let name = self.inner.name().to_string();
        let schema = self.inner.schema();

        let description = schema["description"]
            .as_str()
            .unwrap_or("Cluster code by semantic similarity")
            .to_string();

        let input_schema = schema["parameters"].clone();

        ToolMetadata {
            name,
            description,
            input_schema,
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        use crate::mcp::tools::semantic_search_tools::McpTool as SimpleMcpTool;

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

/// Adapter for analyze_topics tool
pub struct AnalyzeTopicsToolAdapter {
    inner: crate::mcp::tools::semantic_search_tools::AnalyzeTopicsTool,
}

impl AnalyzeTopicsToolAdapter {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(engine: Arc<HybridSearchEngine>) -> Self {
        Self {
            inner: crate::mcp::tools::semantic_search_tools::AnalyzeTopicsTool::new(engine),
        }
    }
}

#[async_trait]
impl McpTool for AnalyzeTopicsToolAdapter {
    fn metadata(&self) -> ToolMetadata {
        use crate::mcp::tools::semantic_search_tools::McpTool as SimpleMcpTool;
        let name = self.inner.name().to_string();
        let schema = self.inner.schema();

        let description = schema["description"]
            .as_str()
            .unwrap_or("Extract semantic topics from codebase")
            .to_string();

        let input_schema = schema["parameters"].clone();

        ToolMetadata {
            name,
            description,
            input_schema,
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        use crate::mcp::tools::semantic_search_tools::McpTool as SimpleMcpTool;

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
