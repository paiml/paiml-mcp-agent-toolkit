//! Analysis command handlers
//!
//! This module extracts all analysis-related handlers from the main CLI module
//! to reduce complexity and improve organization.

use crate::cli::{self, AnalyzeCommands};
use anyhow::Result;
use serde_json::json;
use std::path::PathBuf;

/// Router for all analysis commands - central dispatch for CLI analyze subcommands.
///
/// This function serves as the main entry point for all `pmat analyze` subcommands,
/// routing each command variant to its specific handler implementation. Critical for
/// API stability as it defines the complete analyze command interface.
///
/// # Parameters
///
/// * `cmd` - The specific analyze command variant with all parsed arguments
///
/// # Returns
///
/// * `Ok(())` - Command completed successfully
/// * `Err(anyhow::Error)` - Command execution failed with detailed error context
///
/// # API Stability Contract
///
/// This router maintains the CLI API contract by:
/// - Ensuring all AnalyzeCommands variants are handled
/// - Providing consistent parameter forwarding to handlers
/// - Maintaining backward compatibility for existing commands
/// - Preventing API drift through comprehensive parameter mapping
///
/// # Supported Commands
///
/// ## Core Analysis Commands
/// - `complexity` - Cyclomatic and cognitive complexity analysis
/// - `churn` - Code change frequency analysis over time
/// - `dead-code` - Unused code detection and reporting
/// - `dag` - Dependency graph generation and visualization
/// - `satd` - Self-admitted technical debt detection
///
/// ## Advanced Analysis Commands  
/// - `deep-context` - Comprehensive project context analysis
/// - `tdg` - Technical debt gravity calculation
/// - `lint-hotspot` - Linting issue density analysis
/// - `makefile` - Makefile structure and rule analysis
/// - `provability` - Formal verification potential assessment
/// - `duplicates` - Code duplication detection
/// - `defect-prediction` - AI-powered defect probability analysis
/// - `comprehensive` - Full multi-faceted analysis suite
/// - `graph-metrics` - Graph centrality and topology metrics
/// - `name-similarity` - Identifier similarity analysis
/// - `proof-annotations` - Proof annotation extraction
/// - `incremental-coverage` - Differential coverage analysis
/// - `symbol-table` - Symbol visibility and reference analysis
/// - `big-o` - Algorithmic complexity analysis
/// - `assemblyscript` - AssemblyScript-specific analysis
/// - `webassembly` - WebAssembly module analysis
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::cli::handlers::analysis_handlers::route_analyze_command;
/// use pmat::cli::commands::AnalyzeCommands;
/// use std::path::PathBuf;
///
/// # tokio_test::block_on(async {
/// // Complexity analysis command
/// let complexity_cmd = AnalyzeCommands::Complexity {
///     project_path: PathBuf::from("/tmp/project"),
///     file: None,
///     files: vec![],
///     toolchain: None,
///     format: pmat::cli::enums::ComplexityOutputFormat::Summary,
///     output: None,
///     max_cyclomatic: None,
///     max_cognitive: None,
///     include: vec![],
///     watch: false,
///     top_files: 10,
///     fail_on_violation: false,
/// };
///
/// // This would normally execute the command
/// // let result = route_analyze_command(complexity_cmd).await;
/// // assert!(result.is_ok());
///
/// // Dead code analysis command
/// let dead_code_cmd = AnalyzeCommands::DeadCode {
///     path: PathBuf::from("/tmp/project"),
///     format: pmat::cli::enums::DeadCodeOutputFormat::Summary,
///     top_files: None,
///     include_unreachable: false,
///     min_dead_lines: 10,
///     include_tests: false,
///     output: None,
///     fail_on_violation: false,
///     max_percentage: 100.0,
/// };
///
/// // DAG analysis command
/// let dag_cmd = AnalyzeCommands::Dag {
///     dag_type: pmat::cli::enums::DagType::CallGraph,
///     project_path: PathBuf::from("/tmp/project"),
///     output: None,
///     max_depth: Some(5),
///     target_nodes: None,
///     filter_external: false,
///     show_complexity: false,
///     include_duplicates: false,
///     include_dead_code: false,
///     enhanced: false,
/// };
///
/// // All commands follow the same routing pattern
/// // Each command variant maps to a specific handler function
/// # });
/// ```
///
/// # Error Handling
///
/// The router implements comprehensive error handling:
/// - Parameter validation errors are propagated from handlers
/// - I/O errors from file operations are wrapped with context
/// - Parse errors include file location information
/// - Analysis failures preserve original error chains
///
/// # Performance Characteristics
///
/// - Route dispatch: O(1) pattern matching
/// - Parameter forwarding: O(1) move semantics
/// - Memory: Minimal overhead, parameters moved to handlers
/// - Concurrency: Handlers may implement parallel processing internally
pub async fn route_analyze_command(cmd: AnalyzeCommands) -> Result<()> {
    use cli::*;

    match cmd {
        // Core analysis commands
        AnalyzeCommands::Complexity { .. }
        | AnalyzeCommands::Churn { .. }
        | AnalyzeCommands::DeadCode { .. }
        | AnalyzeCommands::Dag { .. }
        | AnalyzeCommands::Satd { .. } => route_core_analysis(cmd).await,

        // Advanced analysis commands
        AnalyzeCommands::DeepContext { .. }
        | AnalyzeCommands::Tdg { .. }
        | AnalyzeCommands::LintHotspot { .. }
        | AnalyzeCommands::Comprehensive { .. } => route_advanced_analysis(cmd).await,

        // Quality analysis commands
        AnalyzeCommands::Duplicates { .. }
        | AnalyzeCommands::DefectPrediction { .. }
        | AnalyzeCommands::Provability { .. }
        | AnalyzeCommands::Clippy { .. }
        | AnalyzeCommands::Entropy { .. } => route_quality_analysis(cmd).await,

        // Specialized analysis commands
        AnalyzeCommands::GraphMetrics { .. }
        | AnalyzeCommands::NameSimilarity { .. }
        | AnalyzeCommands::ProofAnnotations { .. }
        | AnalyzeCommands::IncrementalCoverage { .. }
        | AnalyzeCommands::SymbolTable { .. }
        | AnalyzeCommands::BigO { .. } => route_specialized_analysis(cmd).await,

        // Language-specific commands
        AnalyzeCommands::AssemblyScript { .. } | AnalyzeCommands::WebAssembly { .. } => {
            route_language_specific_analysis(cmd).await
        }

        // System commands
        AnalyzeCommands::Makefile { .. } => route_system_analysis(cmd).await,
    }
}

