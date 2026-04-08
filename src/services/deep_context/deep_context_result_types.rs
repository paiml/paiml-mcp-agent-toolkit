// Deep context result types - core output structures
// Included from mod.rs - shares parent module scope (no `use` imports)

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Context for deep operations.
pub struct DeepContext {
    pub metadata: ContextMetadata,
    pub file_tree: AnnotatedFileTree,
    pub analyses: AnalysisResults,
    pub quality_scorecard: QualityScorecard,
    pub template_provenance: Option<TemplateProvenance>,
    pub defect_summary: DefectSummary,
    pub hotspots: Vec<DefectHotspot>,
    pub recommendations: Vec<PrioritizedRecommendation>,
    pub qa_verification: Option<QAVerificationResult>,
    pub build_info: Option<crate::models::project_meta::BuildInfo>,
    pub project_overview: Option<crate::models::project_meta::ProjectOverview>,
}

/// Extended structure for QA verification that includes additional fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepContextResult {
    // Core fields from DeepContext
    pub metadata: ContextMetadata,
    pub file_tree: Vec<String>, // List of file paths for quality_gates
    pub analyses: AnalysisResults,
    pub quality_scorecard: QualityScorecard,
    pub template_provenance: Option<TemplateProvenance>,
    pub defect_summary: DefectSummary,
    pub hotspots: Vec<DefectHotspot>,
    pub recommendations: Vec<PrioritizedRecommendation>,
    pub qa_verification: Option<QAVerificationResult>,

    // Additional fields expected by quality_gates
    pub complexity_metrics: Option<ComplexityMetricsForQA>,
    pub dead_code_analysis: Option<DeadCodeAnalysis>,
    pub ast_summaries: Option<Vec<AstSummary>>,
    pub churn_analysis: Option<CodeChurnAnalysis>,
    pub language_stats: Option<FxHashMap<String, usize>>,

    // Project metadata fields
    pub build_info: Option<crate::models::project_meta::BuildInfo>,
    pub project_overview: Option<crate::models::project_meta::ProjectOverview>,
}

/// Summary of AST analysis for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstSummary {
    pub path: String,
    pub language: String,
    pub total_items: usize,
    pub functions: usize,
    pub classes: usize,
    pub imports: usize,
}

/// Dead code analysis structure expected by `quality_gates`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeAnalysis {
    pub summary: DeadCodeSummary,
    pub dead_functions: Vec<String>,
    pub warnings: Vec<String>,
}

/// Dead code summary structure expected by `quality_gates`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeSummary {
    pub total_functions: usize,
    pub dead_functions: usize,
    pub total_lines: usize,
    pub total_dead_lines: usize,
    pub dead_percentage: f64,
}

/// Complexity metrics structure expected by `quality_gates`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplexityMetricsForQA {
    pub files: Vec<FileComplexityMetricsForQA>,
    pub summary: ComplexitySummaryForQA,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// File complexity metrics for q a.
pub struct FileComplexityMetricsForQA {
    pub path: std::path::PathBuf,
    pub functions: Vec<FunctionComplexityForQA>,
    pub total_cyclomatic: u32,
    pub total_cognitive: u32,
    pub total_lines: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Function complexity for q a.
pub struct FunctionComplexityForQA {
    pub name: String,
    pub cyclomatic: u32,
    pub cognitive: u32,
    pub nesting_depth: u32,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Complexity summary for q a.
pub struct ComplexitySummaryForQA {
    pub total_files: usize,
    pub total_functions: usize,
}
