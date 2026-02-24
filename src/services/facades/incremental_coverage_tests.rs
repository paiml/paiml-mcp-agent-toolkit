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
