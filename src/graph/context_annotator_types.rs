/// Annotates code with graph-derived context information
#[derive(Debug, Clone)]
pub struct GraphContextAnnotator {
    pub pagerank_threshold: f64,
    pub community_relevance: f64,
}

impl Default for GraphContextAnnotator {
    fn default() -> Self {
        GraphContextAnnotator {
            pagerank_threshold: 0.1,
            community_relevance: 0.8,
        }
    }
}

#[derive(Debug, Clone)]
/// Context annotation.
pub struct ContextAnnotation {
    pub file_path: String,
    pub importance_score: f64,
    pub community_id: usize,
    pub related_files: Vec<String>,
    pub complexity_rank: String,
}
