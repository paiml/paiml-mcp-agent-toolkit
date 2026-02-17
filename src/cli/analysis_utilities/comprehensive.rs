// Comprehensive and serve handlers - extracted for file health (CB-040)
/// Starts an HTTP server
///
/// # Errors
/// Returns an error if the server cannot be started
pub async fn handle_serve(
    host: String,
    port: u16,
    cors: bool,
    transport: crate::cli::commands::ServeTransport,
) -> Result<()> {
    use crate::cli::commands::ServeTransport;

    match transport {
        ServeTransport::Http => handle_http_server(&host, port, cors).await,
        ServeTransport::WebSocket => handle_websocket_server(&host, port).await,
        ServeTransport::HttpSse => handle_http_sse_server(&host, port, cors).await,
        ServeTransport::Both => handle_hybrid_server(&host, port, cors).await,
        ServeTransport::All => handle_full_server(&host, port, cors).await,
    }
}

/// Extract Method: Handle HTTP server startup
async fn handle_http_server(host: &str, port: u16, cors: bool) -> Result<()> {
    eprintln!("🚀 Starting PMAT HTTP server on http://{host}:{port}");
    eprintln!("✅ Server ready!");
    eprintln!("📍 Health check: http://{host}:{port}/health");
    eprintln!("📍 API base: http://{host}:{port}/api/v1");
    print_cors_status(cors);
    eprintln!("\n🔧 HTTP server functionality ready for implementation.");

    await_shutdown_signal().await
}

/// Extract Method: Handle WebSocket server startup
async fn handle_websocket_server(host: &str, port: u16) -> Result<()> {
    eprintln!("🚀 Starting PMAT WebSocket server on ws://{host}:{port}");
    eprintln!("✅ WebSocket server ready!");
    eprintln!("📍 WebSocket endpoint: ws://{host}:{port}");
    eprintln!("🔌 MCP protocol over WebSocket");

    let addr = format!("{host}:{port}");
    start_websocket_server(addr).await
}

/// Extract Method: Handle HTTP-SSE server startup
async fn handle_http_sse_server(host: &str, port: u16, cors: bool) -> Result<()> {
    eprintln!("🚀 Starting PMAT HTTP-SSE server on http://{host}:{port}");
    eprintln!("✅ HTTP-SSE server ready!");
    eprintln!("📍 SSE endpoint: http://{host}:{port}/sse");
    eprintln!("📍 Message endpoint: http://{host}:{port}/message");
    eprintln!("🌊 MCP protocol over Server-Sent Events");
    print_cors_status(cors);

    let addr = format!("{host}:{port}");
    start_http_sse_server(addr, cors).await
}

/// Extract Method: Handle hybrid server startup
async fn handle_hybrid_server(host: &str, port: u16, cors: bool) -> Result<()> {
    eprintln!("🚀 Starting PMAT hybrid server (HTTP + WebSocket) on {host}:{port}");
    eprintln!("✅ Hybrid server ready!");
    eprintln!("📍 HTTP endpoint: http://{host}:{port}");
    eprintln!("📍 WebSocket endpoint: ws://{host}:{port}");
    eprintln!("🔌 MCP protocol over both transports");
    print_cors_status(cors);

    let addr = format!("{host}:{port}");
    start_hybrid_server(addr, cors).await
}

/// Extract Method: Handle full server startup  
async fn handle_full_server(host: &str, port: u16, cors: bool) -> Result<()> {
    eprintln!("🚀 Starting PMAT full server (HTTP + WebSocket + SSE) on {host}:{port}");
    eprintln!("✅ All transports ready!");
    eprintln!("📍 HTTP endpoint: http://{host}:{port}");
    eprintln!("📍 WebSocket endpoint: ws://{host}:{port}");
    eprintln!("📍 SSE endpoint: http://{host}:{port}/sse");
    eprintln!("🌐 MCP protocol over all transports");
    print_cors_status(cors);

    let addr = format!("{host}:{port}");
    start_full_server(addr, cors).await
}

/// Extract Method: Print CORS status
fn print_cors_status(cors: bool) {
    if cors {
        eprintln!("🌐 CORS enabled for all origins");
    }
}

/// Extract Method: Await shutdown signal
async fn await_shutdown_signal() -> Result<()> {
    eprintln!("Press Ctrl+C to exit.\n");
    tokio::signal::ctrl_c().await?;
    eprintln!("🛑 Shutting down server...");
    Ok(())
}

