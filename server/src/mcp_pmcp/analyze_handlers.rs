//! Analysis tool handlers for the pmcp-based MCP server.
//!
//! This module contains tool handlers for various code analysis operations
//! including complexity analysis, technical debt detection, and code quality metrics.

use crate::mcp_pmcp::tool_functions;
use async_trait::async_trait;
use pmcp::{Error, RequestHandlerExtra, Result, ToolHandler};
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use tracing::debug;

// Re-export for convenience
pub use self::{
    ChurnTool as AnalyzeDeepContextTool, ComplexityTool as AnalyzeComplexityTool,
    CouplingTool as AnalyzeBigOTool, DeadCodeTool as AnalyzeDeadCodeTool,
    LintHotspotTool as AnalyzeDagTool, SatdTool as AnalyzeSatdTool,
    TdgTool as AnalyzeTdgTool, TdgCompareTool as AnalyzeTdgCompareTool,
};

// Complexity Analysis Tool

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
/// ```
///
/// # Examples
///
/// ```rust
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
/// // let result = tool.handle(args, Default::default()).await?;
/// # Ok(())
/// # }
/// ```
pub struct ComplexityTool;

impl ComplexityTool {
    /// Creates a new complexity analysis tool handler.
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
            .map_err(|e| Error::validation(format!("Invalid arguments: {}", e)))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let results =
            tool_functions::analyze_complexity(&paths, params.top_files, params.threshold)
                .await
                .map_err(|e| Error::internal(format!("Complexity analysis failed: {}", e)))?;

        Ok(results)
    }
}

// SATD (Self-Admitted Technical Debt) Analysis Tool

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
/// ```
///
/// # Examples
///
/// ```rust
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
            .map_err(|e| Error::validation(format!("Invalid arguments: {}", e)))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let results = tool_functions::analyze_satd(&paths, params.include_resolved)
            .await
            .map_err(|e| Error::internal(format!("SATD analysis failed: {}", e)))?;

        Ok(results)
    }
}

// Dead Code Analysis Tool

#[derive(Debug, Deserialize)]
struct DeadCodeArgs {
    paths: Vec<String>,
    #[serde(default)]
    include_tests: bool,
}

pub struct DeadCodeTool;

impl DeadCodeTool {
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
            .map_err(|e| Error::validation(format!("Invalid arguments: {}", e)))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let results = tool_functions::analyze_dead_code(&paths, params.include_tests)
            .await
            .map_err(|e| Error::internal(format!("Dead code analysis failed: {}", e)))?;

        Ok(results)
    }
}

// Lint Hotspot Analysis Tool

#[derive(Debug, Deserialize)]
struct LintHotspotArgs {
    paths: Vec<String>,
    #[serde(default)]
    top_files: Option<usize>,
}

pub struct LintHotspotTool;

impl LintHotspotTool {
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
            .map_err(|e| Error::validation(format!("Invalid arguments: {}", e)))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let results = tool_functions::analyze_lint_hotspots(&paths, params.top_files)
            .await
            .map_err(|e| Error::internal(format!("Lint hotspot analysis failed: {}", e)))?;

        Ok(results)
    }
}

// Churn Analysis Tool

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
            .map_err(|e| Error::validation(format!("Invalid arguments: {}", e)))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let results = tool_functions::analyze_churn(&paths, params.days, params.top_files)
            .await
            .map_err(|e| Error::internal(format!("Churn analysis failed: {}", e)))?;

        Ok(results)
    }
}

// Coupling Analysis Tool

#[derive(Debug, Deserialize)]
struct CouplingArgs {
    paths: Vec<String>,
    #[serde(default)]
    threshold: Option<f64>,
}

pub struct CouplingTool;

impl CouplingTool {
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
            .map_err(|e| Error::validation(format!("Invalid arguments: {}", e)))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let results = tool_functions::analyze_coupling(&paths, params.threshold)
            .await
            .map_err(|e| Error::internal(format!("Coupling analysis failed: {}", e)))?;

        Ok(results)
    }
}

// TDG (Technical Debt Grading) Analysis Tool

#[derive(Debug, Deserialize)]
struct TdgArgs {
    paths: Vec<String>,
    #[serde(default)]
    threshold: Option<f64>,
    #[serde(default)]
    top_files: Option<usize>,
    #[serde(default)]
    include_components: Option<bool>,
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
/// ```
///
/// # Examples
///
/// ```rust
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
/// // let result = tool.handle(args, Default::default()).await?;
/// # Ok(())
/// # }
/// ```
pub struct TdgTool;

impl TdgTool {
    /// Creates a new TDG analysis tool handler.
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
            .map_err(|e| Error::validation(format!("Invalid arguments: {}", e)))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        let results = tool_functions::analyze_tdg(
            &paths, 
            params.threshold, 
            params.top_files, 
            params.include_components
        )
        .await
        .map_err(|e| Error::internal(format!("TDG analysis failed: {}", e)))?;

        Ok(results)
    }
}

// TDG Comparison Tool

#[derive(Debug, Deserialize)]
struct TdgCompareArgs {
    path1: String,
    path2: String,
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
/// ```
pub struct TdgCompareTool;

impl TdgCompareTool {
    /// Creates a new TDG comparison tool handler.
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
            .map_err(|e| Error::validation(format!("Invalid arguments: {}", e)))?;

        let path1 = PathBuf::from(params.path1);
        let path2 = PathBuf::from(params.path2);

        let results = tool_functions::compare_tdg(&path1, &path2)
            .await
            .map_err(|e| Error::internal(format!("TDG comparison failed: {}", e)))?;

        Ok(results)
    }
}
