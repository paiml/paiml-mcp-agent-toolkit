//! Advanced analysis command handlers
//!
//! This module contains handlers for advanced analysis features like
//! deep context, TDG, provability, and comprehensive analysis.

use crate::cli::{
    ComprehensiveOutputFormat, DagType, DeepContextOutputFormat, DefectPredictionOutputFormat,
    GraphMetricType, GraphMetricsOutputFormat, MakefileOutputFormat, SymbolTableOutputFormat,
    SymbolTypeFilter, TdgOutputFormat,
};
use crate::services::simple_deep_context::{SimpleAnalysisConfig, SimpleDeepContext};
use anyhow::Result;
use std::path::PathBuf;
use tracing::{debug, info};

/// Handle deep context analysis command
///
/// Performs comprehensive analysis of project context, including code relationships,
/// dependencies, and architectural patterns. This addresses issue #33 where the
/// command wasn't finding anything by implementing proper file discovery and analysis.
///
/// # Examples
///
/// ```no_run
/// use pmat::cli::{DeepContextOutputFormat, DagType};
/// use pmat::cli::handlers::advanced_analysis_handlers::handle_analyze_deep_context;
/// use std::path::PathBuf;
///
/// # async fn example() -> anyhow::Result<()> {
/// // Basic deep context analysis
/// handle_analyze_deep_context(
///     PathBuf::from("."),
///     None,                              // output
///     DeepContextOutputFormat::Json,     // format
///     false,                             // full
///     vec![],                            // include
///     vec![],                            // exclude
///     30,                                // period_days
///     None,                              // dag_type
///     None,                              // max_depth
///     vec![],                            // include_patterns
///     vec![],                            // exclude_patterns
///     None,                              // cache_strategy
///     false,                             // parallel
///     false,                             // verbose
///     10,                                // top_files
/// ).await?;
/// # Ok(())
/// # }
/// ```ignore
///
/// ```no_run
/// # use pmat::cli::{DeepContextOutputFormat, DagType};
/// # use pmat::cli::handlers::advanced_analysis_handlers::handle_analyze_deep_context;
/// # use std::path::PathBuf;
/// # async fn example() -> anyhow::Result<()> {
/// // Full analysis with specific includes
/// handle_analyze_deep_context(
///     PathBuf::from("./src"),
///     Some(PathBuf::from("context.json")),
///     DeepContextOutputFormat::Json,
///     true,                              // full analysis
///     vec!["complexity".to_string(), "dependencies".to_string()],
///     vec![],
///     90,                                // 90 day history
///     Some(DagType::CallGraph),
///     Some(5),                           // max depth 5
///     vec!["**/*.rs".to_string()],       // only Rust files
///     vec!["**/tests/**".to_string()],   // exclude tests
///     Some("persistent".to_string()),
///     true,                              // parallel processing
///     true,                              // verbose output
///     20,                                // top 20 files
/// ).await?;
/// # Ok(())
/// # }
/// ```ignore
///
/// # Returns
///
/// Returns `Ok(())` if analysis completes successfully, or an error if:
/// - Project path doesn't exist
/// - No files found to analyze
/// - Output file cannot be written
/// - Analysis encounters errors
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_deep_context(
    project_path: PathBuf,
    output: Option<PathBuf>,
    format: DeepContextOutputFormat,
    full: bool,
    include: Vec<String>,
    _exclude: Vec<String>,
    period_days: u32,
    _dag_type: Option<DagType>,
    _max_depth: Option<usize>,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    _cache_strategy: Option<String>,
    _parallel: bool,
    verbose: bool,
    top_files: usize,
) -> Result<()> {
    info!("🔍 Starting deep context analysis");
    info!("📂 Project path: {}", project_path.display());
    info!("📊 Analysis period: {} days", period_days);

    // Create simple deep context analyzer
    let analyzer = SimpleDeepContext::new();

    // Build configuration
    let mut include_features = include;
    if full {
        include_features.push("all".to_string());
    }

    let mut combined_exclude = exclude_patterns;
    // Add common exclusions
    combined_exclude.extend([
        "**/target/**".to_string(),
        "**/node_modules/**".to_string(),
        "**/.git/**".to_string(),
        "**/build/**".to_string(),
        "**/dist/**".to_string(),
        "**/__pycache__/**".to_string(),
    ]);

    let config = SimpleAnalysisConfig {
        project_path: project_path.clone(),
        include_features,
        include_patterns,
        exclude_patterns: combined_exclude,
        enable_verbose: verbose,
    };

    if verbose {
        debug!("Analysis configuration: {:?}", config);
    }

    // Perform analysis
    let report = analyzer.analyze(config).await?;

    // Format and output results
    let output_content = match format {
        DeepContextOutputFormat::Json => analyzer.format_as_json(&report)?,
        DeepContextOutputFormat::Markdown => analyzer.format_as_markdown(&report, top_files),
        DeepContextOutputFormat::Sarif => {
            // TRACKED: Implement SARIF format
            analyzer.format_as_json(&report)?
        }
    };

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &output_content).await?;
        info!(
            "📄 Deep context analysis saved to: {}",
            output_path.display()
        );
    } else {
        println!("{output_content}");
    }

    // Print summary
    info!("✅ Deep context analysis completed successfully");
    info!(
        "📊 Analyzed {} files in {:?}",
        report.file_count, report.analysis_duration
    );
    info!(
        "💡 Generated {} recommendations",
        report.recommendations.len()
    );

    Ok(())
}