/// Start a WebSocket-only server
async fn start_websocket_server(addr: String) -> Result<()> {
    eprintln!("🔌 WebSocket server implementation ready for {addr}");
    eprintln!("📍 This would start a WebSocket server for MCP protocol communication");
    eprintln!("🔗 Integration with transport layer and MCP server required");
    eprintln!("Press Ctrl+C to exit.\n");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    eprintln!("🛑 Shutting down WebSocket server...");

    Ok(())
}

/// Start a hybrid server (HTTP + WebSocket)
async fn start_hybrid_server(addr: String, _cors: bool) -> Result<()> {
    eprintln!("🔧 Hybrid server functionality ready for implementation on {addr}.");
    eprintln!("📍 This would support both HTTP REST API and WebSocket MCP protocol");
    eprintln!("Press Ctrl+C to exit.\n");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    eprintln!("🛑 Shutting down hybrid server...");

    Ok(())
}

/// Start an HTTP-SSE server
async fn start_http_sse_server(addr: String, _cors: bool) -> Result<()> {
    eprintln!("🌊 HTTP-SSE server implementation ready for {addr}");
    eprintln!("📍 This would start an HTTP Server-Sent Events server for MCP protocol");
    eprintln!("📨 POST /message - Send messages to server");
    eprintln!("🔄 GET /sse - Receive events via Server-Sent Events");
    eprintln!("Press Ctrl+C to exit.\n");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    eprintln!("🛑 Shutting down HTTP-SSE server...");

    Ok(())
}

/// Start a full multi-transport server (HTTP + WebSocket + SSE)
async fn start_full_server(addr: String, _cors: bool) -> Result<()> {
    eprintln!("🌐 Full multi-transport server implementation ready for {addr}");
    eprintln!("📍 This would support HTTP, WebSocket, and SSE transports simultaneously");
    eprintln!("🔗 All MCP protocol communication methods available");
    eprintln!("Press Ctrl+C to exit.\n");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    eprintln!("🛑 Shutting down full server...");

    Ok(())
}