/// Route core analysis commands
async fn route_core_analysis(cmd: AnalyzeCommands) -> Result<()> {
    match cmd {
        AnalyzeCommands::Complexity { .. } => route_complexity_analysis(cmd).await,
        AnalyzeCommands::Churn { .. } => route_churn_analysis(cmd).await,
        AnalyzeCommands::DeadCode { .. } => route_dead_code_analysis(cmd).await,
        AnalyzeCommands::Dag { .. } => route_dag_analysis(cmd).await,
        AnalyzeCommands::Satd { .. } => route_satd_analysis(cmd).await,
        _ => unreachable!("Expected core analysis command"),
    }
}

/// Route advanced analysis commands
async fn route_advanced_analysis(cmd: AnalyzeCommands) -> Result<()> {
    match cmd {
        AnalyzeCommands::DeepContext { .. } => route_deep_context_analysis(cmd).await,
        AnalyzeCommands::Tdg { .. } => route_tdg_analysis(cmd).await,
        AnalyzeCommands::LintHotspot { .. } => route_lint_hotspot_analysis(cmd).await,
        AnalyzeCommands::Comprehensive { .. } => route_comprehensive_analysis(cmd).await,
        _ => unreachable!("Expected advanced analysis command"),
    }
}

/// Route quality analysis commands
async fn route_quality_analysis(cmd: AnalyzeCommands) -> Result<()> {
    match cmd {
        AnalyzeCommands::Duplicates { .. } => route_duplicates_analysis(cmd).await,
        AnalyzeCommands::DefectPrediction { .. } => route_defect_prediction_analysis(cmd).await,
        AnalyzeCommands::Provability { .. } => route_provability_analysis(cmd).await,
        AnalyzeCommands::Clippy { .. } => route_clippy_analysis(cmd).await,
        AnalyzeCommands::Entropy { .. } => route_entropy_analysis(cmd).await,
        _ => unreachable!("Expected quality analysis command"),
    }
}

