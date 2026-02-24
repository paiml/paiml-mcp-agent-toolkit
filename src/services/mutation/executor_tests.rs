// include!()'d from executor.rs
// Tests for MutantExecutor

#[test]
fn test_parse_compilation_error() {
    let executor = MutantExecutor::new(PathBuf::from("."));
    let output = "error[E0308]: mismatched types\n  --> src/lib.rs:10:5";

    let (status, failures, error) = executor.parse_test_output(output);

    assert_eq!(status, MutantStatus::CompileError);
    assert!(failures.is_empty());
    assert!(error.is_some());
}

#[test]
fn test_parse_test_failure() {
    let executor = MutantExecutor::new(PathBuf::from("."));
    let output = "running 3 tests\ntest test_add ... FAILED\ntest test_sub ... ok\n\ntest result: FAILED. 1 passed; 1 failed";

    let (status, failures, error) = executor.parse_test_output(output);

    assert_eq!(status, MutantStatus::Killed);
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0], "test_add");
    assert!(error.is_none());
}

#[test]
fn test_parse_all_tests_passed() {
    let executor = MutantExecutor::new(PathBuf::from("."));
    let output = "running 3 tests\ntest test_add ... ok\ntest test_sub ... ok\n\ntest result: ok. 2 passed; 0 failed";

    let (status, failures, error) = executor.parse_test_output(output);

    assert_eq!(status, MutantStatus::Survived);
    assert!(failures.is_empty());
    assert!(error.is_none());
}

#[test]
fn test_extract_multiple_failures() {
    let executor = MutantExecutor::new(PathBuf::from("."));
    let output = "test test_add ... FAILED\ntest test_sub ... FAILED\ntest test_mul ... ok";

    let failures = executor.extract_test_failures(output);

    assert_eq!(failures.len(), 2);
    assert!(failures.contains(&"test_add".to_string()));
    assert!(failures.contains(&"test_sub".to_string()));
}

#[tokio::test]
async fn test_atomic_write_basic() {
    use std::fs;
    use tempfile::NamedTempFile;

    // Create a temp file for testing
    let temp_file = NamedTempFile::new().unwrap();
    let test_path = temp_file.path().to_path_buf();

    // Write initial content
    fs::write(&test_path, "original content").unwrap();

    // Create executor and write new content atomically
    let executor = MutantExecutor::new(PathBuf::from("."));
    let new_content = "new content that should be atomic";

    executor
        .atomic_write(&test_path, new_content)
        .await
        .unwrap();

    // Verify content was written correctly
    let final_content = fs::read_to_string(&test_path).unwrap();
    assert_eq!(final_content, new_content);

    // Verify no temp file left behind
    let temp_path = test_path.with_extension("pmat_tmp");
    assert!(!temp_path.exists(), "Temp file should be cleaned up");
}

#[tokio::test]
async fn test_atomic_write_preserves_on_error() {
    use std::fs;
    use tempfile::TempDir;

    // Create a temp directory
    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path().join("test_file.rs");

    // Write original content
    fs::write(&test_path, "original content").unwrap();

    // Create executor
    let executor = MutantExecutor::new(PathBuf::from("."));

    // Try to write to non-existent directory (should fail)
    let bad_path = PathBuf::from("/nonexistent/directory/file.rs");
    let result = executor.atomic_write(&bad_path, "new content").await;

    // Verify operation failed
    assert!(
        result.is_err(),
        "Should fail to write to nonexistent directory"
    );

    // Verify original file unchanged
    let final_content = fs::read_to_string(&test_path).unwrap();
    assert_eq!(
        final_content, "original content",
        "Original file should be unchanged"
    );
}
