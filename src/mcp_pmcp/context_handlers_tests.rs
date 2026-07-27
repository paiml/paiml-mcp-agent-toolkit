// Tests for context_handlers: property tests and coverage tests.
// Included from context_handlers.rs -- do NOT add `use` imports or `#!` inner attributes.

#[cfg_attr(coverage_nightly, coverage(off))]
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
        let tool = GitCloneTool;
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
        let tool = GitStatusTool;
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
        // CARGO_MANIFEST_DIR is the git repo root
        let git_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

        // Skip if not a git repo
        if !git_root.join(".git").exists() {
            eprintln!("Skipping: .git not found at {:?}", git_root);
            return;
        }

        let args = json!({
            "path": git_root.to_str().unwrap()
        });
        let result = tool.handle(args, test_extra()).await;
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
        let tool = ContextGenerateTool;
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
        let tool = ContextAnalyzeTool;
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
        let tool = ContextSummaryTool;
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