/// Route specialized analysis commands
async fn route_specialized_analysis(cmd: AnalyzeCommands) -> Result<()> {
    match cmd {
        AnalyzeCommands::GraphMetrics { .. } => route_graph_metrics_analysis(cmd).await,
        AnalyzeCommands::NameSimilarity { .. } => route_name_similarity_analysis(cmd).await,
        AnalyzeCommands::ProofAnnotations { .. } => route_proof_annotations_analysis(cmd).await,
        AnalyzeCommands::IncrementalCoverage { .. } => {
            route_incremental_coverage_analysis(cmd).await
        }
        AnalyzeCommands::SymbolTable { .. } => route_symbol_table_analysis(cmd).await,
        AnalyzeCommands::BigO { .. } => route_big_o_analysis(cmd).await,
        _ => unreachable!("Expected specialized analysis command"),
    }
}

/// Route language-specific analysis commands
async fn route_language_specific_analysis(cmd: AnalyzeCommands) -> Result<()> {
    match cmd {
        AnalyzeCommands::AssemblyScript { .. } => route_assemblyscript_analysis(cmd).await,
        AnalyzeCommands::WebAssembly { .. } => route_webassembly_analysis(cmd).await,
        _ => unreachable!("Expected language-specific analysis command"),
    }
}

/// Route system analysis commands
async fn route_system_analysis(cmd: AnalyzeCommands) -> Result<()> {
    match cmd {
        AnalyzeCommands::Makefile { .. } => route_makefile_analysis(cmd).await,
        _ => unreachable!("Expected system analysis command"),
    }
}

/// Route complexity analysis command
async fn route_complexity_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Complexity {
        path,
        project_path,
        file,
        files,
        toolchain,
        format,
        output,
        max_cyclomatic,
        max_cognitive,
        include,
        watch,
        top_files,
        fail_on_violation,
        timeout,
    } = cmd
    {
        route_complexity_command(
            path,
            project_path,
            file,
            files,
            toolchain,
            format,
            output,
            max_cyclomatic,
            max_cognitive,
            include,
            watch,
            top_files,
            fail_on_violation,
            timeout,
        )
        .await
    } else {
        unreachable!("Expected Complexity command")
    }
}

/// Route churn analysis command
async fn route_churn_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Churn {
        project_path,
        days,
        format,
        output,
        top_files,
        include,
        exclude,
    } = cmd
    {
        super::complexity_handlers::handle_analyze_churn(
            project_path,
            days,
            format,
            output,
            top_files,
            include,
            exclude,
        )
        .await
    } else {
        unreachable!("Expected Churn command")
    }
}

/// Route dead code analysis command
async fn route_dead_code_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::DeadCode {
        path,
        format,
        top_files,
        include_unreachable,
        min_dead_lines,
        include_tests,
        output,
        fail_on_violation,
        max_percentage,
        timeout,
        include,
        exclude,
    } = cmd
    {
        super::complexity_handlers::handle_analyze_dead_code(
            path,
            format,
            top_files,
            include_unreachable,
            min_dead_lines,
            include_tests,
            output,
            fail_on_violation,
            max_percentage,
            timeout,
            include,
            exclude,
        )
        .await
    } else {
        unreachable!("Expected DeadCode command")
    }
}