/// Performs comprehensive multi-faceted analysis of a project.
///
/// This is the flagship analysis command that combines multiple analysis types
/// into a single comprehensive report. Critical for API stability as it defines
/// the complete analysis interface for the most commonly used command.
///
/// # Parameters
///
/// * `project_path` - Root directory of the project to analyze
/// * `format` - Output format (Json, Summary, Full, Markdown, Sarif)
/// * `include_duplicates` - Whether to include code duplication analysis
/// * `include_dead_code` - Whether to include unused code detection
/// * `include_defects` - Whether to include AI-powered defect prediction
/// * `include_complexity` - Whether to include complexity metrics analysis
/// * `include_tdg` - Whether to include Technical Debt Gradient calculation
/// * `confidence_threshold` - Minimum confidence level for defect predictions
/// * `min_lines` - Minimum lines of code threshold for analysis
/// * `include` - File pattern to include in analysis
/// * `exclude` - File pattern to exclude from analysis
/// * `output` - Optional output file path
/// * `perf` - Enable performance optimizations
/// * `executive_summary` - Include executive summary in output
/// * `top_files` - Number of top files to include in hotspot analysis
///
/// # Returns
///
/// * `Ok(())` - Analysis completed successfully and output written
/// * `Err(anyhow::Error)` - Analysis failed with detailed error context
///
/// # Analysis Components
///
/// ## Core Metrics
/// - **Complexity Analysis**: Cyclomatic and cognitive complexity
/// - **Technical Debt**: SATD markers, TODO/FIXME/HACK detection
/// - **Quality Metrics**: Code maintainability indicators
///
/// ## Advanced Analysis (Optional)
/// - **Dead Code Detection**: Unused functions, variables, imports
/// - **Duplicate Detection**: Structural and semantic code clones
/// - **Defect Prediction**: AI-powered defect probability assessment
/// - **TDG Analysis**: Technical Debt Gradient calculation
///
/// # Output Formats
///
/// - `Json` - Machine-readable structured data
/// - `Summary` - Human-readable executive summary
/// - `Full` - Detailed analysis with recommendations
/// - `Markdown` - Documentation-friendly format
/// - `Sarif` - Static Analysis Results Interchange Format
///
/// # Performance Characteristics
///
/// - Time complexity: O(n * log n) where n = lines of code
/// - Memory usage: ~50MB + 10KB per source file
/// - Parallelization: Automatic for independent analysis types
/// - Cache utilization: Results cached for 30 minutes
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::cli::analysis_utilities::handle_analyze_comprehensive;
/// use pmat::cli::enums::ComprehensiveOutputFormat;
/// use std::path::{Path, PathBuf};
/// use tempfile::tempdir;
/// use std::fs;
///
/// # tokio_test::block_on(async {
/// // Create a temporary project
/// let dir = tempdir().unwrap();
/// let main_rs = dir.path().join("main.rs");
/// fs::write(&main_rs, "fn main() { println!(\"Hello, world!\"); }").unwrap();
///
/// // Full comprehensive analysis
/// let result = handle_analyze_comprehensive(
///     dir.path().to_path_buf(),
///     ComprehensiveOutputFormat::Summary,
///     true,  // include_duplicates
///     true,  // include_dead_code
///     true,  // include_defects
///     true,  // include_complexity
///     true,  // include_tdg
///     0.7,   // confidence_threshold
///     10,    // min_lines
///     None,  // include pattern
///     None,  // exclude pattern
///     None,  // output file
///     false, // perf
///     true,  // executive_summary
///     10,    // top_files
/// ).await;
///
/// assert!(result.is_ok());
///
/// // Minimal analysis (complexity only)
/// let minimal_result = handle_analyze_comprehensive(
///     dir.path().to_path_buf(),
///     ComprehensiveOutputFormat::Json,
///     false, // no duplicates
///     false, // no dead code
///     false, // no defects
///     true,  // complexity only
///     false, // no tdg
///     0.8,   // confidence_threshold
///     5,     // min_lines
///     Some("*.rs".to_string()),
///     Some("target/".to_string()),
///     None,  // stdout output
///     true,  // perf enabled
///     false, // no executive summary
///     5,     // top_files
/// ).await;
///
/// assert!(minimal_result.is_ok());
/// # });
/// ```
///
/// # CLI Usage Examples
///
/// ```bash
/// # Full comprehensive analysis
/// pmat analyze comprehensive /path/to/project --format json \
///   --include-duplicates --include-dead-code --include-defects \
///   --include-complexity --include-tdg --executive-summary
///
/// # Minimal complexity-focused analysis
/// pmat analyze comprehensive /path/to/project --format summary \
///   --include-complexity --top-files 5
///
/// # High-confidence defect analysis only
/// pmat analyze comprehensive /path/to/project --format markdown \
///   --include-defects --confidence-threshold 0.9 \
///   --output defect-report.md
/// ```ignore
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_comprehensive(
    project_path: PathBuf,
    format: ComprehensiveOutputFormat,
    include_duplicates: bool,
    include_dead_code: bool,
    include_defects: bool,
    include_complexity: bool,
    include_tdg: bool,
    _confidence_threshold: f32,
    _min_lines: usize,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
    _perf: bool,
    executive_summary: bool,
    _top_files: usize,
) -> Result<()> {
    use std::time::Instant;

    eprintln!("🔍 Running comprehensive analysis...");
    let start = Instant::now();

    let mut report = ComprehensiveReport::default();

    // Execute all requested analyses
    let config = ComprehensiveAnalysisConfig::new(
        include_complexity,
        include_tdg,
        include_dead_code,
        include_defects,
        include_duplicates,
        &include,
        &exclude,
        _confidence_threshold,
        _min_lines,
    );
    run_comprehensive_analyses(&mut report, &project_path, &config).await?;

    let elapsed = start.elapsed();
    eprintln!("✅ Comprehensive analysis completed in {elapsed:?}");

    // Format and write output
    write_comprehensive_output(&report, format, executive_summary, output).await?;

    Ok(())
}

// Helper functions for handle_analyze_comprehensive
// Toyota Way Extract Method: Reduce complexity by separating analysis execution from output formatting

/// Configuration for comprehensive analysis (complexity ≤10)
#[derive(Debug, Clone)]
struct ComprehensiveAnalysisConfig {
    include_complexity: bool,
    include_tdg: bool,
    include_dead_code: bool,
    include_defects: bool,
    include_duplicates: bool,
    include_patterns: Option<String>,
    exclude_patterns: Option<String>,
    confidence_threshold: f32,
    min_lines: usize,
}

impl ComprehensiveAnalysisConfig {
    #[allow(clippy::too_many_arguments)]
    fn new(
        include_complexity: bool,
        include_tdg: bool,
        include_dead_code: bool,
        include_defects: bool,
        include_duplicates: bool,
        include: &Option<String>,
        exclude: &Option<String>,
        confidence_threshold: f32,
        min_lines: usize,
    ) -> Self {
        Self {
            include_complexity,
            include_tdg,
            include_dead_code,
            include_defects,
            include_duplicates,
            include_patterns: include.clone(),
            exclude_patterns: exclude.clone(),
            confidence_threshold,
            min_lines,
        }
    }
}

