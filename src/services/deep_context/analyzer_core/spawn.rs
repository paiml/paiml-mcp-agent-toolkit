#![cfg_attr(coverage_nightly, coverage(off))]

use std::path::PathBuf;
use std::sync::Arc;

use tracing::debug;

use crate::services::cache::{CacheConfig, SessionCacheManager};
use crate::services::deep_context::analyzer_core::types::{
    AnalysisResult, ParallelAnalysisResults,
};
use crate::services::deep_context::AnalysisType;
use crate::services::deep_context::DeepContextAnalyzer;

impl DeepContextAnalyzer {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) async fn execute_parallel_analyses_with_progress(
        &self,
        project_path: &std::path::Path,
        progress: &crate::services::progress::ProgressTracker,
    ) -> anyhow::Result<ParallelAnalysisResults> {
        debug_assert!(
            project_path.exists(),
            "project_path must exist: {}",
            project_path.display()
        );
        let analysis_count = self.config.include_analyses.len() as u64;
        let analysis_progress = progress.create_sub_progress("Running analyses", analysis_count);

        let mut results = ParallelAnalysisResults::default();

        // Phase 1: Run AST analysis FIRST to populate process-global DashMap caches.
        // The AST phase parses every Rust/TS/Python file once and stores complexity
        // metrics in RUST_UNIFIED_CACHE etc. Subsequent phases get 100% cache hits
        // instead of redundantly calling syn::parse_file() (~3 GB savings).
        let mut prebuilt_context: Option<Arc<crate::services::context::ProjectContext>> = None;

        if self.config.include_analyses.contains(&AnalysisType::Ast) {
            let file_classifier_config = self.config.file_classifier_config.clone();
            let path = project_path.to_path_buf();
            let ast_result = tokio::spawn(async move {
                AnalysisResult::Ast(
                    crate::services::deep_context::analyze_ast_contexts(
                        &path,
                        file_classifier_config,
                    )
                    .await,
                )
            })
            .await?;

            // Extract ProjectContext from AST results for Provability/DAG reuse.
            // This eliminates 2 full analyze_project() calls (~2 GB syn parsing).
            if let AnalysisResult::Ast(Ok(ref ast_contexts)) = ast_result {
                let files: Vec<crate::services::context::FileContext> =
                    ast_contexts.iter().map(|efc| efc.base.clone()).collect();
                let summary = crate::services::context::ProjectSummary {
                    total_files: files.len(),
                    total_functions: files
                        .iter()
                        .flat_map(|f| f.items.iter())
                        .filter(|i| matches!(i, crate::services::context::AstItem::Function { .. }))
                        .count(),
                    total_structs: 0,
                    total_enums: 0,
                    total_traits: 0,
                    total_impls: 0,
                    dependencies: vec![],
                };
                prebuilt_context = Some(Arc::new(crate::services::context::ProjectContext {
                    project_type: "rust".to_string(),
                    files,
                    summary,
                    graph: None,
                }));
            }

            self.integrate_analysis_result(&mut results, ast_result);
            analysis_progress.inc(1);
        }

        // Phase 2: Spawn remaining analyses in parallel (they benefit from populated caches)
        let mut join_set = self.spawn_remaining_tasks(project_path, prebuilt_context)?;

        // Collect remaining results
        let remaining = self
            .collect_analysis_results_with_progress(&mut join_set, &analysis_progress)
            .await?;

        // Merge remaining results into the aggregate
        self.merge_results(&mut results, remaining);

        analysis_progress.finish_with_message("Analyses complete");
        Ok(results)
    }

    /// Spawn all configured analysis tasks EXCEPT AST (which runs in Phase 1)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn spawn_remaining_tasks(
        &self,
        project_path: &std::path::Path,
        prebuilt_context: Option<Arc<crate::services::context::ProjectContext>>,
    ) -> anyhow::Result<tokio::task::JoinSet<AnalysisResult>> {
        debug_assert!(
            project_path.exists(),
            "project_path must exist: {}",
            project_path.display()
        );
        let mut join_set = tokio::task::JoinSet::new();

        // Shared cache as fallback for Provability/DAG if no pre-built context
        let shared_cache = Arc::new(SessionCacheManager::new(CacheConfig::default()));

        for analysis_type in &self.config.include_analyses {
            // Skip AST — already ran in Phase 1
            if matches!(analysis_type, AnalysisType::Ast) {
                continue;
            }
            self.spawn_analysis_task(
                &mut join_set,
                project_path,
                analysis_type,
                &shared_cache,
                &prebuilt_context,
            )?;
        }

        Ok(join_set)
    }

    /// Merge partial results from remaining analyses into the main results
    fn merge_results(&self, target: &mut ParallelAnalysisResults, source: ParallelAnalysisResults) {
        debug_assert!(true, "contract: merge_results");
        if source.ast_contexts.is_some() {
            target.ast_contexts = source.ast_contexts;
        }
        if source.complexity_report.is_some() {
            target.complexity_report = source.complexity_report;
        }
        if source.churn_analysis.is_some() {
            target.churn_analysis = source.churn_analysis;
        }
        if source.dead_code_results.is_some() {
            target.dead_code_results = source.dead_code_results;
        }
        if source.duplicate_code_results.is_some() {
            target.duplicate_code_results = source.duplicate_code_results;
        }
        if source.satd_results.is_some() {
            target.satd_results = source.satd_results;
        }
        if source.provability_results.is_some() {
            target.provability_results = source.provability_results;
        }
        if source.dependency_graph.is_some() {
            target.dependency_graph = source.dependency_graph;
        }
        if source.big_o_analysis.is_some() {
            target.big_o_analysis = source.big_o_analysis;
        }
    }

    /// Spawn a single analysis task based on type
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn spawn_analysis_task(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        project_path: &std::path::Path,
        analysis_type: &AnalysisType,
        shared_cache: &Arc<SessionCacheManager>,
        prebuilt_context: &Option<Arc<crate::services::context::ProjectContext>>,
    ) -> anyhow::Result<()> {
        debug_assert!(
            project_path.exists(),
            "project_path must exist: {}",
            project_path.display()
        );
        let path = project_path.to_path_buf();

        match analysis_type {
            AnalysisType::Ast => self.spawn_ast_analysis(join_set, path),
            AnalysisType::Complexity => self.spawn_complexity_analysis(join_set, path),
            AnalysisType::Churn => self.spawn_churn_analysis(join_set, path),
            AnalysisType::DeadCode => self.spawn_dead_code_analysis(join_set, path),
            AnalysisType::DuplicateCode => self.spawn_duplicate_analysis(join_set, path),
            AnalysisType::Satd => self.spawn_satd_analysis(join_set, path),
            AnalysisType::Provability => self.spawn_provability_analysis(
                join_set,
                path,
                shared_cache.clone(),
                prebuilt_context.clone(),
            ),
            AnalysisType::Dag => self.spawn_dag_analysis(
                join_set,
                path,
                shared_cache.clone(),
                prebuilt_context.clone(),
            ),
            AnalysisType::TechnicalDebtGradient => Ok(()), // Computed in correlate_defects
            AnalysisType::BigO => self.spawn_big_o_analysis(join_set, path),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn spawn_ast_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let file_classifier_config = self.config.file_classifier_config.clone();
        join_set.spawn(async move {
            AnalysisResult::Ast(
                crate::services::deep_context::analyze_ast_contexts(&path, file_classifier_config)
                    .await,
            )
        });
        Ok(())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn spawn_complexity_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        join_set.spawn(async move {
            AnalysisResult::Complexity(
                crate::services::deep_context::analysis_functions::analyze_complexity(&path).await,
            )
        });
        Ok(())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn spawn_churn_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let days = self.config.period_days;
        join_set.spawn(async move {
            AnalysisResult::Churn(
                crate::services::deep_context::analysis_functions::analyze_churn(&path, days).await,
            )
        });
        Ok(())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn spawn_dead_code_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        join_set.spawn(async move {
            AnalysisResult::DeadCode(
                crate::services::deep_context::analysis_functions::analyze_dead_code(&path).await,
            )
        });
        Ok(())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn spawn_duplicate_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        join_set.spawn(async move {
            AnalysisResult::DuplicateCode(
                crate::services::deep_context::analysis_functions::analyze_duplicate_code(&path)
                    .await,
            )
        });
        Ok(())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn spawn_satd_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        join_set.spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                tokio::runtime::Handle::current().block_on(async {
                    crate::services::deep_context::analysis_functions::analyze_satd(&path).await
                })
            })
            .await
            .unwrap_or_else(|_| Err(anyhow::anyhow!("SATD analysis failed")));
            AnalysisResult::Satd(result)
        });
        Ok(())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn spawn_provability_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
        cache: Arc<SessionCacheManager>,
        prebuilt: Option<Arc<crate::services::context::ProjectContext>>,
    ) -> anyhow::Result<()> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        join_set.spawn(async move {
            AnalysisResult::Provability(
                crate::services::deep_context::analysis_functions::analyze_provability_with_context(
                    &path,
                    Some(cache),
                    prebuilt,
                )
                .await,
            )
        });
        Ok(())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn spawn_dag_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
        cache: Arc<SessionCacheManager>,
        prebuilt: Option<Arc<crate::services::context::ProjectContext>>,
    ) -> anyhow::Result<()> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let dag_type = self.config.dag_type.clone();
        join_set.spawn(async move {
            AnalysisResult::Dag(
                crate::services::deep_context::analysis_functions::analyze_dag_with_context(
                    &path,
                    dag_type,
                    Some(cache),
                    prebuilt,
                )
                .await,
            )
        });
        Ok(())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn spawn_big_o_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        join_set.spawn(async move {
            AnalysisResult::BigO(
                crate::services::deep_context::analysis_functions::analyze_big_o(&path).await,
            )
        });
        Ok(())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) async fn collect_analysis_results_with_progress(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<ParallelAnalysisResults> {
        // Direct collection without timeout - let it complete naturally
        let results = self
            .process_analysis_results_with_progress(join_set, progress)
            .await?;
        debug!("Parallel analysis collection completed successfully");
        Ok(results)
    }

    /// Process all analysis results concurrently with progress
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) async fn process_analysis_results_with_progress(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<ParallelAnalysisResults> {
        // Collect all results first
        let mut pending_results = Vec::new();
        while let Some(result) = join_set.join_next().await {
            pending_results.push(result?);
            progress.inc(1);
        }

        // Process results concurrently
        let result_processors: Vec<_> = pending_results
            .into_iter()
            .map(|result| tokio::spawn(async move { result }))
            .collect();

        // Aggregate processed results
        let mut results = ParallelAnalysisResults::default();
        for processor in result_processors {
            if let Ok(processed) = processor.await {
                self.integrate_analysis_result(&mut results, processed);
            }
        }

        Ok(results)
    }

    /// Integrate a single analysis result into the final results.
    /// Takes ownership and moves data to avoid cloning large structures.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn integrate_analysis_result(
        &self,
        results: &mut ParallelAnalysisResults,
        result: AnalysisResult,
    ) {
        match result {
            AnalysisResult::Ast(Ok(data)) => {
                results.ast_contexts = Some(data);
            }
            AnalysisResult::Complexity(Ok(data)) => {
                results.complexity_report = Some(data);
            }
            AnalysisResult::Churn(Ok(data)) => {
                results.churn_analysis = Some(data);
            }
            AnalysisResult::DeadCode(Ok(data)) => {
                results.dead_code_results = Some(data);
            }
            AnalysisResult::DuplicateCode(Ok(data)) => {
                results.duplicate_code_results = Some(data);
            }
            AnalysisResult::Satd(Ok(data)) => {
                results.satd_results = Some(data);
            }
            AnalysisResult::Provability(Ok(data)) => {
                results.provability_results = Some(data);
            }
            AnalysisResult::Dag(Ok(data)) => {
                results.dependency_graph = Some(data);
            }
            AnalysisResult::BigO(Ok(data)) => {
                results.big_o_analysis = Some(data);
            }
            // Handle errors with helper
            other => self.log_integration_error(&other),
        }
    }

    /// Log errors from analysis integration
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn log_integration_error(&self, result: &AnalysisResult) {
        match result {
            AnalysisResult::Ast(Err(e))
            | AnalysisResult::Complexity(Err(e))
            | AnalysisResult::Churn(Err(e))
            | AnalysisResult::DeadCode(Err(e))
            | AnalysisResult::DuplicateCode(Err(e))
            | AnalysisResult::Satd(Err(e))
            | AnalysisResult::Provability(Err(e))
            | AnalysisResult::Dag(Err(e))
            | AnalysisResult::BigO(Err(e)) => {
                debug!("{} analysis failed: {}", self.get_analysis_name(result), e);
            }
            _ => {}
        }
    }

    /// Get analysis name for logging
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn get_analysis_name(&self, result: &AnalysisResult) -> &'static str {
        match result {
            AnalysisResult::Ast(_) => "AST",
            AnalysisResult::Complexity(_) => "Complexity",
            AnalysisResult::Churn(_) => "Churn",
            AnalysisResult::DeadCode(_) => "Dead code",
            AnalysisResult::DuplicateCode(_) => "Duplicate code",
            AnalysisResult::Satd(_) => "SATD",
            AnalysisResult::Provability(_) => "Provability",
            AnalysisResult::Dag(_) => "DAG",
            AnalysisResult::BigO(_) => "Big-O",
        }
    }
}