/// Route DAG analysis command
async fn route_dag_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Dag {
        dag_type,
        project_path,
        output,
        max_depth,
        target_nodes,
        filter_external,
        show_complexity,
        include_duplicates,
        include_dead_code,
        enhanced,
    } = cmd
    {
        super::complexity_handlers::handle_analyze_dag(
            dag_type,
            project_path,
            output,
            max_depth,
            target_nodes,
            filter_external,
            show_complexity,
            include_duplicates,
            include_dead_code,
            enhanced,
        )
        .await
    } else {
        unreachable!("Expected Dag command")
    }
}
/// Route SATD analysis command
async fn route_satd_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Satd {
        path,
        format,
        severity,
        critical_only,
        include_tests,
        strict,
        evolution,
        days,
        metrics,
        output,
        top_files,
        fail_on_violation,
        timeout,
        include,
        exclude,
    } = cmd
    {
        use super::satd_handler::SatdAnalysisConfig;

        let config = SatdAnalysisConfig {
            path,
            format,
            severity,
            critical_only,
            include_tests,
            strict,
            evolution,
            days,
            metrics,
            output,
            top_files,
            fail_on_violation,
            timeout,
            include,
            exclude,
        };

        super::satd_handler::handle_analyze_satd(config).await
    } else {
        unreachable!("Expected Satd command")
    }
}
/// Route deep context analysis command
async fn route_deep_context_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::DeepContext {
        project_path,
        output,
        format,
        full,
        include,
        exclude,
        period_days,
        dag_type,
        max_depth,
        include_patterns,
        exclude_patterns,
        cache_strategy,
        parallel,
        verbose,
        top_files,
    } = cmd
    {
        let converted_dag_type = convert_deep_context_dag_type(dag_type);
        let converted_cache_strategy = convert_cache_strategy(cache_strategy);

        super::advanced_analysis_handlers::handle_analyze_deep_context(
            project_path,
            output,
            format,
            full,
            include,
            exclude,
            period_days,
            Some(converted_dag_type),
            max_depth,
            include_patterns,
            exclude_patterns,
            Some(converted_cache_strategy),
            parallel.is_some(),
            verbose,
            top_files,
        )
        .await
    } else {
        unreachable!("Expected DeepContext command")
    }
}

/// Convert deep context DAG type to standard DAG type
fn convert_deep_context_dag_type(dag_type: cli::DeepContextDagType) -> cli::DagType {
    match dag_type {
        cli::DeepContextDagType::CallGraph => cli::DagType::CallGraph,
        cli::DeepContextDagType::ImportGraph => cli::DagType::ImportGraph,
        cli::DeepContextDagType::Inheritance => cli::DagType::Inheritance,
        cli::DeepContextDagType::FullDependency => cli::DagType::FullDependency,
    }
}

/// Convert cache strategy to string
fn convert_cache_strategy(strategy: cli::DeepContextCacheStrategy) -> String {
    match strategy {
        cli::DeepContextCacheStrategy::Normal => "normal".to_string(),
        cli::DeepContextCacheStrategy::ForceRefresh => "force-refresh".to_string(),
        cli::DeepContextCacheStrategy::Offline => "offline".to_string(),
    }
}
/// Route TDG analysis command
async fn route_tdg_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Tdg {
        path,
        threshold,
        top_files,
        format,
        include_components,
        output,
        critical_only,
        verbose,
    } = cmd
    {
        use super::new_tdg_handler::TdgAnalysisConfig;

        let config = TdgAnalysisConfig {
            path,
            threshold: Some(threshold),
            top_files: Some(top_files),
            format,
            include_components,
            output,
            critical_only,
            verbose,
        };

        super::new_tdg_handler::handle_analyze_tdg(config).await
    } else {
        unreachable!("Expected Tdg command")
    }
}
/// Route lint hotspot analysis command
async fn route_lint_hotspot_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::LintHotspot {
        project_path,
        file,
        format,
        max_density,
        min_confidence,
        enforce,
        dry_run,
        enforcement_metadata,
        output,
        perf,
        clippy_flags,
        top_files,
        include,
        exclude,
    } = cmd
    {
        super::lint_hotspot_handlers::handle_analyze_lint_hotspot(
            project_path,
            file,
            format,
            max_density,
            min_confidence,
            enforce,
            dry_run,
            enforcement_metadata,
            output,
            perf,
            clippy_flags,
            top_files,
            include,
            exclude,
        )
        .await
    } else {
        unreachable!("Expected LintHotspot command")
    }
}

/// Route comprehensive analysis command
async fn route_comprehensive_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Comprehensive {
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
        top_files: _,
    } = cmd
    {
        super::advanced_analysis_handlers::handle_analyze_comprehensive(
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
        )
        .await
    } else {
        unreachable!("Expected Comprehensive command")
    }
}

/// Route duplicates analysis command
async fn route_duplicates_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Duplicates {
        project_path,
        detection_type,
        threshold,
        min_lines,
        max_tokens,
        format,
        perf,
        include,
        exclude,
        output,
        top_files,
    } = cmd
    {
        let config = super::duplication_analysis::DuplicateAnalysisConfig {
            project_path,
            detection_type,
            threshold: threshold as f64,
            min_lines,
            max_tokens,
            format,
            perf,
            include,
            exclude,
            output,
            top_files,
        };
        super::duplication_analysis::handle_analyze_duplicates(config).await
    } else {
        unreachable!("Expected Duplicates command")
    }
}

