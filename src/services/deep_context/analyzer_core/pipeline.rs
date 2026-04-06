#![cfg_attr(coverage_nightly, coverage(off))]

use std::path::{Path, PathBuf};

use chrono::Utc;
use rustc_hash::FxHashMap;
use tracing::{debug, info};

use crate::services::deep_context::analyzer_core::params::DeepContextBuildParams;
use crate::services::deep_context::analyzer_core::types::ParallelAnalysisResults;
use crate::services::deep_context::DeepContextAnalyzer;
use crate::services::deep_context::{
    AnalysisResults, AnnotatedFileTree, CrossLangReference, DeepContext, DefectHotspot,
    DefectSummary, PrioritizedRecommendation, QualityScorecard, TemplateProvenance,
};
use crate::services::deep_context::{CacheStats, ContextMetadata};
use crate::services::quality_gates::QAVerificationResult;

impl DeepContextAnalyzer {
    pub async fn analyze_project(&self, project_path: &PathBuf) -> anyhow::Result<DeepContext> {
        debug_assert!(
            project_path.exists(),
            "project_path must exist: {}",
            project_path.display()
        );
        let start_time = std::time::Instant::now();
        info!(
            "Starting deep context analysis for project: {:?}",
            project_path
        );

        // Create progress tracker
        let progress = crate::services::progress::ProgressTracker::new(true);
        let main_progress = progress.create_spinner("Analyzing project...");

        // Execute all analysis phases using extracted methods
        let mut file_tree = self
            .execute_discovery_phase(project_path, &main_progress)
            .await?;
        let analyses = self
            .execute_analysis_phase(project_path, &progress, &main_progress)
            .await?;
        self.enrich_file_tree_if_dag_present(&mut file_tree, &analyses, &main_progress)?;
        let cross_refs = self
            .execute_cross_reference_phase(&analyses, &main_progress)
            .await?;
        let (defect_summary, hotspots) = self
            .execute_defect_correlation_phase(&analyses, &main_progress)
            .await?;
        let quality_scorecard = self
            .execute_quality_scoring_phase(&analyses, &defect_summary, &main_progress)
            .await?;
        let recommendations = self
            .execute_recommendations_phase(&analyses, &defect_summary, &main_progress)
            .await?;
        let template_provenance = self
            .execute_template_provenance_phase(&analyses, &main_progress)
            .await?;
        let (build_info, project_overview) = self
            .execute_metadata_analysis_phase(project_path, &main_progress)
            .await?;

        // Build the deep context from all phases
        let analysis_duration = start_time.elapsed();
        let build_params = DeepContextBuildParams {
            project_path,
            file_tree,
            analyses,
            cross_refs,
            quality_scorecard,
            template_provenance,
            defect_summary,
            hotspots,
            recommendations,
            build_info,
            project_overview,
            analysis_duration,
        };
        let mut deep_context = self.build_deep_context(build_params);

        // Execute final QA verification phase
        deep_context.qa_verification = Some(
            self.execute_qa_verification_phase(&deep_context, &main_progress)
                .await?,
        );

        // Complete progress tracking
        main_progress.finish_with_message("Analysis complete!");
        progress.clear();

        info!("Deep context analysis completed in {:?}", analysis_duration);
        Ok(deep_context)
    }

    // EXTRACTED METHODS - Toyota Way Extract Method Pattern
    // Each method has single responsibility and <10 complexity

