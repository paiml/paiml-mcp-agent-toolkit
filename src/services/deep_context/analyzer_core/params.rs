#![cfg_attr(coverage_nightly, coverage(off))]

use std::path::Path;

use rustc_hash::FxHashMap;

use crate::models::project_meta::{BuildInfo, ProjectOverview};
use crate::services::deep_context::analyzer_core::types::ParallelAnalysisResults;
use crate::services::deep_context::{
    AnnotatedFileTree, CrossLangReference, DefectHotspot, DefectSummary, PrioritizedRecommendation,
    QualityScorecard, TemplateProvenance,
};

/// Parameters for building deep context
pub(crate) struct DeepContextBuildParams<'a> {
    pub(crate) project_path: &'a Path,
    pub(crate) file_tree: AnnotatedFileTree,
    pub(crate) analyses: ParallelAnalysisResults,
    pub(crate) cross_refs: FxHashMap<String, Vec<CrossLangReference>>,
    pub(crate) quality_scorecard: QualityScorecard,
    pub(crate) template_provenance: Option<TemplateProvenance>,
    pub(crate) defect_summary: DefectSummary,
    pub(crate) hotspots: Vec<DefectHotspot>,
    pub(crate) recommendations: Vec<PrioritizedRecommendation>,
    pub(crate) build_info: Option<BuildInfo>,
    pub(crate) project_overview: Option<ProjectOverview>,
    pub(crate) analysis_duration: std::time::Duration,
}
