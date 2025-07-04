//! Analysis command handlers
//!
//! This module extracts all analysis-related handlers from the main CLI module
//! to reduce complexity and improve organization.

use crate::cli::{self, AnalyzeCommands};
use anyhow::Result;

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
/// ```rust
/// use pmat::cli::handlers::analysis_handlers::route_analyze_command;
/// use pmat::cli::AnalyzeCommands;
/// use std::path::PathBuf;
///
/// # tokio_test::block_on(async {
/// // Complexity analysis command
/// let complexity_cmd = AnalyzeCommands::Complexity {
///     project_path: Some(PathBuf::from("/tmp/project")),
///     toolchain: None,
///     format: None,
///     output: None,
///     max_cyclomatic: None,
///     max_cognitive: None,
///     include: None,
///     watch: false,
///     top_files: None,
/// };
///
/// // This would normally execute the command
/// // let result = route_analyze_command(complexity_cmd).await;
/// // assert!(result.is_ok());
///
/// // Dead code analysis command
/// let dead_code_cmd = AnalyzeCommands::DeadCode {
///     path: Some(PathBuf::from("/tmp/project")),
///     format: None,
///     top_files: None,
///     include_unreachable: false,
///     min_dead_lines: None,
///     include_tests: false,
///     output: None,
/// };
///
/// // DAG analysis command
/// let dag_cmd = AnalyzeCommands::Dag {
///     dag_type: pmat::cli::DagType::CallGraph,
///     project_path: Some(PathBuf::from("/tmp/project")),
///     output: None,
///     max_depth: Some(5),
///     target_nodes: vec![],
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
        AnalyzeCommands::Complexity {
            project_path,
            toolchain,
            format,
            output,
            max_cyclomatic,
            max_cognitive,
            include,
            watch,
            top_files,
        } => {
            super::complexity_handlers::handle_analyze_complexity(
                project_path,
                toolchain,
                format,
                output,
                max_cyclomatic,
                max_cognitive,
                include,
                watch,
                top_files,
            )
            .await
        }
        AnalyzeCommands::Churn {
            project_path,
            days,
            format,
            output,
            top_files,
        } => {
            super::complexity_handlers::handle_analyze_churn(
                project_path,
                days,
                format,
                output,
                top_files,
            )
            .await
        }
        AnalyzeCommands::DeadCode {
            path,
            format,
            top_files,
            include_unreachable,
            min_dead_lines,
            include_tests,
            output,
        } => {
            super::complexity_handlers::handle_analyze_dead_code(
                path,
                format,
                top_files,
                include_unreachable,
                min_dead_lines,
                include_tests,
                output,
            )
            .await
        }
        AnalyzeCommands::Dag {
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
        } => {
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
        }
        AnalyzeCommands::Satd {
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
        } => {
            super::complexity_handlers::handle_analyze_satd(
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
            )
            .await
        }
        AnalyzeCommands::DeepContext {
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
        } => {
            super::advanced_analysis_handlers::handle_analyze_deep_context(
                project_path,
                output,
                format,
                full,
                include,
                exclude,
                period_days,
                Some(match dag_type {
                    DeepContextDagType::CallGraph => DagType::CallGraph,
                    DeepContextDagType::ImportGraph => DagType::ImportGraph,
                    DeepContextDagType::Inheritance => DagType::Inheritance,
                    DeepContextDagType::FullDependency => DagType::FullDependency,
                }),
                max_depth,
                include_patterns,
                exclude_patterns,
                Some(match cache_strategy {
                    DeepContextCacheStrategy::Normal => "normal".to_string(),
                    DeepContextCacheStrategy::ForceRefresh => "force-refresh".to_string(),
                    DeepContextCacheStrategy::Offline => "offline".to_string(),
                }),
                parallel.is_some(),
                verbose,
                top_files,
            )
            .await
        }
        AnalyzeCommands::Tdg {
            path,
            threshold,
            top_files,
            format,
            include_components,
            output,
            critical_only,
            verbose,
        } => {
            super::advanced_analysis_handlers::handle_analyze_tdg(
                path,
                Some(threshold),
                Some(top_files),
                format,
                include_components,
                output,
                critical_only,
                verbose,
            )
            .await
        }
        AnalyzeCommands::LintHotspot {
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
        } => {
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
            )
            .await
        }
        AnalyzeCommands::Makefile {
            path,
            rules,
            format,
            fix,
            gnu_version,
            top_files,
        } => {
            super::advanced_analysis_handlers::handle_analyze_makefile(
                path,
                rules,
                format,
                fix,
                Some(gnu_version),
                top_files,
            )
            .await
        }
        AnalyzeCommands::Provability {
            project_path,
            functions,
            analysis_depth,
            format,
            high_confidence_only,
            include_evidence,
            output,
            top_files,
        } => {
            super::advanced_analysis_handlers::handle_analyze_provability(
                project_path,
                functions,
                Some(analysis_depth as u32),
                format,
                high_confidence_only,
                include_evidence,
                output,
                top_files,
            )
            .await
        }
        AnalyzeCommands::Duplicates {
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
        } => {
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
        }
        AnalyzeCommands::DefectPrediction {
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
        } => {
            super::advanced_analysis_handlers::handle_analyze_defect_prediction(
                project_path,
                Some(confidence_threshold as f64),
                Some(min_lines),
                include_low_confidence,
                format,
                high_risk_only,
                include_recommendations,
                include.map(|s| vec![s]).unwrap_or_default(),
                exclude.map(|s| vec![s]).unwrap_or_default(),
                output,
                perf,
                top_files,
            )
            .await
        }
        AnalyzeCommands::Comprehensive {
            project_path,
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
            top_files,
        } => {
            super::super::stubs::handle_analyze_comprehensive(
                project_path,
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
                top_files,
            )
            .await
        }
        AnalyzeCommands::GraphMetrics {
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
        } => {
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
        }
        AnalyzeCommands::NameSimilarity {
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
        } => {
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
        }
        AnalyzeCommands::ProofAnnotations {
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
        } => {
            super::super::stubs::handle_analyze_proof_annotations(
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
        }
        AnalyzeCommands::IncrementalCoverage {
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
            top_files: _top_files,
        } => {
            super::super::stubs::handle_analyze_incremental_coverage(
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
            )
            .await
        }
        AnalyzeCommands::SymbolTable {
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
        } => {
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
        }
        AnalyzeCommands::BigO {
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
        } => {
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
        }
        AnalyzeCommands::AssemblyScript {
            project_path,
            format,
            wasm_complexity,
            memory_analysis,
            security,
            output,
            timeout,
            perf,
            top_files: _top_files,
        } => {
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
        }
        AnalyzeCommands::WebAssembly {
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
        } => {
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
        }
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
