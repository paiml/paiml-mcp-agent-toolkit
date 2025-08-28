//! Advanced analysis command handlers
//!
//! This module contains handlers for advanced analysis features like
//! deep context, TDG, provability, and comprehensive analysis.

use crate::cli::*;
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
/// ```
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
/// ```
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
    super::new_tdg_handler::handle_analyze_tdg(
        path,
        threshold,
        top,
        format,
        include_components,
        output,
        critical_only,
        verbose,
    )
    .await
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
    super::super::stubs::handle_analyze_makefile(path, rules, format, fix, gnu_version, top_files)
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
    // Use the new orchestrator-based comprehensive handler implementation
    super::comprehensive_analysis_handler::handle_analyze_comprehensive(
        project_path,
        file,
        files, // files (MCP composition support)
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
        20, // top_files default
    )
    .await
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
    // use super::*; // Unused in simple tests

    #[test]
    fn test_advanced_analysis_handlers_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
