/// Analysis request configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub project_path: String,
    pub analysis_types: Vec<AnalysisType>,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub max_depth: Option<usize>,
    pub parallel: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnalysisType {
    DuplicateDetection,
    DeadCodeAnalysis,
    ComplexityMetrics,
    DependencyGraph,
    DefectPrediction,
    NameSimilarity,
}

impl AnalysisRequest {
    /// Generates a deterministic cache key for this analysis request.
    ///
    /// The cache key is derived from the SHA256 hash of the project path and analysis types,
    /// ensuring that identical requests produce identical cache keys while different
    /// requests produce different keys.
    ///
    /// # Performance
    ///
    /// - Time: O(n) where n = combined length of path and analysis types
    /// - Space: O(1) - fixed 64-byte hash output
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::code_intelligence::{AnalysisRequest, AnalysisType};
    ///
    /// let request = AnalysisRequest {
    ///     project_path: "/home/user/project".to_string(),
    ///     analysis_types: vec![AnalysisType::DuplicateDetection, AnalysisType::DeadCodeAnalysis],
    ///     include_patterns: vec!["*.rs".to_string()],
    ///     exclude_patterns: vec!["target/".to_string()],
    ///     max_depth: Some(5),
    ///     parallel: true,
    /// };
    ///
    /// let key1 = request.cache_key();
    /// let key2 = request.cache_key();
    ///
    /// // Cache keys are deterministic
    /// assert_eq!(key1, key2);
    /// assert_eq!(key1.len(), 64); // SHA256 produces 64-character hex string
    ///
    /// // Different requests produce different keys
    /// let mut different_request = request.clone();
    /// different_request.project_path = "/different/path".to_string();
    /// assert_ne!(key1, different_request.cache_key());
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn cache_key(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(self.project_path.as_bytes());
        for t in &self.analysis_types {
            hasher.update(format!("{t:?}").as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }
}

/// Comprehensive analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub duplicates: Option<CloneReport>,
    pub dead_code: Option<DeadCodeReport>,
    pub complexity_metrics: Option<ComplexityReport>,
    pub dependency_graph: Option<DependencyGraphReport>,
    pub defect_predictions: Option<Vec<DefectScore>>,
    pub graph_metrics: Option<GraphMetricsReport>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityReport {
    pub total_files: usize,
    pub average_complexity: f32,
    pub hotspots: Vec<ComplexityHotspot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityHotspot {
    pub file_path: String,
    pub function_name: String,
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraphReport {
    pub nodes: usize,
    pub edges: usize,
    pub circular_dependencies: Vec<Vec<String>>,
    pub mermaid_diagram: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectScore {
    pub entity: String,
    pub score: f32,
    pub confidence: f32,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetricsReport {
    pub centrality_scores: Vec<CentralityScore>,
    pub clustering_coefficient: f32,
    pub modularity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralityScore {
    pub node: String,
    pub degree: f32,
    pub betweenness: f32,
    pub closeness: f32,
    pub pagerank: f32,
}