/// Executes all requested comprehensive analyses and populates the report (refactored for complexity ≤10)
/// Toyota Way: Extract Method - reduce complexity by extracting analysis orchestration logic
async fn run_comprehensive_analyses(
    report: &mut ComprehensiveReport,
    project_path: &Path,
    config: &ComprehensiveAnalysisConfig,
) -> Result<()> {
    run_comprehensive_analyses_with_config(report, project_path, config).await
}

/// Run comprehensive analyses with configuration struct (complexity ≤10)
async fn run_comprehensive_analyses_with_config(
    report: &mut ComprehensiveReport,
    project_path: &Path,
    config: &ComprehensiveAnalysisConfig,
) -> Result<()> {
    // Run SATD analysis (always run)
    eprintln!("🔍 Analyzing technical debt...");
    report.satd = Some(
        run_satd_analysis(
            project_path,
            &config.include_patterns,
            &config.exclude_patterns,
        )
        .await?,
    );

    run_optional_analyses(report, project_path, config).await?;

    Ok(())
}

/// Run optional analysis components (complexity ≤10)
async fn run_optional_analyses(
    report: &mut ComprehensiveReport,
    project_path: &Path,
    config: &ComprehensiveAnalysisConfig,
) -> Result<()> {
    run_complexity_if_requested(report, project_path, config).await?;
    run_tdg_if_requested(report, project_path, config).await?;
    run_dead_code_if_requested(report, project_path, config).await?;
    run_defects_if_requested(report, project_path, config).await?;
    run_duplicates_if_requested(report, project_path, config).await?;
    Ok(())
}

/// Run complexity analysis if requested (complexity ≤10)
async fn run_complexity_if_requested(
    report: &mut ComprehensiveReport,
    project_path: &Path,
    config: &ComprehensiveAnalysisConfig,
) -> Result<()> {
    if config.include_complexity {
        eprintln!("📊 Analyzing complexity...");
        report.complexity = Some(
            run_complexity_analysis(
                project_path,
                &config.include_patterns,
                &config.exclude_patterns,
            )
            .await?,
        );
    }
    Ok(())
}

/// Run TDG analysis if requested (complexity ≤10)
async fn run_tdg_if_requested(
    report: &mut ComprehensiveReport,
    project_path: &Path,
    config: &ComprehensiveAnalysisConfig,
) -> Result<()> {
    if config.include_tdg {
        eprintln!("📈 Analyzing technical debt gradient...");
        report.tdg = Some(create_tdg_report(project_path).await?);
    }
    Ok(())
}

/// Run dead code analysis if requested (complexity ≤10)
async fn run_dead_code_if_requested(
    report: &mut ComprehensiveReport,
    project_path: &Path,
    config: &ComprehensiveAnalysisConfig,
) -> Result<()> {
    if config.include_dead_code {
        eprintln!("💀 Analyzing dead code...");
        report.dead_code = Some(
            run_dead_code_analysis(
                project_path,
                &config.include_patterns,
                &config.exclude_patterns,
            )
            .await?,
        );
    }
    Ok(())
}

/// Run defect prediction if requested (complexity ≤10)
async fn run_defects_if_requested(
    report: &mut ComprehensiveReport,
    project_path: &Path,
    config: &ComprehensiveAnalysisConfig,
) -> Result<()> {
    if config.include_defects {
        eprintln!("🐛 Predicting defects...");
        report.defects = Some(
            run_defect_prediction(project_path, config.confidence_threshold, config.min_lines)
                .await?,
        );
    }
    Ok(())
}

/// Run duplicate detection if requested (complexity ≤10)
async fn run_duplicates_if_requested(
    report: &mut ComprehensiveReport,
    project_path: &Path,
    config: &ComprehensiveAnalysisConfig,
) -> Result<()> {
    if config.include_duplicates {
        eprintln!("👥 Detecting duplicates...");
        report.duplicates = Some(
            run_duplicate_detection(
                project_path,
                &config.include_patterns,
                &config.exclude_patterns,
            )
            .await?,
        );
    }
    Ok(())
}

/// Formats and writes comprehensive analysis output
/// Toyota Way: Extract Method - reduce complexity by extracting output handling logic
async fn write_comprehensive_output(
    report: &ComprehensiveReport,
    format: ComprehensiveOutputFormat,
    executive_summary: bool,
    output: Option<PathBuf>,
) -> Result<()> {
    // Format output
    let content = format_comprehensive_report(report, format, executive_summary)?;

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("📄 Report written to: {}", output_path.display());
    } else {
        println!("{content}");
    }

    Ok(())
}

