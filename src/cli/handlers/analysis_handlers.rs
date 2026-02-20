//! Analysis command handlers
//!
//! This module extracts all analysis-related handlers from the main CLI module
//! to reduce complexity and improve organization.
#![cfg_attr(coverage_nightly, coverage(off))]

use crate::cli::{self, AnalyzeCommands};
use anyhow::Result;
use std::path::{Path, PathBuf};

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
/// - Ensuring all `AnalyzeCommands` variants are handled
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
    use cli::AnalyzeCommands;

    match cmd {
        // Core analysis commands
        AnalyzeCommands::Complexity { .. }
        | AnalyzeCommands::Churn { .. }
        | AnalyzeCommands::DeadCode { .. }
        | AnalyzeCommands::Defects { .. }
        | AnalyzeCommands::Dag { .. }
        | AnalyzeCommands::Satd { .. } => route_core_analysis(cmd).await,

        // Advanced analysis commands
        AnalyzeCommands::DeepContext { .. }
        | AnalyzeCommands::Tdg { .. }
        | AnalyzeCommands::BuildTdg { .. }
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
        | AnalyzeCommands::CoverageImprove { .. }
        | AnalyzeCommands::SymbolTable { .. }
        | AnalyzeCommands::BigO { .. } => route_specialized_analysis(cmd).await,

        // Language-specific commands
        AnalyzeCommands::AssemblyScript { .. }
        | AnalyzeCommands::WebAssembly { .. }
        | AnalyzeCommands::Wasm { .. } => route_language_specific_analysis(cmd).await,

        // Deep WASM analysis (feature-gated)
        #[cfg(feature = "deep-wasm")]
        AnalyzeCommands::DeepWasm { .. } => route_deep_wasm_analysis(cmd).await,

        // Mutation testing (feature-gated)
        #[cfg(feature = "mutation-testing")]
        AnalyzeCommands::Mutate { .. } => route_mutation_testing(cmd).await,

        // System commands
        AnalyzeCommands::Makefile { .. } => route_system_analysis(cmd).await,

        // Semantic analysis commands (PMAT-SEARCH-011)
        AnalyzeCommands::Cluster { .. } | AnalyzeCommands::Topics { .. } => {
            route_semantic_analysis(cmd).await
        }

        // MLOps model analysis (PMAT-500)
        AnalyzeCommands::Models { .. } => route_model_analysis(cmd).await,
    }
}

