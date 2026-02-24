/// Complete TDG baseline for a project
///
/// Captures quality state of all files at a specific point in time.
/// Uses content hashing for efficient deduplication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TdgBaseline {
    /// PMAT version that created this baseline
    pub version: String,

    /// When this baseline was created
    pub created_at: DateTime<Utc>,

    /// Git context (commit, branch, author) when baseline was created
    pub git_context: Option<GitContext>,

    /// Per-file TDG scores indexed by file path
    pub files: HashMap<PathBuf, BaselineEntry>,

    /// Aggregate statistics across all files
    pub summary: BaselineSummary,
}

/// Single file entry in baseline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineEntry {
    /// Blake3 content hash for deduplication
    pub content_hash: Blake3Hash,

    /// TDG score for this file
    pub score: TdgScore,

    /// Component breakdown (complexity, duplication, etc.)
    pub components: ComponentScores,

    /// Git context when this file was analyzed
    pub git_context: Option<GitContext>,
}

/// Aggregate statistics for baseline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineSummary {
    /// Total number of files analyzed
    pub total_files: usize,

    /// Average TDG score across all files
    pub avg_score: f32,

    /// Distribution of grades (A+, A, B+, etc.)
    pub grade_distribution: HashMap<Grade, usize>,

    /// Count by programming language
    pub languages: HashMap<String, usize>,
}

/// Comparison result between two baselines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComparison {
    /// Files with improved scores
    pub improved: Vec<FileComparison>,

    /// Files with regressed scores
    pub regressed: Vec<FileComparison>,

    /// Files with unchanged scores
    pub unchanged: Vec<PathBuf>,

    /// Files added since baseline
    pub added: Vec<PathBuf>,

    /// Files removed since baseline
    pub removed: Vec<PathBuf>,
}

/// Detailed comparison for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileComparison {
    /// File path
    pub path: PathBuf,

    /// Score in old baseline
    pub old_score: TdgScore,

    /// Score in new baseline
    pub new_score: TdgScore,

    /// Score delta (positive = improvement, negative = regression)
    pub delta: f32,

    /// Grade change (old, new)
    pub grade_change: (Grade, Grade),
}
