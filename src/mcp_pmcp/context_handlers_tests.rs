// Tests for context_handlers: property tests and coverage tests.
// Included from context_handlers.rs -- do NOT add `use` imports or `#!` inner attributes.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    /// A real directory with one analysable source file.
    ///
    /// These tests used to pass `"/nonexistent/path"` and assert `is_ok()`,
    /// which meant they exercised none of the options they are named for — the
    /// tools walked an absent tree, found nothing, and reported success. Now
    /// that missing paths are rejected (GH #639) the fixture has to be real.
    fn fixture_dir() -> tempfile::TempDir {
        let dir = tempfile::TempDir::new().expect("tempdir");
        std::fs::write(
            dir.path().join("sample.rs"),
            "fn sample(a: i32) -> i32 {\n    if a > 2 {\n        a * 3\n    } else {\n        a\n    }\n}\n",
        )
        .expect("write fixture");
        dir
    }
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
        // A path that does not exist must be rejected: reporting zero
        // findings for it is indistinguishable from a clean result, and an
        // MCP client has no exit code to check (GH #639).
        let err = result.expect_err("nonexistent path must be rejected");
        assert!(
            err.to_string().contains("not found"),
            "error should name the missing path, got: {err}"
        );
    }

    #[tokio::test]
    async fn test_context_generate_tool_json_format() {
        let tool = ContextGenerateTool::new();
        let fixture = fixture_dir();
        let fixture_dir_path = fixture.path().display().to_string();
        let args = json!({
            "paths": [fixture_dir_path.as_str()],
            "format": "json"
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_ok());
    }

    /// These two used to assert the STUB: `format: "markdown"` returned the
    /// literal "Context in markdown format (not implemented)" as a string with
    /// isError=false, and the assertion `value["markdown"].is_string()` was
    /// satisfied by exactly that. A format the tool cannot render is now
    /// rejected instead of faked.
    #[tokio::test]
    async fn test_context_generate_tool_markdown_format_is_rejected() {
        let tool = ContextGenerateTool::new();
        let fixture = fixture_dir();
        let fixture_dir_path = fixture.path().display().to_string();
        let args = json!({
            "paths": [fixture_dir_path.as_str()],
            "format": "markdown"
        });
        let err = tool
            .handle(args, test_extra())
            .await
            .expect_err("markdown is not rendered, so it must not report success");
        assert!(err.to_string().contains("Unsupported format"), "{err}");
    }

    #[tokio::test]
    async fn test_context_generate_tool_xml_format_is_rejected() {
        let tool = ContextGenerateTool::new();
        let fixture = fixture_dir();
        let fixture_dir_path = fixture.path().display().to_string();
        let args = json!({
            "paths": [fixture_dir_path.as_str()],
            "format": "xml"
        });
        let err = tool
            .handle(args, test_extra())
            .await
            .expect_err("xml is not rendered, so it must not report success");
        assert!(err.to_string().contains("Unsupported format"), "{err}");
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
        let fixture = fixture_dir();
        let fixture_dir_path = fixture.path().display().to_string();
        let args = json!({
            "paths": [fixture_dir_path.as_str()],
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
        // "Succeed with zeros" is exactly the false pass: an MCP client cannot
        // tell a summary of nothing from a summary of a clean tree, and it has
        // no exit code to check (GH #639).
        let err = result.expect_err("nonexistent path must be rejected");
        assert!(
            err.to_string().contains("not found"),
            "error should name the missing path, got: {err}"
        );
    }

    #[tokio::test]
    async fn test_context_summary_tool_with_level() {
        let tool = ContextSummaryTool::new();
        let fixture = fixture_dir();
        let fixture_dir_path = fixture.path().display().to_string();
        let args = json!({
            "paths": [fixture_dir_path.as_str()],
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

    // === JSON-RPC error CLASS for caller mistakes (round-4 R16) ===

    /// A `level` outside the enum this tool's own schema advertises is a bad
    /// ARGUMENT. It was dispatched first and the refusal wrapped in
    /// `Error::internal`, so `level:"deep"` came back as -32603 — a client
    /// cannot tell a rejected enum from a crashed server, and retries.
    #[tokio::test]
    async fn an_unknown_level_is_invalid_params_not_an_internal_error() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        std::fs::write(dir.path().join("a.rs"), "pub fn a() {}\n").expect("write fixture");

        let tool = ContextSummaryTool::new();
        let err = tool
            .handle(
                json!({"paths": [dir.path().display().to_string()], "level": "deep"}),
                test_extra(),
            )
            .await
            .expect_err("`deep` is not one of brief/normal/detailed");
        assert!(
            matches!(err, Error::Validation(_)),
            "a schema-enum violation is -32602; got {err:?}"
        );
        assert!(
            err.to_string().contains("brief"),
            "the refusal must name the accepted values: {err}"
        );
    }

    /// A mistyped repository path is a bad ARGUMENT, and the answer must not
    /// leak the errno from spawning `git` in a directory that is not there.
    #[tokio::test]
    async fn a_missing_git_path_is_invalid_params_and_leaks_no_errno() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let missing = dir.path().join("no-such-repo");

        let tool = GitStatusTool::new();
        let err = tool
            .handle(
                json!({"path": missing.display().to_string()}),
                test_extra(),
            )
            .await
            .expect_err("a nonexistent repository path must be refused");
        assert!(
            matches!(err, Error::Validation(_)),
            "a mistyped path is -32602; got {err:?}"
        );
        assert!(
            !err.to_string().contains("os error"),
            "the caller gets a path, not a spawn errno: {err}"
        );
    }
}
