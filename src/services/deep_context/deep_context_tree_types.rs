// Deep context tree and metadata types - file tree annotations, metadata, analysis results
// Included from mod.rs - shares parent module scope (no `use` imports)

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Metadata for context.
pub struct ContextMetadata {
    pub generated_at: DateTime<Utc>,
    pub tool_version: String,
    pub project_root: PathBuf,
    pub cache_stats: CacheStats,
    pub analysis_duration: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Statistics for cache.
pub struct CacheStats {
    pub hit_rate: f64,
    pub memory_efficiency: f64,
    pub time_saved_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Annotated file tree.
pub struct AnnotatedFileTree {
    pub root: AnnotatedNode,
    pub total_files: usize,
    pub total_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Annotated node.
pub struct AnnotatedNode {
    pub name: String,
    pub path: PathBuf,
    pub node_type: NodeType,
    pub children: Vec<AnnotatedNode>,
    pub annotations: NodeAnnotations,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
/// Type classification for node.
pub enum NodeType {
    Directory,
    #[default]
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Node annotations.
pub struct NodeAnnotations {
    pub defect_score: Option<f32>,
    pub complexity_score: Option<f32>,
    pub cognitive_complexity: Option<u16>,
    pub churn_score: Option<f32>,
    pub dead_code_items: usize,
    pub satd_items: usize,
    pub centrality: Option<f32>,
    pub test_coverage: Option<f32>,
    pub big_o_complexity: Option<String>,
    pub memory_complexity: Option<String>,
    pub duplication_score: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Analysis results.
pub struct AnalysisResults {
    pub ast_contexts: Vec<EnhancedFileContext>,
    pub complexity_report: Option<ComplexityReport>,
    pub churn_analysis: Option<CodeChurnAnalysis>,
    pub dependency_graph: Option<DependencyGraph>,
    pub dead_code_results: Option<crate::models::dead_code::DeadCodeRankingResult>,
    pub duplicate_code_results: Option<crate::services::duplicate_detector::CloneReport>,
    pub satd_results: Option<SATDAnalysisResult>,
    pub provability_results:
        Option<Vec<crate::services::lightweight_provability_analyzer::ProofSummary>>,
    pub cross_language_refs: Vec<CrossLangReference>,
    pub big_o_analysis: Option<crate::services::big_o_analyzer::BigOAnalysisReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Context for enhanced file operations.
pub struct EnhancedFileContext {
    pub base: FileContext,
    pub complexity_metrics: Option<FileComplexityMetrics>,
    pub churn_metrics: Option<FileChurnMetrics>,
    pub defects: DefectAnnotations,
    pub symbol_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// File churn metrics.
pub struct FileChurnMetrics {
    pub commits: u32,
    pub authors: u32,
    pub lines_added: u32,
    pub lines_deleted: u32,
    pub last_modified: DateTime<Utc>,
}
