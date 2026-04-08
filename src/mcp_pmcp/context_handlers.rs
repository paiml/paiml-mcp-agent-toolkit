//! Context and git tool handlers for the pmcp-based MCP server.

use crate::mcp_pmcp::tool_functions;
use async_trait::async_trait;
use pmcp::{Error, RequestHandlerExtra, Result, ToolHandler};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use tracing::debug;

// Re-export with expected names
pub use self::{
    ContextGenerateTool as GenerateContextTool, ContextSummaryTool as ScaffoldProjectTool,
    GitStatusTool as GitTool,
};

// --- Struct definitions ---

/// Git clone tool.
pub struct GitCloneTool;
/// Git status tool.
pub struct GitStatusTool;
/// Context generate tool.
pub struct ContextGenerateTool;
/// Context analyze tool.
pub struct ContextAnalyzeTool;
/// Context summary tool.
pub struct ContextSummaryTool;

// --- Include files ---

// Git tool handlers: GitCloneTool and GitStatusTool args, impls, Default, ToolHandler
include!("context_handlers_git.rs");

// Context tool handlers: ContextGenerateTool, ContextAnalyzeTool, ContextSummaryTool args, impls, Default, ToolHandler
include!("context_handlers_context.rs");

// Property tests and coverage tests for all context/git handlers
include!("context_handlers_tests.rs");
