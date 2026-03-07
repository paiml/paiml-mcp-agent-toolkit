#![cfg_attr(coverage_nightly, coverage(off))]

use crate::models::churn::CodeChurnAnalysis;
use crate::models::dag::DependencyGraph;
use crate::services::complexity::ComplexityReport;
use crate::services::deep_context::EnhancedFileContext;
use crate::services::satd_detector::SATDAnalysisResult;

/// Structure for collecting parallel analysis results
#[derive(Default)]
pub(crate) struct ParallelAnalysisResults {
    pub(crate) ast_contexts: Option<Vec<EnhancedFileContext>>,
    pub(crate) complexity_report: Option<ComplexityReport>,
    pub(crate) churn_analysis: Option<CodeChurnAnalysis>,
    pub(crate) dependency_graph: Option<DependencyGraph>,
    pub(crate) dead_code_results: Option<crate::models::dead_code::DeadCodeRankingResult>,
    pub(crate) duplicate_code_results: Option<crate::services::duplicate_detector::CloneReport>,
    pub(crate) satd_results: Option<SATDAnalysisResult>,
    pub(crate) provability_results:
        Option<Vec<crate::services::lightweight_provability_analyzer::ProofSummary>>,
    pub(crate) big_o_analysis: Option<crate::services::big_o_analyzer::BigOAnalysisReport>,
}

pub(crate) enum AnalysisResult {
    Ast(anyhow::Result<Vec<EnhancedFileContext>>),
    Complexity(anyhow::Result<ComplexityReport>),
    Churn(anyhow::Result<CodeChurnAnalysis>),
    DeadCode(anyhow::Result<crate::models::dead_code::DeadCodeRankingResult>),
    DuplicateCode(anyhow::Result<crate::services::duplicate_detector::CloneReport>),
    Satd(anyhow::Result<SATDAnalysisResult>),
    Provability(
        anyhow::Result<Vec<crate::services::lightweight_provability_analyzer::ProofSummary>>,
    ),
    Dag(anyhow::Result<DependencyGraph>),
    BigO(anyhow::Result<crate::services::big_o_analyzer::BigOAnalysisReport>),
}