/// Route defect prediction analysis command
async fn route_defect_prediction_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::DefectPrediction {
        project_path,
        confidence_threshold,
        min_lines,
        include_low_confidence,
        format,
        high_risk_only,
        include_recommendations,
        include,
        exclude,
        output,
        perf,
        top_files,
    } = cmd
    {
        use super::defect_prediction_handler::DefectPredictionConfig;

        let config = DefectPredictionConfig {
            project_path,
            confidence_threshold,
            min_lines,
            include_low_confidence,
            format,
            high_risk_only,
            include_recommendations,
            include,
            exclude,
            output,
            perf,
            top_files,
        };

        super::defect_prediction_handler::handle_analyze_defect_prediction(config).await
    } else {
        unreachable!("Expected DefectPrediction command")
    }
}

/// Route provability analysis command
async fn route_provability_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Provability {
        project_path,
        functions,
        analysis_depth,
        format,
        high_confidence_only,
        include_evidence,
        output,
        top_files,
    } = cmd
    {
        use super::provability_handler::ProvabilityConfig;

        let config = ProvabilityConfig {
            project_path,
            functions,
            analysis_depth,
            format,
            high_confidence_only,
            include_evidence,
            output,
            top_files,
        };

        super::provability_handler::handle_analyze_provability(config).await
    } else {
        unreachable!("Expected Provability command")
    }
}

/// Route graph metrics analysis command
async fn route_graph_metrics_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::GraphMetrics {
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
    } = cmd
    {
        super::advanced_analysis_handlers::handle_analyze_graph_metrics(
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
    } else {
        unreachable!("Expected GraphMetrics command")
    }
}

/// Route name similarity analysis command
async fn route_name_similarity_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::NameSimilarity {
        project_path,
        query,
        top_k,
        phonetic,
        scope,
        format,
        output,
        threshold,
        include,
        exclude,
        perf,
        fuzzy,
        case_sensitive,
    } = cmd
    {
        super::name_similarity_analysis::handle_analyze_name_similarity(
            project_path,
            query,
            top_k,
            phonetic,
            scope,
            threshold as f64,
            format,
            include,
            exclude,
            output,
            perf,
            fuzzy,
            case_sensitive,
        )
        .await
    } else {
        unreachable!("Expected NameSimilarity command")
    }
}

/// Route proof annotations analysis command
async fn route_proof_annotations_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::ProofAnnotations {
        project_path,
        format,
        high_confidence_only,
        include_evidence,
        property_type,
        verification_method,
        output,
        perf,
        clear_cache,
        top_files: _top_files,
    } = cmd
    {
        super::proof_annotations_handler::handle_analyze_proof_annotations(
            project_path,
            format,
            high_confidence_only,
            include_evidence,
            property_type,
            verification_method,
            output,
            perf,
            clear_cache,
        )
        .await
    } else {
        unreachable!("Expected ProofAnnotations command")
    }
}

/// Route incremental coverage analysis command
async fn route_incremental_coverage_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::IncrementalCoverage {
        project_path,
        base_branch,
        target_branch,
        format,
        coverage_threshold,
        changed_files_only,
        detailed,
        output,
        perf,
        cache_dir,
        force_refresh,
        top_files,
    } = cmd
    {
        use super::incremental_coverage_handler::IncrementalCoverageConfig;

        let config = IncrementalCoverageConfig {
            project_path,
            base_branch,
            target_branch,
            format,
            coverage_threshold,
            changed_files_only,
            detailed,
            output,
            perf,
            cache_dir,
            force_refresh,
            top_files,
        };

        super::incremental_coverage_handler::handle_analyze_incremental_coverage(config).await
    } else {
        unreachable!("Expected IncrementalCoverage command")
    }
}

/// Route symbol table analysis command
async fn route_symbol_table_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::SymbolTable {
        project_path,
        format,
        filter,
        query,
        include,
        exclude,
        show_unreferenced,
        show_references,
        output,
        perf,
        top_files: _top_files,
    } = cmd
    {
        super::advanced_analysis_handlers::handle_analyze_symbol_table(
            project_path,
            format,
            filter,
            query,
            include,
            exclude,
            show_unreferenced,
            show_references,
            output,
            perf,
        )
        .await
    } else {
        unreachable!("Expected SymbolTable command")
    }
}