/// Route core analysis commands
async fn route_core_analysis(cmd: AnalyzeCommands) -> Result<()> {
    match cmd {
        AnalyzeCommands::Complexity { .. } => route_complexity_analysis(cmd).await,
        AnalyzeCommands::Churn { .. } => route_churn_analysis(cmd).await,
        AnalyzeCommands::DeadCode { .. } => route_dead_code_analysis(cmd).await,
        AnalyzeCommands::Defects { .. } => route_defects_analysis(cmd).await,
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
        AnalyzeCommands::BuildTdg { .. } => route_build_tdg_analysis(cmd).await,
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
        AnalyzeCommands::CoverageImprove {
            project_path,
            target,
            max_iterations,
            fast,
            mutation_threshold,
            focus,
            exclude,
            output,
            format,
        } => {
            crate::cli::handlers::coverage_improve_handler::handle_coverage_improve(
                project_path,
                target,
                max_iterations,
                fast,
                mutation_threshold,
                focus,
                exclude,
                output,
                format,
            )
            .await
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
        #[cfg(feature = "wasm-ast")]
        AnalyzeCommands::Wasm { .. } => route_wasm_analysis(cmd).await,
        #[cfg(not(feature = "wasm-ast"))]
        AnalyzeCommands::Wasm { .. } => {
            anyhow::bail!(
                "WASM analysis requires the 'wasm-ast' feature. Build with --features wasm-ast"
            )
        }
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
        ml: _, // GH-97: ML flag (not yet implemented in handler)
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
        max_depth,
    } = cmd
    {
        super::dead_code_handlers::handle_analyze_dead_code(
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
            max_depth,
        )
        .await
    } else {
        unreachable!("Expected DeadCode command")
    }
}

/// Route defects analysis command
async fn route_defects_analysis(cmd: AnalyzeCommands) -> Result<()> {
    use crate::cli::handlers::analyze_defects_handler::{handle_analyze_defects, OutputFormat};
    use crate::services::defect_detector::Severity;

    if let AnalyzeCommands::Defects {
        path,
        file,
        severity,
        format,
        output: _,
    } = cmd
    {
        // Convert format enum to handler's OutputFormat
        let output_format = match format {
            cli::DefectsOutputFormat::Text => OutputFormat::Text,
            cli::DefectsOutputFormat::Json => OutputFormat::Json,
            cli::DefectsOutputFormat::Junit => OutputFormat::Junit,
        };

        // Parse severity filter if provided
        let severity_filter = severity
            .as_deref()
            .and_then(|s| match s.to_lowercase().as_str() {
                "critical" => Some(Severity::Critical),
                "high" => Some(Severity::High),
                "medium" => Some(Severity::Medium),
                "low" => Some(Severity::Low),
                _ => None,
            });

        let exit_code = handle_analyze_defects(
            path.as_deref(),
            file.as_deref(),
            severity_filter,
            output_format,
        )
        .await?;

        // Exit with the handler's exit code (1 if critical defects found)
        if exit_code != 0 {
            std::process::exit(exit_code);
        }

        Ok(())
    } else {
        unreachable!("Expected Defects command")
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
        extended,
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
            extended,
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
        ml: _, // GH-97: ML flag (not yet implemented in handler)
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

/// Run cargo build step for build-tdg command
fn run_cargo_build(path: &Path, release: bool) -> Result<()> {
    use std::process::Command;
    println!("📦 Building project...");
    let mut build_cmd = Command::new("cargo");
    build_cmd.arg("build");
    if release {
        build_cmd.arg("--release");
    }
    build_cmd.current_dir(path);
    let status = build_cmd.status()?;
    if !status.success() {
        anyhow::bail!("Build failed with exit code: {:?}", status.code());
    }
    println!("✅ Build successful\n");
    Ok(())
}

/// Check for quality regressions against baseline
fn check_quality_regression(path: &Path) -> Result<()> {
    let baseline_path = path.join(".pmat/baseline.json");
    if !baseline_path.exists() {
        println!(
            "⚠️  No baseline found at {}, skipping regression check",
            baseline_path.display()
        );
        println!("   Run 'pmat tdg baseline create' to create a baseline");
        return Ok(());
    }
    println!("🔍 Checking for quality regressions...");
    let status = std::process::Command::new("pmat")
        .args([
            "tdg",
            "check-regression",
            "--baseline",
            baseline_path.to_str().unwrap_or(".pmat/baseline.json"),
            "--path",
            path.to_str().unwrap_or("."),
            "--fail-on-regression",
        ])
        .status();
    match status {
        Ok(s) if s.success() => println!("✅ No regressions detected"),
        Ok(_) => anyhow::bail!("Quality regression detected"),
        Err(e) => println!("⚠️  Could not run regression check: {}", e),
    }
    Ok(())
}

/// Route build-tdg analysis command (build + TDG quality gate)
async fn route_build_tdg_analysis(cmd: AnalyzeCommands) -> Result<()> {
    let AnalyzeCommands::BuildTdg {
        path,
        release,
        threshold,
        fail_on_regression,
        tdg_only,
        top_files,
        format,
        output,
    } = cmd
    else {
        unreachable!("Expected BuildTdg command")
    };

    use super::new_tdg_handler::TdgAnalysisConfig;

    if !tdg_only {
        run_cargo_build(&path, release)?;
    }

    println!("📊 Running TDG analysis...");
    let config = TdgAnalysisConfig {
        path: path.clone(),
        threshold: Some(threshold),
        top_files: Some(top_files),
        format,
        include_components: false,
        output,
        critical_only: false,
        verbose: false,
    };

    let result = super::new_tdg_handler::handle_analyze_tdg(config).await;

    if fail_on_regression {
        check_quality_regression(&path)?;
    }

    result
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
            threshold: f64::from(threshold),
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
            f64::from(threshold),
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

/// Route `AssemblyScript` analysis command
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

#[cfg(feature = "wasm-ast")]
async fn route_wasm_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Wasm {
        wasm_file,
        format,
        verify,
        security,
        profile,
        baseline,
        output,
        verbose,
    } = cmd
    {
        super::wasm_handler::handle_analyze_wasm(
            wasm_file, format, verify, security, profile, baseline, output, verbose,
        )
        .await
    } else {
        unreachable!("Expected Wasm command")
    }
}

/// Route Deep WASM analysis command
#[cfg(feature = "deep-wasm")]
async fn route_deep_wasm_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::DeepWasm {
        source_path,
        wasm_file,
        dwarf_file,
        source_map,
        language,
        focus,
        format,
        output,
        strict,
        include_mir,
        include_llvm_ir,
        track_memory,
        detect_deadlocks,
    } = cmd
    {
        super::deep_wasm_handlers::handle_deep_wasm(super::deep_wasm_handlers::DeepWasmOptions {
            source_path,
            wasm_file,
            dwarf_file,
            source_map,
            language,
            focus,
            format,
            output,
            strict,
            _include_mir: include_mir,
            _include_llvm_ir: include_llvm_ir,
            _track_memory: track_memory,
            _detect_deadlocks: detect_deadlocks,
        })
        .await
    } else {
        unreachable!("Expected DeepWasm command")
    }
}

/// Route Mutation Testing command (feature-gated)
#[cfg(feature = "mutation-testing")]
async fn route_mutation_testing(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Mutate {
        path,
        operators,
        ml_predict,
        distributed,
        workers,
        progress,
        min_score,
        ci_learning,
        ci_provider,
        auto_train_threshold,
        format,
        output,
    } = cmd
    {
        let config = super::mutation_handlers::MutationTestConfig::new(
            operators,
            ml_predict,
            distributed,
            workers,
            progress,
            min_score,
            ci_learning,
            ci_provider,
            auto_train_threshold,
        );
        super::mutation_handlers::handle_mutate(path, config, format, output).await
    } else {
        unreachable!("Expected Mutate command")
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
#[allow(clippy::too_many_arguments)]
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
    // Silently accept both for backwards compatibility
    let analysis_path = project_path.unwrap_or(path);

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
        use crate::mcp_pmcp::tools::auto_clippy_fix::auto_clippy_fix;

        let confidence_level = Some(confidence.clone());
        let codes = if fix_codes.is_empty() {
            None
        } else {
            Some(fix_codes.clone())
        };

        let result = auto_clippy_fix(
            Some(project_path.to_string_lossy().to_string()),
            confidence_level,
            Some(dry_run),
            codes,
        )
        .await?;

        if let Some(output_path) = output {
            use std::fs;
            let content = serde_json::to_string_pretty(&result)?;
            fs::write(&output_path, content)?;
            eprintln!("📁 Results written to {}", output_path.display());
        } else {
            eprintln!("{result:?}");
        }

        Ok(())
    } else {
        unreachable!("Expected Clippy command")
    }
}

/// Route entropy analysis command
///
/// Refactored to reduce complexity from 25 to <20 by extracting helper functions
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
        use crate::entropy::EntropyAnalyzer;

        let config = create_entropy_config(min_severity, include_tests);
        let analyzer = EntropyAnalyzer::with_config(config);

        let analysis_path = file.unwrap_or(project_path);
        let report = analyzer.analyze(&analysis_path).await?;

        let output_content = format_entropy_report(&report, format, top_violations)?;

        output_entropy_results(output, &output_content)?;

        Ok(())
    } else {
        unreachable!("Expected Entropy command")
    }
}

