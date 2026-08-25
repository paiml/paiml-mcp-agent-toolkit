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
//! - `analyze_forensics_handlers.rs` — Reachability, hardcoded paths, vacuous tests (#1029)
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

// Re-export for convenience.
//
// NOTE (R17-1): AnalyzeDagTool / AnalyzeDeepContextTool / AnalyzeBigOTool are
// *not* aliases for LintHotspotTool / ChurnTool / CouplingTool. They are
// distinct structs defined below that dispatch to the correct
// `tool_functions::*` implementation. The earlier aliases mis-routed three
// MCP tools (see R15 #3 bench matrix).
pub use self::{
    ComplexityTool as AnalyzeComplexityTool, DeadCodeTool as AnalyzeDeadCodeTool,
    SatdTool as AnalyzeSatdTool, TdgCompareTool as AnalyzeTdgCompareTool,
    TdgTool as AnalyzeTdgTool,
};

// Complexity analysis tool handler
include!("analyze_complexity_handler.rs");

// SATD and dead code analysis tool handlers
include!("analyze_debt_handlers.rs");

// Lint hotspot, churn, and coupling analysis tool handlers
include!("analyze_metrics_handlers.rs");

// TDG scoring and comparison tool handlers
include!("analyze_tdg_tool_handlers.rs");

// #1029: reachability, hardcoded-path and vacuous-test tool handlers — the
// three analyzers that were CLI-only by omission.
include!("analyze_forensics_handlers.rs");

// Tests: the three forensic analyzers exposed in #1029
include!("analyze_forensics_handlers_tests.rs");

// Tests: complexity, debt, and metrics tool handlers
include!("analyze_handlers_tests.rs");

// Tests: TDG tools, deserialization, and integration tests
include!("analyze_handlers_tests_tdg.rs");
