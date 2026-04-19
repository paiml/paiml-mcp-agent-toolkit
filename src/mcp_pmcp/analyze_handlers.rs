//! Analysis tool handlers for the pmcp-based MCP server.
//!
//! This module contains tool handlers for various code analysis operations
//! including complexity analysis, technical debt detection, and code quality metrics.
//!
//! # Module Structure
//!
//! The implementation is split across include files for maintainability:
//! - `analyze_complexity_handler.rs` — Complexity analysis tool
//! - `analyze_debt_handlers.rs` — SATD and dead code analysis tools
//! - `analyze_metrics_handlers.rs` — Lint hotspot, churn, and coupling tools
//! - `analyze_tdg_tool_handlers.rs` — TDG scoring and comparison tools
//! - `analyze_handlers_tests.rs` — Unit tests (complexity, debt, metrics)
//! - `analyze_handlers_tests_tdg.rs` — Unit tests (TDG, deserialization, integration)

use crate::mcp_pmcp::tool_functions;
use crate::mcp_pmcp::tool_schemas::{build_tool_info, paths_object_schema};
use async_trait::async_trait;
use pmcp::types::ToolInfo;
use pmcp::{Error, RequestHandlerExtra, Result, ToolHandler};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use tracing::debug;

// Re-export for convenience
pub use self::{
    ChurnTool as AnalyzeDeepContextTool, ComplexityTool as AnalyzeComplexityTool,
    CouplingTool as AnalyzeBigOTool, DeadCodeTool as AnalyzeDeadCodeTool,
    LintHotspotTool as AnalyzeDagTool, SatdTool as AnalyzeSatdTool,
    TdgCompareTool as AnalyzeTdgCompareTool, TdgTool as AnalyzeTdgTool,
};

// Complexity analysis tool handler
include!("analyze_complexity_handler.rs");

// SATD and dead code analysis tool handlers
include!("analyze_debt_handlers.rs");

// Lint hotspot, churn, and coupling analysis tool handlers
include!("analyze_metrics_handlers.rs");

// TDG scoring and comparison tool handlers
include!("analyze_tdg_tool_handlers.rs");

// Tests: complexity, debt, and metrics tool handlers
include!("analyze_handlers_tests.rs");

// Tests: TDG tools, deserialization, and integration tests
include!("analyze_handlers_tests_tdg.rs");