/// Create entropy configuration from CLI parameters
fn create_entropy_config(
    min_severity: crate::cli::EntropySeverity,
    include_tests: bool,
) -> crate::entropy::EntropyConfig {
    use crate::cli::EntropySeverity;
    use crate::entropy::violation_detector::Severity;
    use crate::entropy::EntropyConfig;

    let min_sev = match min_severity {
        EntropySeverity::Low => Severity::Low,
        EntropySeverity::Medium => Severity::Medium,
        EntropySeverity::High => Severity::High,
    };

    let mut config = EntropyConfig {
        min_severity: min_sev,
        ..Default::default()
    };

    if !include_tests {
        config.exclude_paths.push("**/*test*.rs".to_string());
        config.exclude_paths.push("tests/**".to_string());
    }

    config
}

/// Format entropy report based on output format
fn format_entropy_report(
    report: &crate::entropy::EntropyReport,
    format: crate::cli::EntropyOutputFormat,
    top_violations: usize,
) -> Result<String> {
    use crate::cli::EntropyOutputFormat;

    match format {
        EntropyOutputFormat::Summary => Ok(format_summary_report(report, top_violations)),
        EntropyOutputFormat::Detailed => Ok(report.format_report()),
        EntropyOutputFormat::Json => Ok(serde_json::to_string_pretty(&report)?),
        EntropyOutputFormat::Markdown => Ok(format_markdown_report(report, top_violations)),
    }
}

