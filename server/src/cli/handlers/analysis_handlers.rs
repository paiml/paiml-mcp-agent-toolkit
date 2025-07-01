//! Analysis command handlers
//!
//! This module extracts all analysis-related handlers from the main CLI module
//! to reduce complexity and improve organization.

use crate::cli::{self, AnalyzeCommands};
use anyhow::Result;

/// Router for all analysis commands
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
        } => {
            super::complexity_handlers::handle_analyze_churn(project_path, days, format, output)
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
            evolution,
            days,
            metrics,
            output,
        } => {
            super::complexity_handlers::handle_analyze_satd(
                path,
                format,
                severity,
                critical_only,
                include_tests,
                evolution,
                days,
                metrics,
                output,
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
            )
            .await
        }
        AnalyzeCommands::Tdg {
            path,
            threshold,
            top,
            format,
            include_components,
            output,
            critical_only,
            verbose,
        } => {
            super::advanced_analysis_handlers::handle_analyze_tdg(
                path,
                Some(threshold),
                Some(top),
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
            format,
            max_density,
            min_confidence,
            enforce,
            dry_run,
            enforcement_metadata,
            output,
            perf,
            clippy_flags,
        } => {
            super::lint_hotspot_handlers::handle_analyze_lint_hotspot(
                project_path,
                format,
                max_density,
                min_confidence,
                enforce,
                dry_run,
                enforcement_metadata,
                output,
                perf,
                clippy_flags,
            )
            .await
        }
        AnalyzeCommands::Makefile {
            path,
            rules,
            format,
            fix,
            gnu_version,
        } => {
            super::advanced_analysis_handlers::handle_analyze_makefile(
                path,
                rules,
                format,
                fix,
                Some(gnu_version),
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
        } => {
            super::advanced_analysis_handlers::handle_analyze_provability(
                project_path,
                functions,
                Some(analysis_depth as u32),
                format,
                high_confidence_only,
                include_evidence,
                output,
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