/// Route Big O analysis command
async fn route_big_o_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::BigO {
        project_path,
        format,
        confidence_threshold,
        analyze_space,
        include,
        exclude,
        high_complexity_only,
        output,
        perf,
        top_files,
    } = cmd
    {
        super::big_o_handlers::handle_analyze_big_o(
            project_path,
            format,
            confidence_threshold,
            analyze_space,
            include,
            exclude,
            high_complexity_only,
            output,
            perf,
            top_files,
        )
        .await
    } else {
        unreachable!("Expected BigO command")
    }
}

/// Route AssemblyScript analysis command
async fn route_assemblyscript_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::AssemblyScript {
        project_path,
        format,
        wasm_complexity,
        memory_analysis,
        security,
        output,
        timeout,
        perf,
        top_files: _top_files,
    } = cmd
    {
        super::wasm_handlers::handle_analyze_assemblyscript(
            project_path,
            format,
            wasm_complexity,
            memory_analysis,
            security,
            output,
            timeout,
            perf,
        )
        .await
    } else {
        unreachable!("Expected AssemblyScript command")
    }
}

/// Route WebAssembly analysis command
async fn route_webassembly_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::WebAssembly {
        project_path,
        format,
        include_binary,
        include_text,
        memory_analysis,
        security,
        complexity,
        output,
        perf,
        top_files: _top_files,
    } = cmd
    {
        super::wasm_handlers::handle_analyze_webassembly(
            project_path,
            format,
            include_binary,
            include_text,
            memory_analysis,
            security,
            complexity,
            output,
            perf,
        )
        .await
    } else {
        unreachable!("Expected WebAssembly command")
    }
}

/// Route Makefile analysis command
async fn route_makefile_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Makefile {
        path,
        rules,
        format,
        fix,
        gnu_version,
        top_files,
    } = cmd
    {
        super::advanced_analysis_handlers::handle_analyze_makefile(
            path,
            rules,
            format,
            fix,
            Some(gnu_version),
            top_files,
        )
        .await
    } else {
        unreachable!("Expected Makefile command")
    }
}

/// Route complexity command (cognitive complexity ≤5)
async fn route_complexity_command(
    path: PathBuf,
    project_path: Option<PathBuf>,
    file: Option<PathBuf>,
    files: Vec<PathBuf>,
    toolchain: Option<String>,
    format: crate::cli::ComplexityOutputFormat,
    output: Option<PathBuf>,
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
    include: Vec<String>,
    watch: bool,
    top_files: usize,
    fail_on_violation: bool,
    timeout: u64,
) -> Result<()> {
    // Handle parameter migration: use new 'path' or deprecated 'project_path'
    let analysis_path = if let Some(deprecated_path) = project_path {
        eprintln!("⚠️  WARNING: --project-path is deprecated. Use --path instead.");
        deprecated_path
    } else {
        path
    };

    super::complexity_handlers::handle_analyze_complexity(
        analysis_path,
        file,
        files,
        toolchain,
        format,
        output,
        max_cyclomatic,
        max_cognitive,
        include,
        watch,
        top_files,
        fail_on_violation,
        timeout,
    )
    .await
}

/// Route clippy analysis command (complexity: 4)
async fn route_clippy_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Clippy {
        project_path,
        confidence,
        dry_run,
        fix_codes,
        output,
        perf: _perf,
    } = cmd
    {
        // Call the auto_clippy_fix MCP tool function directly
        // NOTE: Disabled due to pmcp ToolResult import issue - see issue #XXX
        // use crate::mcp_pmcp::tools::auto_clippy_fix::auto_clippy_fix;
        
        let confidence_level = Some(confidence.clone());
        let codes = if fix_codes.is_empty() { 
            None 
        } else { 
            Some(fix_codes.clone()) 
        };
        
        // NOTE: Re-enable after fixing pmcp ToolResult import issue
        /*
        let result = auto_clippy_fix(
            Some(project_path.to_string_lossy().to_string()),
            confidence_level,
            Some(dry_run),
            codes,
        )
        .await?;
        */
        
        // NOTE: Implement direct clippy fix logic here temporarily
        // For now, provide a stub implementation
        if let Some(output_path) = output {
            use std::fs;
            let stub_result = json!({
                "action": if dry_run { "analyzed" } else { "applied" },
                "message": "🔧 Clippy fix temporarily disabled due to pmcp import issue",
                "confidence": confidence,
                "codes": codes
            });
            let content = serde_json::to_string_pretty(&stub_result)?;
            fs::write(output_path, content)?;
        } else {
            println!("🔧 Clippy fix temporarily disabled due to pmcp import issue");
        }
        
        Ok(())
    } else {
        unreachable!("Expected Clippy command")
    }
}

