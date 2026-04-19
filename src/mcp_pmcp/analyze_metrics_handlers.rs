// Lint Hotspot Analysis Tool

/// MCP args for analyze.lint-hotspot: paths to scan for lint violations, optional top_files limit.
#[derive(Debug, Deserialize)]
struct LintHotspotArgs {
    paths: Vec<String>,
    #[serde(default)]
    top_files: Option<usize>,
}

/// Lint hotspot tool.
pub struct LintHotspotTool;

impl LintHotspotTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for LintHotspotTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for LintHotspotTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling analyze.lint-hotspot with args: {}", args);

        let params: LintHotspotArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let results = tool_functions::analyze_lint_hotspots(&paths, params.top_files)
            .await
            .map_err(|e| Error::internal(format!("Lint hotspot analysis failed: {e}")))?;

        Ok(results)
    }

    fn metadata(&self) -> Option<ToolInfo> {
        let extra = json!({
            "top_files": { "type": "integer", "description": "Return only the top N files with the most lint violations" }
        });
        // Registered as `analyze_dag` in server.rs; keep the registered name for consistency.
        Some(build_tool_info(
            "analyze_dag",
            "Generate a dependency graph (DAG) / lint hotspot report highlighting files with the most violations.",
            paths_object_schema(extra, vec!["paths"]),
        ))
    }
}

// Churn Analysis Tool

/// MCP args for analyze.churn: paths to analyze for git change frequency, optional days window and top_files limit.
#[derive(Debug, Deserialize)]
struct ChurnArgs {
    paths: Vec<String>,
    #[serde(default)]
    days: Option<u32>,
    #[serde(default)]
    top_files: Option<usize>,
}

/// Churn tool.
pub struct ChurnTool;

impl ChurnTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ChurnTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for ChurnTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling analyze.churn with args: {}", args);

        let params: ChurnArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let results = tool_functions::analyze_churn(&paths, params.days, params.top_files)
            .await
            .map_err(|e| Error::internal(format!("Churn analysis failed: {e}")))?;

        Ok(results)
    }

    fn metadata(&self) -> Option<ToolInfo> {
        let extra = json!({
            "days":      { "type": "integer", "description": "Git history window in days (default: 90)" },
            "top_files": { "type": "integer", "description": "Return only the top N files by churn" }
        });
        // Registered as `analyze_deep_context` in server.rs.
        Some(build_tool_info(
            "analyze_deep_context",
            "Comprehensive code analysis combining git churn history with code metrics.",
            paths_object_schema(extra, vec!["paths"]),
        ))
    }
}

// Coupling Analysis Tool

/// MCP args for analyze.coupling: paths to analyze for module coupling, optional similarity threshold.
#[derive(Debug, Deserialize)]
struct CouplingArgs {
    paths: Vec<String>,
    #[serde(default)]
    threshold: Option<f64>,
}

/// Coupling tool.
pub struct CouplingTool;

impl CouplingTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CouplingTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for CouplingTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling analyze.coupling with args: {}", args);

        let params: CouplingArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let results = tool_functions::analyze_coupling(&paths, params.threshold)
            .await
            .map_err(|e| Error::internal(format!("Coupling analysis failed: {e}")))?;

        Ok(results)
    }

    fn metadata(&self) -> Option<ToolInfo> {
        let extra = json!({
            "threshold": { "type": "number", "description": "Minimum similarity threshold for reporting coupled modules (0.0-1.0)" }
        });
        // Registered as `analyze_big_o` in server.rs.
        Some(build_tool_info(
            "analyze_big_o",
            "Big-O complexity / module coupling analysis.",
            paths_object_schema(extra, vec!["paths"]),
        ))
    }
}

// ============================================================================
// R17-1: Correct handlers for analyze_dag / analyze_big_o / analyze_deep_context
//
// These handlers replace the old type aliases that mis-routed these three MCP
// tools to lint-hotspot / coupling / churn. Each now dispatches to the right
// `tool_functions::*` implementation.
// ============================================================================

// --- DAG analysis tool ---

/// MCP args for analyze_dag: project paths and optional DAG type filter.
#[derive(Debug, Deserialize)]
struct AnalyzeDagArgs {
    paths: Vec<String>,
    #[serde(default)]
    dag_type: Option<String>,
}

/// DAG analysis tool. Generates dependency graphs (call / import / inheritance).
pub struct AnalyzeDagTool;

impl AnalyzeDagTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AnalyzeDagTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for AnalyzeDagTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling analyze_dag with args: {}", args);

        let params: AnalyzeDagArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let results = tool_functions::analyze_dag(&paths, params.dag_type)
            .await
            .map_err(|e| Error::internal(format!("DAG analysis failed: {e}")))?;

        Ok(results)
    }
}

// --- Big-O analysis tool ---

/// MCP args for analyze_big_o: project paths and optional top_files limit.
#[derive(Debug, Deserialize)]
struct AnalyzeBigOArgs {
    paths: Vec<String>,
    #[serde(default)]
    top_files: Option<usize>,
}

/// Big-O analysis tool. Classifies function time complexity.
pub struct AnalyzeBigOTool;

impl AnalyzeBigOTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AnalyzeBigOTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for AnalyzeBigOTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling analyze_big_o with args: {}", args);

        let params: AnalyzeBigOArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let results = tool_functions::analyze_big_o(&paths, params.top_files)
            .await
            .map_err(|e| Error::internal(format!("Big-O analysis failed: {e}")))?;

        Ok(results)
    }
}

// --- Deep context analysis tool ---

/// MCP args for analyze_deep_context: project paths and optional include patterns.
#[derive(Debug, Deserialize)]
struct AnalyzeDeepContextArgs {
    paths: Vec<String>,
    #[serde(default)]
    include_patterns: Option<Vec<String>>,
}

/// Deep context analysis tool. Runs the full deep-context pipeline.
pub struct AnalyzeDeepContextTool;

impl AnalyzeDeepContextTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AnalyzeDeepContextTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for AnalyzeDeepContextTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug!("Handling analyze_deep_context with args: {}", args);

        let params: AnalyzeDeepContextArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let results = tool_functions::analyze_deep_context(&paths, params.include_patterns)
            .await
            .map_err(|e| Error::internal(format!("Deep context analysis failed: {e}")))?;

        Ok(results)
    }
}
