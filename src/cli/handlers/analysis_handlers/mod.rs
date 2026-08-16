//! Analysis command handlers
//!
//! This module extracts all analysis-related handlers from the main CLI module
//! to reduce complexity and improve organization.
#![cfg_attr(coverage_nightly, coverage(off))]

mod advanced_routes;
mod core_routes;
mod entropy_semantic;
pub mod perf_report;
mod platform_routes;

use crate::cli::{self, AnalyzeCommands};
use anyhow::Result;

/// Refuse an `--ml` flag whose scorer is not wired into the handler (GH-97).
///
/// ONE RULE, TWO COMMANDS. `analyze complexity --ml` and `analyze tdg --ml` both
/// promised "trained ML models instead of heuristic weighted sums" and both
/// destructured the flag into `_`, so each returned the heuristic numbers under
/// an ML banner — a relabelling, not a different result. `complexity` was fixed
/// with a bail! written inline and `tdg` was left behind; the refusal lives in
/// one place now so the next `--ml` cannot drift from it.
///
/// # Errors
/// Always, when `ml` is set. That is the point: an honest refusal beats a
/// silent no-op.
pub(super) fn reject_unimplemented_ml(ml: bool, command: &str, scores: &str) -> Result<()> {
    if !ml {
        return Ok(());
    }
    anyhow::bail!(
        "--ml is not implemented: {scores} are still computed by the heuristic formulas, \
         so this flag would relabel them without changing them. \
         Re-run `{command}` without --ml (see GH-97)."
    )
}

#[cfg(test)]
pub(crate) use advanced_routes::{convert_cache_strategy, convert_deep_context_dag_type};
#[cfg(test)]
pub(crate) use entropy_semantic::{
    create_entropy_config, format_markdown_violations, format_violation_list, get_top_violations,
    output_entropy_results,
};

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
/// ```ignore
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn route_analyze_command(cmd: AnalyzeCommands) -> Result<()> {
    // `--perf` has exactly one implementation, and it lives here. Nine of the
    // thirteen subcommands that advertise the flag used to drop it on the floor
    // (see `perf_report` for the roll-call), so `--perf` produced byte-identical
    // output. Timing the dispatch is the only way to make that impossible: a
    // subcommand cannot forget to honour a flag it never sees.
    let perf_label = perf_report::perf_command_label(&cmd);
    let started = std::time::Instant::now();

    let result = dispatch_analyze_command(cmd).await;

    if let Some(label) = perf_label {
        perf_report::emit(label, started.elapsed());
    }

    result
}

/// Dispatch to the per-family routers. Split out of [`route_analyze_command`]
/// so the `--perf` measurement wraps every analyze subcommand exactly once.
async fn dispatch_analyze_command(cmd: AnalyzeCommands) -> Result<()> {
    use cli::AnalyzeCommands;

    match cmd {
        // Core analysis commands
        AnalyzeCommands::Bottleneck { .. }
        | AnalyzeCommands::Complexity { .. }
        | AnalyzeCommands::Churn { .. }
        | AnalyzeCommands::DeadCode { .. }
        | AnalyzeCommands::Defects { .. }
        | AnalyzeCommands::Dag { .. }
        | AnalyzeCommands::Satd { .. } => route_core_analysis(cmd).await,

        AnalyzeCommands::Reachability { .. } => route_reachability(cmd).await,

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
        AnalyzeCommands::DeepWasm { .. } => platform_routes::route_deep_wasm_analysis(cmd).await,

        // Mutation testing (feature-gated)
        #[cfg(feature = "mutation-testing")]
        AnalyzeCommands::Mutate { .. } => platform_routes::route_mutation_testing(cmd).await,

        // System commands
        AnalyzeCommands::Makefile { .. } => route_system_analysis(cmd).await,

        // Semantic analysis commands (PMAT-SEARCH-011)
        AnalyzeCommands::Cluster { .. } | AnalyzeCommands::Topics { .. } => {
            entropy_semantic::route_semantic_analysis(cmd).await
        }

        // MLOps model analysis (PMAT-500)
        AnalyzeCommands::Models { .. } => platform_routes::route_model_analysis(cmd).await,
    }
}

