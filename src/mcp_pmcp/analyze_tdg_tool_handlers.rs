// TDG (Technical Debt Grading) Analysis Tool

/// MCP args for analyze.tdg: paths to grade, optional threshold, top_files limit, component breakdown, and git context.
#[derive(Debug, Deserialize)]
struct TdgArgs {
    paths: Vec<String>,
    #[serde(default)]
    threshold: Option<f64>,
    #[serde(default)]
    top_files: Option<usize>,
    #[serde(default)]
    include_components: Option<bool>,
    #[serde(default)] // Sprint 65: Git-commit correlation
    with_git_context: Option<bool>,
}

/// Tool handler for analyzing Technical Debt Grading (TDG) scores.
///
/// This tool calculates comprehensive code quality scores using orthogonal metrics:
/// structural complexity, semantic complexity, code duplication, coupling,
/// documentation coverage, and code consistency.
///
/// # Arguments
///
/// The tool accepts JSON arguments with the following schema:
/// ```json
/// {
///   "paths": ["src/", "tests/"],        // Required: paths to analyze
///   "threshold": 75.0,                  // Optional: minimum score threshold
///   "top_files": 10,                    // Optional: number of files to return
///   "include_components": true          // Optional: include metric breakdown
/// }
/// ```ignore
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::mcp_pmcp::analyze_handlers::AnalyzeTdgTool;
/// use pmcp::ToolHandler;
/// use serde_json::json;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let tool = AnalyzeTdgTool::new();
/// let args = json!({
///     "paths": ["src/"],
///     "threshold": 80.0,
///     "top_files": 5,
///     "include_components": true
/// });
///
/// // In practice, this would be called by the MCP server
/// // let result = tool.handle(args, test_extra()).await?;
/// # Ok(())
/// # }
/// ```
pub struct TdgTool;

impl TdgTool {
    /// Creates a new TDG analysis tool handler.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for TdgTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for TdgTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling analyze.tdg with args: {}", args);

        let params: TdgArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let results = tool_functions::analyze_tdg(
            &paths,
            params.threshold,
            params.top_files,
            params.include_components,
            params.with_git_context, // Sprint 65: Git-commit correlation
        )
        .await
        .map_err(|e| Error::internal(format!("TDG analysis failed: {e}")))?;

        Ok(results)
    }
}

// TDG Comparison Tool

/// MCP args for analyze.tdg_compare: two paths (path1, path2) to diff TDG scores, optional git context.
#[derive(Debug, Deserialize)]
struct TdgCompareArgs {
    path1: String,
    path2: String,
    #[serde(default)] // Sprint 65: Git-commit correlation
    with_git_context: Option<bool>,
}

/// Tool handler for comparing TDG scores between two files or directories.
///
/// This tool compares the technical debt grading scores between two codebases
/// or files, showing improvements, regressions, and overall changes.
///
/// # Arguments
///
/// The tool accepts JSON arguments with the following schema:
/// ```json
/// {
///   "path1": "src/old_version/",       // Required: first path to compare
///   "path2": "src/new_version/"        // Required: second path to compare
/// }
/// ```ignore
pub struct TdgCompareTool;

impl TdgCompareTool {
    /// Creates a new TDG comparison tool handler.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for TdgCompareTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for TdgCompareTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling analyze.tdg_compare with args: {}", args);

        let params: TdgCompareArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let path1 = PathBuf::from(params.path1);
        let path2 = PathBuf::from(params.path2);

        let results = tool_functions::compare_tdg(&path1, &path2, params.with_git_context)
            .await
            .map_err(|e| Error::internal(format!("TDG comparison failed: {e}")))?;

        Ok(results)
    }
}
