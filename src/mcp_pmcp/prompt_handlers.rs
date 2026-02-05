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
/// // let result = tool.handle(args, test_extra()).await?;
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use tokio_util::sync::CancellationToken;

    fn test_extra() -> RequestHandlerExtra {
        RequestHandlerExtra::new("test-request".to_string(), CancellationToken::new())
    }

    // === DefectAwarePromptTool Tests ===

    #[test]
    fn test_defect_aware_prompt_tool_new() {
        let tool = DefectAwarePromptTool::new();
        let _ = tool;
    }

    #[test]
    fn test_defect_aware_prompt_tool_default() {
        let tool = DefectAwarePromptTool::default();
        let _ = tool;
    }

    #[tokio::test]
    async fn test_defect_aware_prompt_tool_invalid_args() {
        let tool = DefectAwarePromptTool::new();
        let args = json!({"invalid": "args"});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_defect_aware_prompt_tool_missing_task() {
        let tool = DefectAwarePromptTool::new();
        let args = json!({
            "context": "Some context",
            "summary_path": "/tmp/summary.yaml"
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_defect_aware_prompt_tool_missing_context() {
        let tool = DefectAwarePromptTool::new();
        let args = json!({
            "task": "Build something",
            "summary_path": "/tmp/summary.yaml"
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_defect_aware_prompt_tool_missing_summary_path() {
        let tool = DefectAwarePromptTool::new();
        let args = json!({
            "task": "Build something",
            "context": "Some context"
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_defect_aware_prompt_tool_nonexistent_summary() {
        let tool = DefectAwarePromptTool::new();
        let args = json!({
            "task": "Build HTTP client",
            "context": "External API integration",
            "summary_path": "/nonexistent/summary.yaml"
        });
        let result = tool.handle(args, test_extra()).await;
        // Should return success with a "failed" status in the JSON
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["status"], "failed");
        assert!(value["message"].as_str().unwrap().contains("not found"));
    }

    #[tokio::test]
    async fn test_defect_aware_prompt_tool_invalid_yaml() {
        let tool = DefectAwarePromptTool::new();

        // Create temp file with invalid YAML
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "invalid: yaml: content: [").unwrap();

        let args = json!({
            "task": "Build HTTP client",
            "context": "External API integration",
            "summary_path": temp_file.path().to_str().unwrap()
        });
        let result = tool.handle(args, test_extra()).await;
        // Should return success with a "failed" status due to parse error
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["status"], "failed");
    }

    #[tokio::test]
    async fn test_defect_aware_prompt_tool_with_valid_summary() {
        let tool = DefectAwarePromptTool::new();

        // Create temp file with valid OipSummary YAML format
        let mut temp_file = NamedTempFile::new().unwrap();
        let yaml_content = r#"
metadata:
  repositories_analyzed: 5
  commits_analyzed: 1000
  analysis_date: "2024-01-01"

organizational_insights:
  top_defect_categories:
    - category: "null_pointer"
      frequency: 15
      confidence: 0.85
      quality_signals:
        avg_tdg_score: 65.0
        max_tdg_score: 80.0
        avg_complexity: 12.5
        avg_test_coverage: 0.75
        satd_instances: 3
        avg_lines_changed: 45.0
        avg_files_per_commit: 2.5

code_quality_thresholds:
  tdg_minimum: 70.0
  test_coverage_minimum: 0.80
  max_function_length: 50
  max_cyclomatic_complexity: 15
"#;
        writeln!(temp_file, "{}", yaml_content).unwrap();

        let args = json!({
            "task": "Build HTTP client with retry logic",
            "context": "External API integration for payment processing",
            "summary_path": temp_file.path().to_str().unwrap()
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["status"], "completed");
        assert!(value["prompt"].is_string());
        assert!(value["metadata"].is_object());
    }

    // === Re-export Tests ===

    #[test]
    fn test_re_export_exists() {
        // Test that the re-export is accessible
        let _: GenerateDefectAwarePromptTool = DefectAwarePromptTool::new();
    }

    // === Args Deserialization Tests ===

    #[test]
    fn test_defect_aware_prompt_args_deserialization() {
        let json_str = r#"{
            "task": "Implement HTTP client",
            "context": "External API integration",
            "summary_path": "/tmp/summary.yaml"
        }"#;
        let args: DefectAwarePromptArgs = serde_json::from_str(json_str).unwrap();
        assert_eq!(args.task, "Implement HTTP client");
        assert_eq!(args.context, "External API integration");
        assert_eq!(args.summary_path, "/tmp/summary.yaml");
    }

    #[test]
    fn test_defect_aware_prompt_args_with_special_characters() {
        let json_str = r#"{
            "task": "Build \"HTTP\" client with <retry> logic",
            "context": "External API & integration",
            "summary_path": "/tmp/path with spaces/summary.yaml"
        }"#;
        let args: DefectAwarePromptArgs = serde_json::from_str(json_str).unwrap();
        assert!(args.task.contains("HTTP"));
        assert!(args.context.contains("&"));
        assert!(args.summary_path.contains(" "));
    }

    #[test]
    fn test_defect_aware_prompt_args_unicode() {
        let json_str = r#"{
            "task": "Build internationalization support",
            "context": "Support for Japanese characters",
            "summary_path": "/tmp/summary.yaml"
        }"#;
        let args: DefectAwarePromptArgs = serde_json::from_str(json_str).unwrap();
        assert_eq!(args.task, "Build internationalization support");
    }

    // === Tool Handler Trait Tests ===

    #[tokio::test]
    async fn test_tool_handler_trait_implementation() {
        // Verify that DefectAwarePromptTool implements ToolHandler
        use pmcp::ToolHandler;

        let tool: Box<dyn ToolHandler + Send + Sync> = Box::new(DefectAwarePromptTool::new());

        // Call handle with invalid args to verify trait works
        let args = json!({});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    // === Debug Trait Tests ===

    #[test]
    fn test_defect_aware_prompt_args_debug() {
        let args = DefectAwarePromptArgs {
            task: "test".to_string(),
            context: "ctx".to_string(),
            summary_path: "/tmp/s.yaml".to_string(),
        };
        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("task"));
        assert!(debug_str.contains("context"));
        assert!(debug_str.contains("summary_path"));
    }

    // === Memory Tests ===

    #[test]
    fn test_defect_aware_prompt_tool_is_zero_sized() {
        // DefectAwarePromptTool is a unit struct, should have zero size
        assert_eq!(std::mem::size_of::<DefectAwarePromptTool>(), 0);
    }

    #[test]
    fn test_defect_aware_prompt_tool_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<DefectAwarePromptTool>();
    }

    #[test]
    fn test_defect_aware_prompt_tool_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<DefectAwarePromptTool>();
    }
}
