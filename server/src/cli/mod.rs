//! CLI module for the paiml-mcp-agent-toolkit
//!
//! This module implements the command-line interface using a modular architecture
//! to reduce complexity and improve testability.

pub mod analysis;
pub mod analysis_helpers;
pub mod analysis_utilities;
pub mod args;
pub mod command_dispatcher;
pub mod command_structure;
pub mod commands;
pub mod coverage_helpers;
pub mod dead_code_formatter;
pub mod defect_formatter;
pub mod defect_helpers;
pub mod defect_prediction_helpers;
pub mod diagnose;
pub mod enums;
pub mod formatting_helpers;
pub mod handlers;
pub mod language_analyzer;
pub mod name_similarity_helpers;
pub mod proof_annotation_formatter;
pub mod proof_annotation_helpers;
pub mod provability_helpers;
pub mod symbol_table_helpers;
pub mod tdg_helpers;

// Re-export commonly used types from submodules
pub use commands::{
    AgentCommands, AnalyzeCommands, Cli, Commands, EnforceCommands, Mode, RefactorCommands,
};
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
    pub is_mcp_server: bool,
}

/// Parse CLI early to extract tracing configuration
///
/// # Examples
///
/// ```rust,ignore
/// use pmat::cli::parse_early_for_tracing;
///
/// // This function reads from std::env::args() and RUST_LOG
/// let args = parse_early_for_tracing();
///
/// // The function always returns valid EarlyCliArgs
/// // Values depend on actual command line arguments
/// ```
pub fn parse_early_for_tracing() -> EarlyCliArgs {
    let args: Vec<String> = std::env::args().collect();

    let verbose = args.iter().any(|arg| arg == "-v" || arg == "--verbose");
    let debug = args.iter().any(|arg| arg == "--debug");
    let trace = args.iter().any(|arg| arg == "--trace");

    // Check if this is an MCP server command
    let is_mcp_server = args.len() >= 3 && args[1] == "agent" && args[2] == "mcp-server";

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
        is_mcp_server,
    }
}

pub async fn run(server: Arc<StatelessTemplateServer>) -> anyhow::Result<()> {
    let cli = match parse_with_suggestions() {
        Ok(cli) => cli,
        Err(suggestion_msg) => {
            eprintln!("{}", suggestion_msg);
            std::process::exit(2); // Exit with "misuse" code for command errors
        }
    };

    debug!("CLI arguments parsed");

    // Handle forced mode
    if let Some(commands::Mode::Mcp) = cli.mode {
        info!("Forced MCP mode detected");
        return crate::run_mcp_server(server).await;
    }

    // Use command dispatcher for improved modularity
    CommandDispatcher::execute_command(cli.command, server).await
}

/// Parse CLI with command suggestions on failure
fn parse_with_suggestions() -> Result<Cli, String> {
    use crate::utils::command_suggestions::CommandSuggester;
    use clap::Parser;

    // Try to parse normally first
    match Cli::try_parse() {
        Ok(cli) => Ok(cli),
        Err(clap_error) => {
            // Handle special cases where clap handles --version and --help
            use clap::error::ErrorKind;
            match clap_error.kind() {
                ErrorKind::DisplayHelp => {
                    // Print the help message to stdout
                    print!("{}", clap_error);
                    std::process::exit(0);
                }
                ErrorKind::DisplayVersion => {
                    // Print the version to stdout
                    print!("{}", clap_error);
                    std::process::exit(0);
                }
                _ => {
                    // For other errors, provide suggestions
                    let args: Vec<String> = std::env::args().skip(1).collect();
                    let suggester = CommandSuggester::new();

                    // Get suggestion based on the failed arguments
                    if let Some(suggestion) = suggester.suggest_command(&args) {
                        let error_msg = format!(
                            "error: unrecognized subcommand\n\n{}\n\nFor more information, try 'pmat --help'",
                            suggestion
                        );
                        Err(error_msg)
                    } else {
                        // If no suggestion, show the original clap error with examples
                        let examples = CommandSuggester::get_help_examples();
                        let error_msg = format!("{}{}", clap_error, examples);
                        Err(error_msg)
                    }
                }
            }
        }
    }
}

// Helper functions made public for testing

use std::path::Path;

/// Detects the primary programming language of a project based on marker files
///
/// # Examples
///
/// ```rust
/// use pmat::cli::detect_primary_language;
/// use std::path::Path;
/// use tempfile::tempdir;
/// use std::fs;
///
/// let dir = tempdir().unwrap();
/// fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();
///
/// let lang = detect_primary_language(dir.path());
/// assert_eq!(lang, Some("rust".to_string()));
/// ```
fn has_ruchy_files(path: &Path) -> bool {
    use walkdir::WalkDir;
    WalkDir::new(path)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .any(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "ruchy" || ext == "rh")
                .unwrap_or(false)
        })
}

fn detect_by_project_files(path: &Path) -> Option<String> {
    // Project marker files in priority order
    const MARKERS: &[(&str, &str)] = &[
        ("Cargo.toml", "rust"),
        ("pyproject.toml", "python-uv"),
        ("setup.py", "python-uv"),
        ("build.gradle", "kotlin"),
        ("build.gradle.kts", "kotlin"),
    ];
    
    for (file, lang) in MARKERS {
        if path.join(file).exists() {
            return Some(lang.to_string());
        }
    }
    
    // Special handling for JS/TS projects
    if path.join("package.json").exists() {
        if path.join("deno.json").exists() || path.join("deno.jsonc").exists() {
            return Some("deno".to_string());
        }
        return Some("deno".to_string());
    }
    
    None
}

fn should_exclude_dir(name: &str) -> bool {
    name.starts_with('.') || matches!(name, "target" | "node_modules" | "build" | "dist" | "archive")
}

