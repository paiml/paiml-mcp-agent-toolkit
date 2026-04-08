#[derive(Debug, Clone, Serialize)]
/// Demo content.
pub struct DemoContent {
    pub mermaid_diagram: String,
    pub system_diagram: Option<String>,
    pub files_analyzed: usize,
    pub functions_analyzed: usize,
    pub avg_complexity: f64,
    pub p90_complexity: u32,
    pub hotspot_functions: usize,
    pub quality_score: f64,
    pub tech_debt_hours: u32,
    pub hotspots: Vec<EnhancedHotspot>,
    pub language_stats: std::collections::HashMap<String, LanguageStats>,
    pub ast_time_ms: u64,
    pub complexity_time_ms: u64,
    pub churn_time_ms: u64,
    pub dag_time_ms: u64,
    pub recommendations: Vec<RepositoryRecommendation>,
    pub polyglot_analysis: Option<PolyglotAnalysis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Hotspot identified in enhanced analysis.
pub struct EnhancedHotspot {
    pub function: String,
    pub file: String,
    pub path: String,
    pub complexity: u32,
    pub loc: u32,
    pub language: String,
    pub churn_score: u32,
    pub refactor_suggestion: String,
}

#[derive(Debug, Clone, Serialize)]
/// Statistics for language.
pub struct LanguageStats {
    pub file_count: usize,
    pub function_count: usize,
    pub avg_complexity: f64,
    pub total_loc: u32,
}

// Legacy hotspot for compatibility
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
/// Hotspot identified in hotspot analysis.
pub struct Hotspot {
    pub file: String,
    pub complexity: u32,
    pub churn_score: u32,
}

#[derive(Clone)]
/// State representation for demo.
pub struct DemoState {
    pub repository: std::path::PathBuf,
    pub analysis_results: AnalysisResults,
    pub mermaid_cache: Arc<DashMap<u64, String>>,
    pub system_diagram: Option<String>,
}

impl std::fmt::Debug for DemoState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DemoState")
            .field("repository", &self.repository)
            .field("analysis_results", &"<AnalysisResults>")
            .field(
                "mermaid_cache",
                &format!("<{} entries>", self.mermaid_cache.len()),
            )
            .field("system_diagram", &self.system_diagram.is_some())
            .finish()
    }
}

#[derive(Clone, Serialize)]
/// Analysis results.
pub struct AnalysisResults {
    pub files_analyzed: usize,
    pub avg_complexity: f64,
    pub tech_debt_hours: u32,
    pub complexity_report: crate::services::complexity::ComplexityReport,
    pub churn_analysis: crate::models::churn::CodeChurnAnalysis,
    pub dependency_graph: DependencyGraph,
    pub tdg_summary: Option<crate::models::tdg::TDGSummary>,
}
