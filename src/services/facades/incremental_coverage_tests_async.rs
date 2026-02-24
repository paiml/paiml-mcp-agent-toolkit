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