/// Format summary report
fn format_summary_report(report: &crate::entropy::EntropyReport, top_violations: usize) -> String {
    let violations = get_top_violations(&report.actionable_violations, top_violations);

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
        format_violation_list(&violations)
    )
}

/// Format markdown report
fn format_markdown_report(report: &crate::entropy::EntropyReport, top_violations: usize) -> String {
    let max_violations = if top_violations == 0 {
        usize::MAX
    } else {
        top_violations
    };

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
        format_markdown_violations(&report.actionable_violations, max_violations)
    )
}

/// Get top N violations from list
fn get_top_violations(
    violations: &[crate::entropy::violation_detector::ActionableViolation],
    top_n: usize,
) -> Vec<crate::entropy::violation_detector::ActionableViolation> {
    if top_n > 0 && violations.len() > top_n {
        violations.iter().take(top_n).cloned().collect()
    } else {
        violations.to_vec()
    }
}

/// Format violation list for summary
fn format_violation_list(
    violations: &[crate::entropy::violation_detector::ActionableViolation],
) -> String {
    violations
        .iter()
        .enumerate()
        .map(|(i, v)| {
            format!(
                "{}. {} (saves {} lines)\n   Fix: {}",
                i + 1,
                v.message,
                v.estimated_loc_reduction,
                v.fix_suggestion
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Format violations for markdown output
fn format_markdown_violations(
    violations: &[crate::entropy::violation_detector::ActionableViolation],
    max_count: usize,
) -> String {
    violations
        .iter()
        .take(max_count)
        .map(|v| {
            format!(
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
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Output entropy results to file or stdout
fn output_entropy_results(output: Option<std::path::PathBuf>, content: &str) -> Result<()> {
    use std::fs;

    if let Some(output_path) = output {
        fs::write(output_path, content)?;
    } else {
        println!("{content}");
    }

    Ok(())
}

/// Index workspace and validate document count
fn index_workspace(
    engine: &mut crate::services::local_semantic::LocalSemanticEngine,
    workspace: &Path,
    language: Option<&str>,
) -> Result<usize> {
    println!("🔍 Indexing source files...");
    let num_docs = engine
        .index_directory(workspace, language)
        .map_err(|e| anyhow::anyhow!("Failed to index directory: {}", e))?;
    if num_docs == 0 {
        anyhow::bail!("No source files found to analyze");
    }
    println!("📁 Indexed {} source files", num_docs);
    Ok(num_docs)
}

/// Output clustering results in the requested format
fn output_cluster_results(
    result: &crate::services::local_semantic::LocalClusterResult,
    format: &crate::cli::enums::OutputFormat,
) -> Result<()> {
    match format {
        crate::cli::enums::OutputFormat::Json => {
            let json_output = serde_json::json!({
                "method": result.method,
                "num_documents": result.num_documents,
                "num_clusters": result.clusters.len(),
                "clusters": result.clusters.iter().map(|c| serde_json::json!({
                    "id": c.id, "size": c.size,
                    "files": c.files.iter().map(|f| f.display().to_string()).collect::<Vec<_>>()
                })).collect::<Vec<_>>()
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        _ => {
            println!("\n📊 Clustering Results ({}):", result.method);
            println!("   Documents: {}", result.num_documents);
            println!("   Clusters: {}\n", result.clusters.len());
            for cluster in &result.clusters {
                println!("   Cluster {} ({} files):", cluster.id, cluster.size);
                for file in cluster.files.iter().take(5) {
                    println!("     - {}", file.display());
                }
                if cluster.files.len() > 5 {
                    println!("     ... and {} more", cluster.files.len() - 5);
                }
                println!();
            }
        }
    }
    Ok(())
}

/// Output topic extraction results in the requested format
fn output_topic_results(
    result: &crate::services::local_semantic::LocalTopicResult,
    format: &crate::cli::enums::OutputFormat,
) -> Result<()> {
    match format {
        crate::cli::enums::OutputFormat::Json => {
            let json_output = serde_json::json!({
                "num_documents": result.num_documents,
                "num_topics": result.topics.len(),
                "topics": result.topics.iter().map(|t| serde_json::json!({
                    "id": t.id, "document_count": t.document_count,
                    "top_terms": t.top_terms.iter().map(|(term, weight)| {
                        serde_json::json!({"term": term, "weight": weight})
                    }).collect::<Vec<_>>()
                })).collect::<Vec<_>>()
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        _ => {
            println!("\n📊 Topic Extraction Results:");
            println!("   Documents: {}", result.num_documents);
            println!("   Topics: {}\n", result.topics.len());
            for topic in &result.topics {
                println!(
                    "   Topic {} ({} documents):",
                    topic.id, topic.document_count
                );
                println!("     Top terms:");
                for (term, weight) in topic.top_terms.iter().take(10) {
                    println!("       - {} ({:.3})", term, weight);
                }
                println!();
            }
        }
    }
    Ok(())
}

/// Route semantic analysis commands (PMAT-SEARCH-011)
/// Uses local aprender-based analysis - NO external API required
async fn route_semantic_analysis(cmd: AnalyzeCommands) -> Result<()> {
    use crate::services::local_semantic::LocalSemanticEngine;

    let workspace = std::env::current_dir().unwrap_or_default();
    let mut engine = LocalSemanticEngine::new();

    match cmd {
        AnalyzeCommands::Cluster {
            method,
            k,
            language,
            format,
        } => {
            let method_str = match method {
                crate::cli::commands::ClusterMethod::Kmeans => "kmeans",
                crate::cli::commands::ClusterMethod::Hierarchical => "hierarchical",
                crate::cli::commands::ClusterMethod::Dbscan => "dbscan",
            };
            index_workspace(&mut engine, &workspace, language.as_deref())?;
            println!("🧮 Running {} clustering...", method_str);
            let result = engine
                .cluster(method_str, k)
                .map_err(|e| anyhow::anyhow!("Clustering failed: {}", e))?;
            output_cluster_results(&result, &format)
        }
        AnalyzeCommands::Topics {
            num_topics,
            language,
            format,
        } => {
            index_workspace(&mut engine, &workspace, language.as_deref())?;
            println!("🔬 Extracting {} topics using LDA...", num_topics);
            let result = engine
                .extract_topics(num_topics, language)
                .map_err(|e| anyhow::anyhow!("Topic extraction failed: {}", e))?;
            output_topic_results(&result, &format)
        }
        _ => unreachable!("Expected semantic analysis command"),
    }
}

/// Route MLOps model analysis (PMAT-500)
async fn route_model_analysis(cmd: AnalyzeCommands) -> Result<()> {
    use cli::AnalyzeCommands;

    if let AnalyzeCommands::Models {
        path,
        format,
        check,
    } = cmd
    {
        let project_path = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());

        let model_files = super::comply_cb_detect::walkdir_model_files(&project_path);

        if model_files.is_empty() {
            println!(
                "No model files found (*.gguf, *.apr, *.safetensors) in {}",
                project_path.display()
            );
            return Ok(());
        }

        // Detect Git LFS patterns
        let lfs_patterns = detect_lfs_patterns(&project_path);

        // Collect metadata for each model file
        let mut entries: Vec<ModelInventoryEntry> = Vec::new();
        let mut total_size: u64 = 0;

        for file_path in &model_files {
            let file_size = std::fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);
            total_size += file_size;

            let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let format_name = super::comply_cb_detect::ModelFormat::from_extension(ext)
                .map(|f| f.name())
                .unwrap_or("Unknown");

            let rel = file_path
                .strip_prefix(&project_path)
                .unwrap_or(file_path)
                .display()
                .to_string();

            let filename = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            entries.push(ModelInventoryEntry {
                file: rel,
                format: format_name.to_string(),
                size_bytes: file_size,
                lfs_tracked: is_lfs_tracked(filename, &lfs_patterns),
            });
        }

        match format {
            cli::OutputFormat::Json => {
                print_model_inventory_json(&entries, total_size)?;
            }
            _ => {
                print_model_inventory_table(&entries, total_size);
            }
        }

        // Optionally run compliance checks
        if check {
            println!();
            let violations = collect_model_violations(&project_path);
            if violations.is_empty() {
                println!("✅ All model files pass quality checks");
            } else {
                for v in &violations {
                    let icon = match v.severity {
                        super::comply_cb_detect::Severity::Error => "❌",
                        super::comply_cb_detect::Severity::Warning => "⚠️",
                        _ => "ℹ️",
                    };
                    println!("{} {}: {} ({})", icon, v.pattern_id, v.description, v.file);
                }
            }
        }

        Ok(())
    } else {
        unreachable!("Expected Models command")
    }
}

struct ModelInventoryEntry {
    file: String,
    format: String,
    size_bytes: u64,
    lfs_tracked: bool,
}

fn format_size(bytes: u64) -> String {
    batuta_common::fmt::format_bytes(bytes)
}

fn print_model_inventory_table(entries: &[ModelInventoryEntry], total_size: u64) {
    let has_lfs = entries.iter().any(|e| e.lfs_tracked);
    let width = if has_lfs { 78 } else { 72 };

    println!(
        "Model Inventory ({} files, {} total)",
        entries.len(),
        format_size(total_size)
    );
    println!("{}", "─".repeat(width));
    if has_lfs {
        println!(
            "{:<40} {:<12} {:>12} {:>6}",
            "File", "Format", "Size", "LFS"
        );
    } else {
        println!("{:<40} {:<12} {:>12}", "File", "Format", "Size");
    }
    println!("{}", "─".repeat(width));
    for entry in entries {
        let display_file = if entry.file.len() > 38 {
            format!("...{}", &entry.file[entry.file.len() - 35..])
        } else {
            entry.file.clone()
        };
        if has_lfs {
            println!(
                "{:<40} {:<12} {:>12} {:>6}",
                display_file,
                entry.format,
                format_size(entry.size_bytes),
                if entry.lfs_tracked { "Yes" } else { "-" }
            );
        } else {
            println!(
                "{:<40} {:<12} {:>12}",
                display_file,
                entry.format,
                format_size(entry.size_bytes)
            );
        }
    }
    println!("{}", "─".repeat(width));
}

fn print_model_inventory_json(entries: &[ModelInventoryEntry], total_size: u64) -> Result<()> {
    let json_entries: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| {
            serde_json::json!({
                "file": e.file,
                "format": e.format,
                "size_bytes": e.size_bytes,
                "size_human": format_size(e.size_bytes),
                "lfs_tracked": e.lfs_tracked,
            })
        })
        .collect();

    let output = serde_json::json!({
        "model_count": entries.len(),
        "total_size_bytes": total_size,
        "total_size_human": format_size(total_size),
        "models": json_entries,
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Parse .gitattributes files to find LFS-tracked patterns
fn detect_lfs_patterns(project_path: &std::path::Path) -> Vec<String> {
    let mut patterns = Vec::new();
    let gitattr_path = project_path.join(".gitattributes");
    if let Ok(content) = std::fs::read_to_string(&gitattr_path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if trimmed.contains("filter=lfs") {
                // Extract the pattern (first whitespace-separated token)
                if let Some(pattern) = trimmed.split_whitespace().next() {
                    patterns.push(pattern.to_string());
                }
            }
        }
    }
    patterns
}

/// Check if a filename matches any LFS glob pattern
fn is_lfs_tracked(filename: &str, lfs_patterns: &[String]) -> bool {
    for pattern in lfs_patterns {
        // Simple glob matching: *.ext
        if let Some(ext_pattern) = pattern.strip_prefix("*.") {
            if let Some(file_ext) = filename.rsplit('.').next() {
                if file_ext.eq_ignore_ascii_case(ext_pattern) {
                    return true;
                }
            }
        } else if pattern == filename {
            return true;
        }
    }
    false
}

fn collect_model_violations(
    project_path: &std::path::Path,
) -> Vec<super::comply_cb_detect::CbPatternViolation> {
    let mut all = Vec::new();
    all.extend(super::comply_cb_detect::detect_cb1000_missing_model_card(
        project_path,
    ));
    all.extend(super::comply_cb_detect::detect_cb1001_oversized_tensor_count(project_path));
    all.extend(super::comply_cb_detect::detect_cb1002_missing_tokenizer(
        project_path,
    ));
    all.extend(super::comply_cb_detect::detect_cb1004_missing_architecture(
        project_path,
    ));
    all.extend(super::comply_cb_detect::detect_cb1005_quantization_mismatch(project_path));
    all.extend(super::comply_cb_detect::detect_cb1006_sharded_without_index(project_path));
    all.extend(super::comply_cb_detect::detect_cb1007_excessive_file_size(
        project_path,
    ));
    all.extend(super::comply_cb_detect::detect_cb1008_apr_missing_crc(
        project_path,
    ));
    all
}

// Tests re-unified via Operation Logical Atomism
#[cfg(test)]
#[path = "analysis_handlers_tests.rs"]
mod tests;
