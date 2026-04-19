use crate::models::churn::ChurnOutputFormat;

// Import handlers from extracted module (CB-040)
use crate::handlers::tools_advanced::{
    handle_analyze_dead_code, handle_analyze_deep_context, handle_analyze_lint_hotspot,
    handle_analyze_makefile_lint, handle_analyze_provability, handle_analyze_satd,
    handle_analyze_tdg, handle_quality_driven_development,
};
use crate::models::mcp::{
    GenerateTemplateArgs, ListTemplatesArgs, McpRequest, McpResponse, ScaffoldProjectArgs,
    SearchTemplatesArgs, ToolCallParams, ValidateTemplateArgs,
};
use crate::models::template::{ParameterSpec, TemplateResource};
use crate::services::git_analysis::GitAnalysisService;
use crate::services::template_service;
use crate::TemplateServerTrait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{error, info};

/// R22-1 / D101 — Reject missing, null, or empty/whitespace-only
/// `project_path` arguments across MCP analysis handlers.
///
/// This is the parity fix for R21-5 / D99 (PR #371), which added the same
/// guard to the `src/agent/mcp_server_protocol.rs` handlers. The live
/// dispatcher in `src/handlers/tools/` was still silently defaulting to
/// `std::env::current_dir()`, which lets a remote MCP client exfiltrate
/// information about the server's launch directory by sending `{}` or
/// `{"project_path": null}` / `{"project_path": ""}`.
///
/// The helper takes the already-deserialised `Option<String>` (since the
/// handlers use typed arg structs like `AnalyzeComplexityArgs`) and returns
/// a validated `PathBuf` or an error string that the caller maps to
/// JSON-RPC `-32602` (Invalid params).
///
/// Mirrors the R21-1 / D100 fail-loud pattern already in
/// `handle_analyze_dead_code`.
fn require_project_path(project_path_arg: Option<String>) -> Result<PathBuf, String> {
    let Some(raw) = project_path_arg else {
        return Err(
            "'project_path' is required and must be a non-empty string — \
null/missing is rejected to avoid silently analyzing the server's current \
directory (R22-1 / D101)"
                .to_string(),
        );
    };
    if raw.trim().is_empty() {
        return Err(
            "'project_path' must be a non-empty string — empty/whitespace values \
are rejected to avoid silently analyzing the server's current directory \
(R22-1 / D101)"
                .to_string(),
        );
    }
    Ok(PathBuf::from(raw))
}

// --- Tool call dispatch (routing, classification) ---
include!("core_tools_dispatch.rs");

// --- Template tool handlers (generate, list, validate, scaffold, search, server info) ---
include!("core_tools_template_handlers.rs");

// --- Code churn analysis handlers and formatters ---
include!("core_tools_churn.rs");