fn count_extension(ext: &str, lang_counts: &mut std::collections::HashMap<&'static str, usize>) {
    match ext {
        "rs" => *lang_counts.entry("rust").or_insert(0) += 1,
        "ts" | "tsx" | "js" | "jsx" => *lang_counts.entry("deno").or_insert(0) += 1,
        "py" => *lang_counts.entry("python-uv").or_insert(0) += 1,
        "kt" | "kts" => *lang_counts.entry("kotlin").or_insert(0) += 1,
        _ => {}
    }
}

fn detect_by_file_extensions(path: &Path) -> Option<String> {
    use walkdir::WalkDir;
    let mut lang_counts = std::collections::HashMap::new();
    
    for entry in WalkDir::new(path)
        .max_depth(5)
        .into_iter()
        .filter_entry(|e| {
            let file_name = e.file_name().to_str().unwrap_or("");
            !should_exclude_dir(file_name)
        })
        .flatten()
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                count_extension(ext, &mut lang_counts);
            }
        }
    }
    
    lang_counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(lang, _)| lang.to_string())
}

pub fn detect_primary_language(path: &Path) -> Option<String> {
    // Check for Ruchy files first
    if has_ruchy_files(path) {
        return Some("ruchy".to_string());
    }
    
    // Check project marker files
    if let Some(lang) = detect_by_project_files(path) {
        return Some(lang);
    }
    
    // Fall back to file extension counting
    detect_by_file_extensions(path)
}

/// Detect primary language with confidence score
fn detect_with_confidence_by_markers(path: &Path) -> Option<(String, f64)> {
    // Project markers with 100% confidence
    const CONFIDENT_MARKERS: &[(&str, &str)] = &[
        ("Cargo.toml", "rust"),
        ("pyproject.toml", "python-uv"),
        ("setup.py", "python-uv"),
        ("build.gradle", "kotlin"),
        ("build.gradle.kts", "kotlin"),
    ];
    
    for (file, lang) in CONFIDENT_MARKERS {
        if path.join(file).exists() {
            return Some((lang.to_string(), 100.0));
        }
    }
    
    // Special JS/TS handling
    if path.join("package.json").exists() {
        let confidence = if path.join("deno.json").exists() || path.join("deno.jsonc").exists() {
            100.0
        } else {
            90.0
        };
        return Some(("deno".to_string(), confidence));
    }
    
    None
}

fn count_files_by_extension(path: &Path) -> Option<(String, f64)> {
    use walkdir::WalkDir;
    let mut lang_counts = std::collections::HashMap::new();
    let mut total_files = 0;
    
    for entry in WalkDir::new(path)
        .max_depth(5)
        .into_iter()
        .filter_entry(|e| {
            let file_name = e.file_name().to_str().unwrap_or("");
            !should_exclude_dir(file_name)
        })
        .flatten()
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                let lang = match ext {
                    "rs" => Some("rust"),
                    "ts" | "tsx" | "js" | "jsx" => Some("deno"),
                    "py" => Some("python-uv"),
                    "kt" | "kts" => Some("kotlin"),
                    _ => None,
                };
                
                if let Some(l) = lang {
                    *lang_counts.entry(l).or_insert(0) += 1;
                    total_files += 1;
                }
            }
        }
    }
    
    if total_files == 0 {
        return None;
    }
    
    lang_counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(lang, count)| {
            let confidence = (count as f64 / total_files as f64) * 100.0;
            (lang.to_string(), confidence)
        })
}

pub fn detect_primary_language_with_confidence(path: &Path) -> Option<(String, f64)> {
    // Try project markers first
    if let Some(result) = detect_with_confidence_by_markers(path) {
        return Some(result);
    }
    
    // Fall back to file counting
    count_files_by_extension(path)
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

/// Converts CLI DAG type to internal model DAG type
///
/// # Examples
///
/// ```rust
/// use pmat::cli::{convert_dag_type, DeepContextDagType};
/// use pmat::models::dag::DagType;
///
/// let cli_type = DeepContextDagType::CallGraph;
/// let model_type = convert_dag_type(cli_type);
/// assert!(matches!(model_type, DagType::CallGraph));
/// ```
pub fn convert_dag_type(dag_type: DeepContextDagType) -> crate::models::dag::DagType {
    match dag_type {
        DeepContextDagType::CallGraph => crate::models::dag::DagType::CallGraph,
        DeepContextDagType::ImportGraph => crate::models::dag::DagType::ImportGraph,
        DeepContextDagType::Inheritance => crate::models::dag::DagType::Inheritance,
        DeepContextDagType::FullDependency => crate::models::dag::DagType::FullDependency,
    }
}

/// Converts cache strategy (currently a pass-through)
///
/// # Examples
///
/// ```rust
/// use pmat::cli::{convert_cache_strategy, DeepContextCacheStrategy};
///
/// let strategy = DeepContextCacheStrategy::Normal;
/// let converted = convert_cache_strategy(strategy);
///
/// // Currently returns the same strategy
/// assert_eq!(converted, DeepContextCacheStrategy::Normal);
/// ```
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

/// Parses a string into an AnalysisType enum
///
/// # Examples
///
/// ```rust
/// use pmat::cli::{parse_analysis_type, AnalysisType};
///
/// assert_eq!(parse_analysis_type("complexity").unwrap(), AnalysisType::Complexity);
/// assert_eq!(parse_analysis_type("tdg").unwrap(), AnalysisType::TechnicalDebt);
/// assert_eq!(parse_analysis_type("big-o").unwrap(), AnalysisType::BigO);
/// assert!(parse_analysis_type("invalid").is_err());
/// ```
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
        top_files: 10, // Default to 10
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

#[cfg(test)]
mod analysis_utilities_property_tests;