/// Route core analysis commands
async fn route_core_analysis(cmd: AnalyzeCommands) -> Result<()> {
    match cmd {
        AnalyzeCommands::Bottleneck { .. } => core_routes::route_bottleneck_analysis(cmd).await,
        AnalyzeCommands::Complexity { .. } => core_routes::route_complexity_analysis(cmd).await,
        AnalyzeCommands::Churn { .. } => core_routes::route_churn_analysis(cmd).await,
        AnalyzeCommands::DeadCode { .. } => core_routes::route_dead_code_analysis(cmd).await,
        AnalyzeCommands::Defects { .. } => core_routes::route_defects_analysis(cmd).await,
        AnalyzeCommands::Dag { .. } => core_routes::route_dag_analysis(cmd).await,
        AnalyzeCommands::Satd { .. } => core_routes::route_satd_analysis(cmd).await,
        _ => unreachable!("Expected core analysis command"),
    }
}

/// Route advanced analysis commands
async fn route_advanced_analysis(cmd: AnalyzeCommands) -> Result<()> {
    match cmd {
        AnalyzeCommands::DeepContext { .. } => {
            advanced_routes::route_deep_context_analysis(cmd).await
        }
        AnalyzeCommands::Tdg { .. } => advanced_routes::route_tdg_analysis(cmd).await,
        AnalyzeCommands::BuildTdg { .. } => advanced_routes::route_build_tdg_analysis(cmd).await,
        AnalyzeCommands::LintHotspot { .. } => {
            advanced_routes::route_lint_hotspot_analysis(cmd).await
        }
        AnalyzeCommands::Comprehensive { .. } => {
            advanced_routes::route_comprehensive_analysis(cmd).await
        }
        _ => unreachable!("Expected advanced analysis command"),
    }
}

/// Route quality analysis commands
async fn route_quality_analysis(cmd: AnalyzeCommands) -> Result<()> {
    match cmd {
        AnalyzeCommands::Duplicates { .. } => advanced_routes::route_duplicates_analysis(cmd).await,
        AnalyzeCommands::DefectPrediction { .. } => {
            advanced_routes::route_defect_prediction_analysis(cmd).await
        }
        AnalyzeCommands::Provability { .. } => {
            advanced_routes::route_provability_analysis(cmd).await
        }
        AnalyzeCommands::Clippy { .. } => advanced_routes::route_clippy_analysis(cmd).await,
        AnalyzeCommands::Entropy { .. } => entropy_semantic::route_entropy_analysis(cmd).await,
        _ => unreachable!("Expected quality analysis command"),
    }
}