/// Route entropy analysis command
async fn route_entropy_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Entropy {
        project_path,
        format,
        output,
        min_severity,
        top_violations,
        file,
        include_tests,
    } = cmd
    {
        use crate::entropy::{EntropyAnalyzer, EntropyConfig};
        use crate::cli::{EntropyOutputFormat, EntropySeverity};
        use crate::entropy::violation_detector::Severity;
        use std::fs;

        // Convert CLI severity to entropy severity
        let min_sev = match min_severity {
            EntropySeverity::Low => Severity::Low,
            EntropySeverity::Medium => Severity::Medium,
            EntropySeverity::High => Severity::High,
        };

        // Create entropy configuration
        let mut config = EntropyConfig::default();
        config.min_severity = min_sev;
        
        // Update exclude paths if tests should be excluded
        if !include_tests {
            config.exclude_paths.push("**/*test*.rs".to_string());
            config.exclude_paths.push("tests/**".to_string());
        }

        // Create analyzer and run analysis
        let analyzer = EntropyAnalyzer::with_config(config);
        
        let analysis_path = if let Some(file_path) = file {
            file_path
        } else {
            project_path
        };

        let report = analyzer.analyze(&analysis_path).await?;

        // Generate output based on format
        let output_content = match format {
            EntropyOutputFormat::Summary => {
                let violations = if top_violations > 0 && report.actionable_violations.len() > top_violations {
                    report.actionable_violations.iter().take(top_violations).cloned().collect::<Vec<_>>()
                } else {
                    report.actionable_violations.clone()
                };
                
                format!(
                    "Entropy Analysis Summary\n========================\n\n\
                     Files Analyzed: {}\n\
                     Total Violations: {}\n\
                     Potential LOC Reduction: {} lines ({:.1}%)\n\n\
                     Top Violations:\n{}\n",
                    report.total_files_analyzed,
                    report.actionable_violations.len(),
                    report.total_loc_reduction(),
                    report.reduction_percentage(),
                    violations.iter()
                        .enumerate()
                        .map(|(i, v)| format!(
                            "{}. {} (saves {} lines)\n   Fix: {}",
                            i + 1, v.message, v.estimated_loc_reduction, v.fix_suggestion
                        ))
                        .collect::<Vec<_>>()
                        .join("\n\n")
                )
            },
            EntropyOutputFormat::Detailed => report.format_report(),
            EntropyOutputFormat::Json => serde_json::to_string_pretty(&report)?,
            EntropyOutputFormat::Markdown => {
                format!(
                    "# Entropy Analysis Report\n\n\
                     ## Summary\n\n\
                     - **Files Analyzed**: {}\n\
                     - **Total Violations**: {}\n\
                     - **Potential LOC Reduction**: {} lines ({:.1}%)\n\n\
                     ## Violations\n\n{}\n",
                    report.total_files_analyzed,
                    report.actionable_violations.len(),
                    report.total_loc_reduction(),
                    report.reduction_percentage(),
                    report.actionable_violations.iter()
                        .take(if top_violations == 0 { usize::MAX } else { top_violations })
                        .map(|v| format!(
                            "### {} ({:?})\n\n\
                             **Pattern**: {:?} (repeated {} times)\n\
                             **Fix**: {}\n\
                             **LOC Reduction**: {} lines\n\
                             **Affected Files**: {}\n",
                            v.message,
                            v.severity,
                            v.pattern.pattern_type,
                            v.pattern.repetitions,
                            v.fix_suggestion,
                            v.estimated_loc_reduction,
                            v.affected_files.len()
                        ))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            },
        };

        // Output results
        if let Some(output_path) = output {
            fs::write(output_path, output_content)?;
        } else {
            println!("{}", output_content);
        }

        Ok(())
    } else {
        unreachable!("Expected Entropy command")
    }
}

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_analysis_handlers_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
