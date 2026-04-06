// Comprehensive and serve handlers - extracted for file health (CB-040)
// Split into submodules for maintainability (PMAT-503)

// Serve handlers (HTTP, WebSocket, SSE, hybrid, full server)
include!("comprehensive_serve.rs");

// Type definitions (reports, violations, quality gate results)
include!("comprehensive_types.rs");

// Analysis runner functions (complexity, SATD, TDG, dead code, defects, duplicates)
include!("comprehensive_runners.rs");

// Report formatting (JSON, Markdown, section writers)
include!("comprehensive_formatting.rs");

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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