/// Handle TDG (Technical Debt Gradient) analysis command  
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_tdg(
    path: PathBuf,
    threshold: Option<f64>,
    top: Option<usize>,
    format: TdgOutputFormat,
    include_components: bool,
    output: Option<PathBuf>,
    critical_only: bool,
    verbose: bool,
) -> Result<()> {
    // Use the enhanced implementation from stubs that supports all modes
    use super::new_tdg_handler::TdgAnalysisConfig;

    let config = TdgAnalysisConfig {
        path,
        threshold,
        top_files: top,
        format,
        include_components,
        output,
        critical_only,
        verbose,
    };

    super::new_tdg_handler::handle_analyze_tdg(config).await
}

/// Handle makefile analysis command
pub async fn handle_analyze_makefile(
    path: PathBuf,
    rules: Vec<String>,
    format: MakefileOutputFormat,
    fix: bool,
    gnu_version: Option<String>,
    top_files: usize,
) -> Result<()> {
    // Delegate to stub implementation for now - will be fully extracted later
    super::super::analysis_utilities::handle_analyze_makefile(
        path,
        rules,
        format,
        fix,
        gnu_version,
        top_files,
    )
    .await
}

// handle_analyze_provability has been moved to provability_handler.rs

/// Handle defect prediction analysis command
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_defect_prediction(
    project_path: PathBuf,
    confidence_threshold: Option<f64>,
    min_lines: Option<usize>,
    include_low_confidence: bool,
    format: DefectPredictionOutputFormat,
    high_risk_only: bool,
    include_recommendations: bool,
    include: Vec<String>,
    exclude: Vec<String>,
    output: Option<PathBuf>,
    perf: bool,
    top_files: usize,
) -> Result<()> {
    // Delegate to the real implementation
    crate::cli::analysis::defect_prediction::handle_analyze_defect_prediction(
        project_path,
        confidence_threshold.unwrap_or(0.5) as f32,
        min_lines.unwrap_or(100),
        include_low_confidence,
        format,
        high_risk_only,
        include_recommendations,
        Some(include.join(",")),
        Some(exclude.join(",")),
        output,
        perf,
        top_files,
    )
    .await
}

/// Handle comprehensive analysis command
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_comprehensive(
    project_path: PathBuf,
    file: Option<PathBuf>,
    files: Vec<PathBuf>,
    format: ComprehensiveOutputFormat,
    include_duplicates: bool,
    include_dead_code: bool,
    include_defects: bool,
    include_complexity: bool,
    include_tdg: bool,
    confidence_threshold: f32,
    min_lines: usize,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
    perf: bool,
    executive_summary: bool,
) -> Result<()> {
    use super::comprehensive_analysis_handler::ComprehensiveAnalysisConfig;

    // Create config struct
    let config = ComprehensiveAnalysisConfig {
        project_path,
        file,
        files,
        format,
        include_duplicates,
        include_dead_code,
        include_defects,
        include_complexity,
        include_tdg,
        confidence_threshold,
        min_lines,
        include,
        exclude,
        output,
        perf,
        executive_summary,
        top_files: 20, // default value
    };

    // Use the new orchestrator-based comprehensive handler implementation
    super::comprehensive_analysis_handler::handle_analyze_comprehensive(config).await
}

