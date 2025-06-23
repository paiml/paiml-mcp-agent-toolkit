//! CLI module for the paiml-mcp-agent-toolkit
//!
//! This module implements the command-line interface using a modular architecture
//! to reduce complexity and improve testability.

pub mod analysis;
pub mod analysis_helpers;
pub mod args;
pub mod command_dispatcher;
pub mod command_structure;
pub mod commands;
pub mod coverage_helpers;
pub mod defect_helpers;
pub mod defect_prediction_helpers;
pub mod diagnose;
pub mod enums;
pub mod formatting_helpers;
pub mod handlers;
pub mod name_similarity_helpers;
pub mod proof_annotation_formatter;
pub mod proof_annotation_helpers;
pub mod provability_helpers;
pub mod stubs;
pub mod stubs_refactored;
pub mod symbol_table_helpers;
pub mod tdg_helpers;

// Re-export commonly used types from submodules
pub use commands::{AnalyzeCommands, Cli, Commands, Mode, RefactorCommands};
pub use enums::*;

// Type definitions for handler compatibility
#[derive(Debug, Clone)]
pub struct NameInfo {
    pub name: String,
    pub kind: String,
    pub file_path: PathBuf,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct NameSimilarityResult {
    pub name: String,
    pub kind: String,
    pub file_path: PathBuf,
    pub line: usize,
    pub similarity: f32,
    pub phonetic_match: bool,
    pub fuzzy_match: bool,
}

#[derive(Debug, Clone)]
pub struct DuplicateHandlerConfig {
    pub project_path: PathBuf,
    pub detection_type: DuplicateType,
    pub threshold: f32,
    pub min_lines: usize,
    pub max_tokens: usize,
    pub format: DuplicateOutputFormat,
    pub perf: bool,
    pub include: Option<String>,
    pub exclude: Option<String>,
    pub output: Option<PathBuf>,
}

use crate::stateless_server::StatelessTemplateServer;
use clap::Parser;
use command_dispatcher::CommandDispatcher;
use std::sync::Arc;
use tracing::{debug, info};

/// Early CLI args struct for tracing initialization
#[derive(Debug, Clone)]
pub struct EarlyCliArgs {
    pub verbose: bool,
    pub debug: bool,
    pub trace: bool,
    pub trace_filter: Option<String>,
}

/// Parse CLI early to extract tracing configuration
pub fn parse_early_for_tracing() -> EarlyCliArgs {
    let args: Vec<String> = std::env::args().collect();

    let verbose = args.iter().any(|arg| arg == "-v" || arg == "--verbose");
    let debug = args.iter().any(|arg| arg == "--debug");
    let trace = args.iter().any(|arg| arg == "--trace");

    let trace_filter = args
        .iter()
        .position(|arg| arg == "--trace-filter")
        .and_then(|pos| args.get(pos + 1))
        .cloned()
        .or_else(|| std::env::var("RUST_LOG").ok());

    EarlyCliArgs {
        verbose,
        debug,
        trace,
        trace_filter,
    }
}

pub async fn run(server: Arc<StatelessTemplateServer>) -> anyhow::Result<()> {
    let cli = Cli::parse();
    debug!("CLI arguments parsed");

    // Handle forced mode
    if let Some(commands::Mode::Mcp) = cli.mode {
        info!("Forced MCP mode detected");
        return crate::run_mcp_server(server).await;
    }

    // Use command dispatcher for improved modularity
    CommandDispatcher::execute_command(cli.command, server).await
}

// Helper functions made public for testing

use std::path::Path;

pub fn detect_primary_language(path: &Path) -> Option<String> {
    use walkdir::WalkDir;

    // First check for project marker files
    if path.join("Cargo.toml").exists() {
        return Some("rust".to_string());
    }
    if path.join("package.json").exists() {
        // Could be Node.js or Deno - check for deno.json/deno.jsonc
        if path.join("deno.json").exists() || path.join("deno.jsonc").exists() {
            return Some("deno".to_string());
        }
        return Some("deno".to_string()); // Default TypeScript/JS to deno for now
    }
    if path.join("pyproject.toml").exists() || path.join("setup.py").exists() {
        return Some("python-uv".to_string());
    }
    if path.join("build.gradle").exists() || path.join("build.gradle.kts").exists() {
        return Some("kotlin".to_string());
    }

    // Fall back to counting file extensions
    let mut lang_counts = std::collections::HashMap::new();

    for entry in WalkDir::new(path).max_depth(3).into_iter().flatten() {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                match ext.to_str() {
                    Some("rs") => *lang_counts.entry("rust").or_insert(0) += 1,
                    Some("ts") | Some("tsx") | Some("js") | Some("jsx") => {
                        *lang_counts.entry("deno").or_insert(0) += 1
                    }
                    Some("py") => *lang_counts.entry("python-uv").or_insert(0) += 1,
                    Some("kt") | Some("kts") => *lang_counts.entry("kotlin").or_insert(0) += 1,
                    _ => {}
                }
            }
        }
    }

    lang_counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(lang, _)| lang.to_string())
}

