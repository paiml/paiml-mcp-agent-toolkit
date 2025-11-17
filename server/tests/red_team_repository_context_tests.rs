// EXTREME TDD: Red Team Repository Context Tests (RED Phase)
//
// Test real repository context building for Red Team Mode
// These tests will fail until RepositoryContext::from_path() is implemented

use pmat::red_team::RepositoryContext;
use std::path::Path;
use tempfile::TempDir;

// RED Test 1: Build context from git repository
#[test]
fn test_build_context_from_git_repo() {
    // Create temporary git repo
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    // Create a test file
    std::fs::write(repo_path.join("test.txt"), "Hello").unwrap();

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(repo_path)
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(["commit", "-m", "feat: Initial commit"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    // Build context from path (THIS WILL FAIL - not implemented yet)
    let context = RepositoryContext::from_path(repo_path).unwrap();

    // Should have git history
    assert!(context.has_git_history());

    // Should find the commit
    let commits = context.get_recent_commits(10);
    assert_eq!(commits.len(), 1);
    assert!(commits[0].message.contains("Initial commit"));
}

// RED Test 2: Build context without git repository
#[test]
fn test_build_context_from_non_git_directory() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Build context from non-git directory (THIS WILL FAIL - not implemented yet)
    let context = RepositoryContext::from_path(repo_path).unwrap();

    // Should not have git history
    assert!(!context.has_git_history());

    // Should have empty commits
    let commits = context.get_recent_commits(10);
    assert_eq!(commits.len(), 0);
}

// RED Test 3: Detect test files in repository
#[test]
fn test_detect_test_files() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Create directories and test files
    std::fs::create_dir_all(repo_path.join("tests")).unwrap();
    std::fs::create_dir_all(repo_path.join("src")).unwrap();
    std::fs::write(repo_path.join("tests/test_foo.rs"), "// test").unwrap();
    std::fs::write(repo_path.join("src/lib.rs"), "// lib").unwrap();

    // Build context from path
    let context = RepositoryContext::from_path(repo_path).unwrap();

    // Should detect test files
    let test_files = context.get_test_files();
    assert_eq!(test_files.len(), 1);
    assert!(test_files[0].to_string_lossy().contains("test_foo.rs"));
}

// RED Test 4: Find coverage reports
#[test]
fn test_find_coverage_reports() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Create coverage report
    std::fs::create_dir_all(repo_path.join("target/coverage")).unwrap();
    std::fs::write(
        repo_path.join("target/coverage/lcov.info"),
        "SF:src/lib.rs\nLF:10\nLH:8\nend_of_record\n",
    )
    .unwrap();

    // Build context (THIS WILL FAIL - not implemented yet)
    let context = RepositoryContext::from_path(repo_path).unwrap();

    // Should find coverage report
    assert!(context.has_coverage_report());

    // Should parse coverage percentage
    let coverage = context.get_coverage_percentage();
    assert!(coverage > 0.0);
    assert!(coverage <= 100.0);
}

// RED Test 5: Scan for test results
#[test]
fn test_scan_test_results() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Create test output
    std::fs::create_dir_all(repo_path.join("target/test-results")).unwrap();
    std::fs::write(
        repo_path.join("target/test-results/output.txt"),
        "test result: ok. 10 passed; 2 failed; 3 ignored",
    )
    .unwrap();

    // Build context (THIS WILL FAIL - not implemented yet)
    let context = RepositoryContext::from_path(repo_path).unwrap();

    // Should detect test results
    let test_info = context.get_test_execution_info();
    assert!(test_info.has_results);
    assert_eq!(test_info.passed_count, 10);
    assert_eq!(test_info.failed_count, 2);
    assert_eq!(test_info.ignored_count, 3);
}

// RED Test 6: Search for grep patterns in codebase
#[test]
fn test_code_grep_search() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Create files with patterns
    std::fs::write(repo_path.join("file1.rs"), "use sled::Db;").unwrap();
    std::fs::write(repo_path.join("file2.rs"), "use sled::Tree;").unwrap();
    std::fs::write(repo_path.join("file3.rs"), "use libsql::Connection;").unwrap();

    // Build context (THIS WILL FAIL - not implemented yet)
    let context = RepositoryContext::from_path(repo_path).unwrap();

    // Should find grep matches
    let sled_matches = context.grep_codebase("sled");
    assert_eq!(sled_matches.len(), 2);

    let libsql_matches = context.grep_codebase("libsql");
    assert_eq!(libsql_matches.len(), 1);
}

// Integration Test 7: Real repository analysis (GREEN phase - now enabled!)
#[test]
fn test_analyze_current_repository() {
    // Test on the actual pmat repository
    let repo_path = Path::new(".");

    // Build context from current repository
    let context = RepositoryContext::from_path(repo_path).unwrap();

    // Should have git history
    assert!(context.has_git_history());

    // Should have commits
    let commits = context.get_recent_commits(10);
    assert!(!commits.is_empty());

    // Should find test files
    let test_files = context.get_test_files();
    assert!(!test_files.is_empty());
}
