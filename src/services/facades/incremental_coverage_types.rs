/// Request for incremental coverage analysis
#[derive(Debug, Clone)]
pub struct IncrementalCoverageRequest {
    pub project_path: PathBuf,
    pub base_branch: String,
    pub target_branch: Option<String>,
    /// Minimum coverage, as a PERCENTAGE in 0-100 — the same units `--help`
    /// documents (`[default: 80.0]`).
    ///
    /// Every renderer used to multiply this by 100 on the way out, so the
    /// documented default was announced and applied as "8000.0%" and no file
    /// could ever be above threshold (GH #658).
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
    /// Mean line coverage of the measured changed files, 0-100. `None` when no
    /// changed file had coverage data.
    pub coverage_percentage: Option<f64>,
    pub files_above_threshold: usize,
    pub files_below_threshold: usize,
    /// Changed files whose coverage could not be measured (no entry in the
    /// coverage artifact). Counted, never scored.
    pub files_not_measured: usize,
    pub changed_files: Vec<ChangedFileCoverage>,
    pub summary: String,
}

/// Coverage information for a changed file.
///
/// `None` means pmat did not measure the field — not that it measured zero.
/// These were fabricated constants: `coverage_before` was 0.75 (0.0 for added
/// files), `coverage_after` was 0.85 and `lines_covered`/`lines_total` were
/// 85/100 for every file, under a comment reading "Mock coverage analysis for
/// now". See `contracts/pmat-no-fabrication-v1.yaml`, `measured_or_absent`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedFileCoverage {
    pub file_path: String,
    /// Coverage before the change. Always `None`: measuring it needs a coverage
    /// artifact for the base branch, and pmat has only the working tree's.
    pub coverage_before: Option<f64>,
    /// Line coverage percentage (0-100) read from the project's coverage
    /// artifact, or `None` when the file has no entry in it.
    pub coverage_after: Option<f64>,
    /// `coverage_after - coverage_before`; `None` while `coverage_before` is.
    pub coverage_delta: Option<f64>,
    pub status: CoverageStatus,
    pub lines_covered: usize,
    pub lines_total: usize,
}

/// Coverage status for a file
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CoverageStatus {
    Improved,
    Degraded,
    Unchanged,
    New,
    Deleted,
    /// Coverage was not measured for this file, so no direction can be claimed.
    NotMeasured,
}
