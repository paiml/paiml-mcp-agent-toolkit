//! Prompt generation tool handlers for the pmcp-based MCP server.
//!
//! This module contains tool handlers for generating AI prompts enriched with
//! organizational intelligence and defect patterns from OIP analysis.

use crate::mcp_pmcp::tool_functions;
use async_trait::async_trait;
use pmcp::{Error, RequestHandlerExtra, Result, ToolHandler};
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use tracing::debug;

// Re-export for convenience
pub use self::DefectAwarePromptTool as GenerateDefectAwarePromptTool;

// Defect-Aware Prompt Generation Tool

#[derive(Debug, Deserialize)]
struct DefectAwarePromptArgs {
    task: String,
    context: String,
    summary_path: String,
}

/// Tool handler for generating defect-aware AI prompts.
///
/// This tool generates context-aware AI prompts that include organizational
/// quality standards and historical defect patterns from OIP (Organizational
/// Intelligence Plugin) analysis. It helps ensure AI-generated code aligns
/// with organizational best practices and avoids common defect patterns.
///
/// # Arguments
///
/// The tool accepts JSON arguments with the following schema:
/// ```json
/// {
///   "task": "Implement HTTP client with retry logic",
///   "context": "External API integration for payment processing",
///   "summary_path": "/tmp/paiml-summary.yaml"
/// }
/// ```
///
/// # Returns
///
/// Returns a JSON object containing:
/// - `status`: "completed" or "failed"
/// - `prompt`: Generated AI prompt with organizational context
/// - `metadata`: Analysis metadata (repos, commits, patterns)
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::mcp_pmcp::prompt_handlers::GenerateDefectAwarePromptTool;
/// use pmcp::ToolHandler;
/// use serde_json::json;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let tool = GenerateDefectAwarePromptTool::new();
/// let args = json!({
///     "task": "Build configuration parser",
///     "context": "Microservices configuration management",
///     "summary_path": "/tmp/org-summary.yaml"
/// });
///
/// // In practice, this would be called by the MCP server
/// // let result = tool.handle(args, Default::default()).await?;
/// # Ok(())
/// # }
/// ```
pub struct DefectAwarePromptTool;

impl DefectAwarePromptTool {
    /// Creates a new defect-aware prompt generation tool handler.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefectAwarePromptTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for DefectAwarePromptTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling generate_defect_aware_prompt with args: {}", args);

        let params: DefectAwarePromptArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let summary_path = PathBuf::from(params.summary_path);

        let results =
            tool_functions::generate_defect_aware_prompt(params.task, params.context, summary_path)
                .await
                .map_err(|e| Error::internal(format!("Prompt generation failed: {e}")))?;

        Ok(results)
    }
}
