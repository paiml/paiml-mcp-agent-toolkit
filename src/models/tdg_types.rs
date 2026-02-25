/// Summary statistics for TDG across a codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TDGSummary {
    /// Total number of files analyzed
    pub total_files: usize,

    /// Number of files with critical TDG scores
    pub critical_files: usize,

    /// Number of files with warning TDG scores
    pub warning_files: usize,

    /// Average TDG score across all files
    pub average_tdg: f64,

    /// 95th percentile TDG score
    pub p95_tdg: f64,

    /// 99th percentile TDG score
    pub p99_tdg: f64,

    /// Estimated technical debt in hours
    pub estimated_debt_hours: f64,

    /// Files with highest TDG scores
    pub hotspots: Vec<TDGHotspot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TDGHotspot {
    /// File path
    pub path: String,

    /// TDG score
    pub tdg_score: f64,

    /// Primary contributor to high TDG
    pub primary_factor: String,

    /// Estimated hours to refactor
    pub estimated_hours: f64,
}

/// TDG calculation result with detailed breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TDGAnalysis {
    /// The calculated TDG score
    pub score: TDGScore,

    /// Detailed explanation of the calculation
    pub explanation: String,

    /// Specific recommendations for reducing TDG
    pub recommendations: Vec<TDGRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TDGRecommendation {
    /// Type of recommendation
    pub recommendation_type: RecommendationType,

    /// Specific action to take
    pub action: String,

    /// Expected TDG reduction
    pub expected_reduction: f64,

    /// Estimated effort in hours
    pub estimated_hours: f64,

    /// Priority level (1-5, 5 being highest)
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationType {
    /// Reduce function complexity
    ReduceComplexity,

    /// Stabilize frequently changing code
    StabilizeChurn,

    /// Reduce coupling between modules
    ReduceCoupling,

    /// Address domain-specific risks
    AddressDomainRisk,

    /// Remove duplicate code
    RemoveDuplication,

    /// Split large files
    SplitFile,

    /// Add test coverage
    AddTests,
}

/// TDG distribution for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TDGDistribution {
    /// Histogram buckets
    pub buckets: Vec<TDGBucket>,

    /// Total number of files
    pub total_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TDGBucket {
    /// Lower bound of the bucket (inclusive)
    pub min: f64,

    /// Upper bound of the bucket (exclusive)
    pub max: f64,

    /// Number of files in this bucket
    pub count: usize,

    /// Percentage of total files
    pub percentage: f64,
}

// Additional types for SATD analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatdItem {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub comment_text: String,
    pub debt_type: String,
    pub severity: SatdSeverity,
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SatdSeverity {
    Low,
    Medium,
    High,
    Critical,
}