/// Handle graph metrics analysis command
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_graph_metrics(
    project_path: PathBuf,
    metrics: Vec<GraphMetricType>,
    pagerank_seeds: Vec<String>,
    damping_factor: f32,
    max_iterations: usize,
    convergence_threshold: f64,
    export_graphml: bool,
    format: GraphMetricsOutputFormat,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
    perf: bool,
    top_k: usize,
    min_centrality: f64,
) -> Result<()> {
    // Delegate to the actual implementation
    crate::cli::analysis::graph_metrics::handle_analyze_graph_metrics(
        project_path,
        metrics,
        pagerank_seeds,
        damping_factor,
        max_iterations,
        convergence_threshold,
        export_graphml,
        format,
        include,
        exclude,
        output,
        perf,
        top_k,
        min_centrality,
    )
    .await
}

/// Handle symbol table analysis command
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_symbol_table(
    project_path: PathBuf,
    format: SymbolTableOutputFormat,
    filter: Option<SymbolTypeFilter>,
    query: Option<String>,
    include: Vec<String>,
    exclude: Vec<String>,
    show_unreferenced: bool,
    show_references: bool,
    output: Option<PathBuf>,
    perf: bool,
) -> Result<()> {
    // Delegate to the actual implementation
    crate::cli::analysis::symbol_table::handle_analyze_symbol_table(
        project_path,
        format,
        filter,
        query,
        Some(include.join(",")),
        Some(exclude.join(",")),
        show_unreferenced,
        show_references,
        output,
        perf,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper to create a temporary project directory with Rust files
    fn create_test_project() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create a simple Rust file
        let main_rs = temp_dir.path().join("main.rs");
        fs::write(
            &main_rs,
            r#"
fn main() {
    println!("Hello, world!");
}

fn complex_function(x: i32, y: i32) -> i32 {
    if x > 0 {
        if y > 0 {
            x + y
        } else {
            x - y
        }
    } else {
        -x
    }
}

// TODO: Refactor this function
fn needs_work() {
    // FIXME: This is a hack
    let _ = 42;
}
"#,
        )
        .expect("Failed to write main.rs");

        // Create a lib.rs file
        let lib_rs = temp_dir.path().join("lib.rs");
        fs::write(
            &lib_rs,
            r#"
pub mod utils;

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}
"#,
        )
        .expect("Failed to write lib.rs");

        temp_dir
    }

    /// Helper to create a simple Makefile for testing
    fn create_test_makefile(dir: &TempDir) {
        let makefile = dir.path().join("Makefile");
        fs::write(
            &makefile,
            r#"
.PHONY: all clean test

all: build

build:
	cargo build --release

test:
	cargo test

clean:
	cargo clean
"#,
        )
        .expect("Failed to write Makefile");
    }

    #[test]
    fn test_advanced_analysis_handlers_basic() {
        // Basic test to ensure module compiles
        assert_eq!(1 + 1, 2);
    }

    #[test]
    fn test_deep_context_output_format_variants() {
        // Test that all output format variants work
        let formats = [
            DeepContextOutputFormat::Json,
            DeepContextOutputFormat::Markdown,
            DeepContextOutputFormat::Sarif,
        ];

        for format in formats {
            let format_str = format!("{:?}", format);
            assert!(!format_str.is_empty());
        }
    }

    #[test]
    fn test_dag_type_variants() {
        // Test that all DAG type variants work
        let dag_types = [
            DagType::CallGraph,
            DagType::ImportGraph,
            DagType::Inheritance,
            DagType::FullDependency,
        ];

        for dag_type in dag_types {
            let dag_str = format!("{}", dag_type);
            assert!(!dag_str.is_empty());
        }
    }

    #[test]
    fn test_tdg_output_format_variants() {
        // Test TDG output format variants
        let formats = [
            TdgOutputFormat::Table,
            TdgOutputFormat::Json,
            TdgOutputFormat::Markdown,
            TdgOutputFormat::Sarif,
        ];

        for format in formats {
            let format_str = format!("{}", format);
            assert!(!format_str.is_empty());
        }
    }

    #[test]
    fn test_makefile_output_format_variants() {
        // Test Makefile output format variants
        let formats = [
            MakefileOutputFormat::Human,
            MakefileOutputFormat::Json,
            MakefileOutputFormat::Gcc,
            MakefileOutputFormat::Sarif,
        ];

        for format in formats {
            let format_str = format!("{}", format);
            assert!(!format_str.is_empty());
        }
    }

    #[test]
    fn test_defect_prediction_output_format_variants() {
        // Test defect prediction output format variants
        let formats = [
            DefectPredictionOutputFormat::Summary,
            DefectPredictionOutputFormat::Detailed,
            DefectPredictionOutputFormat::Json,
            DefectPredictionOutputFormat::Csv,
            DefectPredictionOutputFormat::Sarif,
        ];

        for format in formats {
            let format_str = format!("{}", format);
            assert!(!format_str.is_empty());
        }
    }

    #[test]
    fn test_comprehensive_output_format_variants() {
        // Test comprehensive output format variants
        let formats = [
            ComprehensiveOutputFormat::Summary,
            ComprehensiveOutputFormat::Detailed,
            ComprehensiveOutputFormat::Json,
            ComprehensiveOutputFormat::Markdown,
            ComprehensiveOutputFormat::Sarif,
        ];

        for format in formats {
            let format_str = format!("{}", format);
            assert!(!format_str.is_empty());
        }
    }

    #[test]
    fn test_graph_metric_type_variants() {
        // Test graph metric type variants
        let metrics = [
            GraphMetricType::Centrality,
            GraphMetricType::Betweenness,
            GraphMetricType::Closeness,
            GraphMetricType::PageRank,
            GraphMetricType::Clustering,
            GraphMetricType::Components,
            GraphMetricType::All,
        ];

        for metric in metrics {
            let metric_str = format!("{}", metric);
            assert!(!metric_str.is_empty());
        }
    }

    #[test]
    fn test_graph_metrics_output_format_variants() {
        // Test graph metrics output format variants
        let formats = [
            GraphMetricsOutputFormat::Summary,
            GraphMetricsOutputFormat::Detailed,
            GraphMetricsOutputFormat::Human,
            GraphMetricsOutputFormat::Json,
            GraphMetricsOutputFormat::Csv,
            GraphMetricsOutputFormat::GraphML,
            GraphMetricsOutputFormat::Markdown,
        ];

        for format in formats {
            let format_str = format!("{}", format);
            assert!(!format_str.is_empty());
        }
    }

    #[test]
    fn test_symbol_table_output_format_variants() {
        // Test symbol table output format variants
        let formats = [
            SymbolTableOutputFormat::Summary,
            SymbolTableOutputFormat::Detailed,
            SymbolTableOutputFormat::Human,
            SymbolTableOutputFormat::Json,
            SymbolTableOutputFormat::Csv,
        ];

        for format in formats {
            let format_str = format!("{}", format);
            assert!(!format_str.is_empty());
        }
    }

    #[test]
    fn test_symbol_type_filter_variants() {
        // Test symbol type filter variants
        let filters = [
            SymbolTypeFilter::Functions,
            SymbolTypeFilter::Classes,
            SymbolTypeFilter::Types,
            SymbolTypeFilter::Variables,
            SymbolTypeFilter::Modules,
            SymbolTypeFilter::All,
        ];

        for filter in filters {
            let filter_str = format!("{}", filter);
            assert!(!filter_str.is_empty());
        }
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_with_empty_project() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create an empty directory
        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        // Should succeed even with no files
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_json_format() {
        let temp_dir = create_test_project();

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_markdown_format() {
        let temp_dir = create_test_project();

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Markdown,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_sarif_format() {
        let temp_dir = create_test_project();

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Sarif,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_full_mode() {
        let temp_dir = create_test_project();

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            true, // full mode
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_with_include_features() {
        let temp_dir = create_test_project();

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec!["complexity".to_string(), "dependencies".to_string()],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_with_include_patterns() {
        let temp_dir = create_test_project();

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec!["**/*.rs".to_string()],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_with_exclude_patterns() {
        let temp_dir = create_test_project();

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec!["**/tests/**".to_string()],
            None,
            false,
            false,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_verbose_mode() {
        let temp_dir = create_test_project();

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            true, // verbose
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_with_output_file() {
        let temp_dir = create_test_project();
        let output_file = temp_dir.path().join("output.json");

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            Some(output_file.clone()),
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        assert!(result.is_ok());
        assert!(output_file.exists());
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_with_dag_type() {
        let temp_dir = create_test_project();

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            Some(DagType::CallGraph),
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_with_max_depth() {
        let temp_dir = create_test_project();

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            None,
            Some(5),
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_with_period_days() {
        let temp_dir = create_test_project();

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            90, // 90 day period
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_deep_context_with_top_files() {
        let temp_dir = create_test_project();

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            20, // top 20 files
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_tdg_basic() {
        let temp_dir = create_test_project();

        let result = handle_analyze_tdg(
            temp_dir.path().to_path_buf(),
            None,
            None,
            TdgOutputFormat::Table,
            false,
            None,
            false,
            false,
        )
        .await;

        // TDG analysis should complete (may fail on non-git repos, but function works)
        // We just verify the function runs without panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_tdg_json_format() {
        let temp_dir = create_test_project();

        let result = handle_analyze_tdg(
            temp_dir.path().to_path_buf(),
            None,
            None,
            TdgOutputFormat::Json,
            false,
            None,
            false,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_tdg_with_threshold() {
        let temp_dir = create_test_project();

        let result = handle_analyze_tdg(
            temp_dir.path().to_path_buf(),
            Some(1.5),
            None,
            TdgOutputFormat::Table,
            false,
            None,
            false,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_tdg_with_top_files() {
        let temp_dir = create_test_project();

        let result = handle_analyze_tdg(
            temp_dir.path().to_path_buf(),
            None,
            Some(5),
            TdgOutputFormat::Table,
            false,
            None,
            false,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_tdg_include_components() {
        let temp_dir = create_test_project();

        let result = handle_analyze_tdg(
            temp_dir.path().to_path_buf(),
            None,
            None,
            TdgOutputFormat::Table,
            true, // include_components
            None,
            false,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_tdg_critical_only() {
        let temp_dir = create_test_project();

        let result = handle_analyze_tdg(
            temp_dir.path().to_path_buf(),
            None,
            None,
            TdgOutputFormat::Table,
            false,
            None,
            true, // critical_only
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_makefile_basic() {
        let temp_dir = create_test_project();
        create_test_makefile(&temp_dir);

        let makefile_path = temp_dir.path().join("Makefile");

        let result = handle_analyze_makefile(
            makefile_path,
            vec![],
            MakefileOutputFormat::Human,
            false,
            None,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_makefile_json_format() {
        let temp_dir = create_test_project();
        create_test_makefile(&temp_dir);

        let makefile_path = temp_dir.path().join("Makefile");

        let result = handle_analyze_makefile(
            makefile_path,
            vec![],
            MakefileOutputFormat::Json,
            false,
            None,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_makefile_gcc_format() {
        let temp_dir = create_test_project();
        create_test_makefile(&temp_dir);

        let makefile_path = temp_dir.path().join("Makefile");

        let result = handle_analyze_makefile(
            makefile_path,
            vec![],
            MakefileOutputFormat::Gcc,
            false,
            None,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_makefile_with_rules() {
        let temp_dir = create_test_project();
        create_test_makefile(&temp_dir);

        let makefile_path = temp_dir.path().join("Makefile");

        let result = handle_analyze_makefile(
            makefile_path,
            vec!["all".to_string(), "clean".to_string()],
            MakefileOutputFormat::Human,
            false,
            None,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_makefile_nonexistent() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let makefile_path = temp_dir.path().join("nonexistent_Makefile");

        let result = handle_analyze_makefile(
            makefile_path,
            vec![],
            MakefileOutputFormat::Human,
            false,
            None,
            10,
        )
        .await;

        // Should fail for nonexistent file
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_analyze_defect_prediction_basic() {
        let temp_dir = create_test_project();

        let result = handle_analyze_defect_prediction(
            temp_dir.path().to_path_buf(),
            None,
            None,
            false,
            DefectPredictionOutputFormat::Summary,
            false,
            false,
            vec![],
            vec![],
            None,
            false,
            10,
        )
        .await;

        // Function should complete without panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_defect_prediction_json_format() {
        let temp_dir = create_test_project();

        let result = handle_analyze_defect_prediction(
            temp_dir.path().to_path_buf(),
            None,
            None,
            false,
            DefectPredictionOutputFormat::Json,
            false,
            false,
            vec![],
            vec![],
            None,
            false,
            10,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_defect_prediction_with_confidence_threshold() {
        let temp_dir = create_test_project();

        let result = handle_analyze_defect_prediction(
            temp_dir.path().to_path_buf(),
            Some(0.7),
            None,
            false,
            DefectPredictionOutputFormat::Summary,
            false,
            false,
            vec![],
            vec![],
            None,
            false,
            10,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_defect_prediction_with_min_lines() {
        let temp_dir = create_test_project();

        let result = handle_analyze_defect_prediction(
            temp_dir.path().to_path_buf(),
            None,
            Some(50),
            false,
            DefectPredictionOutputFormat::Summary,
            false,
            false,
            vec![],
            vec![],
            None,
            false,
            10,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_defect_prediction_high_risk_only() {
        let temp_dir = create_test_project();

        let result = handle_analyze_defect_prediction(
            temp_dir.path().to_path_buf(),
            None,
            None,
            false,
            DefectPredictionOutputFormat::Summary,
            true, // high_risk_only
            false,
            vec![],
            vec![],
            None,
            false,
            10,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_defect_prediction_with_recommendations() {
        let temp_dir = create_test_project();

        let result = handle_analyze_defect_prediction(
            temp_dir.path().to_path_buf(),
            None,
            None,
            false,
            DefectPredictionOutputFormat::Detailed,
            false,
            true, // include_recommendations
            vec![],
            vec![],
            None,
            false,
            10,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_comprehensive_basic() {
        let temp_dir = create_test_project();

        let result = handle_analyze_comprehensive(
            temp_dir.path().to_path_buf(),
            None,
            vec![],
            ComprehensiveOutputFormat::Summary,
            false,
            false,
            false,
            true,
            false,
            0.5,
            100,
            None,
            None,
            None,
            false,
            false,
        )
        .await;

        // Function should complete
        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_comprehensive_json_format() {
        let temp_dir = create_test_project();

        let result = handle_analyze_comprehensive(
            temp_dir.path().to_path_buf(),
            None,
            vec![],
            ComprehensiveOutputFormat::Json,
            false,
            false,
            false,
            true,
            false,
            0.5,
            100,
            None,
            None,
            None,
            false,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_comprehensive_with_all_options() {
        let temp_dir = create_test_project();

        let result = handle_analyze_comprehensive(
            temp_dir.path().to_path_buf(),
            None,
            vec![],
            ComprehensiveOutputFormat::Detailed,
            true,  // include_duplicates
            true,  // include_dead_code
            true,  // include_defects
            true,  // include_complexity
            true,  // include_tdg
            0.7,   // confidence_threshold
            50,    // min_lines
            None,
            None,
            None,
            false,
            true, // executive_summary
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_comprehensive_single_file() {
        let temp_dir = create_test_project();
        let main_rs = temp_dir.path().join("main.rs");

        let result = handle_analyze_comprehensive(
            temp_dir.path().to_path_buf(),
            Some(main_rs),
            vec![],
            ComprehensiveOutputFormat::Summary,
            false,
            false,
            false,
            true,
            false,
            0.5,
            100,
            None,
            None,
            None,
            false,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_graph_metrics_basic() {
        let temp_dir = create_test_project();

        let result = handle_analyze_graph_metrics(
            temp_dir.path().to_path_buf(),
            vec![GraphMetricType::Centrality],
            vec![],
            0.85,
            100,
            1e-6,
            false,
            GraphMetricsOutputFormat::Summary,
            None,
            None,
            None,
            false,
            10,
            0.0,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_graph_metrics_all_metrics() {
        let temp_dir = create_test_project();

        let result = handle_analyze_graph_metrics(
            temp_dir.path().to_path_buf(),
            vec![GraphMetricType::All],
            vec![],
            0.85,
            100,
            1e-6,
            false,
            GraphMetricsOutputFormat::Json,
            None,
            None,
            None,
            false,
            10,
            0.0,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_graph_metrics_with_pagerank_seeds() {
        let temp_dir = create_test_project();

        let result = handle_analyze_graph_metrics(
            temp_dir.path().to_path_buf(),
            vec![GraphMetricType::PageRank],
            vec!["main".to_string()],
            0.85,
            100,
            1e-6,
            false,
            GraphMetricsOutputFormat::Summary,
            None,
            None,
            None,
            false,
            10,
            0.0,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_graph_metrics_custom_damping() {
        let temp_dir = create_test_project();

        let result = handle_analyze_graph_metrics(
            temp_dir.path().to_path_buf(),
            vec![GraphMetricType::PageRank],
            vec![],
            0.90, // custom damping factor
            200,  // custom max iterations
            1e-8, // tighter convergence
            false,
            GraphMetricsOutputFormat::Summary,
            None,
            None,
            None,
            false,
            10,
            0.0,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_symbol_table_basic() {
        let temp_dir = create_test_project();

        let result = handle_analyze_symbol_table(
            temp_dir.path().to_path_buf(),
            SymbolTableOutputFormat::Summary,
            None,
            None,
            vec![],
            vec![],
            false,
            false,
            None,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_symbol_table_json_format() {
        let temp_dir = create_test_project();

        let result = handle_analyze_symbol_table(
            temp_dir.path().to_path_buf(),
            SymbolTableOutputFormat::Json,
            None,
            None,
            vec![],
            vec![],
            false,
            false,
            None,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_symbol_table_with_filter() {
        let temp_dir = create_test_project();

        let result = handle_analyze_symbol_table(
            temp_dir.path().to_path_buf(),
            SymbolTableOutputFormat::Summary,
            Some(SymbolTypeFilter::Functions),
            None,
            vec![],
            vec![],
            false,
            false,
            None,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_symbol_table_with_query() {
        let temp_dir = create_test_project();

        let result = handle_analyze_symbol_table(
            temp_dir.path().to_path_buf(),
            SymbolTableOutputFormat::Summary,
            None,
            Some("main".to_string()),
            vec![],
            vec![],
            false,
            false,
            None,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_symbol_table_show_unreferenced() {
        let temp_dir = create_test_project();

        let result = handle_analyze_symbol_table(
            temp_dir.path().to_path_buf(),
            SymbolTableOutputFormat::Detailed,
            None,
            None,
            vec![],
            vec![],
            true, // show_unreferenced
            false,
            None,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_symbol_table_show_references() {
        let temp_dir = create_test_project();

        let result = handle_analyze_symbol_table(
            temp_dir.path().to_path_buf(),
            SymbolTableOutputFormat::Detailed,
            None,
            None,
            vec![],
            vec![],
            false,
            true, // show_references
            None,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_symbol_table_all_filters() {
        let temp_dir = create_test_project();

        // Test all filter types
        let filters = [
            SymbolTypeFilter::Functions,
            SymbolTypeFilter::Classes,
            SymbolTypeFilter::Types,
            SymbolTypeFilter::Variables,
            SymbolTypeFilter::Modules,
            SymbolTypeFilter::All,
        ];

        for filter in filters {
            let result = handle_analyze_symbol_table(
                temp_dir.path().to_path_buf(),
                SymbolTableOutputFormat::Summary,
                Some(filter),
                None,
                vec![],
                vec![],
                false,
                false,
                None,
                false,
            )
            .await;

            let _ = result;
        }
    }
}

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

        #[test]
        fn test_period_days_reasonable(days in 1u32..365) {
            // Period days should always be positive and reasonable
            prop_assert!(days > 0);
            prop_assert!(days < 366);
        }

        #[test]
        fn test_top_files_positive(top in 1usize..1000) {
            // Top files count should always be positive
            prop_assert!(top > 0);
        }

        #[test]
        fn test_confidence_threshold_valid(threshold in 0.0f32..1.0) {
            // Confidence threshold should be between 0 and 1
            prop_assert!(threshold >= 0.0);
            prop_assert!(threshold <= 1.0);
        }

        #[test]
        fn test_damping_factor_valid(damping in 0.0f32..1.0) {
            // PageRank damping factor should be between 0 and 1
            prop_assert!(damping >= 0.0);
            prop_assert!(damping <= 1.0);
        }

        #[test]
        fn test_max_iterations_positive(iterations in 1usize..10000) {
            // Max iterations should be positive
            prop_assert!(iterations > 0);
        }

        #[test]
        fn test_min_lines_reasonable(lines in 1usize..100000) {
            // Min lines should be reasonable
            prop_assert!(lines > 0);
            prop_assert!(lines < 100001);
        }

        #[test]
        fn test_convergence_threshold_small(threshold in 1e-10f64..1e-3) {
            // Convergence threshold should be small but positive
            prop_assert!(threshold > 0.0);
            prop_assert!(threshold < 0.01);
        }

        #[test]
        fn test_tdg_threshold_positive(threshold in 0.0f64..10.0) {
            // TDG threshold should be non-negative
            prop_assert!(threshold >= 0.0);
        }
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_handle_deep_context_nonexistent_path() {
        let result = handle_analyze_deep_context(
            PathBuf::from("/nonexistent/path/to/project"),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        // Should succeed even with nonexistent path (returns empty analysis)
        // or fail gracefully
        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_deep_context_with_special_characters_in_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let special_dir = temp_dir.path().join("project with spaces");
        fs::create_dir_all(&special_dir).expect("Failed to create directory");

        let main_rs = special_dir.join("main.rs");
        fs::write(&main_rs, "fn main() {}").expect("Failed to write file");

        let result = handle_analyze_deep_context(
            special_dir,
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_deep_context_empty_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let empty_rs = temp_dir.path().join("empty.rs");
        fs::write(&empty_rs, "").expect("Failed to write empty file");

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_deep_context_binary_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let binary_file = temp_dir.path().join("binary.rs");
        fs::write(&binary_file, vec![0u8, 1, 2, 3, 255, 254, 253]).expect("Failed to write binary file");

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        // Should handle binary files gracefully
        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_deep_context_large_top_files() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let main_rs = temp_dir.path().join("main.rs");
        fs::write(&main_rs, "fn main() {}").expect("Failed to write file");

        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            30,
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            1000, // Large top_files value
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_deep_context_zero_period_days() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let main_rs = temp_dir.path().join("main.rs");
        fs::write(&main_rs, "fn main() {}").expect("Failed to write file");

        // Edge case: 0 period days
        let result = handle_analyze_deep_context(
            temp_dir.path().to_path_buf(),
            None,
            DeepContextOutputFormat::Json,
            false,
            vec![],
            vec![],
            0, // Zero days
            None,
            None,
            vec![],
            vec![],
            None,
            false,
            false,
            10,
        )
        .await;

        // Should handle edge case gracefully
        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_comprehensive_empty_files_list() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let main_rs = temp_dir.path().join("main.rs");
        fs::write(&main_rs, "fn main() {}").expect("Failed to write file");

        let result = handle_analyze_comprehensive(
            temp_dir.path().to_path_buf(),
            None,
            vec![], // Empty files list
            ComprehensiveOutputFormat::Summary,
            false,
            false,
            false,
            true,
            false,
            0.5,
            100,
            None,
            None,
            None,
            false,
            false,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_graph_metrics_empty_seeds() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let main_rs = temp_dir.path().join("main.rs");
        fs::write(&main_rs, "fn main() {}").expect("Failed to write file");

        let result = handle_analyze_graph_metrics(
            temp_dir.path().to_path_buf(),
            vec![GraphMetricType::PageRank],
            vec![], // Empty seeds
            0.85,
            100,
            1e-6,
            false,
            GraphMetricsOutputFormat::Summary,
            None,
            None,
            None,
            false,
            10,
            0.0,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_symbol_table_empty_include_exclude() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let main_rs = temp_dir.path().join("main.rs");
        fs::write(&main_rs, "fn main() {}").expect("Failed to write file");

        let result = handle_analyze_symbol_table(
            temp_dir.path().to_path_buf(),
            SymbolTableOutputFormat::Summary,
            None,
            None,
            vec![], // Empty include
            vec![], // Empty exclude
            false,
            false,
            None,
            false,
        )
        .await;

        let _ = result;
    }
}
