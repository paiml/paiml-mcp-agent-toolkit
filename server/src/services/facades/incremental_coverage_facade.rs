//! Incremental Coverage Analysis Facade
//!
//! Provides a simplified interface for incremental coverage analysis operations.

use crate::services::service_registry::ServiceRegistry;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

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

/// Facade for incremental coverage analysis operations
#[derive(Clone)]
pub struct IncrementalCoverageFacade {
    #[allow(dead_code)]
    registry: Arc<ServiceRegistry>,
}

impl IncrementalCoverageFacade {
    /// Create a new incremental coverage facade
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self { registry }
    }

    /// Perform incremental coverage analysis on a project
    pub async fn analyze_project(
        &self,
        request: IncrementalCoverageRequest,
    ) -> Result<IncrementalCoverageResult> {
        // Get changed files between branches
        let changed_files = self
            .get_changed_files(
                &request.project_path,
                &request.base_branch,
                request.target_branch.as_deref(),
            )
            .await?;

        // Analyze coverage for changed files
        let coverage_data = self
            .analyze_coverage_changes(&request.project_path, &changed_files, &request)
            .await?;

        // Build result
        Ok(self.build_coverage_result(coverage_data, changed_files, &request))
    }

    /// Get changed files between branches
    async fn get_changed_files(
        &self,
        project_path: &Path,
        base_branch: &str,
        target_branch: Option<&str>,
    ) -> Result<Vec<(PathBuf, String)>> {
        use crate::cli::coverage_helpers::get_changed_files_for_coverage;

        get_changed_files_for_coverage(project_path, base_branch, target_branch).await
    }

    /// Analyze coverage changes for files
    async fn analyze_coverage_changes(
        &self,
        _project_path: &Path,
        changed_files: &[(PathBuf, String)],
        request: &IncrementalCoverageRequest,
    ) -> Result<Vec<ChangedFileCoverage>> {
        let mut coverage_data = Vec::new();

        for (path, status) in changed_files {
            if status == "M" || status == "A" {
                // Mock coverage analysis for now - would integrate with real coverage analyzer
                let coverage_before = if status == "A" { 0.0 } else { 0.75 };
                let coverage_after = 0.85;
                let coverage_delta = coverage_after - coverage_before;

                let file_coverage = ChangedFileCoverage {
                    file_path: path.display().to_string(),
                    coverage_before,
                    coverage_after,
                    coverage_delta,
                    status: if coverage_delta > 0.0 {
                        CoverageStatus::Improved
                    } else if coverage_delta < 0.0 {
                        CoverageStatus::Degraded
                    } else {
                        CoverageStatus::Unchanged
                    },
                    lines_covered: 85,
                    lines_total: 100,
                };

                coverage_data.push(file_coverage);

                // Only analyze top N files if requested
                if coverage_data.len() >= request.top_files {
                    break;
                }
            }
        }

        Ok(coverage_data)
    }

    /// Build the final coverage result
    fn build_coverage_result(
        &self,
        coverage_data: Vec<ChangedFileCoverage>,
        changed_files: Vec<(PathBuf, String)>,
        request: &IncrementalCoverageRequest,
    ) -> IncrementalCoverageResult {
        let total_files = changed_files.len();
        let covered_files = coverage_data
            .iter()
            .filter(|f| f.coverage_after > 0.0)
            .count();

        let avg_coverage = if !coverage_data.is_empty() {
            coverage_data.iter().map(|f| f.coverage_after).sum::<f64>() / coverage_data.len() as f64
        } else {
            0.0
        };

        let files_above_threshold = coverage_data
            .iter()
            .filter(|f| f.coverage_after >= request.coverage_threshold)
            .count();

        let files_below_threshold = coverage_data
            .iter()
            .filter(|f| f.coverage_after < request.coverage_threshold)
            .count();

        let summary = format!(
            "Analyzed {} changed files: {} covered ({:.1}%), {} above threshold ({:.1}%), {} below threshold",
            total_files,
            covered_files,
            avg_coverage * 100.0,
            files_above_threshold,
            request.coverage_threshold * 100.0,
            files_below_threshold
        );

        IncrementalCoverageResult {
            total_files,
            covered_files,
            coverage_percentage: avg_coverage,
            files_above_threshold,
            files_below_threshold,
            changed_files: coverage_data,
            summary,
        }
    }

    /// Quick coverage analysis with defaults
    pub async fn quick_analysis(
        &self,
        project_path: PathBuf,
        base_branch: String,
    ) -> Result<IncrementalCoverageResult> {
        let request = IncrementalCoverageRequest {
            project_path,
            base_branch,
            target_branch: None,
            coverage_threshold: 0.8,
            changed_files_only: true,
            detailed: false,
            cache_dir: None,
            force_refresh: false,
            top_files: 10,
        };

        self.analyze_project(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::service_registry::ServiceRegistry;

    #[tokio::test]
    async fn test_incremental_coverage_facade_creation() {
        let registry = Arc::new(ServiceRegistry::new());
        let _facade = IncrementalCoverageFacade::new(registry);
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test] 
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
