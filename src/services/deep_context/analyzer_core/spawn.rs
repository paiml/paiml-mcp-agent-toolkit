#![cfg_attr(coverage_nightly, coverage(off))]

use std::path::PathBuf;

use tracing::debug;

use crate::services::deep_context::AnalysisType;
use crate::services::deep_context::analyzer_core::types::{AnalysisResult, ParallelAnalysisResults};
use crate::services::deep_context::DeepContextAnalyzer;

impl DeepContextAnalyzer {
    pub(crate) async fn execute_parallel_analyses_with_progress(
        &self,
        project_path: &std::path::Path,
        progress: &crate::services::progress::ProgressTracker,
    ) -> anyhow::Result<ParallelAnalysisResults> {
        // Step 1: Spawn all analysis tasks with progress tracking
        let mut join_set = self.spawn_analysis_tasks(project_path)?;

        // Create sub-progress bars for different analyses
        let analysis_count = self.config.include_analyses.len() as u64;
        let analysis_progress = progress.create_sub_progress("Running analyses", analysis_count);

        // Step 2: Collect and process results - NO TIMEOUT!
        let results = self
            .collect_analysis_results_with_progress(&mut join_set, &analysis_progress)
            .await?;

        analysis_progress.finish_with_message("Analyses complete");
        Ok(results)
    }

    /// Spawn all configured analysis tasks
    pub(crate) fn spawn_analysis_tasks(
        &self,
        project_path: &std::path::Path,
    ) -> anyhow::Result<tokio::task::JoinSet<AnalysisResult>> {
        let mut join_set = tokio::task::JoinSet::new();

        for analysis_type in &self.config.include_analyses {
            self.spawn_analysis_task(&mut join_set, project_path, analysis_type)?;
        }

        Ok(join_set)
    }

    /// Spawn a single analysis task based on type
    pub(crate) fn spawn_analysis_task(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        project_path: &std::path::Path,
        analysis_type: &AnalysisType,
    ) -> anyhow::Result<()> {
        let path = project_path.to_path_buf();

        match analysis_type {
            AnalysisType::Ast => self.spawn_ast_analysis(join_set, path),
            AnalysisType::Complexity => self.spawn_complexity_analysis(join_set, path),
            AnalysisType::Churn => self.spawn_churn_analysis(join_set, path),
            AnalysisType::DeadCode => self.spawn_dead_code_analysis(join_set, path),
            AnalysisType::DuplicateCode => self.spawn_duplicate_analysis(join_set, path),
            AnalysisType::Satd => self.spawn_satd_analysis(join_set, path),
            AnalysisType::Provability => self.spawn_provability_analysis(join_set, path),
            AnalysisType::Dag => self.spawn_dag_analysis(join_set, path),
            AnalysisType::TechnicalDebtGradient => Ok(()), // Computed in correlate_defects
            AnalysisType::BigO => self.spawn_big_o_analysis(join_set, path),
        }
    }

    pub(crate) fn spawn_ast_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        let file_classifier_config = self.config.file_classifier_config.clone();
        join_set.spawn(async move {
            AnalysisResult::Ast(
                crate::services::deep_context::analyze_ast_contexts(&path, file_classifier_config)
                    .await,
            )
        });
        Ok(())
    }

    pub(crate) fn spawn_complexity_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        join_set.spawn(async move {
            AnalysisResult::Complexity(
                crate::services::deep_context::analysis_functions::analyze_complexity(&path).await,
            )
        });
        Ok(())
    }

    pub(crate) fn spawn_churn_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        let days = self.config.period_days;
        join_set.spawn(async move {
            AnalysisResult::Churn(
                crate::services::deep_context::analysis_functions::analyze_churn(&path, days)
                    .await,
            )
        });
        Ok(())
    }

    pub(crate) fn spawn_dead_code_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        join_set.spawn(async move {
            AnalysisResult::DeadCode(
                crate::services::deep_context::analysis_functions::analyze_dead_code(&path).await,
            )
        });
        Ok(())
    }

    pub(crate) fn spawn_duplicate_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        join_set.spawn(async move {
            AnalysisResult::DuplicateCode(
                crate::services::deep_context::analysis_functions::analyze_duplicate_code(&path)
                    .await,
            )
        });
        Ok(())
    }

    pub(crate) fn spawn_satd_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
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

    pub(crate) fn spawn_provability_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        join_set.spawn(async move {
            AnalysisResult::Provability(
                crate::services::deep_context::analysis_functions::analyze_provability(&path)
                    .await,
            )
        });
        Ok(())
    }

    pub(crate) fn spawn_dag_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        let dag_type = self.config.dag_type.clone();
        join_set.spawn(async move {
            AnalysisResult::Dag(
                crate::services::deep_context::analysis_functions::analyze_dag(&path, dag_type)
                    .await,
            )
        });
        Ok(())
    }

    pub(crate) fn spawn_big_o_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        join_set.spawn(async move {
            AnalysisResult::BigO(
                crate::services::deep_context::analysis_functions::analyze_big_o(&path).await,
            )
        });
        Ok(())
    }

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

    /// Integrate a single analysis result into the final results
    pub(crate) fn integrate_analysis_result(
        &self,
        results: &mut ParallelAnalysisResults,
        result: AnalysisResult,
    ) {
        match &result {
            AnalysisResult::Ast(Ok(data)) => {
                results.ast_contexts = Some(data.clone());
            }
            AnalysisResult::Complexity(Ok(data)) => {
                results.complexity_report = Some(data.clone());
            }
            AnalysisResult::Churn(Ok(data)) => {
                results.churn_analysis = Some(data.clone());
            }
            AnalysisResult::DeadCode(Ok(data)) => {
                results.dead_code_results = Some(data.clone());
            }
            AnalysisResult::DuplicateCode(Ok(data)) => {
                results.duplicate_code_results = Some(data.clone());
            }
            AnalysisResult::Satd(Ok(data)) => {
                results.satd_results = Some(data.clone());
            }
            AnalysisResult::Provability(Ok(data)) => {
                results.provability_results = Some(data.clone());
            }
            AnalysisResult::Dag(Ok(data)) => {
                results.dependency_graph = Some(data.clone());
            }
            AnalysisResult::BigO(Ok(data)) => {
                results.big_o_analysis = Some(data.clone());
            }
            // Handle errors with helper
            _ => self.log_integration_error(&result),
        }
    }

    /// Log errors from analysis integration
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
