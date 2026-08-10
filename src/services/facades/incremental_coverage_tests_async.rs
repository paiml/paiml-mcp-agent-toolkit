#[tokio::test]
async fn test_analyze_coverage_changes_modified_file() {
    let facade = create_test_facade();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let request = IncrementalCoverageRequest {
        project_path: temp_dir.path().to_path_buf(),
        base_branch: "main".to_string(),
        target_branch: None,
        coverage_threshold: 80.0,
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
    // GH #658: this used to assert `coverage_before == 0.75`, i.e. it pinned
    // the mock. Baseline coverage needs a coverage artifact for the base
    // branch, which is not on disk, so it is not measured.
    assert_eq!(result[0].coverage_before, None);
    assert_eq!(result[0].coverage_delta, None);
}

#[tokio::test]
async fn test_analyze_coverage_changes_added_file() {
    let facade = create_test_facade();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let request = IncrementalCoverageRequest {
        project_path: temp_dir.path().to_path_buf(),
        base_branch: "main".to_string(),
        target_branch: None,
        coverage_threshold: 80.0,
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
    // GH #658: previously asserted the mock's `coverage_before == 0.0`. An
    // added file has no baseline to measure either — absent, not zero.
    assert_eq!(result[0].coverage_before, None);
}

#[tokio::test]
async fn test_analyze_coverage_changes_deleted_file_ignored() {
    let facade = create_test_facade();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let request = IncrementalCoverageRequest {
        project_path: temp_dir.path().to_path_buf(),
        base_branch: "main".to_string(),
        target_branch: None,
        coverage_threshold: 80.0,
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
        coverage_threshold: 80.0,
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

    // This used to assert `result.len() == 2`, i.e. that `--top-files 2` stopped
    // the *analysis* after 2 files. That was the bug: the summary counts were
    // then derived from the truncated vector while `total_files` came from the
    // full changed-file list, so `files_not_measured` just echoed `--top-files`
    // and the remaining changed files were unaccounted for. `--top-files` is a
    // display limit, applied in `build_coverage_result`, so every changed file
    // is analyzed here.
    assert_eq!(result.len(), 4, "every changed file must be analyzed");

    // ...and the display limit still caps what is rendered, without moving any count.
    let summarized = facade.build_coverage_result(result, changed_files, &request);
    assert_eq!(
        summarized.changed_files.len(),
        2,
        "--top-files 2 caps the displayed list"
    );
    assert_eq!(
        summarized.total_files, 4,
        "--top-files must not shrink total_files"
    );
}

#[tokio::test]
async fn test_analyze_coverage_changes_coverage_status() {
    let facade = create_test_facade();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let request = IncrementalCoverageRequest {
        project_path: temp_dir.path().to_path_buf(),
        base_branch: "main".to_string(),
        target_branch: None,
        coverage_threshold: 80.0,
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
    // GH #658: this used to assert `Improved`, which followed from the mock's
    // constant before=0.75 / after=0.85. With no coverage artifact for the
    // project there is no direction to claim.
    assert_eq!(result[0].status, CoverageStatus::NotMeasured);
    assert_eq!(result[0].coverage_delta, None);
}

#[tokio::test]
async fn test_get_changed_files_nonexistent_repo() {
    let facade = create_test_facade();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // This used to assert that a non-git directory came back as an empty
    // changelist ("should return empty list (not error)"). That was the bug:
    // `git diff` failing was swallowed, so incremental-coverage rendered an
    // all-zero "clean" gate report and exited 0 for a directory it could not
    // diff at all. A coverage gate must never report a pass because the diff
    // could not be taken.
    let result = facade.get_changed_files(temp_dir.path(), "main", None).await;

    let err = result
        .expect_err("a non-git directory must surface as an error, not an empty changelist")
        .to_string();
    assert!(
        err.contains("git diff"),
        "the error must say the diff failed: {err}"
    );
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
        coverage_threshold: 80.0,
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
        coverage_percentage: Some(80.0),
        files_above_threshold: 3,
        files_below_threshold: 2,
        files_not_measured: 0,
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
        coverage_threshold: 80.0,
        changed_files_only: true,
        detailed: false,
        cache_dir: None,
        force_refresh: false,
        top_files: 10,
    };

    let coverage_data = vec![ChangedFileCoverage {
        file_path: "test.rs".to_string(),
        coverage_before: Some(70.0),
        coverage_after: Some(85.0),
        coverage_delta: Some(15.0),
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
        coverage_threshold: 80.0,
        changed_files_only: true,
        detailed: false,
        cache_dir: None,
        force_refresh: false,
        top_files: 10,
    };

    let coverage_data = vec![ChangedFileCoverage {
        file_path: "uncovered.rs".to_string(),
        coverage_before: Some(0.0),
        coverage_after: Some(0.0),
        coverage_delta: Some(0.0),
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
