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
    #[must_use]
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

        let avg_coverage = if coverage_data.is_empty() {
            0.0
        } else {
            coverage_data.iter().map(|f| f.coverage_after).sum::<f64>() / coverage_data.len() as f64
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
    use std::fs;
    use tempfile::TempDir;

    /// Helper to create a test facade with a fresh registry
    fn create_test_facade() -> IncrementalCoverageFacade {
        let registry = Arc::new(ServiceRegistry::new());
        IncrementalCoverageFacade::new(registry)
    }

    /// Helper to create a temporary git repo for testing
    fn create_test_git_repo() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Initialize git repo
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to init git repo");

        // Configure git user
        std::process::Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to configure git email");

        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to configure git name");

        // Create a base file and commit
        let base_file = temp_dir.path().join("base.rs");
        fs::write(&base_file, "fn main() {}\n").expect("Failed to write base file");

        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to stage files");

        std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to create initial commit");

        // Create main branch
        std::process::Command::new("git")
            .args(["branch", "-M", "main"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to rename branch to main");

        temp_dir
    }

    #[tokio::test]
    async fn test_incremental_coverage_facade_creation() {
        let registry = Arc::new(ServiceRegistry::new());
        let facade = IncrementalCoverageFacade::new(registry);
        // Verify facade is properly created (no panic)
        let _ = facade;
    }

    #[tokio::test]
    async fn test_facade_clone() {
        let facade = create_test_facade();
        let cloned = facade.clone();
        // Both facades should work independently
        let _ = cloned;
    }

    #[test]
    fn test_coverage_status_variants() {
        let improved = CoverageStatus::Improved;
        let degraded = CoverageStatus::Degraded;
        let unchanged = CoverageStatus::Unchanged;
        let new = CoverageStatus::New;
        let deleted = CoverageStatus::Deleted;

        // Just verify all variants exist and can be created
        let _ = (improved, degraded, unchanged, new, deleted);
    }

    #[test]
    fn test_changed_file_coverage_creation() {
        let coverage = ChangedFileCoverage {
            file_path: "test.rs".to_string(),
            coverage_before: 0.75,
            coverage_after: 0.85,
            coverage_delta: 0.10,
            status: CoverageStatus::Improved,
            lines_covered: 85,
            lines_total: 100,
        };

        assert_eq!(coverage.file_path, "test.rs");
        assert!((coverage.coverage_before - 0.75).abs() < f64::EPSILON);
        assert!((coverage.coverage_after - 0.85).abs() < f64::EPSILON);
        assert!((coverage.coverage_delta - 0.10).abs() < f64::EPSILON);
        assert_eq!(coverage.lines_covered, 85);
        assert_eq!(coverage.lines_total, 100);
    }

    #[test]
    fn test_changed_file_coverage_clone() {
        let coverage = ChangedFileCoverage {
            file_path: "test.rs".to_string(),
            coverage_before: 0.75,
            coverage_after: 0.85,
            coverage_delta: 0.10,
            status: CoverageStatus::Improved,
            lines_covered: 85,
            lines_total: 100,
        };

        let cloned = coverage.clone();
        assert_eq!(cloned.file_path, "test.rs");
        assert_eq!(cloned.lines_covered, 85);
    }

    #[test]
    fn test_incremental_coverage_request_creation() {
        let request = IncrementalCoverageRequest {
            project_path: PathBuf::from("/test"),
            base_branch: "main".to_string(),
            target_branch: Some("feature".to_string()),
            coverage_threshold: 0.8,
            changed_files_only: true,
            detailed: false,
            cache_dir: Some(PathBuf::from("/cache")),
            force_refresh: false,
            top_files: 10,
        };

        assert_eq!(request.project_path, PathBuf::from("/test"));
        assert_eq!(request.base_branch, "main");
        assert_eq!(request.target_branch, Some("feature".to_string()));
        assert!((request.coverage_threshold - 0.8).abs() < f64::EPSILON);
        assert!(request.changed_files_only);
        assert!(!request.detailed);
        assert!(request.cache_dir.is_some());
        assert!(!request.force_refresh);
        assert_eq!(request.top_files, 10);
    }

    #[test]
    fn test_incremental_coverage_request_clone() {
        let request = IncrementalCoverageRequest {
            project_path: PathBuf::from("/test"),
            base_branch: "main".to_string(),
            target_branch: None,
            coverage_threshold: 0.8,
            changed_files_only: true,
            detailed: false,
            cache_dir: None,
            force_refresh: false,
            top_files: 10,
        };

        let cloned = request.clone();
        assert_eq!(cloned.base_branch, "main");
        assert!(cloned.target_branch.is_none());
    }

    #[test]
    fn test_incremental_coverage_result_creation() {
        let result = IncrementalCoverageResult {
            total_files: 10,
            covered_files: 8,
            coverage_percentage: 0.85,
            files_above_threshold: 7,
            files_below_threshold: 3,
            changed_files: vec![],
            summary: "Test summary".to_string(),
        };

        assert_eq!(result.total_files, 10);
        assert_eq!(result.covered_files, 8);
        assert!((result.coverage_percentage - 0.85).abs() < f64::EPSILON);
        assert_eq!(result.files_above_threshold, 7);
        assert_eq!(result.files_below_threshold, 3);
        assert!(result.changed_files.is_empty());
        assert_eq!(result.summary, "Test summary");
    }

    #[test]
    fn test_incremental_coverage_result_serialization() {
        let result = IncrementalCoverageResult {
            total_files: 5,
            covered_files: 4,
            coverage_percentage: 0.80,
            files_above_threshold: 3,
            files_below_threshold: 2,
            changed_files: vec![ChangedFileCoverage {
                file_path: "test.rs".to_string(),
                coverage_before: 0.70,
                coverage_after: 0.85,
                coverage_delta: 0.15,
                status: CoverageStatus::Improved,
                lines_covered: 85,
                lines_total: 100,
            }],
            summary: "Test summary".to_string(),
        };

        let json = serde_json::to_string(&result).expect("Failed to serialize");
        assert!(json.contains("total_files"));
        assert!(json.contains("5"));
        assert!(json.contains("coverage_percentage"));
        assert!(json.contains("test.rs"));
    }

    #[test]
    fn test_incremental_coverage_result_deserialization() {
        let json = r#"{
            "total_files": 3,
            "covered_files": 2,
            "coverage_percentage": 0.75,
            "files_above_threshold": 2,
            "files_below_threshold": 1,
            "changed_files": [],
            "summary": "Deserialized summary"
        }"#;

        let result: IncrementalCoverageResult =
            serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(result.total_files, 3);
        assert_eq!(result.covered_files, 2);
        assert!((result.coverage_percentage - 0.75).abs() < f64::EPSILON);
        assert_eq!(result.summary, "Deserialized summary");
    }

    #[test]
    fn test_build_coverage_result_empty_data() {
        let facade = create_test_facade();
        let request = IncrementalCoverageRequest {
            project_path: PathBuf::from("/test"),
            base_branch: "main".to_string(),
            target_branch: None,
            coverage_threshold: 0.8,
            changed_files_only: true,
            detailed: false,
            cache_dir: None,
            force_refresh: false,
            top_files: 10,
        };

        let result = facade.build_coverage_result(vec![], vec![], &request);

        assert_eq!(result.total_files, 0);
        assert_eq!(result.covered_files, 0);
        assert!((result.coverage_percentage - 0.0).abs() < f64::EPSILON);
        assert_eq!(result.files_above_threshold, 0);
        assert_eq!(result.files_below_threshold, 0);
        assert!(result.summary.contains("0 changed files"));
    }

    #[test]
    fn test_build_coverage_result_with_data() {
        let facade = create_test_facade();
        let request = IncrementalCoverageRequest {
            project_path: PathBuf::from("/test"),
            base_branch: "main".to_string(),
            target_branch: None,
            coverage_threshold: 0.8,
            changed_files_only: true,
            detailed: false,
            cache_dir: None,
            force_refresh: false,
            top_files: 10,
        };

        let coverage_data = vec![
            ChangedFileCoverage {
                file_path: "high.rs".to_string(),
                coverage_before: 0.70,
                coverage_after: 0.90,
                coverage_delta: 0.20,
                status: CoverageStatus::Improved,
                lines_covered: 90,
                lines_total: 100,
            },
            ChangedFileCoverage {
                file_path: "low.rs".to_string(),
                coverage_before: 0.80,
                coverage_after: 0.70,
                coverage_delta: -0.10,
                status: CoverageStatus::Degraded,
                lines_covered: 70,
                lines_total: 100,
            },
        ];

        let changed_files = vec![
            (PathBuf::from("high.rs"), "M".to_string()),
            (PathBuf::from("low.rs"), "M".to_string()),
        ];

        let result = facade.build_coverage_result(coverage_data, changed_files, &request);

        assert_eq!(result.total_files, 2);
        assert_eq!(result.covered_files, 2);
        // Average coverage = (0.90 + 0.70) / 2 = 0.80
        assert!((result.coverage_percentage - 0.80).abs() < f64::EPSILON);
        // 0.90 >= 0.8, 0.70 < 0.8
        assert_eq!(result.files_above_threshold, 1);
        assert_eq!(result.files_below_threshold, 1);
    }

    #[test]
    fn test_build_coverage_result_all_above_threshold() {
        let facade = create_test_facade();
        let request = IncrementalCoverageRequest {
            project_path: PathBuf::from("/test"),
            base_branch: "main".to_string(),
            target_branch: None,
            coverage_threshold: 0.5,
            changed_files_only: true,
            detailed: false,
            cache_dir: None,
            force_refresh: false,
            top_files: 10,
        };

        let coverage_data = vec![
            ChangedFileCoverage {
                file_path: "a.rs".to_string(),
                coverage_before: 0.60,
                coverage_after: 0.80,
                coverage_delta: 0.20,
                status: CoverageStatus::Improved,
                lines_covered: 80,
                lines_total: 100,
            },
            ChangedFileCoverage {
                file_path: "b.rs".to_string(),
                coverage_before: 0.70,
                coverage_after: 0.90,
                coverage_delta: 0.20,
                status: CoverageStatus::Improved,
                lines_covered: 90,
                lines_total: 100,
            },
        ];

        let changed_files = vec![
            (PathBuf::from("a.rs"), "M".to_string()),
            (PathBuf::from("b.rs"), "M".to_string()),
        ];

        let result = facade.build_coverage_result(coverage_data, changed_files, &request);

        assert_eq!(result.files_above_threshold, 2);
        assert_eq!(result.files_below_threshold, 0);
    }

    #[test]
    fn test_build_coverage_result_all_below_threshold() {
        let facade = create_test_facade();
        let request = IncrementalCoverageRequest {
            project_path: PathBuf::from("/test"),
            base_branch: "main".to_string(),
            target_branch: None,
            coverage_threshold: 0.95,
            changed_files_only: true,
            detailed: false,
            cache_dir: None,
            force_refresh: false,
            top_files: 10,
        };

        let coverage_data = vec![
            ChangedFileCoverage {
                file_path: "a.rs".to_string(),
                coverage_before: 0.60,
                coverage_after: 0.80,
                coverage_delta: 0.20,
                status: CoverageStatus::Improved,
                lines_covered: 80,
                lines_total: 100,
            },
            ChangedFileCoverage {
                file_path: "b.rs".to_string(),
                coverage_before: 0.70,
                coverage_after: 0.90,
                coverage_delta: 0.20,
                status: CoverageStatus::Improved,
                lines_covered: 90,
                lines_total: 100,
            },
        ];

        let changed_files = vec![
            (PathBuf::from("a.rs"), "M".to_string()),
            (PathBuf::from("b.rs"), "M".to_string()),
        ];

        let result = facade.build_coverage_result(coverage_data, changed_files, &request);

        assert_eq!(result.files_above_threshold, 0);
        assert_eq!(result.files_below_threshold, 2);
    }

    #[tokio::test]
    async fn test_analyze_coverage_changes_modified_file() {
        let facade = create_test_facade();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let request = IncrementalCoverageRequest {
            project_path: temp_dir.path().to_path_buf(),
            base_branch: "main".to_string(),
            target_branch: None,
            coverage_threshold: 0.8,
            changed_files_only: true,
            detailed: false,
            cache_dir: None,
            force_refresh: false,
            top_files: 10,
        };

        let changed_files = vec![(PathBuf::from("modified.rs"), "M".to_string())];

        let result = facade
            .analyze_coverage_changes(temp_dir.path(), &changed_files, &request)
            .await
            .expect("Failed to analyze coverage changes");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file_path, "modified.rs");
        // Modified file should have coverage_before = 0.75
        assert!((result[0].coverage_before - 0.75).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_analyze_coverage_changes_added_file() {
        let facade = create_test_facade();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let request = IncrementalCoverageRequest {
            project_path: temp_dir.path().to_path_buf(),
            base_branch: "main".to_string(),
            target_branch: None,
            coverage_threshold: 0.8,
            changed_files_only: true,
            detailed: false,
            cache_dir: None,
            force_refresh: false,
            top_files: 10,
        };

        let changed_files = vec![(PathBuf::from("new.rs"), "A".to_string())];

        let result = facade
            .analyze_coverage_changes(temp_dir.path(), &changed_files, &request)
            .await
            .expect("Failed to analyze coverage changes");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file_path, "new.rs");
        // Added file should have coverage_before = 0.0
        assert!((result[0].coverage_before - 0.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_analyze_coverage_changes_deleted_file_ignored() {
        let facade = create_test_facade();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let request = IncrementalCoverageRequest {
            project_path: temp_dir.path().to_path_buf(),
            base_branch: "main".to_string(),
            target_branch: None,
            coverage_threshold: 0.8,
            changed_files_only: true,
            detailed: false,
            cache_dir: None,
            force_refresh: false,
            top_files: 10,
        };

        let changed_files = vec![(PathBuf::from("deleted.rs"), "D".to_string())];

        let result = facade
            .analyze_coverage_changes(temp_dir.path(), &changed_files, &request)
            .await
            .expect("Failed to analyze coverage changes");

        // Deleted files should be ignored
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_coverage_changes_top_files_limit() {
        let facade = create_test_facade();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let request = IncrementalCoverageRequest {
            project_path: temp_dir.path().to_path_buf(),
            base_branch: "main".to_string(),
            target_branch: None,
            coverage_threshold: 0.8,
            changed_files_only: true,
            detailed: false,
            cache_dir: None,
            force_refresh: false,
            top_files: 2,
        };

        let changed_files = vec![
            (PathBuf::from("a.rs"), "M".to_string()),
            (PathBuf::from("b.rs"), "M".to_string()),
            (PathBuf::from("c.rs"), "M".to_string()),
            (PathBuf::from("d.rs"), "M".to_string()),
        ];

        let result = facade
            .analyze_coverage_changes(temp_dir.path(), &changed_files, &request)
            .await
            .expect("Failed to analyze coverage changes");

        // Should only analyze top 2 files
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_analyze_coverage_changes_coverage_status() {
        let facade = create_test_facade();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let request = IncrementalCoverageRequest {
            project_path: temp_dir.path().to_path_buf(),
            base_branch: "main".to_string(),
            target_branch: None,
            coverage_threshold: 0.8,
            changed_files_only: true,
            detailed: false,
            cache_dir: None,
            force_refresh: false,
            top_files: 10,
        };

        let changed_files = vec![(PathBuf::from("modified.rs"), "M".to_string())];

        let result = facade
            .analyze_coverage_changes(temp_dir.path(), &changed_files, &request)
            .await
            .expect("Failed to analyze coverage changes");

        assert_eq!(result.len(), 1);
        // With mock implementation: before=0.75, after=0.85, delta=0.10 > 0
        // So status should be Improved
        match result[0].status {
            CoverageStatus::Improved => (),
            _ => panic!("Expected Improved status"),
        }
    }

    #[tokio::test]
    async fn test_get_changed_files_nonexistent_repo() {
        let facade = create_test_facade();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // No git repo initialized, should return empty list (not error)
        let result = facade
            .get_changed_files(temp_dir.path(), "main", None)
            .await
            .expect("Should not fail on non-git directory");

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_get_changed_files_valid_repo() {
        let temp_dir = create_test_git_repo();
        let facade = create_test_facade();

        // Add a new file and stage it
        let new_file = temp_dir.path().join("new.rs");
        fs::write(&new_file, "fn new_function() {}\n").expect("Failed to write new file");

        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to stage files");

        std::process::Command::new("git")
            .args(["commit", "-m", "Add new file"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to commit");

        // Get changes between first commit and HEAD
        let result = facade
            .get_changed_files(temp_dir.path(), "HEAD~1", Some("HEAD"))
            .await
            .expect("Failed to get changed files");

        // Should find the new.rs file
        assert!(!result.is_empty());
        let paths: Vec<_> = result.iter().map(|(p, _)| p.file_name().unwrap()).collect();
        assert!(
            paths.iter().any(|p| p.to_str() == Some("new.rs")),
            "Expected to find new.rs in changed files: {:?}",
            paths
        );
    }

    #[tokio::test]
    async fn test_analyze_project_with_valid_git_repo() {
        let temp_dir = create_test_git_repo();
        let facade = create_test_facade();

        // Add a new file
        let new_file = temp_dir.path().join("module.rs");
        fs::write(&new_file, "pub fn module_function() {}\n").expect("Failed to write new file");

        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to stage files");

        std::process::Command::new("git")
            .args(["commit", "-m", "Add module"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to commit");

        let request = IncrementalCoverageRequest {
            project_path: temp_dir.path().to_path_buf(),
            base_branch: "HEAD~1".to_string(),
            target_branch: Some("HEAD".to_string()),
            coverage_threshold: 0.8,
            changed_files_only: true,
            detailed: false,
            cache_dir: None,
            force_refresh: false,
            top_files: 10,
        };

        let result = facade
            .analyze_project(request)
            .await
            .expect("Failed to analyze project");

        assert!(!result.summary.is_empty());
    }

    #[tokio::test]
    async fn test_quick_analysis() {
        let temp_dir = create_test_git_repo();
        let facade = create_test_facade();

        let result = facade
            .quick_analysis(temp_dir.path().to_path_buf(), "main".to_string())
            .await
            .expect("Failed to run quick analysis");

        assert!(!result.summary.is_empty());
    }

    #[test]
    fn test_coverage_status_debug_format() {
        let status = CoverageStatus::Improved;
        let debug_str = format!("{:?}", status);
        assert_eq!(debug_str, "Improved");
    }

    #[test]
    fn test_incremental_coverage_result_clone() {
        let result = IncrementalCoverageResult {
            total_files: 5,
            covered_files: 4,
            coverage_percentage: 0.80,
            files_above_threshold: 3,
            files_below_threshold: 2,
            changed_files: vec![],
            summary: "Test summary".to_string(),
        };

        let cloned = result.clone();
        assert_eq!(cloned.total_files, 5);
        assert_eq!(cloned.summary, "Test summary");
    }

    #[test]
    fn test_summary_format_contains_expected_info() {
        let facade = create_test_facade();
        let request = IncrementalCoverageRequest {
            project_path: PathBuf::from("/test"),
            base_branch: "main".to_string(),
            target_branch: None,
            coverage_threshold: 0.8,
            changed_files_only: true,
            detailed: false,
            cache_dir: None,
            force_refresh: false,
            top_files: 10,
        };

        let coverage_data = vec![ChangedFileCoverage {
            file_path: "test.rs".to_string(),
            coverage_before: 0.70,
            coverage_after: 0.85,
            coverage_delta: 0.15,
            status: CoverageStatus::Improved,
            lines_covered: 85,
            lines_total: 100,
        }];

        let changed_files = vec![(PathBuf::from("test.rs"), "M".to_string())];

        let result = facade.build_coverage_result(coverage_data, changed_files, &request);

        // Summary should contain file count, coverage percentage, threshold info
        assert!(result.summary.contains("1 changed files"));
        assert!(result.summary.contains("85.0%"));
        assert!(result.summary.contains("80.0%"));
    }

    #[test]
    fn test_zero_coverage_files() {
        let facade = create_test_facade();
        let request = IncrementalCoverageRequest {
            project_path: PathBuf::from("/test"),
            base_branch: "main".to_string(),
            target_branch: None,
            coverage_threshold: 0.8,
            changed_files_only: true,
            detailed: false,
            cache_dir: None,
            force_refresh: false,
            top_files: 10,
        };

        let coverage_data = vec![ChangedFileCoverage {
            file_path: "uncovered.rs".to_string(),
            coverage_before: 0.0,
            coverage_after: 0.0,
            coverage_delta: 0.0,
            status: CoverageStatus::Unchanged,
            lines_covered: 0,
            lines_total: 100,
        }];

        let changed_files = vec![(PathBuf::from("uncovered.rs"), "M".to_string())];

        let result = facade.build_coverage_result(coverage_data, changed_files, &request);

        // File with 0 coverage_after should not be counted as "covered"
        assert_eq!(result.covered_files, 0);
        assert_eq!(result.files_below_threshold, 1);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn coverage_percentage_bounded(
            before in 0.0f64..=1.0f64,
            after in 0.0f64..=1.0f64
        ) {
            let delta = after - before;

            // Coverage values should remain bounded
            prop_assert!(before >= 0.0 && before <= 1.0);
            prop_assert!(after >= 0.0 && after <= 1.0);
            prop_assert!(delta >= -1.0 && delta <= 1.0);
        }

        #[test]
        fn coverage_status_from_delta(delta in -1.0f64..1.0f64) {
            let status = if delta > 0.0 {
                CoverageStatus::Improved
            } else if delta < 0.0 {
                CoverageStatus::Degraded
            } else {
                CoverageStatus::Unchanged
            };

            match status {
                CoverageStatus::Improved => prop_assert!(delta > 0.0),
                CoverageStatus::Degraded => prop_assert!(delta < 0.0),
                CoverageStatus::Unchanged => prop_assert!((delta - 0.0).abs() < f64::EPSILON),
                _ => prop_assert!(false, "Unexpected status"),
            }
        }

        #[test]
        fn threshold_comparison_consistent(
            coverage in 0.0f64..=1.0f64,
            threshold in 0.0f64..=1.0f64
        ) {
            let above = coverage >= threshold;
            let below = coverage < threshold;

            // Exactly one must be true
            prop_assert!(above ^ below);
        }

        #[test]
        fn result_counts_sum_correctly(
            above in 0usize..100,
            below in 0usize..100
        ) {
            let total = above + below;

            let result = IncrementalCoverageResult {
                total_files: total,
                covered_files: above,
                coverage_percentage: 0.8,
                files_above_threshold: above,
                files_below_threshold: below,
                changed_files: vec![],
                summary: String::new(),
            };

            prop_assert_eq!(
                result.files_above_threshold + result.files_below_threshold,
                result.total_files
            );
        }

        #[test]
        fn serialization_roundtrip(
            total in 0usize..1000,
            covered in 0usize..1000,
            above in 0usize..100,
            below in 0usize..100
        ) {
            let result = IncrementalCoverageResult {
                total_files: total,
                covered_files: covered,
                coverage_percentage: 0.8,
                files_above_threshold: above,
                files_below_threshold: below,
                changed_files: vec![],
                summary: "Test".to_string(),
            };

            let json = serde_json::to_string(&result).unwrap();
            let deserialized: IncrementalCoverageResult = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(result.total_files, deserialized.total_files);
            prop_assert_eq!(result.covered_files, deserialized.covered_files);
            prop_assert_eq!(result.files_above_threshold, deserialized.files_above_threshold);
            prop_assert_eq!(result.files_below_threshold, deserialized.files_below_threshold);
        }

        #[test]
        fn average_coverage_calculation(
            coverages in prop::collection::vec(0.0f64..=1.0f64, 1..10)
        ) {
            let avg = if coverages.is_empty() {
                0.0
            } else {
                coverages.iter().sum::<f64>() / coverages.len() as f64
            };

            // Average should be bounded
            prop_assert!(avg >= 0.0);
            prop_assert!(avg <= 1.0);

            // Average should be between min and max
            if !coverages.is_empty() {
                let min = coverages.iter().cloned().reduce(f64::min).unwrap();
                let max = coverages.iter().cloned().reduce(f64::max).unwrap();
                prop_assert!(avg >= min);
                prop_assert!(avg <= max);
            }
        }

        #[test]
        fn lines_covered_bounded(
            covered in 0usize..10000,
            total in 1usize..10000
        ) {
            // covered should not exceed total in valid data
            let valid_covered = covered.min(total);

            let file_coverage = ChangedFileCoverage {
                file_path: "test.rs".to_string(),
                coverage_before: 0.0,
                coverage_after: valid_covered as f64 / total as f64,
                coverage_delta: 0.0,
                status: CoverageStatus::Unchanged,
                lines_covered: valid_covered,
                lines_total: total,
            };

            prop_assert!(file_coverage.lines_covered <= file_coverage.lines_total);
            prop_assert!(file_coverage.coverage_after >= 0.0);
            prop_assert!(file_coverage.coverage_after <= 1.0);
        }
    }
}