/// Route specialized analysis commands
async fn route_specialized_analysis(cmd: AnalyzeCommands) -> Result<()> {
    match cmd {
        AnalyzeCommands::GraphMetrics { .. } => {
            platform_routes::route_graph_metrics_analysis(cmd).await
        }
        AnalyzeCommands::NameSimilarity { .. } => {
            platform_routes::route_name_similarity_analysis(cmd).await
        }
        AnalyzeCommands::ProofAnnotations { .. } => {
            platform_routes::route_proof_annotations_analysis(cmd).await
        }
        AnalyzeCommands::IncrementalCoverage { .. } => {
            platform_routes::route_incremental_coverage_analysis(cmd).await
        }
        AnalyzeCommands::CoverageImprove {
            path,
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
            let path = project_path.unwrap_or(path);
            crate::cli::handlers::coverage_improve_handler::handle_coverage_improve(
                path,
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
        AnalyzeCommands::SymbolTable { .. } => {
            platform_routes::route_symbol_table_analysis(cmd).await
        }
        AnalyzeCommands::BigO { .. } => platform_routes::route_big_o_analysis(cmd).await,
        _ => unreachable!("Expected specialized analysis command"),
    }
}

/// Route language-specific analysis commands
async fn route_language_specific_analysis(cmd: AnalyzeCommands) -> Result<()> {
    match cmd {
        AnalyzeCommands::AssemblyScript { .. } => {
            platform_routes::route_assemblyscript_analysis(cmd).await
        }
        AnalyzeCommands::WebAssembly { .. } => {
            platform_routes::route_webassembly_analysis(cmd).await
        }
        #[cfg(feature = "wasm-ast")]
        AnalyzeCommands::Wasm { .. } => platform_routes::route_wasm_analysis(cmd).await,
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
        AnalyzeCommands::Makefile { .. } => platform_routes::route_makefile_analysis(cmd).await,
        _ => unreachable!("Expected system analysis command"),
    }
}

// Tests re-unified via Operation Logical Atomism
#[cfg(test)]
#[path = "../analysis_handlers_tests.rs"]
mod tests;

#[cfg(test)]
mod ml_refusal_tests {
    //! `analyze complexity --ml` and `analyze tdg --ml` are the same defect:
    //! both promised "trained ML models instead of heuristic weighted sums" and
    //! both threw the flag away, so each printed the heuristic numbers under an
    //! ML banner. The refusal is one function so the two cannot drift.
    use super::reject_unimplemented_ml;

    #[test]
    fn an_unset_flag_is_not_refused() {
        assert!(reject_unimplemented_ml(false, "analyze tdg", "TDG scores").is_ok());
    }

    #[test]
    fn tdg_ml_is_refused_rather_than_relabelled() {
        let err = reject_unimplemented_ml(true, "analyze tdg", "TDG scores")
            .expect_err("--ml returned heuristic scores, so it must not be accepted");
        let err = err.to_string();
        assert!(err.contains("--ml is not implemented"), "{err}");
        assert!(err.contains("TDG scores"), "{err}");
        assert!(err.contains("analyze tdg"), "{err}");
    }

    #[test]
    fn complexity_ml_keeps_its_own_wording() {
        let err = reject_unimplemented_ml(true, "analyze complexity", "complexity scores")
            .unwrap_err()
            .to_string();
        assert!(err.contains("complexity scores"), "{err}");
        assert!(err.contains("analyze complexity"), "{err}");
    }
}

/// Report tracked `.rs` files that no compilation unit reaches.
async fn route_reachability(cmd: cli::AnalyzeCommands) -> anyhow::Result<()> {
    use crate::services::reachability;
    let cli::AnalyzeCommands::Reachability {
        path,
        format,
        fail_on_orphan,
    } = cmd
    else {
        unreachable!("Expected Reachability command")
    };

    let (roots, tracked) = reachability::discover(&path)?;
    if roots.is_empty() {
        // Refuse rather than report "0 orphans" over a tree we never walked:
        // an unmeasured run must not look like a clean one.
        anyhow::bail!(
            "no cargo targets found under {} — `cargo metadata --no-deps` returned none, \
             so reachability could not be measured (this is not a clean result)",
            path.display()
        );
    }
    let report = reachability::analyze(&path, &roots, &tracked);

    if format == "json" {
        println!(
            "{}",
            serde_json::json!({
                "reachable": report.reachable,
                "roots": report.roots,
                "orphan_count": report.orphans.len(),
                "orphan_lines": report.orphan_lines(),
                "orphan_tests": report.orphan_tests(),
                "unresolved_mods": report.unresolved.len(),
                "summary": report.summary(),
                "orphans": report.orphans.iter().map(|o| serde_json::json!({
                    "file": o.path, "lines": o.lines, "tests": o.tests
                })).collect::<Vec<_>>(),
            })
        );
    } else {
        println!("{}", report.summary());
        for o in report.orphans.iter().take(40) {
            println!("  {}  ({} lines, {} tests)", o.path, o.lines, o.tests);
        }
        if report.orphans.len() > 40 {
            println!("  … and {} more", report.orphans.len() - 40);
        }
        for u in report.unresolved.iter().take(10) {
            println!("  unresolved: {u}");
        }
    }

    if fail_on_orphan && !report.orphans.is_empty() {
        std::process::exit(1);
    }
    Ok(())
}