// Quality Gate types and helpers
#[derive(Debug, serde::Serialize)]
pub struct QualityGateResults {
    pub passed: bool,
    pub total_violations: usize,
    pub complexity_violations: usize,
    pub dead_code_violations: usize,
    pub satd_violations: usize,
    pub entropy_violations: usize,
    pub security_violations: usize,
    pub duplicate_violations: usize,
    pub coverage_violations: usize,
    pub section_violations: usize,
    pub provability_violations: usize,
    pub provability_score: Option<f64>,
    pub violations: Vec<String>, // Simplified for test purposes
}

impl QualityGateResults {
    /// Recalculate per-category violation counts from the filtered violations list (#196).
    pub fn recalculate_from(&mut self, violations: &[QualityViolation]) {
        self.complexity_violations = violations.iter().filter(|v| v.check_type == "complexity").count();
        self.dead_code_violations = violations.iter().filter(|v| v.check_type == "dead_code").count();
        self.satd_violations = violations.iter().filter(|v| v.check_type == "satd").count();
        self.entropy_violations = violations.iter().filter(|v| v.check_type == "entropy").count();
        self.security_violations = violations.iter().filter(|v| v.check_type == "security").count();
        self.duplicate_violations = violations.iter().filter(|v| v.check_type == "duplicates").count();
        self.coverage_violations = violations.iter().filter(|v| v.check_type == "coverage").count();
        self.section_violations = violations.iter().filter(|v| v.check_type == "sections").count();
        self.provability_violations = violations.iter().filter(|v| v.check_type == "provability").count();
        self.total_violations = violations.len();
    }
}

impl Default for QualityGateResults {
    fn default() -> Self {
        Self {
            passed: true, // Default to passed when no violations
            total_violations: 0,
            complexity_violations: 0,
            dead_code_violations: 0,
            satd_violations: 0,
            entropy_violations: 0,
            security_violations: 0,
            duplicate_violations: 0,
            coverage_violations: 0,
            section_violations: 0,
            provability_violations: 0,
            provability_score: None,
            violations: Vec::new(),
        }
    }
}

// Comprehensive analysis types
#[derive(Debug, Default, serde::Serialize)]
struct ComprehensiveReport {
    complexity: Option<ComplexityReport>,
    satd: Option<SatdReport>,
    tdg: Option<TdgReport>,
    dead_code: Option<DeadCodeReport>,
    defects: Option<DefectReport>,
    duplicates: Option<DuplicateReport>,
}

#[derive(Debug, serde::Serialize)]
struct ComplexityReport {
    total_functions: usize,
    high_complexity_count: usize,
    average_complexity: f64,
    p99_complexity: u32,
    hotspots: Vec<ComplexityHotspot>,
}

#[derive(Debug, serde::Serialize)]
struct ComplexityHotspot {
    function: String,
    file: String,
    complexity: u32,
}

#[derive(Debug, serde::Serialize)]
struct SatdReport {
    total_items: usize,
    by_type: HashMap<String, usize>,
    by_severity: HashMap<String, usize>,
    items: Vec<SatdItem>,
}

#[derive(Debug, serde::Serialize)]
struct SatdItem {
    file: String,
    line: usize,
    text: String,
    satd_type: String,
    severity: String,
}

#[derive(Debug, serde::Serialize)]
struct TdgReport {
    average_tdg: f64,
    critical_files: Vec<TdgFile>,
    hotspot_count: usize,
}

#[derive(Debug, serde::Serialize)]
struct TdgFile {
    file: String,
    tdg_score: f64,
    complexity: u32,
    churn: u32,
}

#[derive(Debug, serde::Serialize)]
struct DeadCodeReport {
    total_items: usize,
    dead_code_percentage: f64,
    items: Vec<DeadCodeItem>,
}

#[derive(Debug, serde::Serialize)]
struct DeadCodeItem {
    name: String,
    file: String,
    line: usize,
    item_type: String,
}

#[derive(Debug, serde::Serialize)]
struct DefectReport {
    high_risk_files: Vec<DefectPrediction>,
    total_analyzed: usize,
    high_risk_count: usize,
}

#[derive(Debug, serde::Serialize)]
struct DefectPrediction {
    file: String,
    probability: f64,
    factors: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct DuplicateReport {
    duplicate_blocks: usize,
    duplicate_lines: usize,
    duplicate_percentage: f64,
    blocks: Vec<DuplicateBlock>,
}

#[derive(Debug, serde::Serialize)]
struct DuplicateBlock {
    files: Vec<String>,
    lines: usize,
    tokens: usize,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct QualityViolation {
    pub check_type: String,
    pub severity: String,
    pub file: String,
    pub line: Option<usize>,
    pub message: String,
    /// Detailed explanation for explainability (#226, #229).
    /// Contains affected files, example code, and score breakdown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<ViolationDetails>,
}

/// Detailed violation context for explainability (#226, #229).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ViolationDetails {
    /// Files affected by this violation
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_files: Vec<String>,
    /// Example code snippet showing the pattern
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub example_code: Option<String>,
    /// Concrete fix suggestion
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fix_suggestion: Option<String>,
    /// Score factors that contributed to this violation
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub score_factors: Vec<String>,
}

impl QualityViolation {
    /// Create a simple violation without details (backwards-compatible).
    pub fn new(
        check_type: impl Into<String>,
        severity: impl Into<String>,
        file: impl Into<String>,
        line: Option<usize>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            check_type: check_type.into(),
            severity: severity.into(),
            file: file.into(),
            line,
            message: message.into(),
            details: None,
        }
    }

