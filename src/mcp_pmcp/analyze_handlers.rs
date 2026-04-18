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
use async_trait::async_trait;
use pmcp::{Error, RequestHandlerExtra, Result, ToolHandler};
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use tracing::debug;

// Re-export for convenience.
//
// NOTE (D39/D47/D48 fix): These aliases were previously mis-labeled — the
// `AnalyzeBigOTool`, `AnalyzeDeepContextTool`, and `AnalyzeDagTool` names
// aliased `CouplingTool`, `ChurnTool`, and `LintHotspotTool` respectively,
// which dispatched MCP calls to the wrong handlers. The aliases now match
// the underlying behavior: coupling, churn, and lint-hotspot analysis.
pub use self::{
    ChurnTool as AnalyzeChurnTool, ComplexityTool as AnalyzeComplexityTool,
    CouplingTool as AnalyzeCouplingTool, DeadCodeTool as AnalyzeDeadCodeTool,
    LintHotspotTool as AnalyzeLintHotspotsTool, SatdTool as AnalyzeSatdTool,
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
