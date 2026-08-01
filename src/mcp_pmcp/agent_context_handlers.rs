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
    QueryCodeTool,
};
use async_trait::async_trait;
use pmcp::{Error as PmcpError, RequestHandlerExtra, Result as PmcpResult, ToolHandler};
use serde_json::Value;
use std::sync::Arc;

/// Prefixes of the error strings these four tools produce for a **caller's**
/// mistake, as opposed to a failure of ours.
///
/// The inner `McpTool` trait carries errors as bare `String`s, so the variant is
/// the only classification available here. Every prefix below is produced by
/// exactly one construct in `crate::mcp::tools::agent_context_tools`:
///
/// * `"Missing required parameter"` — `params[…].as_str().ok_or(…)`
/// * `"Invalid function_id format"` — `parse_function_id`
/// * `"Function not found"` — the caller named a symbol the index does not
///   have; the same shape as `tool_schemas::resolve_existing_paths`' "path(s)
///   not found", which is already a validation error.
///
/// Everything else (index-build failures, IO) stays an internal error. Mapping
/// these to `Error::validation` is what lets the stdio transport re-code them
/// -32602 instead of -32603: `-32603 Internal error: Invalid function_id format`
/// told the host that pmat had faulted when the host had sent `no_separator`.
const CLIENT_FAULT_PREFIXES: &[&str] = &[
    "Missing required parameter",
    "Invalid function_id format",
    "Function not found",
];

/// Turn an `McpTool` error string into the pmcp error its origin deserves.
fn adapt_tool_error(message: String) -> PmcpError {
    if CLIENT_FAULT_PREFIXES
        .iter()
        .any(|p| message.starts_with(*p))
    {
        return PmcpError::validation(message);
    }
    PmcpError::internal(message)
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
                    adapt_tool_error(message.to_string()),
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
                    adapt_tool_error(message.to_string()),
                    PmcpError::Internal(_)
                ),
                "{message} must not be reported as a bad argument"
            );
        }
    }
}
