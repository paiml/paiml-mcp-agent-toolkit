// Vectorized tool handler implementations
// Included from mod.rs - shares parent module scope (no `use` imports here)
//
// # Honest-failure policy (R21-1 / KAIZEN-0200 / D90 / D92)
//
// These seven "vectorized" tools were pure stubs that returned hardcoded fake
// payloads (150 files / 25 duplicates, 256 nodes / 1024 edges, fake
// `src/handlers/request.rs:45` locations, fake 1250 symbols, fake 78.5%
// coverage, fake 450 functions, fake 250 files / 25000 lines). They
// masqueraded as real analysis, which is strictly worse than failing — clients
// silently believed the fake numbers and made decisions on them.
//
// Per the R17-4 serve fail-loud policy, every invocation now funnels through a
// single `write_unimplemented_message()` helper and returns a JSON-RPC error
// with code `-32001` (application-defined — "not implemented"). The tool still
// appears in `tools/list` so discovery works, but its description is marked
// "(unimplemented stub — KAIZEN-0200)" so callers see the state before they
// invoke it.

/// JSON-RPC error code for "tool not implemented". `-32001` is in the
/// application-reserved range and mirrors the convention used by other pmat
/// fail-loud call sites.
const UNIMPLEMENTED_ERROR_CODE: i32 = -32001;

/// Build the canonical "not yet implemented" error message for a vectorized
/// tool invocation. Extracted so unit tests can pin the exact text.
fn write_unimplemented_message(tool_name: &str) -> String {
    format!(
        "Tool '{tool_name}' is not yet implemented. Tracking: KAIZEN-0200. \
This is a known stub that was previously masquerading as real output — \
see R21-1 / D90 / D92."
    )
}

/// Return the canonical "unimplemented" McpResponse for a vectorized tool.
fn unimplemented_response(request_id: Value, tool_name: &str) -> McpResponse {
    McpResponse::error(
        request_id,
        UNIMPLEMENTED_ERROR_CODE,
        write_unimplemented_message(tool_name),
    )
}

/// Handle vectorized duplicate detection — FAIL LOUD (was stub with fake data).
async fn handle_duplicates_vectorized(request_id: Value, _args: Option<Value>) -> McpResponse {
    info!("⚠️  analyze_duplicates_vectorized invoked — returning unimplemented (R21-1)");
    unimplemented_response(request_id, "analyze_duplicates_vectorized")
}

/// Handle vectorized graph metrics analysis — FAIL LOUD (was stub with fake data).
async fn handle_graph_metrics_vectorized(request_id: Value, _args: Option<Value>) -> McpResponse {
    info!("⚠️  analyze_graph_metrics_vectorized invoked — returning unimplemented (R21-1)");
    unimplemented_response(request_id, "analyze_graph_metrics_vectorized")
}

/// Handle vectorized name similarity search — FAIL LOUD (was stub with fake data).
async fn handle_name_similarity_vectorized(request_id: Value, _args: Option<Value>) -> McpResponse {
    info!("⚠️  analyze_name_similarity_vectorized invoked — returning unimplemented (R21-1)");
    unimplemented_response(request_id, "analyze_name_similarity_vectorized")
}

/// Handle vectorized symbol table analysis — FAIL LOUD (was stub with fake data).
async fn handle_symbol_table_vectorized(request_id: Value, _args: Option<Value>) -> McpResponse {
    info!("⚠️  analyze_symbol_table_vectorized invoked — returning unimplemented (R21-1)");
    unimplemented_response(request_id, "analyze_symbol_table_vectorized")
}

/// Handle vectorized incremental coverage analysis — FAIL LOUD (was stub with fake data).
async fn handle_incremental_coverage_vectorized(
    request_id: Value,
    _args: Option<Value>,
) -> McpResponse {
    info!("⚠️  analyze_incremental_coverage_vectorized invoked — returning unimplemented (R21-1)");
    unimplemented_response(request_id, "analyze_incremental_coverage_vectorized")
}

/// Handle vectorized Big-O complexity analysis — FAIL LOUD (was stub with fake data).
async fn handle_big_o_vectorized(request_id: Value, _args: Option<Value>) -> McpResponse {
    info!("⚠️  analyze_big_o_vectorized invoked — returning unimplemented (R21-1)");
    unimplemented_response(request_id, "analyze_big_o_vectorized")
}

/// Handle enhanced report generation — FAIL LOUD (was stub with fake data).
async fn handle_enhanced_report(request_id: Value, _args: Option<Value>) -> McpResponse {
    info!("⚠️  generate_enhanced_report invoked — returning unimplemented (R21-1)");
    unimplemented_response(request_id, "generate_enhanced_report")
}

