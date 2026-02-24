#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tdg_coverage_tests {
    use super::*;
    use serde_json::json;
    use tokio_util::sync::CancellationToken;

    fn test_extra() -> RequestHandlerExtra {
        RequestHandlerExtra::new("test-request".to_string(), CancellationToken::new())
    }

    // === TdgTool Tests ===

    #[test]
    fn test_tdg_tool_new() {
        let tool = TdgTool::new();
        let _ = tool;
    }

    #[test]
    fn test_tdg_tool_default() {
        let tool = TdgTool::default();
        let _ = tool;
    }

    #[tokio::test]
    async fn test_tdg_tool_invalid_args() {
        let tool = TdgTool::new();
        let args = json!({"invalid": "args"});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tdg_tool_empty_paths() {
        let tool = TdgTool::new();
        let args = json!({"paths": []});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tdg_tool_with_all_options() {
        let tool = TdgTool::new();
        let args = json!({
            "paths": ["/nonexistent/path"],
            "threshold": 75.0,
            "top_files": 10,
            "include_components": true,
            "with_git_context": true
        });
        let result = tool.handle(args, test_extra()).await;
        // Nonexistent path returns error
        assert!(result.is_err());
    }

    // === TdgCompareTool Tests ===

    #[test]
    fn test_tdg_compare_tool_new() {
        let tool = TdgCompareTool::new();
        let _ = tool;
    }

    #[test]
    fn test_tdg_compare_tool_default() {
        let tool = TdgCompareTool::default();
        let _ = tool;
    }

    #[tokio::test]
    async fn test_tdg_compare_tool_invalid_args() {
        let tool = TdgCompareTool::new();
        let args = json!({"invalid": "args"});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tdg_compare_tool_missing_path2() {
        let tool = TdgCompareTool::new();
        let args = json!({"path1": "/some/path"});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tdg_compare_tool_with_git_context() {
        let tool = TdgCompareTool::new();
        let args = json!({
            "path1": "/nonexistent/path1",
            "path2": "/nonexistent/path2",
            "with_git_context": true
        });
        let result = tool.handle(args, test_extra()).await;
        // Nonexistent paths return error
        assert!(result.is_err());
    }

    // === Re-export Tests ===

    #[test]
    fn test_re_exports_exist() {
        // Test that re-exports are accessible
        let _: AnalyzeComplexityTool = ComplexityTool::new();
        let _: AnalyzeSatdTool = SatdTool::new();
        let _: AnalyzeDeadCodeTool = DeadCodeTool::new();
        let _: AnalyzeDagTool = LintHotspotTool::new();
        let _: AnalyzeDeepContextTool = ChurnTool::new();
        let _: AnalyzeBigOTool = CouplingTool::new();
        let _: AnalyzeTdgTool = TdgTool::new();
        let _: AnalyzeTdgCompareTool = TdgCompareTool::new();
    }

    // === Args Deserialization Tests ===

    #[test]
    fn test_complexity_args_deserialization() {
        let json_str = r#"{"paths": ["src/"], "top_files": 5, "threshold": 20}"#;
        let args: ComplexityArgs = serde_json::from_str(json_str).unwrap();
        assert_eq!(args.paths, vec!["src/"]);
        assert_eq!(args.top_files, Some(5));
        assert_eq!(args.threshold, Some(20));
    }

    #[test]
    fn test_complexity_args_minimal() {
        let json_str = r#"{"paths": ["src/"]}"#;
        let args: ComplexityArgs = serde_json::from_str(json_str).unwrap();
        assert_eq!(args.paths, vec!["src/"]);
        assert_eq!(args.top_files, None);
        assert_eq!(args.threshold, None);
    }

    #[test]
    fn test_satd_args_deserialization() {
        let json_str = r#"{"paths": ["src/"], "include_resolved": true}"#;
        let args: SatdArgs = serde_json::from_str(json_str).unwrap();
        assert_eq!(args.paths, vec!["src/"]);
        assert!(args.include_resolved);
    }

    #[test]
    fn test_satd_args_default_include_resolved() {
        let json_str = r#"{"paths": ["src/"]}"#;
        let args: SatdArgs = serde_json::from_str(json_str).unwrap();
        assert!(!args.include_resolved);
    }

    #[test]
    fn test_dead_code_args_deserialization() {
        let json_str = r#"{"paths": ["src/"], "include_tests": true}"#;
        let args: DeadCodeArgs = serde_json::from_str(json_str).unwrap();
        assert_eq!(args.paths, vec!["src/"]);
        assert!(args.include_tests);
    }

    #[test]
    fn test_lint_hotspot_args_deserialization() {
        let json_str = r#"{"paths": ["src/"], "top_files": 10}"#;
        let args: LintHotspotArgs = serde_json::from_str(json_str).unwrap();
        assert_eq!(args.paths, vec!["src/"]);
        assert_eq!(args.top_files, Some(10));
    }

    #[test]
    fn test_churn_args_deserialization() {
        let json_str = r#"{"paths": ["src/"], "days": 30, "top_files": 10}"#;
        let args: ChurnArgs = serde_json::from_str(json_str).unwrap();
        assert_eq!(args.paths, vec!["src/"]);
        assert_eq!(args.days, Some(30));
        assert_eq!(args.top_files, Some(10));
    }

    #[test]
    fn test_coupling_args_deserialization() {
        let json_str = r#"{"paths": ["src/"], "threshold": 0.75}"#;
        let args: CouplingArgs = serde_json::from_str(json_str).unwrap();
        assert_eq!(args.paths, vec!["src/"]);
        assert_eq!(args.threshold, Some(0.75));
    }

    #[test]
    fn test_tdg_args_deserialization() {
        let json_str = r#"{"paths": ["src/"], "threshold": 75.0, "top_files": 5, "include_components": true, "with_git_context": true}"#;
        let args: TdgArgs = serde_json::from_str(json_str).unwrap();
        assert_eq!(args.paths, vec!["src/"]);
        assert_eq!(args.threshold, Some(75.0));
        assert_eq!(args.top_files, Some(5));
        assert_eq!(args.include_components, Some(true));
        assert_eq!(args.with_git_context, Some(true));
    }

    #[test]
    fn test_tdg_compare_args_deserialization() {
        let json_str = r#"{"path1": "old/", "path2": "new/", "with_git_context": true}"#;
        let args: TdgCompareArgs = serde_json::from_str(json_str).unwrap();
        assert_eq!(args.path1, "old/");
        assert_eq!(args.path2, "new/");
        assert_eq!(args.with_git_context, Some(true));
    }

    // === Integration Test with Real Paths ===

    #[tokio::test]
    async fn test_complexity_tool_with_current_file() {
        let tool = ComplexityTool::new();
        let args = json!({
            "paths": [file!()],
            "threshold": 100
        });
        let result = tool.handle(args, test_extra()).await;
        // Should succeed with a real file
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["status"], "completed");
    }

    #[tokio::test]
    async fn test_satd_tool_with_current_file() {
        let tool = SatdTool::new();
        let args = json!({
            "paths": [file!()]
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["status"], "completed");
    }

    #[tokio::test]
    async fn test_dead_code_tool_with_current_file() {
        let tool = DeadCodeTool::new();
        let args = json!({
            "paths": [file!()]
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["status"], "completed");
    }
}
