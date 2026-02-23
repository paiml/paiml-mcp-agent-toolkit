#![cfg_attr(coverage_nightly, coverage(off))]
// DeepContextAnalyzer core analysis methods - split into submodules for file health (CB-040)
//
// Submodules organized by semantic grouping:
// - types: ParallelAnalysisResults, AnalysisResult enum
// - params: DeepContextBuildParams struct
// - pipeline: Main analyze_project and phase execution methods
// - spawn: Task spawning for parallel analyses
// - file_tree: File tree building and centrality enrichment
// - defect: Defect correlation and TDG analysis
// - quality: Quality scoring, recommendations, and QA verification

pub(crate) mod defect;
pub(crate) mod file_tree;
pub(crate) mod params;
pub(crate) mod pipeline;
pub(crate) mod quality;
pub(crate) mod spawn;
pub(crate) mod types;