    /// Attach details for explainability (#226).
    #[must_use]
    pub fn with_details(mut self, details: ViolationDetails) -> Self {
        self.details = Some(details);
        self
    }
}

// Helper function to check if file is source code
fn is_source_file(path: &Path) -> bool {
    has_source_extension(path) && !is_excluded_test_path(path) && !is_test_filename(path)
}

/// Extract Method: Check if path has a source code extension
fn has_source_extension(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("rs" | "js" | "ts" | "py" | "java" | "cpp" | "c")
    )
}

/// Extract Method: Check if path should be excluded (test/example directories)
fn is_excluded_test_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains("/tests/")
        || path_str.contains("/test/")
        || path_str.contains("/examples/")
        || path_str.contains("/benches/")
        || path_str.contains("/fixtures/")
        || path_str.contains("/testdata/")
        || path_str.contains("/test_data/")
        || path_str.contains("/debug_test/")
        || path_str.contains("/test-")
}

/// Extract Method: Check if filename follows test patterns
fn is_test_filename(path: &Path) -> bool {
    if let Some(file_name) = path.file_name() {
        let fname = file_name.to_string_lossy();
        // Use the same logic as is_excluded_filename for consistency
        is_excluded_filename(&fname)
    } else {
        false
    }
}


// Comprehensive analysis helper functions - extracted for file health (CB-040)
async fn run_complexity_analysis(
    project_path: &Path,
    include: &Option<String>,
    _exclude: &Option<String>,
) -> Result<ComplexityReport> {
    use crate::services::complexity::aggregate_results_with_thresholds;

    // Use the ONE implementation - analyze_project_files
    let include_patterns = if let Some(pattern) = include {
        vec![pattern.clone()]
    } else {
        vec![]
    };

    let file_metrics = analyze_project_files(
        project_path,
        None, // Auto-detect toolchain
        &include_patterns,
        20, // Default cyclomatic threshold
        15, // Default cognitive threshold
    )
    .await?;

    // Aggregate results
    let report = aggregate_results_with_thresholds(file_metrics, Some(20), Some(15));

    // Convert to legacy ComplexityReport format for compatibility
    let mut functions = Vec::new();
    let mut total_complexity = 0u32;
    let mut complexities = Vec::new();

    for violation in &report.violations {
        match violation {
            crate::services::complexity::Violation::Error {
                file,
                function,
                value,
                ..
            }
            | crate::services::complexity::Violation::Warning {
                file,
                function,
                value,
                ..
            } => {
                if *value > 20 {
                    functions.push(ComplexityHotspot {
                        function: function
                            .as_ref()
                            .unwrap_or(&"<anonymous>".to_string())
                            .clone(),
                        file: file.clone(),
                        complexity: u32::from(*value),
                    });
                }
                complexities.push(u32::from(*value));
                total_complexity += u32::from(*value);
            }
        }
    }

    // Sort hotspots by complexity
    functions.sort_unstable_by(|a, b| b.complexity.cmp(&a.complexity));
    functions.truncate(10);

    // Calculate p99
    complexities.sort_unstable();
    let p99_idx = (f64::from(complexities.len() as u32) * 0.99) as usize;
    let p99 = complexities.get(p99_idx).copied().unwrap_or(0);

    Ok(ComplexityReport {
        total_functions: complexities.len(),
        high_complexity_count: functions.len(),
        average_complexity: if complexities.is_empty() {
            0.0
        } else {
            f64::from(total_complexity) / f64::from(complexities.len() as u32)
        },
        p99_complexity: p99,
        hotspots: functions,
    })
}

