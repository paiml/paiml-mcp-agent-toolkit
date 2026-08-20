//! pmcp ToolHandler adapters for the 4 `pmat_*` AgentContextTools (KAIZEN-0165).
//!
//! The tools in `crate::mcp::tools::agent_context_tools` implement a custom
//! `McpTool` trait with `execute(Value) -> Result<Value, String>`. pmcp's
//! `Server::builder().tool(...)` requires `pmcp::ToolHandler` with
//! `handle(Value, RequestHandlerExtra) -> pmcp::Result<Value>`. These thin
//! newtype wrappers adapt one to the other so the 4 tools appear in the live
//! MCP stdio tools/list.
//!
//! Each wrapper also forwards `metadata()` to return a `pmcp::types::ToolInfo`
//! sourced from `mcp_tool_schemas/<tool_name>.json` via the KAIZEN-0178
//! build.rs codegen, so tools/list advertises proper description + inputSchema
//! instead of empty metadata.

use crate::mcp::tools::agent_context_tools::{
    FindSimilarTool, GetFunctionTool, IndexManager, IndexStatsTool, McpTool as InnerMcpTool,
    QueryCodeTool, ToolError,
};
use async_trait::async_trait;
use pmcp::{Error as PmcpError, RequestHandlerExtra, Result as PmcpResult, ToolHandler};
use serde_json::Value;
use std::sync::Arc;

/// Turn an `McpTool` error into the pmcp error its origin deserves.
///
/// This used to guess the origin from a hand-maintained list of message
/// PREFIXES (`"Missing required parameter"`, `"Invalid function_id format"`,
/// `"Function not found"`), and everything unlisted fell through to
/// `Error::internal`. Three documented bounds were unlisted, so
/// `{limit: 9999}` against a `maximum: 100`, `{min_similarity: 5.0}` against a
/// `maximum: 1.0` and `{query: "   "}` all reported `-32603 Internal error` —
/// telling the host that pmat had faulted when the host had sent a bad value.
/// A prefix table can only ever describe the messages that existed when it was
/// written; [`ToolError`] carries the classification from the site that knows
/// it, so a new bound cannot silently join the internal bucket.
fn adapt_tool_error(error: ToolError) -> PmcpError {
    match error {
        // `Error::Validation` is what the stdio transport re-codes to -32602.
        ToolError::InvalidParams(message) => PmcpError::validation(message),
        ToolError::Internal(message) => PmcpError::internal(message),
    }
}

macro_rules! impl_pmat_handler {
    ($wrapper:ident, $inner:ident, $tool_name:expr) => {
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
            async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> PmcpResult<Value> {
                self.inner.execute(args).await.map_err(adapt_tool_error)
            }

            fn metadata(&self) -> Option<pmcp::types::ToolInfo> {
                Some(crate::mcp_pmcp::tool_schemas_generated::tool_info_for(
                    $tool_name,
                ))
            }
        }
    };
}

impl_pmat_handler!(PmatQueryCodeHandler, QueryCodeTool, "pmat_query_code");
impl_pmat_handler!(PmatGetFunctionHandler, GetFunctionTool, "pmat_get_function");
impl_pmat_handler!(PmatFindSimilarHandler, FindSimilarTool, "pmat_find_similar");
impl_pmat_handler!(PmatIndexStatsHandler, IndexStatsTool, "pmat_index_stats");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    /// `-32603 Internal error: Invalid function_id format` blamed pmat for the
    /// caller sending `no_separator`. `Error::Validation` is what the stdio
    /// transport re-codes to -32602.
    #[test]
    fn argument_faults_become_validation_errors() {
        for message in [
            "Missing required parameter: function_id",
            "Invalid function_id format. Expected 'file_path::function_name', got: x",
            "Function not found: src/a.rs::gone",
        ] {
            assert!(
                matches!(
                    adapt_tool_error(ToolError::invalid(message)),
                    PmcpError::Validation(_)
                ),
                "{message} is the caller's mistake, not ours"
            );
        }
    }

    /// The other direction: a real failure of ours must not be blamed on the
    /// caller.
    #[test]
    fn our_own_failures_stay_internal() {
        for message in [
            "Failed to build index: permission denied",
            "IO error reading the index",
        ] {
            assert!(
                matches!(
                    adapt_tool_error(ToolError::from(message.to_string())),
                    PmcpError::Internal(_)
                ),
                "{message} must not be reported as a bad argument"
            );
        }
    }

    /// REGRESSION: every documented bound on these four tools' arguments must
    /// arrive as -32602, not -32603.
    ///
    /// The three below are the ones the prefix table missed. They are asserted
    /// through the tools' real `execute` — not by re-stating the message here —
    /// so a future edit that stops classifying one of them fails this test
    /// rather than a hand-copied string.
    #[tokio::test]
    async fn documented_bounds_are_invalid_params_not_internal_errors() {
        let mgr = || Arc::new(IndexManager::new(std::path::PathBuf::from(".")));

        let cases: Vec<(&str, PmcpResult<Value>)> = vec![
            (
                "limit above the schema maximum",
                QueryCodeTool::new(mgr())
                    .execute(serde_json::json!({"query": "x", "limit": 9999}))
                    .await
                    .map_err(adapt_tool_error),
            ),
            (
                "blank query",
                QueryCodeTool::new(mgr())
                    .execute(serde_json::json!({"query": "   "}))
                    .await
                    .map_err(adapt_tool_error),
            ),
            (
                "min_grade outside the schema enum",
                QueryCodeTool::new(mgr())
                    .execute(serde_json::json!({"query": "x", "min_grade": "Z"}))
                    .await
                    .map_err(adapt_tool_error),
            ),
            (
                "min_similarity above the schema maximum",
                FindSimilarTool::new(mgr())
                    .execute(serde_json::json!({"function_id": "a.rs::b", "min_similarity": 5.0}))
                    .await
                    .map_err(adapt_tool_error),
            ),
            (
                "find_similar limit above the schema maximum",
                FindSimilarTool::new(mgr())
                    .execute(serde_json::json!({"function_id": "a.rs::b", "limit": 999}))
                    .await
                    .map_err(adapt_tool_error),
            ),
        ];

        for (what, outcome) in cases {
            match outcome {
                Err(PmcpError::Validation(_)) => {}
                other => panic!(
                    "{what} is a caller-supplied value failing a documented bound; \
                     it must map to -32602 Invalid params, got: {other:?}"
                ),
            }
        }
    }
}
