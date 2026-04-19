// Complexity Analysis Tool

/// MCP args for analyze.complexity: paths to scan, optional top_files limit and complexity threshold.
#[derive(Debug, Deserialize)]
struct ComplexityArgs {
    paths: Vec<String>,
    #[serde(default)]
    top_files: Option<usize>,
    #[serde(default)]
    threshold: Option<u64>,
}

/// Tool handler for analyzing code complexity metrics.
///
/// This tool calculates cyclomatic and cognitive complexity for source files,
/// helping identify areas of code that may be difficult to understand or maintain.
///
/// # Arguments
///
/// The tool accepts JSON arguments with the following schema:
/// ```json
/// {
///   "paths": ["src/", "tests/"],        // Required: paths to analyze
///   "top_files": 10,                    // Optional: number of files to return
///   "threshold": 20                     // Optional: complexity threshold
/// }
/// ```ignore
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::mcp_pmcp::analyze_handlers::AnalyzeComplexityTool;
/// use pmcp::ToolHandler;
/// use serde_json::json;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let tool = AnalyzeComplexityTool::new();
/// let args = json!({
///     "paths": ["src/"],
///     "top_files": 5,
///     "threshold": 15
/// });
///
/// // In practice, this would be called by the MCP server
/// // let result = tool.handle(args, test_extra()).await?;
/// # Ok(())
/// # }
/// ```
pub struct ComplexityTool;

impl ComplexityTool {
    /// Creates a new complexity analysis tool handler.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self
    }
}

impl Default for ComplexityTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for ComplexityTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling analyze.complexity with args: {}", args);

        let params: ComplexityArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let results =
            tool_functions::analyze_complexity(&paths, params.top_files, params.threshold)
                .await
                .map_err(|e| Error::internal(format!("Complexity analysis failed: {e}")))?;

        Ok(results)
    }

    fn metadata(&self) -> Option<ToolInfo> {
        let extra = json!({
            "top_files": { "type": "integer", "description": "Return only the top N most-complex files" },
            "threshold": { "type": "integer", "description": "Minimum cyclomatic complexity to report" }
        });
        Some(build_tool_info(
            "analyze_complexity",
            "Analyze cyclomatic and cognitive complexity for source files.",
            paths_object_schema(extra, vec!["paths"]),
        ))
    }
}