pub fn apply_satd_filters(
    items: Vec<crate::models::tdg::SatdItem>,
    severity: Option<SatdSeverity>,
    critical_only: bool,
) -> Vec<crate::models::tdg::SatdItem> {
    items
        .into_iter()
        .filter(|item| {
            if critical_only {
                matches!(item.severity, crate::models::tdg::SatdSeverity::Critical)
            } else if let Some(min_severity) = &severity {
                // Convert to comparable severity levels
                let item_level = match item.severity {
                    crate::models::tdg::SatdSeverity::Low => 1,
                    crate::models::tdg::SatdSeverity::Medium => 2,
                    crate::models::tdg::SatdSeverity::High => 3,
                    crate::models::tdg::SatdSeverity::Critical => 4,
                };
                let min_level = match min_severity {
                    SatdSeverity::Low => 1,
                    SatdSeverity::Medium => 2,
                    SatdSeverity::High => 3,
                    SatdSeverity::Critical => 4,
                };
                item_level >= min_level
            } else {
                true
            }
        })
        .collect()
}

// Deep context config helpers

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DeepContextConfigParams {
    pub project_path: PathBuf,
    pub output: Option<PathBuf>,
    pub format: DeepContextOutputFormat,
    pub full: bool,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub period_days: u32,
    pub dag_type: DeepContextDagType,
    pub max_depth: Option<usize>,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub cache_strategy: DeepContextCacheStrategy,
    pub parallel: Option<usize>,
    pub verbose: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn build_deep_context_config(
    _project_path: PathBuf,
    _output: Option<PathBuf>,
    _format: DeepContextOutputFormat,
    _full: bool,
    _include: Vec<String>,
    _exclude: Vec<String>,
    _period_days: u32,
    _dag_type: DeepContextDagType,
    _max_depth: Option<usize>,
    _include_patterns: Vec<String>,
    _exclude_patterns: Vec<String>,
    _cache_strategy: DeepContextCacheStrategy,
    _parallel: Option<usize>,
    _verbose: bool,
) -> anyhow::Result<crate::models::deep_context_config::DeepContextConfig> {
    // Return a default DeepContextConfig for now since the structure doesn't match our needs
    Ok(crate::models::deep_context_config::DeepContextConfig {
        entry_points: vec![],
        dead_code_threshold: 0.15,
        complexity_thresholds: Default::default(),
        include_tests: false,
        include_benches: false,
        cross_language_detection: true,
    })
}

pub fn convert_dag_type(dag_type: DeepContextDagType) -> crate::models::dag::DagType {
    match dag_type {
        DeepContextDagType::CallGraph => crate::models::dag::DagType::CallGraph,
        DeepContextDagType::ImportGraph => crate::models::dag::DagType::ImportGraph,
        DeepContextDagType::Inheritance => crate::models::dag::DagType::Inheritance,
        DeepContextDagType::FullDependency => crate::models::dag::DagType::FullDependency,
    }
}

pub fn convert_cache_strategy(strategy: DeepContextCacheStrategy) -> DeepContextCacheStrategy {
    // Just return the same strategy for now - proper cache strategy conversion
    // would need to be implemented based on the actual cache system
    strategy
}

pub fn parse_analysis_filters(
    include: Vec<String>,
    exclude: Vec<String>,
) -> anyhow::Result<(Vec<AnalysisType>, Vec<AnalysisType>)> {
    let include_analysis = include
        .into_iter()
        .map(|s| parse_analysis_type(&s))
        .collect::<Result<Vec<_>, _>>()?;

    let exclude_analysis = exclude
        .into_iter()
        .map(|s| parse_analysis_type(&s))
        .collect::<Result<Vec<_>, _>>()?;

    Ok((include_analysis, exclude_analysis))
}

pub fn parse_analysis_type(s: &str) -> anyhow::Result<AnalysisType> {
    match s.to_lowercase().as_str() {
        "complexity" => Ok(AnalysisType::Complexity),
        "dead-code" | "deadcode" => Ok(AnalysisType::DeadCode),
        "duplication" | "duplicates" => Ok(AnalysisType::Duplication),
        "technical-debt" | "tdg" => Ok(AnalysisType::TechnicalDebt),
        "big-o" | "bigo" => Ok(AnalysisType::BigO),
        "all" => Ok(AnalysisType::All),
        _ => anyhow::bail!("Unknown analysis type: {}", s),
    }
}

// Handler stubs for backward compatibility
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_defect_prediction(
    _project_path: PathBuf,
    _confidence_threshold: f32,
    _min_lines: usize,
    _include_low_confidence: bool,
    _format: DefectPredictionOutputFormat,
    _high_risk_only: bool,
    _include_recommendations: bool,
    _include: Option<String>,
    _exclude: Option<String>,
    _output: Option<PathBuf>,
    _perf: bool,
) -> anyhow::Result<()> {
    // Stub implementation to avoid recursion
    tracing::info!("Defect prediction analysis not yet implemented (from CLI mod)");
    Ok(())
}

pub async fn handle_analyze_duplicates(config: DuplicateHandlerConfig) -> anyhow::Result<()> {
    // Convert to DuplicateAnalysisConfig
    let analysis_config = handlers::duplication_analysis::DuplicateAnalysisConfig {
        project_path: config.project_path,
        detection_type: config.detection_type,
        threshold: config.threshold as f64,
        min_lines: config.min_lines,
        max_tokens: config.max_tokens,
        format: config.format,
        perf: config.perf,
        include: config.include,
        exclude: config.exclude,
        output: config.output,
    };
    handlers::handle_analyze_duplicates(analysis_config).await
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_graph_metrics(
    _project_path: PathBuf,
    _metrics: Vec<GraphMetricType>,
    _pagerank_seeds: Vec<String>,
    _damping_factor: f32,
    _max_iterations: usize,
    _convergence_threshold: f64,
    _export_graphml: bool,
    _format: GraphMetricsOutputFormat,
    _include: Option<String>,
    _exclude: Option<String>,
    _output: Option<PathBuf>,
    _perf: bool,
    _top_k: usize,
    _min_centrality: f64,
) -> anyhow::Result<()> {
    // Stub implementation to avoid recursion
    tracing::info!("Graph metrics analysis not yet implemented (from CLI mod)");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_name_similarity(
    project_path: PathBuf,
    query: String,
    top_k: usize,
    phonetic: bool,
    scope: SearchScope,
    threshold: f32,
    format: NameSimilarityOutputFormat,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
    perf: bool,
    fuzzy: bool,
    case_sensitive: bool,
) -> anyhow::Result<()> {
    // Delegate to the actual implementation
    crate::cli::analysis::name_similarity::handle_analyze_name_similarity(
        project_path,
        query,
        top_k,
        phonetic,
        scope,
        threshold,
        format,
        include,
        exclude,
        output,
        perf,
        fuzzy,
        case_sensitive,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_symbol_table(
    _project_path: PathBuf,
    _format: SymbolTableOutputFormat,
    _filter: Option<SymbolTypeFilter>,
    _query: Option<String>,
    _include: Vec<String>,
    _exclude: Vec<String>,
    _show_unreferenced: bool,
    _show_references: bool,
    _output: Option<PathBuf>,
    _perf: bool,
) -> anyhow::Result<()> {
    // Stub implementation to avoid recursion
    tracing::info!("Symbol table analysis not yet implemented (from CLI mod)");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_comprehensive(
    _project_path: PathBuf,
    _format: ComprehensiveOutputFormat,
    _include_duplicates: bool,
    _include_dead_code: bool,
    _include_defects: bool,
    _include_complexity: bool,
    _include_tdg: bool,
    _confidence_threshold: f32,
    _min_lines: usize,
    _include: Option<String>,
    _exclude: Option<String>,
    _output: Option<PathBuf>,
    _perf: bool,
    _executive_summary: bool,
) -> anyhow::Result<()> {
    // Stub implementation to avoid recursion
    tracing::info!("Comprehensive analysis not yet implemented (from CLI mod)");
    Ok(())
}
