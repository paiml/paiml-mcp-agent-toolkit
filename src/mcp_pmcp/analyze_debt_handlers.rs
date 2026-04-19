// SATD (Self-Admitted Technical Debt) Analysis Tool

/// MCP args for analyze.satd: paths to scan for self-admitted technical debt, optional include_resolved flag.
#[derive(Debug, Deserialize)]
struct SatdArgs {
    paths: Vec<String>,
    #[serde(default)]
    include_resolved: bool,
}

/// Tool handler for detecting self-admitted technical debt in code comments.
///
/// This tool scans source files for TODO, FIXME, HACK, and other markers
/// that indicate technical debt acknowledged by developers.
///
/// # Arguments
///
/// ```json
/// {
///   "paths": ["src/"],              // Required: paths to analyze
///   "include_resolved": false       // Optional: include resolved items
/// }
/// ```ignore
///
/// # Examples
///
/// ```rust,no_run
/// # #[cfg(feature = "pmcp-mcp")]
/// # {
/// use pmat::mcp_pmcp::analyze_handlers::AnalyzeSatdTool;
/// use serde_json::json;
///
/// let tool = AnalyzeSatdTool::new();
/// let args = json!({
///     "paths": ["src/", "tests/"],
///     "include_resolved": false
/// });
/// # }
/// ```
pub struct SatdTool;

impl SatdTool {
    /// Creates a new SATD analysis tool handler.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self
    }
}

impl Default for SatdTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for SatdTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling analyze.satd with args: {}", args);

        let params: SatdArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let results = tool_functions::analyze_satd(&paths, params.include_resolved)
            .await
            .map_err(|e| Error::internal(format!("SATD analysis failed: {e}")))?;

        Ok(results)
    }

    fn metadata(&self) -> Option<ToolInfo> {
        let extra = json!({
            "include_resolved": { "type": "boolean", "description": "Include items already marked resolved" }
        });
        Some(build_tool_info(
            "analyze_satd",
            "Detect self-admitted technical debt (TODO, FIXME, HACK markers) in source code.",
            paths_object_schema(extra, vec!["paths"]),
        ))
    }
}

// Dead Code Analysis Tool

/// MCP args for analyze.dead-code: paths to scan for unreachable code, optional include_tests flag.
#[derive(Debug, Deserialize)]
struct DeadCodeArgs {
    paths: Vec<String>,
    #[serde(default)]
    include_tests: bool,
}

/// Dead code tool.
pub struct DeadCodeTool;

impl DeadCodeTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DeadCodeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for DeadCodeTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling analyze.dead-code with args: {}", args);

        let params: DeadCodeArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let results = tool_functions::analyze_dead_code(&paths, params.include_tests)
            .await
            .map_err(|e| Error::internal(format!("Dead code analysis failed: {e}")))?;

        Ok(results)
    }

    fn metadata(&self) -> Option<ToolInfo> {
        let extra = json!({
            "include_tests": { "type": "boolean", "description": "Include test files when searching for dead code" }
        });
        Some(build_tool_info(
            "analyze_dead_code",
            "Find unreachable or unused code (functions, types, or modules).",
            paths_object_schema(extra, vec!["paths"]),
        ))
    }
}
