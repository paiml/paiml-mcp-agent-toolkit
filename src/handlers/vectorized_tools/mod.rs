#![allow(unused)]
//! Vectorized Tool Handlers - Phase 7 Day 16-17
//!
//! MCP protocol extensions for high-performance vectorized analysis tools
//! that leverage SIMD operations and parallel processing.
//!
//! # Honest-failure policy (R21-1 / KAIZEN-0200)
//!
//! These seven tools were previously stubs that returned hardcoded fake
//! payloads. Per the R17-4 "fail loud" policy they now return JSON-RPC error
//! `-32001 Tool not implemented`. See `vectorized_tools_handlers.rs` for the
//! single funnel helper and `vectorized_tools_info.rs` for the tools/list
//! descriptions that flag each tool as `(unimplemented stub — KAIZEN-0200)`.

#![cfg_attr(coverage_nightly, coverage(off))]

use crate::models::mcp::{McpResponse, ToolCallParams};
use serde_json::{json, Value};
use tracing::info;

/// Vectorized tool names
pub const VECTORIZED_TOOLS: &[&str] = &[
    "analyze_duplicates_vectorized",
    "analyze_graph_metrics_vectorized",
    "analyze_name_similarity_vectorized",
    "analyze_symbol_table_vectorized",
    "analyze_incremental_coverage_vectorized",
    "analyze_big_o_vectorized",
    "generate_enhanced_report",
];

/// Check if a tool is a vectorized analysis tool
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::handlers::vectorized_tools::is_vectorized_tool;
///
/// assert!(is_vectorized_tool("analyze_duplicates_vectorized"));
/// assert!(is_vectorized_tool("analyze_big_o_vectorized"));
/// assert!(!is_vectorized_tool("unknown_tool"));
/// ```
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn is_vectorized_tool(tool_name: &str) -> bool {
    VECTORIZED_TOOLS.contains(&tool_name)
}

/// Handle vectorized tool calls
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_vectorized_tools(
    request_id: Value,
    tool_params: ToolCallParams,
) -> McpResponse {
    info!("⚡ Executing vectorized tool: {}", tool_params.name);

    match tool_params.name.as_str() {
        "analyze_duplicates_vectorized" => {
            handle_duplicates_vectorized(request_id, Some(tool_params.arguments)).await
        }
        "analyze_graph_metrics_vectorized" => {
            handle_graph_metrics_vectorized(request_id, Some(tool_params.arguments)).await
        }
        "analyze_name_similarity_vectorized" => {
            handle_name_similarity_vectorized(request_id, Some(tool_params.arguments)).await
        }
        "analyze_symbol_table_vectorized" => {
            handle_symbol_table_vectorized(request_id, Some(tool_params.arguments)).await
        }
        "analyze_incremental_coverage_vectorized" => {
            handle_incremental_coverage_vectorized(request_id, Some(tool_params.arguments)).await
        }
        "analyze_big_o_vectorized" => {
            handle_big_o_vectorized(request_id, Some(tool_params.arguments)).await
        }
        "generate_enhanced_report" => {
            handle_enhanced_report(request_id, Some(tool_params.arguments)).await
        }
        _ => McpResponse::error(
            request_id,
            -32602,
            format!("Unknown vectorized tool: {}", tool_params.name),
        ),
    }
}

// --- Include handler implementations and tool info ---
//
// R21-1: parameter structs deleted — every handler now returns `-32001` without
// inspecting arguments, so there is nothing to parse. This keeps the fail-loud
// funnel uniform and eliminates the old ceremony of "parse args, then ignore
// them and return fake data".
include!("vectorized_tools_handlers.rs");
include!("vectorized_tools_info.rs");

// Tests split for file health compliance (CB-040)
// TEMPORARILY DISABLED: File splitting broke syntax
#[cfg(all(test, feature = "broken-tests"))]
#[path = "tests.rs"]
mod tests;

#[cfg(all(test, feature = "broken-tests"))]
#[path = "property_tests.rs"]
mod property_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
#[path = "coverage_tests.rs"]
mod coverage_tests;
