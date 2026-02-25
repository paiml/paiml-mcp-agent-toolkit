// Incremental coverage formatting - extracted for file health (CB-040)
// Analysis logic: incremental_coverage_analysis.rs
// Formatters: incremental_coverage_formatters.rs
// Tests: incremental_coverage_tests.rs

#[derive(Debug, Serialize)]
pub struct IncrementalCoverageReport {
    pub base_branch: String,
    pub target_branch: String,
    pub coverage_threshold: f64,
    pub files: Vec<FileCoverageMetrics>,
    pub summary: CoverageSummary,
}

#[derive(Debug, Serialize, Clone)]
pub struct FileCoverageMetrics {
    pub path: PathBuf,
    pub base_coverage: f64,
    pub target_coverage: f64,
    pub coverage_delta: f64,
    pub lines_added: usize,
    pub lines_covered: usize,
    pub lines_uncovered: usize,
}

#[derive(Debug, Serialize)]
pub struct CoverageSummary {
    pub total_files_changed: usize,
    pub files_improved: usize,
    pub files_degraded: usize,
    pub overall_delta: f64,
    pub meets_threshold: bool,
}

// Analysis: convert_coverage_update_to_report
include!("incremental_coverage_analysis.rs");

// Formatters: LCOV, SARIF, summary, detailed, markdown, delta
include!("incremental_coverage_formatters.rs");

// Tests
include!("incremental_coverage_tests.rs");
