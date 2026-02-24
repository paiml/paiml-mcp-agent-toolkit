// ── Types ──────────────────────────────────────────────────────────────────

/// A split plan for a single source file
#[derive(Debug, Clone, Serialize)]
pub struct SplitPlan {
    /// Source file being split
    pub source_file: String,
    /// Total lines in file
    pub total_lines: usize,
    /// Detected clusters
    pub clusters: Vec<SplitCluster>,
    /// Items not assigned to any cluster (singletons)
    pub unclustered: Vec<ClusterItem>,
    /// Impact analysis
    pub impact: SplitImpact,
    /// Louvain modularity score (higher = better cluster separation)
    pub modularity: f64,
}

/// A cluster of related functions that should be extracted together
#[derive(Debug, Clone, Serialize)]
pub struct SplitCluster {
    /// Suggested filename for this cluster (no extension)
    pub suggested_name: String,
    /// Signal that produced the name
    pub naming_signal: String,
    /// Confidence in the suggested name (0.0-1.0)
    pub confidence: f32,
    /// Items in this cluster
    pub items: Vec<ClusterItem>,
    /// Estimated line count
    pub estimated_lines: usize,
    /// Cohesion score: ratio of internal edges to total possible edges
    pub cohesion: f64,
}

/// A single item (function/struct/enum/trait) in a cluster
#[derive(Debug, Clone, Serialize)]
pub struct ClusterItem {
    /// Item name
    pub name: String,
    /// Definition type
    pub definition_type: String,
    /// Line range (start, end)
    pub line_range: (usize, usize),
    /// Functions this item calls (within the file)
    pub calls: Vec<String>,
    /// Functions that call this item (within the file)
    pub called_by: Vec<String>,
}

/// Impact analysis for a split
#[derive(Debug, Clone, Serialize)]
pub struct SplitImpact {
    /// Files that import/use this module
    pub importing_files: Vec<String>,
    /// Potential circular dependency risks
    pub circular_risks: Vec<String>,
}
