//! pmcp ToolHandler adapters for the 4 `pmat_*` AgentContextTools (KAIZEN-0165).
//!
//! The tools in `crate::mcp::tools::agent_context_tools` implement a custom
//! `McpTool` trait with `execute(Value) -> Result<Value, String>`. pmcp's
//! `Server::builder().tool(...)` requires `pmcp::ToolHandler` with
//! `handle(Value, RequestHandlerExtra) -> pmcp::Result<Value>`. These thin
//! newtype wrappers adapt one to the other so the 4 tools appear in the live
//! MCP stdio tools/list.
//!
//! KAIZEN-0174: Forward `description` + `input_schema` from the inner tool via
//! the `ToolHandler::metadata()` override so MCP clients can render forms for
//! these tools. The inner `McpTool::schema()` returns a JSON shaped as
//! `{"name": "...", "description": "...", "parameters": {...}}` — we extract
//! the fields into `pmcp::types::ToolInfo` which pmcp's builder caches at
//! registration time and serves from `tools/list`.

use crate::mcp::tools::agent_context_tools::{
    FindSimilarTool, GetFunctionTool, IndexManager, IndexStatsTool, McpTool as InnerMcpTool,
    QueryCodeTool,
};
use async_trait::async_trait;
use pmcp::types::ToolInfo;
use pmcp::{Error as PmcpError, RequestHandlerExtra, Result as PmcpResult, ToolHandler};
use serde_json::{json, Value};
use std::sync::Arc;

/// Extract `{name, description, input_schema}` from the inner McpTool schema JSON.
///
/// The inner `McpTool::schema()` returns a `{"name", "description", "parameters"}`
/// envelope. pmcp's `ToolInfo` expects a flat `{name, description, input_schema}`.
/// This helper does the translation so every `pmat_*` handler's `metadata()` can
/// feed pmcp's builder the real schema instead of `json!({})`.
fn inner_schema_to_tool_info(inner_schema: &Value) -> ToolInfo {
    let name = inner_schema
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let description = inner_schema
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let input_schema = inner_schema
        .get("parameters")
        .cloned()
        .unwrap_or_else(|| json!({"type": "object"}));
    ToolInfo::new(name, description, input_schema)
}

macro_rules! impl_pmat_handler {
    ($wrapper:ident, $inner:ident) => {
        pub struct $wrapper {
            inner: $inner,
        }

        impl $wrapper {
            pub fn new(mgr: Arc<IndexManager>) -> Self {
                Self {
                    inner: $inner::new(mgr),
                }
            }
        }

        #[async_trait]
        impl ToolHandler for $wrapper {
            async fn handle(
                &self,
                args: Value,
                _extra: RequestHandlerExtra,
            ) -> PmcpResult<Value> {
                self.inner.execute(args).await.map_err(PmcpError::internal)
            }

            fn metadata(&self) -> Option<ToolInfo> {
                Some(inner_schema_to_tool_info(&self.inner.schema()))
            }
        }
    };
}

impl_pmat_handler!(PmatQueryCodeHandler, QueryCodeTool);
impl_pmat_handler!(PmatGetFunctionHandler, GetFunctionTool);
impl_pmat_handler!(PmatFindSimilarHandler, FindSimilarTool);
impl_pmat_handler!(PmatIndexStatsHandler, IndexStatsTool);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::tools::agent_context_tools::IndexManager;
    use std::path::PathBuf;

    fn make_index_manager() -> Arc<IndexManager> {
        Arc::new(IndexManager::new(PathBuf::from(".")))
    }

    #[test]
    fn test_pmat_query_code_has_input_schema() {
        let handler = PmatQueryCodeHandler::new(make_index_manager());
        let info = handler.metadata().expect("metadata should be Some");

        assert_eq!(info.name, "pmat_query_code");
        assert!(
            info.description.is_some(),
            "description should be forwarded"
        );
        assert!(
            info.description.as_deref().unwrap().contains("quality"),
            "description should mention quality filtering"
        );

        let schema = &info.input_schema;
        assert_eq!(schema["type"], "object", "input_schema must be object type");
        assert!(
            schema["properties"]["query"].is_object(),
            "input_schema must include `query` property"
        );
        assert_eq!(
            schema["required"].as_array().expect("required array"),
            &vec![Value::String("query".to_string())]
        );
    }

    #[test]
    fn test_pmat_get_function_has_input_schema() {
        let handler = PmatGetFunctionHandler::new(make_index_manager());
        let info = handler.metadata().expect("metadata should be Some");
        assert_eq!(info.name, "pmat_get_function");
        assert!(info.description.is_some());
        assert_eq!(info.input_schema["type"], "object");
    }

    #[test]
    fn test_pmat_find_similar_has_input_schema() {
        let handler = PmatFindSimilarHandler::new(make_index_manager());
        let info = handler.metadata().expect("metadata should be Some");
        assert_eq!(info.name, "pmat_find_similar");
        assert!(info.description.is_some());
        assert_eq!(info.input_schema["type"], "object");
    }

    #[test]
    fn test_pmat_index_stats_has_input_schema() {
        let handler = PmatIndexStatsHandler::new(make_index_manager());
        let info = handler.metadata().expect("metadata should be Some");
        assert_eq!(info.name, "pmat_index_stats");
        assert!(info.description.is_some());
        assert_eq!(info.input_schema["type"], "object");
    }
}
