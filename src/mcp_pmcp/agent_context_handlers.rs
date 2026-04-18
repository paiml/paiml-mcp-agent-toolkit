//! pmcp ToolHandler adapters for the 4 `pmat_*` AgentContextTools (KAIZEN-0165).
//!
//! The tools in `crate::mcp::tools::agent_context_tools` implement a custom
//! `McpTool` trait with `execute(Value) -> Result<Value, String>`. pmcp's
//! `Server::builder().tool(...)` requires `pmcp::ToolHandler` with
//! `handle(Value, RequestHandlerExtra) -> pmcp::Result<Value>`. These thin
//! newtype wrappers adapt one to the other so the 4 tools appear in the live
//! MCP stdio tools/list.

use crate::mcp::tools::agent_context_tools::{
    FindSimilarTool, GetFunctionTool, IndexManager, IndexStatsTool, McpTool as InnerMcpTool,
    QueryCodeTool,
};
use async_trait::async_trait;
use pmcp::{Error as PmcpError, RequestHandlerExtra, Result as PmcpResult, ToolHandler};
use serde_json::Value;
use std::sync::Arc;

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
        }
    };
}

impl_pmat_handler!(PmatQueryCodeHandler, QueryCodeTool);
impl_pmat_handler!(PmatGetFunctionHandler, GetFunctionTool);
impl_pmat_handler!(PmatFindSimilarHandler, FindSimilarTool);
impl_pmat_handler!(PmatIndexStatsHandler, IndexStatsTool);
