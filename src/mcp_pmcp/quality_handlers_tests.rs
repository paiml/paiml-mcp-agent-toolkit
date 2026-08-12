// Tests for quality gate handlers
// This file is included by quality_handlers.rs via include!()
// NO `use` imports or `#!` inner attributes allowed

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
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

    // === QualityGateTool Tests ===

    #[test]
    fn test_quality_gate_tool_new() {
        let tool = QualityGateTool::new();
        // Verify creation succeeds
        let _ = tool;
    }

    #[test]
    fn test_quality_gate_tool_default() {
        let tool = QualityGateTool;
        let _ = tool;
    }

    #[tokio::test]
    async fn test_quality_gate_tool_invalid_args() {
        let tool = QualityGateTool::new();
        let args = json!({"invalid": "args"});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid arguments"));
    }

    #[tokio::test]
    async fn test_quality_gate_tool_missing_paths() {
        let tool = QualityGateTool::new();
        let args = json!({});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_quality_gate_tool_empty_paths() {
        let tool = QualityGateTool::new();
        let args = json!({"paths": []});
        let result = tool.handle(args, test_extra()).await;
        // Empty paths should return error from tool_functions
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_quality_gate_tool_with_strict() {
        let tool = QualityGateTool::new();
        let fixture = fixture_dir();
        let fixture_dir_path = fixture.path().display().to_string();
        let args = json!({
            "paths": [fixture_dir_path.as_str()],
            "strict": true
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_quality_gate_tool_with_file() {
        let tool = QualityGateTool::new();
        let args = json!({
            "paths": ["."],
            "file": "/nonexistent/specific/file.rs"
        });
        let result = tool.handle(args, test_extra()).await;
        // Should fail because file doesn't exist
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_quality_gate_tool_strict_false() {
        let tool = QualityGateTool::new();
        let fixture = fixture_dir();
        let fixture_dir_path = fixture.path().display().to_string();
        let args = json!({
            "paths": [fixture_dir_path.as_str()],
            "strict": false
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_ok());
    }

    // === QualityGateSummaryTool Tests ===

    #[test]
    fn test_quality_gate_summary_tool_new() {
        let tool = QualityGateSummaryTool::new();
        let _ = tool;
    }

    #[test]
    fn test_quality_gate_summary_tool_default() {
        let tool = QualityGateSummaryTool;
        let _ = tool;
    }

    #[tokio::test]
    async fn test_quality_gate_summary_tool_invalid_args() {
        let tool = QualityGateSummaryTool::new();
        let args = json!({"invalid": "args"});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid arguments"));
    }

    #[tokio::test]
    async fn test_quality_gate_summary_tool_missing_paths() {
        let tool = QualityGateSummaryTool::new();
        let args = json!({});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_quality_gate_summary_tool_empty_paths() {
        let tool = QualityGateSummaryTool::new();
        let args = json!({"paths": []});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_quality_gate_summary_tool_json_format() {
        let tool = QualityGateSummaryTool::new();
        let fixture = fixture_dir();
        let fixture_dir_path = fixture.path().display().to_string();
        let args = json!({
            "paths": [fixture_dir_path.as_str()],
            "format": "json"
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_quality_gate_summary_tool_markdown_format() {
        let tool = QualityGateSummaryTool::new();
        let fixture = fixture_dir();
        let fixture_dir_path = fixture.path().display().to_string();
        let args = json!({
            "paths": [fixture_dir_path.as_str()],
            "format": "markdown"
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.get("markdown").is_some());
    }

    #[tokio::test]
    async fn test_quality_gate_summary_tool_unsupported_format() {
        let tool = QualityGateSummaryTool::new();
        let fixture = fixture_dir();
        let fixture_dir_path = fixture.path().display().to_string();
        let args = json!({
            "paths": [fixture_dir_path.as_str()],
            "format": "xml"
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unsupported format"));
    }

    #[tokio::test]
    async fn test_quality_gate_summary_tool_default_format() {
        let tool = QualityGateSummaryTool::new();
        let fixture = fixture_dir();
        let fixture_dir_path = fixture.path().display().to_string();
        let args = json!({
            "paths": [fixture_dir_path.as_str()]
        });
        let result = tool.handle(args, test_extra()).await;
        // Default format (json) should work
        assert!(result.is_ok());
    }

    // === QualityGateBaselineTool Tests ===

    #[test]
    fn test_quality_gate_baseline_tool_new() {
        let tool = QualityGateBaselineTool::new();
        let _ = tool;
    }

    #[test]
    fn test_quality_gate_baseline_tool_default() {
        let tool = QualityGateBaselineTool;
        let _ = tool;
    }

    #[tokio::test]
    async fn test_quality_gate_baseline_tool_invalid_args() {
        let tool = QualityGateBaselineTool::new();
        let args = json!({"invalid": "args"});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid arguments"));
    }

    #[tokio::test]
    async fn test_quality_gate_baseline_tool_missing_paths() {
        let tool = QualityGateBaselineTool::new();
        let args = json!({});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_quality_gate_baseline_tool_empty_paths() {
        let tool = QualityGateBaselineTool::new();
        let args = json!({"paths": []});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_quality_gate_baseline_tool_with_output() {
        let tool = QualityGateBaselineTool::new();
        let fixture = fixture_dir();
        let fixture_dir_path = fixture.path().display().to_string();
        let args = json!({
            "paths": [fixture_dir_path.as_str()],
            "output": "/tmp/baseline.json"
        });
        let result = tool.handle(args, test_extra()).await;
        // Should succeed and create baseline
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_quality_gate_baseline_tool_without_output() {
        let tool = QualityGateBaselineTool::new();
        let fixture = fixture_dir();
        let fixture_dir_path = fixture.path().display().to_string();
        let args = json!({
            "paths": [fixture_dir_path.as_str()]
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_ok());
    }

    // === QualityGateCompareTool Tests ===

    #[test]
    fn test_quality_gate_compare_tool_new() {
        let tool = QualityGateCompareTool::new();
        let _ = tool;
    }

    #[test]
    fn test_quality_gate_compare_tool_default() {
        let tool = QualityGateCompareTool;
        let _ = tool;
    }

    #[tokio::test]
    async fn test_quality_gate_compare_tool_invalid_args() {
        let tool = QualityGateCompareTool::new();
        let args = json!({"invalid": "args"});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid arguments"));
    }

    #[tokio::test]
    async fn test_quality_gate_compare_tool_missing_baseline() {
        let tool = QualityGateCompareTool::new();
        let args = json!({"paths": ["."]});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_quality_gate_compare_tool_missing_paths() {
        let tool = QualityGateCompareTool::new();
        let args = json!({"baseline": "/tmp/baseline.json"});
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_quality_gate_compare_tool_empty_paths() {
        let tool = QualityGateCompareTool::new();
        let args = json!({
            "baseline": "/tmp/baseline.json",
            "paths": []
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_quality_gate_compare_tool_nonexistent_baseline() {
        let tool = QualityGateCompareTool::new();
        let args = json!({
            "baseline": "/nonexistent/baseline.json",
            "paths": ["."]
        });
        let result = tool.handle(args, test_extra()).await;
        // Should fail because baseline doesn't exist
        assert!(result.is_err());
    }

    // === QualityGateArgs Deserialization Tests ===

    #[test]
    fn test_quality_gate_args_deserialize_minimal() {
        let json = json!({"paths": ["/path/to/project"]});
        let args: QualityGateArgs = serde_json::from_value(json).unwrap();
        assert_eq!(args.paths, vec!["/path/to/project"]);
        assert!(!args.strict);
        assert!(args.file.is_none());
    }

    #[test]
    fn test_quality_gate_args_deserialize_full() {
        let json = json!({
            "paths": ["/path1", "/path2"],
            "strict": true,
            "file": "/specific/file.rs"
        });
        let args: QualityGateArgs = serde_json::from_value(json).unwrap();
        assert_eq!(args.paths, vec!["/path1", "/path2"]);
        assert!(args.strict);
        assert_eq!(args.file, Some("/specific/file.rs".to_string()));
    }

    // === QualityGateSummaryArgs Deserialization Tests ===

    #[test]
    fn test_quality_gate_summary_args_deserialize_minimal() {
        let json = json!({"paths": ["/path"]});
        let args: QualityGateSummaryArgs = serde_json::from_value(json).unwrap();
        assert_eq!(args.paths, vec!["/path"]);
        assert!(args.format.is_none());
    }

    #[test]
    fn test_quality_gate_summary_args_deserialize_full() {
        let json = json!({
            "paths": ["/path1", "/path2"],
            "format": "markdown"
        });
        let args: QualityGateSummaryArgs = serde_json::from_value(json).unwrap();
        assert_eq!(args.paths, vec!["/path1", "/path2"]);
        assert_eq!(args.format, Some("markdown".to_string()));
    }

    // === QualityGateBaselineArgs Deserialization Tests ===

    #[test]
    fn test_quality_gate_baseline_args_deserialize_minimal() {
        let json = json!({"paths": ["/path"]});
        let args: QualityGateBaselineArgs = serde_json::from_value(json).unwrap();
        assert_eq!(args.paths, vec!["/path"]);
        assert!(args.output.is_none());
    }

    #[test]
    fn test_quality_gate_baseline_args_deserialize_full() {
        let json = json!({
            "paths": ["/path"],
            "output": "/output/baseline.json"
        });
        let args: QualityGateBaselineArgs = serde_json::from_value(json).unwrap();
        assert_eq!(args.paths, vec!["/path"]);
        assert_eq!(args.output, Some("/output/baseline.json".to_string()));
    }

    // === QualityGateCompareArgs Deserialization Tests ===

    #[test]
    fn test_quality_gate_compare_args_deserialize() {
        let json = json!({
            "baseline": "/path/to/baseline.json",
            "paths": ["/path1", "/path2"]
        });
        let args: QualityGateCompareArgs = serde_json::from_value(json).unwrap();
        assert_eq!(args.baseline, "/path/to/baseline.json");
        assert_eq!(args.paths, vec!["/path1", "/path2"]);
    }

    #[test]
    fn test_quality_gate_compare_args_deserialize_missing_baseline() {
        let json = json!({"paths": ["/path"]});
        let result: std::result::Result<QualityGateCompareArgs, serde_json::Error> =
            serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_quality_gate_compare_args_deserialize_missing_paths() {
        let json = json!({"baseline": "/baseline.json"});
        let result: std::result::Result<QualityGateCompareArgs, serde_json::Error> =
            serde_json::from_value(json);
        assert!(result.is_err());
    }

    // === JSON-RPC error CLASS for caller mistakes (round-4 R16) ===
    //
    // `-32603 Internal error` says the SERVER faulted and the call is worth
    // retrying. Every case below is the caller's input, and the same server
    // already answers this class with `-32602 Invalid params` elsewhere
    // (`generate_context` with a bad `format`, any nonexistent path). The tests
    // assert the discriminant rather than the wire code because that is what
    // the transport maps: `Error::Validation` ⇒ -32602, `Error::Internal` ⇒
    // -32603.

    /// `{"paths": []}` is a schema violation, not a server fault.
    ///
    /// It reached `check_quality_gates`, which bailed "At least one path must
    /// be provided", and the handler's blanket `Error::internal` reported that
    /// as -32603. The guard now lives in `resolve_existing_paths`, so every
    /// `paths`-shaped tool answers the same way.
    #[tokio::test]
    async fn empty_paths_is_invalid_params_not_an_internal_error() {
        let tool = QualityGateTool::new();
        let err = tool
            .handle(json!({"paths": []}), test_extra())
            .await
            .expect_err("an empty paths list must be refused");
        assert!(
            matches!(err, Error::Validation(_)),
            "empty `paths` is the caller's mistake, so -32602; got {err:?}"
        );
    }

    /// A single unparseable FILE in `paths` is refused the same way the `file`
    /// argument already refuses it.
    ///
    /// `check_quality_gates` calls `analyze_file` directly for a file path and
    /// propagates its "did not parse" bail, which the handler wrapped as
    /// -32603 — one tool reporting one refusal under two codes depending on
    /// which of its two arguments carried the path.
    #[tokio::test]
    async fn an_unparseable_file_in_paths_is_invalid_params_not_an_internal_error() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let bad = dir.path().join("bad.rs");
        std::fs::write(&bad, "fn main( { let x = ;;;\n").expect("write fixture");

        let tool = QualityGateTool::new();
        let err = tool
            .handle(json!({"paths": [bad.display().to_string()]}), test_extra())
            .await
            .expect_err("an unparseable file must be refused");
        assert!(
            matches!(err, Error::Validation(_)),
            "a file that does not parse is bad input, so -32602; got {err:?}"
        );

        // And the two arguments must agree, which is the point of the fix.
        let via_file = tool
            .handle(
                json!({"paths": [dir.path().display().to_string()], "file": bad.display().to_string()}),
                test_extra(),
            )
            .await
            .expect_err("the `file` argument already refused this");
        assert!(matches!(via_file, Error::Validation(_)), "{via_file:?}");
    }
}
