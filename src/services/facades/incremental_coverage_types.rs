/// Request for incremental coverage analysis
#[derive(Debug, Clone)]
pub struct IncrementalCoverageRequest {
    pub project_path: PathBuf,
    pub base_branch: String,
    pub target_branch: Option<String>,
    pub coverage_threshold: f64,
    pub changed_files_only: bool,
    pub detailed: bool,
    pub cache_dir: Option<PathBuf>,
    pub force_refresh: bool,
    pub top_files: usize,
}

/// Result of incremental coverage analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalCoverageResult {
    pub total_files: usize,
    pub covered_files: usize,
    pub coverage_percentage: f64,
    pub files_above_threshold: usize,
    pub files_below_threshold: usize,
    pub changed_files: Vec<ChangedFileCoverage>,
    pub summary: String,
}

/// Coverage information for a changed file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedFileCoverage {
    pub file_path: String,
    pub coverage_before: f64,
    pub coverage_after: f64,
    pub coverage_delta: f64,
    pub status: CoverageStatus,
    pub lines_covered: usize,
    pub lines_total: usize,
}

/// Coverage status for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoverageStatus {
    Improved,
    Degraded,
    Unchanged,
    New,
    Deleted,
}
