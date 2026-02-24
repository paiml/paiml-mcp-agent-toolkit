/// Source location for a finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    /// File path
    pub file: PathBuf,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Function or scope name
    pub scope: Option<String>,
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.file.display(), self.line)
    }
}

/// Suggested fix for a finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixSuggestion {
    /// Description of the fix
    pub description: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Can this fix be auto-applied?
    pub auto_fixable: bool,
    /// Estimated effort (in minutes)
    pub effort_minutes: Option<u32>,
}

/// Individual finding with rich metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Unique identifier
    pub id: String,
    /// Defect category (from UDS)
    pub category: String,
    /// Severity level
    pub severity: Severity,
    /// Source code location
    pub location: SourceLocation,
    /// Human-readable message
    pub message: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// K-means cluster assignment
    pub cluster_id: Option<usize>,
    /// PageRank centrality score
    pub pagerank: Option<f32>,
    /// Louvain community assignment
    pub community: Option<String>,
    /// Isolation Forest anomaly score
    pub anomaly_score: Option<f32>,
    /// Suggested fix
    pub fix_suggestion: Option<FixSuggestion>,
}

/// Cluster of related findings (K-means output)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingCluster {
    /// Cluster ID
    pub id: usize,
    /// Number of findings in cluster
    pub size: usize,
    /// Dominant category in cluster
    pub primary_category: String,
    /// Cluster cohesion score (0.0 - 1.0)
    pub cohesion: f64,
    /// Centroid description
    pub description: String,
    /// Finding IDs in this cluster
    pub finding_ids: Vec<String>,
}

/// Code community detected by Louvain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeCommunity {
    /// Community name (derived from dominant file/module)
    pub name: String,
    /// Modularity score for this community
    pub modularity: f64,
    /// Files in this community
    pub files: Vec<PathBuf>,
    /// Primary issue type in this community
    pub primary_issue: Option<String>,
    /// Number of defects in community
    pub defect_count: usize,
}

/// Anomaly detected by Isolation Forest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyPoint {
    /// Finding ID that is anomalous
    pub finding_id: String,
    /// Anomaly score (0.0 = normal, 1.0 = highly anomalous)
    pub score: f64,
    /// Reason for anomaly
    pub reason: String,
    /// Suggested action
    pub action: String,
}

/// Metric time series for trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricTrend {
    /// Metric name
    pub name: String,
    /// Current value
    pub current: f64,
    /// Trend direction
    pub direction: TrendDirection,
    /// Change percentage
    pub change_percent: f64,
    /// Sparkline data (last N values normalized 0-7)
    pub sparkline: Vec<u8>,
    /// Forecast value (if available)
    pub forecast: Option<f64>,
}