// ---------------------------------------------------------------------------
// Inline unit tests — verify all 7 handlers return the canonical error shape.
// ---------------------------------------------------------------------------

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod fail_loud_tests {
    use super::*;
    use serde_json::json;

    fn assert_unimplemented(resp: &McpResponse, expected_tool: &str) {
        assert!(
            resp.result.is_none(),
            "fail-loud response must not carry a result payload"
        );
        let err = resp
            .error
            .as_ref()
            .expect("fail-loud response must carry an error");
        assert_eq!(
            err.code, UNIMPLEMENTED_ERROR_CODE,
            "error code must be -32001 (application-defined 'not implemented')"
        );
        assert!(
            err.message.contains(expected_tool),
            "error message must name the invoked tool, got: {}",
            err.message
        );
        assert!(
            err.message.contains("not yet implemented"),
            "error message must clearly state unimplemented, got: {}",
            err.message
        );
        assert!(
            err.message.contains("KAIZEN-0200"),
            "error message must reference tracking ticket, got: {}",
            err.message
        );
    }

    #[tokio::test]
    async fn duplicates_vectorized_fails_loud() {
        let r = handle_duplicates_vectorized(json!(1), Some(json!({"project_path": "/tmp"}))).await;
        assert_unimplemented(&r, "analyze_duplicates_vectorized");
    }

    #[tokio::test]
    async fn graph_metrics_vectorized_fails_loud() {
        let r =
            handle_graph_metrics_vectorized(json!(2), Some(json!({"project_path": "/tmp"}))).await;
        assert_unimplemented(&r, "analyze_graph_metrics_vectorized");
    }

    #[tokio::test]
    async fn name_similarity_vectorized_fails_loud() {
        let r = handle_name_similarity_vectorized(
            json!(3),
            Some(json!({"project_path": "/tmp", "query": "foo"})),
        )
        .await;
        assert_unimplemented(&r, "analyze_name_similarity_vectorized");
    }

    #[tokio::test]
    async fn symbol_table_vectorized_fails_loud() {
        let r =
            handle_symbol_table_vectorized(json!(4), Some(json!({"project_path": "/tmp"}))).await;
        assert_unimplemented(&r, "analyze_symbol_table_vectorized");
    }

    #[tokio::test]
    async fn incremental_coverage_vectorized_fails_loud() {
        let r =
            handle_incremental_coverage_vectorized(json!(5), Some(json!({"project_path": "/tmp"})))
                .await;
        assert_unimplemented(&r, "analyze_incremental_coverage_vectorized");
    }

    #[tokio::test]
    async fn big_o_vectorized_fails_loud() {
        let r = handle_big_o_vectorized(json!(6), Some(json!({"project_path": "/tmp"}))).await;
        assert_unimplemented(&r, "analyze_big_o_vectorized");
    }

    #[tokio::test]
    async fn enhanced_report_fails_loud() {
        let r = handle_enhanced_report(json!(7), Some(json!({"project_path": "/tmp"}))).await;
        assert_unimplemented(&r, "generate_enhanced_report");
    }

    #[tokio::test]
    async fn no_args_still_fails_loud_not_silent() {
        // Regression guard: previously a missing `project_path` would return
        // "Missing required parameters". Now we refuse even with args because
        // the whole tool is unimplemented — don't pretend to validate input
        // for a feature that doesn't exist.
        let r = handle_duplicates_vectorized(json!(8), None).await;
        assert_unimplemented(&r, "analyze_duplicates_vectorized");
    }

    #[test]
    fn message_is_stable_and_mentions_tracking() {
        let msg = write_unimplemented_message("analyze_duplicates_vectorized");
        assert!(msg.contains("analyze_duplicates_vectorized"));
        assert!(msg.contains("not yet implemented"));
        assert!(msg.contains("KAIZEN-0200"));
        assert!(msg.contains("masquerading"));
    }

    #[test]
    fn dispatch_through_handle_vectorized_tools_fails_loud() {
        // Verify the public `handle_vectorized_tools` dispatcher funnels every
        // known tool through the unimplemented path.
        use crate::models::mcp::ToolCallParams;
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        for tool in VECTORIZED_TOOLS {
            let params = ToolCallParams {
                name: (*tool).to_string(),
                arguments: json!({"project_path": "/tmp", "query": "foo"}),
            };
            let resp = rt.block_on(handle_vectorized_tools(json!(99), params));
            assert_unimplemented(&resp, tool);
        }
    }
}
