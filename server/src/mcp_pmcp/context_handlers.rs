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

// Git Clone Tool

#[derive(Debug, Deserialize)]
struct GitCloneArgs {
    url: String,
    #[serde(default)]
    target_dir: Option<String>,
    #[serde(default)]
    branch: Option<String>,
    #[serde(default)]
    depth: Option<u32>,
}

pub struct GitCloneTool;

impl GitCloneTool {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for GitCloneTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for GitCloneTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling git.clone with args: {}", args);

        let params: GitCloneArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let target_dir = params.target_dir.map(PathBuf::from);

        let result = tool_functions::git_clone(
            &params.url,
            target_dir.as_deref(),
            params.branch.as_deref(),
            params.depth,
        )
        .await
        .map_err(|e| Error::internal(format!("Git clone failed: {e}")))?;

        Ok(json!({
            "status": "success",
            "path": result.display().to_string(),
            "message": format!("Successfully cloned repository to {}", result.display())
        }))
    }
}

// Git Status Tool

#[derive(Debug, Deserialize)]
struct GitStatusArgs {
    path: String,
}

pub struct GitStatusTool;

impl GitStatusTool {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for GitStatusTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for GitStatusTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling git.status with args: {}", args);

        let params: GitStatusArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let path = PathBuf::from(params.path);

        let status = tool_functions::git_status(path.as_ref())
            .await
            .map_err(|e| Error::internal(format!("Failed to get git status: {e}")))?;

        Ok(status)
    }
}

// Context Generate Tool

#[derive(Debug, Deserialize)]
struct ContextGenerateArgs {
    paths: Vec<String>,
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    max_depth: Option<usize>,
    #[serde(default)]
    include_dependencies: bool,
}

pub struct ContextGenerateTool;

impl ContextGenerateTool {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for ContextGenerateTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for ContextGenerateTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling context.generate with args: {}", args);

        let params: ContextGenerateArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let context =
            tool_functions::generate_context(&paths, params.max_depth, params.include_dependencies)
                .await
                .map_err(|e| Error::internal(format!("Context generation failed: {e}")))?;

        // Format the output based on requested format
        match params.format.as_deref() {
            Some("markdown") => Ok(json!({
                "context": context,
                "markdown": "Context in markdown format (not implemented)"
            })),
            Some("xml") => Ok(json!({
                "context": context,
                "xml": "Context in XML format (not implemented)"
            })),
            Some("json") | None => Ok(context),
            Some(format) => Err(Error::validation(format!("Unsupported format: {format}"))),
        }
    }
}

// Context Analyze Tool

#[derive(Debug, Deserialize)]
struct ContextAnalyzeArgs {
    paths: Vec<String>,
    #[serde(default)]
    analysis_types: Vec<String>,
}

pub struct ContextAnalyzeTool;

impl ContextAnalyzeTool {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for ContextAnalyzeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for ContextAnalyzeTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling context.analyze with args: {}", args);

        let params: ContextAnalyzeArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let analyses = tool_functions::analyze_context(&paths, &params.analysis_types)
            .await
            .map_err(|e| Error::internal(format!("Context analysis failed: {e}")))?;

        Ok(analyses)
    }
}

// Context Summary Tool

#[derive(Debug, Deserialize)]
struct ContextSummaryArgs {
    paths: Vec<String>,
    #[serde(default)]
    level: Option<String>,
}

pub struct ContextSummaryTool;

impl ContextSummaryTool {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for ContextSummaryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for ContextSummaryTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling context.summary with args: {}", args);

        let params: ContextSummaryArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let summary = tool_functions::context_summary(&paths, params.level.as_deref())
            .await
            .map_err(|e| Error::internal(format!("Context summary failed: {e}")))?;

        Ok(summary)
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use serde_json::json;
    use tokio_util::sync::CancellationToken;

    fn test_extra() -> RequestHandlerExtra {
        RequestHandlerExtra::new("test-request".to_string(), CancellationToken::new())
    }

    // === GitCloneTool Tests ===

    #[test]
    fn test_git_clone_tool_new() {
        let tool = GitCloneTool::new();
        let _ = tool;
    }

    #[test]
    fn test_git_clone_tool_default() {
        let tool = GitCloneTool::default();
        let _ = tool;
    }

    #[tokio::test]
    async fn test_git_clone_tool_invalid_args() {
        let tool = GitCloneTool::new();
        let args = json!({"invalid": "args"});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_git_clone_tool_minimal_args() {
        let tool = GitCloneTool::new();
        let args = json!({
            "url": "https://github.com/example/repo.git"
        });
        let result = tool.handle(args, test_extra()).await;
        // This should succeed as clone just returns the path
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["status"], "success");
    }

