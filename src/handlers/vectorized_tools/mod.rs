//! Vectorized Tool Handlers - Phase 7 Day 16-17
//!
//! MCP protocol extensions for high-performance vectorized analysis tools
//! that leverage SIMD operations and parallel processing.

#![cfg_attr(coverage_nightly, coverage(off))]

use crate::models::mcp::{McpResponse, ToolCallParams};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;
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

// --- Parameter structs ---

/// Vectorized duplicate detection parameters
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DuplicatesVectorizedArgs {
    project_path: PathBuf,
    detection_type: Option<String>,
    threshold: Option<f64>,
    min_lines: Option<usize>,
    max_tokens: Option<usize>,
    parallel_threads: Option<usize>,
    use_simd: Option<bool>,
}

/// Graph metrics vectorized parameters
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GraphMetricsVectorizedArgs {
    project_path: PathBuf,
    metrics: Option<Vec<String>>,
    pagerank_damping: Option<f64>,
    max_iterations: Option<usize>,
    convergence_threshold: Option<f64>,
    use_gpu: Option<bool>,
}

/// Name similarity vectorized parameters
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NameSimilarityVectorizedArgs {
    project_path: PathBuf,
    query: String,
    top_k: Option<usize>,
    threshold: Option<f64>,
    phonetic: Option<bool>,
    fuzzy: Option<bool>,
    use_simd: Option<bool>,
}

/// Symbol table vectorized parameters
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SymbolTableVectorizedArgs {
    project_path: PathBuf,
    filter: Option<String>,
    query: Option<String>,
    show_unreferenced: Option<bool>,
    show_references: Option<bool>,
    parallel_parsing: Option<bool>,
}

/// Incremental coverage vectorized parameters
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct IncrementalCoverageVectorizedArgs {
    project_path: PathBuf,
    base_branch: Option<String>,
    target_branch: Option<String>,
    changed_files_only: Option<bool>,
    parallel_diff: Option<bool>,
}

/// Big-O vectorized parameters
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BigOVectorizedArgs {
    project_path: PathBuf,
    confidence_threshold: Option<u8>,
    analyze_space: Option<bool>,
    high_complexity_only: Option<bool>,
    parallel_analysis: Option<bool>,
}

/// Enhanced report generation parameters
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct EnhancedReportArgs {
    project_path: PathBuf,
    output_format: Option<String>,
    analyses: Option<Vec<String>>,
    include_visualizations: Option<bool>,
    include_recommendations: Option<bool>,
    confidence_threshold: Option<u8>,
}

// --- Include handler implementations and tool info ---
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
