// Lint Hotspot Analysis Tool

/// MCP args for analyze.lint-hotspot: paths to scan for lint violations, optional top_files limit.
#[derive(Debug, Deserialize)]
struct LintHotspotArgs {
    paths: Vec<String>,
    #[serde(default)]
    top_files: Option<usize>,
}

pub struct LintHotspotTool;

impl LintHotspotTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
        debug_assert!(true, "contract: handle");
        debug!("Handling analyze.lint-hotspot with args: {}", args);

        let params: LintHotspotArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let results = tool_functions::analyze_lint_hotspots(&paths, params.top_files)
            .await
            .map_err(|e| Error::internal(format!("Lint hotspot analysis failed: {e}")))?;

        Ok(results)
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

pub struct ChurnTool;

impl ChurnTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
        debug_assert!(true, "contract: handle");
        debug!("Handling analyze.churn with args: {}", args);

        let params: ChurnArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let results = tool_functions::analyze_churn(&paths, params.days, params.top_files)
            .await
            .map_err(|e| Error::internal(format!("Churn analysis failed: {e}")))?;

        Ok(results)
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

pub struct CouplingTool;

impl CouplingTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
        debug_assert!(true, "contract: handle");
        debug!("Handling analyze.coupling with args: {}", args);

        let params: CouplingArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let results = tool_functions::analyze_coupling(&paths, params.threshold)
            .await
            .map_err(|e| Error::internal(format!("Coupling analysis failed: {e}")))?;

        Ok(results)
    }
}