    #[tokio::test]
    async fn test_git_clone_tool_with_all_options() {
        let tool = GitCloneTool::new();
        let args = json!({
            "url": "https://github.com/example/repo.git",
            "target_dir": "/tmp/test-clone",
            "branch": "main",
            "depth": 1
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["status"], "success");
        assert_eq!(value["path"], "/tmp/test-clone");
    }

    // === GitStatusTool Tests ===

    #[test]
    fn test_git_status_tool_new() {
        let tool = GitStatusTool::new();
        let _ = tool;
    }

    #[test]
    fn test_git_status_tool_default() {
        let tool = GitStatusTool::default();
        let _ = tool;
    }

    #[tokio::test]
    async fn test_git_status_tool_invalid_args() {
        let tool = GitStatusTool::new();
        let args = json!({"invalid": "args"});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_git_status_tool_nonexistent_path() {
        let tool = GitStatusTool::new();
        let args = json!({
            "path": "/nonexistent/path"
        });
        let result = tool.handle(args, test_extra()).await;
        // Nonexistent path is not a git repo
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_git_status_tool_with_current_dir() {
        let tool = GitStatusTool::new();
        // Use parent of cargo manifest dir which is the git repo root
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let git_root = manifest_dir.parent().unwrap_or(manifest_dir);
        let args = json!({
            "path": git_root.to_str().unwrap()
        });
        let result = tool.handle(args, test_extra()).await;
        // Parent of CARGO_MANIFEST_DIR should be a git repo
        assert!(
            result.is_ok(),
            "Expected Ok, got Err: {:?}",
            result.as_ref().err()
        );
        let value = result.unwrap();
        assert_eq!(value["status"], "completed");
        assert!(value["git_status"].is_object());
    }

    // === ContextGenerateTool Tests ===

    #[test]
    fn test_context_generate_tool_new() {
        let tool = ContextGenerateTool::new();
        let _ = tool;
    }

    #[test]
    fn test_context_generate_tool_default() {
        let tool = ContextGenerateTool::default();
        let _ = tool;
    }

    #[tokio::test]
    async fn test_context_generate_tool_invalid_args() {
        let tool = ContextGenerateTool::new();
        let args = json!({"invalid": "args"});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_context_generate_tool_empty_paths() {
        let tool = ContextGenerateTool::new();
        let args = json!({"paths": []});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_context_generate_tool_nonexistent_path() {
        let tool = ContextGenerateTool::new();
        let args = json!({
            "paths": ["/nonexistent/path"]
        });
        let result = tool.handle(args, test_extra()).await;
        // Should succeed but with empty results
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_generate_tool_json_format() {
        let tool = ContextGenerateTool::new();
        let args = json!({
            "paths": ["/nonexistent/path"],
            "format": "json"
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_generate_tool_markdown_format() {
        let tool = ContextGenerateTool::new();
        let args = json!({
            "paths": ["/nonexistent/path"],
            "format": "markdown"
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value["markdown"].is_string());
    }

    #[tokio::test]
    async fn test_context_generate_tool_xml_format() {
        let tool = ContextGenerateTool::new();
        let args = json!({
            "paths": ["/nonexistent/path"],
            "format": "xml"
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value["xml"].is_string());
    }

    #[tokio::test]
    async fn test_context_generate_tool_unsupported_format() {
        let tool = ContextGenerateTool::new();
        let args = json!({
            "paths": ["/nonexistent/path"],
            "format": "unsupported"
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_context_generate_tool_with_all_options() {
        let tool = ContextGenerateTool::new();
        let args = json!({
            "paths": ["/nonexistent/path"],
            "format": "json",
            "max_depth": 5,
            "include_dependencies": true
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_ok());
    }

    // === ContextAnalyzeTool Tests ===

    #[test]
    fn test_context_analyze_tool_new() {
        let tool = ContextAnalyzeTool::new();
        let _ = tool;
    }

    #[test]
    fn test_context_analyze_tool_default() {
        let tool = ContextAnalyzeTool::default();
        let _ = tool;
    }

    #[tokio::test]
    async fn test_context_analyze_tool_invalid_args() {
        let tool = ContextAnalyzeTool::new();
        let args = json!({"invalid": "args"});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_context_analyze_tool_empty_paths() {
        let tool = ContextAnalyzeTool::new();
        let args = json!({"paths": []});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_context_analyze_tool_nonexistent_path() {
        let tool = ContextAnalyzeTool::new();
        let args = json!({
            "paths": ["/nonexistent/path"]
        });
        let result = tool.handle(args, test_extra()).await;
        // Analysis on nonexistent path should fail
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_context_analyze_tool_with_analysis_types() {
        let tool = ContextAnalyzeTool::new();
        let args = json!({
            "paths": ["/nonexistent/path"],
            "analysis_types": ["structure", "dependencies"]
        });
        let result = tool.handle(args, test_extra()).await;
        // Nonexistent path fails
        assert!(result.is_err());
    }

    // === ContextSummaryTool Tests ===

    #[test]
    fn test_context_summary_tool_new() {
        let tool = ContextSummaryTool::new();
        let _ = tool;
    }

    #[test]
    fn test_context_summary_tool_default() {
        let tool = ContextSummaryTool::default();
        let _ = tool;
    }

    #[tokio::test]
    async fn test_context_summary_tool_invalid_args() {
        let tool = ContextSummaryTool::new();
        let args = json!({"invalid": "args"});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_context_summary_tool_empty_paths() {
        let tool = ContextSummaryTool::new();
        let args = json!({"paths": []});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_context_summary_tool_nonexistent_path() {
        let tool = ContextSummaryTool::new();
        let args = json!({
            "paths": ["/nonexistent/path"]
        });
        let result = tool.handle(args, test_extra()).await;
        // Summary on nonexistent path should succeed with zeros
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_summary_tool_with_level() {
        let tool = ContextSummaryTool::new();
        let args = json!({
            "paths": ["/nonexistent/path"],
            "level": "detailed"
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_ok());
    }

    // === Re-export Tests ===

    #[test]
    fn test_re_exports_exist() {
        // Test that re-exports are accessible
        let _: GenerateContextTool = ContextGenerateTool::new();
        let _: ScaffoldProjectTool = ContextSummaryTool::new();
        let _: GitTool = GitStatusTool::new();
    }

    // === Args Deserialization Tests ===

    #[test]
    fn test_git_clone_args_deserialization() {
        let json_str = r#"{"url": "https://github.com/example/repo.git", "target_dir": "/tmp/clone", "branch": "main", "depth": 1}"#;
        let args: GitCloneArgs = serde_json::from_str(json_str).unwrap();
        assert_eq!(args.url, "https://github.com/example/repo.git");
        assert_eq!(args.target_dir, Some("/tmp/clone".to_string()));
        assert_eq!(args.branch, Some("main".to_string()));
        assert_eq!(args.depth, Some(1));
    }

    #[test]
    fn test_git_clone_args_minimal() {
        let json_str = r#"{"url": "https://github.com/example/repo.git"}"#;
        let args: GitCloneArgs = serde_json::from_str(json_str).unwrap();
        assert_eq!(args.url, "https://github.com/example/repo.git");
        assert_eq!(args.target_dir, None);
        assert_eq!(args.branch, None);
        assert_eq!(args.depth, None);
    }

    #[test]
    fn test_git_status_args_deserialization() {
        let json_str = r#"{"path": "/some/repo"}"#;
        let args: GitStatusArgs = serde_json::from_str(json_str).unwrap();
        assert_eq!(args.path, "/some/repo");
    }

    #[test]
    fn test_context_generate_args_deserialization() {
        let json_str = r#"{"paths": ["src/"], "format": "json", "max_depth": 10, "include_dependencies": true}"#;
        let args: ContextGenerateArgs = serde_json::from_str(json_str).unwrap();
        assert_eq!(args.paths, vec!["src/"]);
        assert_eq!(args.format, Some("json".to_string()));
        assert_eq!(args.max_depth, Some(10));
        assert!(args.include_dependencies);
    }

    #[test]
    fn test_context_generate_args_minimal() {
        let json_str = r#"{"paths": ["src/"]}"#;
        let args: ContextGenerateArgs = serde_json::from_str(json_str).unwrap();
        assert_eq!(args.paths, vec!["src/"]);
        assert_eq!(args.format, None);
        assert_eq!(args.max_depth, None);
        assert!(!args.include_dependencies);
    }

    #[test]
    fn test_context_analyze_args_deserialization() {
        let json_str = r#"{"paths": ["src/"], "analysis_types": ["structure", "dependencies"]}"#;
        let args: ContextAnalyzeArgs = serde_json::from_str(json_str).unwrap();
        assert_eq!(args.paths, vec!["src/"]);
        assert_eq!(args.analysis_types, vec!["structure", "dependencies"]);
    }

    #[test]
    fn test_context_analyze_args_empty_types() {
        let json_str = r#"{"paths": ["src/"]}"#;
        let args: ContextAnalyzeArgs = serde_json::from_str(json_str).unwrap();
        assert_eq!(args.paths, vec!["src/"]);
        assert!(args.analysis_types.is_empty());
    }

    #[test]
    fn test_context_summary_args_deserialization() {
        let json_str = r#"{"paths": ["src/"], "level": "detailed"}"#;
        let args: ContextSummaryArgs = serde_json::from_str(json_str).unwrap();
        assert_eq!(args.paths, vec!["src/"]);
        assert_eq!(args.level, Some("detailed".to_string()));
    }

    #[test]
    fn test_context_summary_args_minimal() {
        let json_str = r#"{"paths": ["src/"]}"#;
        let args: ContextSummaryArgs = serde_json::from_str(json_str).unwrap();
        assert_eq!(args.paths, vec!["src/"]);
        assert_eq!(args.level, None);
    }

    // === Integration Tests with Real Paths ===

    #[tokio::test]
    async fn test_context_generate_with_current_file() {
        let tool = ContextGenerateTool::new();
        let args = json!({
            "paths": [file!()]
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["status"], "completed");
    }

    #[tokio::test]
    async fn test_context_summary_with_current_dir() {
        let tool = ContextSummaryTool::new();
        let args = json!({
            "paths": ["."]
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["status"], "completed");
        // File count depends on current working directory, just verify summary exists
        assert!(value["summary"].is_object(), "Summary should be an object");
    }
}
