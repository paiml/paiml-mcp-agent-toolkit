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

// --- Tool call dispatch (routing, classification) ---
include!("core_tools_dispatch.rs");

// --- Template tool handlers (generate, list, validate, scaffold, search, server info) ---
include!("core_tools_template_handlers.rs");

// --- Code churn analysis handlers and formatters ---
include!("core_tools_churn.rs");