    pub(crate) async fn execute_discovery_phase(
        &self,
        project_path: &PathBuf,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<AnnotatedFileTree> {
        debug_assert!(
            project_path.exists(),
            "project_path must exist: {}",
            project_path.display()
        );
        progress.set_message("Discovering project structure...");
        let file_tree = self.discover_project_structure(project_path).await?;
        debug!("Discovery phase completed");
        Ok(file_tree)
    }

    pub(crate) async fn execute_analysis_phase(
        &self,
        project_path: &Path,
        tracker: &crate::services::progress::ProgressTracker,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<ParallelAnalysisResults> {
        debug_assert!(
            project_path.exists(),
            "project_path must exist: {}",
            project_path.display()
        );
        progress.set_message("Running parallel analyses...");
        let analysis_start = std::time::Instant::now();
        let analyses = self
            .execute_parallel_analyses_with_progress(project_path, tracker)
            .await?;
        info!("Analysis phase completed in {:?}", analysis_start.elapsed());
        debug!("Analysis phase completed");
        Ok(analyses)
    }

    pub(crate) fn enrich_file_tree_if_dag_present(
        &self,
        file_tree: &mut AnnotatedFileTree,
        analyses: &ParallelAnalysisResults,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<()> {
        if let Some(ref dag) = analyses.dependency_graph {
            progress.set_message("Enriching file tree with centrality scores...");
            self.enrich_file_tree_with_centrality(file_tree, dag)?;
            debug!("File tree enriched with centrality scores");
        }
        Ok(())
    }

    pub(crate) async fn execute_cross_reference_phase(
        &self,
        analyses: &ParallelAnalysisResults,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<FxHashMap<String, Vec<CrossLangReference>>> {
        progress.set_message("Resolving cross-language references...");
        let cross_refs = self.build_cross_language_references(analyses).await?;
        debug!("Cross-reference resolution completed");
        Ok(cross_refs)
    }

    pub(crate) async fn execute_defect_correlation_phase(
        &self,
        analyses: &ParallelAnalysisResults,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<(DefectSummary, Vec<DefectHotspot>)> {
        progress.set_message("Correlating defects...");
        let (defect_summary, hotspots) = self.correlate_defects(analyses).await?;
        debug!("Defect correlation completed");
        Ok((defect_summary, hotspots))
    }

    pub(crate) async fn execute_quality_scoring_phase(
        &self,
        analyses: &ParallelAnalysisResults,
        defect_summary: &DefectSummary,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<QualityScorecard> {
        progress.set_message("Calculating quality scores...");
        let quality_scorecard = self
            .calculate_quality_scorecard(analyses, defect_summary)
            .await?;
        debug!("Quality scoring completed");
        Ok(quality_scorecard)
    }

    pub(crate) async fn execute_recommendations_phase(
        &self,
        analyses: &ParallelAnalysisResults,
        defect_summary: &DefectSummary,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<Vec<PrioritizedRecommendation>> {
        progress.set_message("Generating recommendations...");
        let recommendations = self
            .generate_recommendations(analyses, defect_summary)
            .await?;
        debug!("Recommendations generated");
        Ok(recommendations)
    }

    pub(crate) async fn execute_template_provenance_phase(
        &self,
        _analyses: &ParallelAnalysisResults,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<Option<TemplateProvenance>> {
        progress.set_message("Analyzing template provenance...");
        // Legacy function - returns None for now
        Ok(None)
    }

    pub(crate) async fn execute_metadata_analysis_phase(
        &self,
        project_path: &Path,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<(
        Option<crate::models::project_meta::BuildInfo>,
        Option<crate::models::project_meta::ProjectOverview>,
    )> {
        debug_assert!(
            project_path.exists(),
            "project_path must exist: {}",
            project_path.display()
        );
        progress.set_message("Analyzing project metadata...");
        let (build_info, project_overview) = self.analyze_project_metadata(project_path).await?;
        debug!("Project metadata analysis completed");
        Ok((build_info, project_overview))
    }

    pub(crate) fn build_deep_context(&self, params: DeepContextBuildParams) -> DeepContext {
        DeepContext {
            metadata: ContextMetadata {
                generated_at: Utc::now(),
                tool_version: env!("CARGO_PKG_VERSION").to_string(),
                project_root: params.project_path.to_path_buf(),
                cache_stats: CacheStats {
                    hit_rate: 0.0,
                    memory_efficiency: 0.0,
                    time_saved_ms: 0,
                },
                analysis_duration: params.analysis_duration,
            },
            file_tree: params.file_tree,
            analyses: AnalysisResults {
                ast_contexts: params.analyses.ast_contexts.unwrap_or_default(),
                complexity_report: params.analyses.complexity_report,
                churn_analysis: params.analyses.churn_analysis,
                dependency_graph: params.analyses.dependency_graph,
                dead_code_results: params.analyses.dead_code_results,
                duplicate_code_results: params.analyses.duplicate_code_results,
                satd_results: params.analyses.satd_results,
                provability_results: params.analyses.provability_results,
                cross_language_refs: params
                    .cross_refs
                    .into_iter()
                    .flat_map(|(_, refs)| refs)
                    .collect(),
                big_o_analysis: params.analyses.big_o_analysis,
            },
            quality_scorecard: params.quality_scorecard,
            template_provenance: params.template_provenance,
            defect_summary: params.defect_summary,
            hotspots: params.hotspots,
            recommendations: params.recommendations,
            qa_verification: None,
            build_info: params.build_info,
            project_overview: params.project_overview,
        }
    }

    pub(crate) async fn execute_qa_verification_phase(
        &self,
        deep_context: &DeepContext,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<QAVerificationResult> {
        progress.set_message("Running QA verification...");
        let qa_result = self.run_qa_verification(deep_context).await?;
        info!("QA verification completed");
        Ok(qa_result)
    }

    pub(crate) async fn build_cross_language_references(
        &self,
        _analyses: &ParallelAnalysisResults,
    ) -> anyhow::Result<FxHashMap<String, Vec<CrossLangReference>>> {
        // TRACKED: Implement cross-language reference detection
        // This would analyze FFI bindings, WASM exports, Python bindings, etc.
        Ok(FxHashMap::default())
    }
}