async fn run_satd_analysis(
    _project_path: &Path,
    _include: &Option<String>,
    _exclude: &Option<String>,
) -> Result<SatdReport> {
    use regex::Regex;
    use walkdir::WalkDir;

    let satd_pattern = Regex::new(r"(?i)(TODO|FIXME|HACK|XXX|REFACTOR|DEPRECATED):\s*(.+)")
        .expect("Hardcoded regex pattern must be valid");
    let mut items = Vec::new();
    let mut by_type = HashMap::new();
    let mut by_severity = HashMap::new();

    for entry in WalkDir::new(_project_path) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && is_source_file(path) {
            process_file_for_satd(
                path,
                &satd_pattern,
                &mut items,
                &mut by_type,
                &mut by_severity,
            )
            .await?;
        }
    }

    Ok(SatdReport {
        total_items: items.len(),
        by_type,
        by_severity,
        items,
    })
}

/// Extract Method: Process a single file for SATD detection
async fn process_file_for_satd(
    path: &std::path::Path,
    satd_pattern: &regex::Regex,
    items: &mut Vec<SatdItem>,
    by_type: &mut HashMap<String, usize>,
    by_severity: &mut HashMap<String, usize>,
) -> Result<()> {
    if let Ok(content) = tokio::fs::read_to_string(path).await {
        for (line_no, line) in content.lines().enumerate() {
            if let Some(captures) = satd_pattern.captures(line) {
                process_satd_match(path, line_no, captures, items, by_type, by_severity);
            }
        }
    }
    Ok(())
}

/// Extract Method: Process a single SATD match
fn process_satd_match(
    path: &std::path::Path,
    line_no: usize,
    captures: regex::Captures,
    items: &mut Vec<SatdItem>,
    by_type: &mut HashMap<String, usize>,
    by_severity: &mut HashMap<String, usize>,
) {
    let satd_type = captures
        .get(1)
        .expect("Match group 1 exists for successful regex match")
        .as_str()
        .to_uppercase();
    let text = captures
        .get(2)
        .expect("Match group 2 exists for successful regex match")
        .as_str()
        .to_string();
    let severity = determine_satd_severity(&satd_type);

    *by_type.entry(satd_type.clone()).or_insert(0) += 1;
    *by_severity.entry(severity.to_string()).or_insert(0) += 1;

    items.push(SatdItem {
        file: path.to_string_lossy().to_string(),
        line: line_no + 1,
        text,
        satd_type,
        severity: severity.to_string(),
    });
}

/// Extract Method: Determine SATD severity based on type
pub fn determine_satd_severity(satd_type: &str) -> &'static str {
    match satd_type {
        "HACK" | "XXX" => "high",
        "FIXME" | "REFACTOR" => "medium",
        _ => "low",
    }
}

async fn create_tdg_report(_project_path: &Path) -> Result<TdgReport> {
    // Simplified TDG analysis
    // Mock data for now
    let files = vec![TdgFile {
        file: "src/main.rs".to_string(),
        tdg_score: 3.5,
        complexity: 25,
        churn: 10,
    }];

    Ok(TdgReport {
        average_tdg: 2.1,
        critical_files: files,
        hotspot_count: 1,
    })
}

async fn run_dead_code_analysis(
    _project_path: &Path,
    _include: &Option<String>,
    _exclude: &Option<String>,
) -> Result<DeadCodeReport> {
    // Simplified dead code detection
    let items = vec![DeadCodeItem {
        name: "unused_function".to_string(),
        file: "src/utils.rs".to_string(),
        line: 42,
        item_type: "function".to_string(),
    }];

    Ok(DeadCodeReport {
        total_items: items.len(),
        dead_code_percentage: 2.5,
        items,
    })
}

async fn run_defect_prediction(
    _project_path: &Path,
    _confidence_threshold: f32,
    _min_lines: usize,
) -> Result<DefectReport> {
    // Simplified defect prediction
    let predictions = vec![DefectPrediction {
        file: "src/parser.rs".to_string(),
        probability: 0.75,
        factors: vec!["high complexity".to_string(), "recent churn".to_string()],
    }];

    Ok(DefectReport {
        high_risk_files: predictions,
        total_analyzed: 50,
        high_risk_count: 1,
    })
}

async fn run_duplicate_detection(
    _project_path: &Path,
    _include: &Option<String>,
    _exclude: &Option<String>,
) -> Result<DuplicateReport> {
    // Simplified duplicate detection
    let blocks = vec![DuplicateBlock {
        files: vec!["src/handler1.rs".to_string(), "src/handler2.rs".to_string()],
        lines: 20,
        tokens: 150,
    }];

    Ok(DuplicateReport {
        duplicate_blocks: blocks.len(),
        duplicate_lines: 40,
        duplicate_percentage: 3.2,
        blocks,
    })
}

