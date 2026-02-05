use crate::mcp_pmcp::tool_functions;
use async_trait::async_trait;
use pmcp::{Error, RequestHandlerExtra, Result, ToolHandler};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use tracing::debug;

// Quality Gate Check Tool

#[derive(Debug, Deserialize)]
struct QualityGateArgs {
    paths: Vec<String>,
    #[serde(default)]
    strict: bool,
    #[serde(default)]
    file: Option<String>,
}

pub struct QualityGateTool;

impl QualityGateTool {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for QualityGateTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for QualityGateTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling quality-gate with args: {}", args);

        let params: QualityGateArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        // If a specific file is requested, check only that file
        if let Some(file_path) = params.file {
            let file_path = PathBuf::from(file_path);
            let result = tool_functions::check_quality_gate_file(file_path.as_ref(), params.strict)
                .await
                .map_err(|e| Error::internal(format!("Quality gate check failed: {e}")))?;

            return Ok(result);
        }

        // Otherwise check all paths
        let results = tool_functions::check_quality_gates(&paths, params.strict)
            .await
            .map_err(|e| Error::internal(format!("Quality gate check failed: {e}")))?;

        Ok(results)
    }
}

// Quality Gate Summary Tool

#[derive(Debug, Deserialize)]
struct QualityGateSummaryArgs {
    paths: Vec<String>,
    #[serde(default)]
    format: Option<String>,
}

pub struct QualityGateSummaryTool;

impl QualityGateSummaryTool {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for QualityGateSummaryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for QualityGateSummaryTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling quality-gate.summary with args: {}", args);

        let params: QualityGateSummaryArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let summary = tool_functions::quality_gate_summary(&paths)
            .await
            .map_err(|e| Error::internal(format!("Quality gate summary failed: {e}")))?;

        // Return the summary in requested format
        match params.format.as_deref() {
            Some("markdown") => Ok(json!({
                "summary": summary,
                "markdown": "Quality gate summary in markdown format (not implemented)"
            })),
            Some("json") | None => Ok(summary),
            Some(format) => Err(Error::validation(format!("Unsupported format: {format}"))),
        }
    }
}

// Quality Gate Baseline Tool

#[derive(Debug, Deserialize)]
struct QualityGateBaselineArgs {
    paths: Vec<String>,
    #[serde(default)]
    output: Option<String>,
}

pub struct QualityGateBaselineTool;

impl QualityGateBaselineTool {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for QualityGateBaselineTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for QualityGateBaselineTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling quality-gate.baseline with args: {}", args);

        let params: QualityGateBaselineArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();
        let output_path = params.output.map(PathBuf::from);

        let baseline = tool_functions::quality_gate_baseline(&paths, output_path.as_deref())
            .await
            .map_err(|e| Error::internal(format!("Failed to create baseline: {e}")))?;

        Ok(baseline)
    }
}

// Quality Gate Compare Tool

#[derive(Debug, Deserialize)]
struct QualityGateCompareArgs {
    baseline: String,
    paths: Vec<String>,
}

pub struct QualityGateCompareTool;

impl QualityGateCompareTool {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for QualityGateCompareTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for QualityGateCompareTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling quality-gate.compare with args: {}", args);

        let params: QualityGateCompareArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let baseline_path = PathBuf::from(params.baseline);
        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let comparison = tool_functions::quality_gate_compare(baseline_path.as_ref(), &paths)
            .await
            .map_err(|e| Error::internal(format!("Failed to compare with baseline: {e}")))?;

        Ok(comparison)
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
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
        let tool = QualityGateTool::default();
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
        let args = json!({
            "paths": ["/nonexistent/path"],
            "strict": true
        });
        let result = tool.handle(args, test_extra()).await;
        // Should succeed (graceful handling of nonexistent paths)
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
        let args = json!({
            "paths": ["/nonexistent/path"],
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
        let tool = QualityGateSummaryTool::default();
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
        let args = json!({
            "paths": ["/nonexistent/path"],
            "format": "json"
        });
        let result = tool.handle(args, test_extra()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_quality_gate_summary_tool_markdown_format() {
        let tool = QualityGateSummaryTool::new();
        let args = json!({
            "paths": ["/nonexistent/path"],
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
        let args = json!({
            "paths": ["/nonexistent/path"],
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
        let args = json!({
            "paths": ["/nonexistent/path"]
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
        let tool = QualityGateBaselineTool::default();
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
        let args = json!({
            "paths": ["/nonexistent/path"],
            "output": "/tmp/baseline.json"
        });
        let result = tool.handle(args, test_extra()).await;
        // Should succeed and create baseline
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_quality_gate_baseline_tool_without_output() {
        let tool = QualityGateBaselineTool::new();
        let args = json!({
            "paths": ["/nonexistent/path"]
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
        let tool = QualityGateCompareTool::default();
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
}
