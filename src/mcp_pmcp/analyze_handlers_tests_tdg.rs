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
        let tool = TdgTool;
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
        let tool = TdgCompareTool;
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
        let _: AnalyzeTdgTool = TdgTool::new();
        let _: AnalyzeTdgCompareTool = TdgCompareTool::new();
        // R17-1: AnalyzeDagTool / AnalyzeBigOTool / AnalyzeDeepContextTool are
        // now distinct structs, not aliases — they have their own new() ctors.
        let _ = AnalyzeDagTool::new();
        let _ = AnalyzeDeepContextTool::new();
        let _ = AnalyzeBigOTool::new();
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

    // === R17-1 Regression Tests: dispatch correctness ===
    //
    // Each test asserts that the tool's response message contains a marker
    // that is specific to the CORRECT underlying analysis — so that a
    // future mis-wiring (as in R15 #3) would fail fast. Previously,
    // analyze_dag aliased to LintHotspotTool → message mentioned "Lint
    // hotspot"; analyze_big_o → "Coupling analysis"; analyze_deep_context
    // → "Churn analysis". These tests falsify that regression.

    #[tokio::test]
    async fn test_analyze_dag_tool_dispatches_to_dag() {
        let tool = AnalyzeDagTool::new();
        let args = json!({ "paths": [env!("CARGO_MANIFEST_DIR")] });
        let result = tool.handle(args, test_extra()).await;
        // Either succeeds with DAG output, or errors — but must NOT
        // return lint-hotspot / coupling / churn content.
        if let Ok(value) = result {
            let msg = value["message"].as_str().unwrap_or("");
            assert!(
                msg.contains("DAG") || msg.contains("dag"),
                "analyze_dag should produce DAG output, got: {msg}"
            );
            assert!(
                !msg.contains("Lint hotspot"),
                "analyze_dag must NOT dispatch to lint-hotspot (R17-1 regression): {msg}"
            );
            assert!(
                !msg.contains("Coupling"),
                "analyze_dag must NOT dispatch to coupling (R17-1 regression): {msg}"
            );
            assert!(
                !msg.contains("Churn"),
                "analyze_dag must NOT dispatch to churn (R17-1 regression): {msg}"
            );
        }
    }

    #[tokio::test]
    async fn test_analyze_big_o_tool_dispatches_to_big_o() {
        let tool = AnalyzeBigOTool::new();
        let args = json!({ "paths": [env!("CARGO_MANIFEST_DIR")] });
        let result = tool.handle(args, test_extra()).await;
        if let Ok(value) = result {
            let msg = value["message"].as_str().unwrap_or("");
            assert!(
                msg.contains("Big-O") || msg.contains("big_o") || msg.contains("big-o"),
                "analyze_big_o should produce Big-O output, got: {msg}"
            );
            assert!(
                !msg.contains("Coupling"),
                "analyze_big_o must NOT dispatch to coupling (R17-1 regression): {msg}"
            );
            assert!(
                !msg.contains("Lint hotspot"),
                "analyze_big_o must NOT dispatch to lint-hotspot (R17-1 regression): {msg}"
            );
            assert!(
                !msg.contains("Churn"),
                "analyze_big_o must NOT dispatch to churn (R17-1 regression): {msg}"
            );
        }
    }

    #[tokio::test]
    async fn test_analyze_deep_context_tool_dispatches_to_deep_context() {
        let tool = AnalyzeDeepContextTool::new();
        let args = json!({ "paths": [env!("CARGO_MANIFEST_DIR")] });
        let result = tool.handle(args, test_extra()).await;
        if let Ok(value) = result {
            let msg = value["message"].as_str().unwrap_or("");
            assert!(
                msg.contains("Deep context") || msg.contains("deep_context") || msg.contains("deep context"),
                "analyze_deep_context should produce deep-context output, got: {msg}"
            );
            assert!(
                !msg.contains("Churn"),
                "analyze_deep_context must NOT dispatch to churn (R17-1 regression): {msg}"
            );
            assert!(
                !msg.contains("Coupling"),
                "analyze_deep_context must NOT dispatch to coupling (R17-1 regression): {msg}"
            );
            assert!(
                !msg.contains("Lint hotspot"),
                "analyze_deep_context must NOT dispatch to lint-hotspot (R17-1 regression): {msg}"
            );
        }
    }

    #[test]
    fn test_r17_1_dag_tool_is_distinct_from_lint_hotspot() {
        // Static guarantee: the three renamed tools are NOT the same type as
        // the tools they were previously aliased to.
        assert_ne!(
            std::any::TypeId::of::<AnalyzeDagTool>(),
            std::any::TypeId::of::<LintHotspotTool>(),
            "AnalyzeDagTool must not be aliased to LintHotspotTool (R17-1)"
        );
        assert_ne!(
            std::any::TypeId::of::<AnalyzeBigOTool>(),
            std::any::TypeId::of::<CouplingTool>(),
            "AnalyzeBigOTool must not be aliased to CouplingTool (R17-1)"
        );
        assert_ne!(
            std::any::TypeId::of::<AnalyzeDeepContextTool>(),
            std::any::TypeId::of::<ChurnTool>(),
            "AnalyzeDeepContextTool must not be aliased to ChurnTool (R17-1)"
        );
    }
}