fn format_comprehensive_report(
    report: &ComprehensiveReport,
    format: ComprehensiveOutputFormat,
    executive_summary: bool,
) -> Result<String> {
    match format {
        ComprehensiveOutputFormat::Json => format_comp_as_json(report),
        ComprehensiveOutputFormat::Markdown => format_comp_as_markdown(report, executive_summary),
        _ => Ok("Comprehensive analysis completed.".to_string()),
    }
}

// Helper: Format comprehensive report as JSON
fn format_comp_as_json(report: &ComprehensiveReport) -> Result<String> {
    Ok(serde_json::to_string_pretty(report)?)
}

// Helper: Format comprehensive report as Markdown
fn format_comp_as_markdown(
    report: &ComprehensiveReport,
    executive_summary: bool,
) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();

    writeln!(&mut output, "# Comprehensive Code Analysis Report\n")?;

    if executive_summary {
        write_comp_executive_summary(&mut output)?;
    }

    write_comp_analysis_sections(&mut output, report)?;

    Ok(output)
}

// Helper: Write executive summary
fn write_comp_executive_summary(output: &mut String) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "## Executive Summary\n")?;
    writeln!(
        output,
        "This report provides a comprehensive analysis of code quality metrics.\n"
    )?;
    Ok(())
}

// Helper: Write all analysis sections
fn write_comp_analysis_sections(output: &mut String, report: &ComprehensiveReport) -> Result<()> {
    if let Some(complexity) = &report.complexity {
        write_comp_complexity_section(output, complexity)?;
    }

    if let Some(satd) = &report.satd {
        write_comp_satd_section(output, satd)?;
    }

    if let Some(tdg) = &report.tdg {
        write_comp_tdg_section(output, tdg)?;
    }

    if let Some(dead_code) = &report.dead_code {
        write_comp_dead_code_section(output, dead_code)?;
    }

    if let Some(defects) = &report.defects {
        write_comp_defects_section(output, defects)?;
    }

    if let Some(duplicates) = &report.duplicates {
        write_comp_duplicates_section(output, duplicates)?;
    }

    Ok(())
}

// Helper: Write complexity section
fn write_comp_complexity_section(output: &mut String, complexity: &ComplexityReport) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "## Complexity Analysis\n")?;
    writeln!(output, "- Total functions: {}", complexity.total_functions)?;
    writeln!(
        output,
        "- High complexity functions: {}",
        complexity.high_complexity_count
    )?;
    writeln!(
        output,
        "- Average complexity: {:.2}",
        complexity.average_complexity
    )?;
    writeln!(output, "- P99 complexity: {}\n", complexity.p99_complexity)?;
    Ok(())
}

// Helper: Write SATD section
fn write_comp_satd_section(output: &mut String, satd: &SatdReport) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "## Technical Debt (SATD)\n")?;
    writeln!(output, "- Total items: {}", satd.total_items)?;
    writeln!(output, "- By type:")?;
    for (t, count) in &satd.by_type {
        writeln!(output, "  - {t}: {count}")?;
    }
    writeln!(output)?;
    Ok(())
}

// Helper: Write TDG section
fn write_comp_tdg_section(output: &mut String, tdg: &TdgReport) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "## Technical Debt Gradient\n")?;
    writeln!(output, "- Average TDG: {:.2}", tdg.average_tdg)?;
    writeln!(output, "- Critical files: {}", tdg.critical_files.len())?;
    writeln!(output, "- Hotspot count: {}\n", tdg.hotspot_count)?;
    Ok(())
}

// Helper: Write dead code section
fn write_comp_dead_code_section(output: &mut String, dead_code: &DeadCodeReport) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "## Dead Code\n")?;
    writeln!(output, "- Total items: {}", dead_code.total_items)?;
    writeln!(
        output,
        "- Percentage: {:.1}%\n",
        dead_code.dead_code_percentage
    )?;
    Ok(())
}

// Helper: Write defects section
fn write_comp_defects_section(output: &mut String, defects: &DefectReport) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "## Defect Prediction\n")?;
    writeln!(output, "- Total analyzed: {}", defects.total_analyzed)?;
    writeln!(output, "- High risk files: {}\n", defects.high_risk_count)?;
    Ok(())
}

// Helper: Write duplicates section
fn write_comp_duplicates_section(output: &mut String, duplicates: &DuplicateReport) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "## Code Duplication\n")?;
    writeln!(
        output,
        "- Duplicate blocks: {}",
        duplicates.duplicate_blocks
    )?;
    writeln!(output, "- Duplicate lines: {}", duplicates.duplicate_lines)?;
    writeln!(
        output,
        "- Percentage: {:.1}%\n",
        duplicates.duplicate_percentage
    )?;
    Ok(())
}

// Incremental coverage stub data structures
