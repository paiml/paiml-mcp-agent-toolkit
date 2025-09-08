//! Sprint 85 TDD Tests: Code Entropy Reduction
//! 
//! RED Phase: Comprehensive test coverage for complexity hotspots
//! Target: collect_files_recursive complexity 14 → ≤10 (A+ standard)

use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tokio::fs;

/// Test data for file collection scenarios
struct FileCollectionTestData {
    temp_dir: TempDir,
    source_files: Vec<PathBuf>,
    directories: Vec<PathBuf>,
    expected_collected: Vec<PathBuf>,
}

impl FileCollectionTestData {
    async fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path();

        // Create test directory structure
        let directories = vec![
            base_path.join("src"),
            base_path.join("src/handlers"),
            base_path.join("tests"),
            base_path.join("target"), // Should be excluded
            base_path.join(".git"),   // Should be excluded
            base_path.join("node_modules"), // Should be excluded
        ];

        for dir in &directories {
            fs::create_dir_all(dir).await.expect("Failed to create test directory");
        }

        // Create test source files
        let source_files = vec![
            base_path.join("src/main.rs"),
            base_path.join("src/handlers/mod.rs"),
            base_path.join("src/handlers/complexity.rs"),
            base_path.join("tests/integration_test.rs"),
            base_path.join("README.md"), // Not a source file
            base_path.join("target/debug/app"), // In excluded directory
            base_path.join(".git/config"), // In excluded directory
        ];

        for file in &source_files {
            if let Some(parent) = file.parent() {
                fs::create_dir_all(parent).await.ok();
            }
            fs::write(file, "// Test content").await.expect("Failed to create test file");
        }

        let expected_collected = vec![
            base_path.join("src/main.rs"),
            base_path.join("src/handlers/mod.rs"), 
            base_path.join("src/handlers/complexity.rs"),
            base_path.join("tests/integration_test.rs"),
        ];

        Self {
            temp_dir,
            source_files,
            directories,
            expected_collected,
        }
    }
}

// RED PHASE TESTS: Define expected behavior before refactoring

#[tokio::test]
async fn test_collect_files_recursive_basic_functionality() {
    let test_data = FileCollectionTestData::new().await;
    let mut collected_files = Vec::new();

    // This will fail until we implement the refactored version
    let result = collect_files_recursive_new(
        test_data.temp_dir.path(),
        &mut collected_files,
        &None,
        &None,
    ).await;

    assert!(result.is_ok(), "File collection should succeed");
    assert_eq!(
        collected_files.len(),
        test_data.expected_collected.len(),
        "Should collect expected number of source files"
    );
    
    // Verify only source files are collected
    for file in &collected_files {
        assert!(is_source_file_new(file), "Only source files should be collected: {:?}", file);
    }
}

#[tokio::test]
async fn test_exclude_pattern_filtering() {
    let test_data = FileCollectionTestData::new().await;
    let mut collected_files = Vec::new();
    let exclude_pattern = Some("handlers".to_string());

    let result = collect_files_recursive_new(
        test_data.temp_dir.path(),
        &mut collected_files,
        &None,
        &exclude_pattern,
    ).await;

    assert!(result.is_ok(), "File collection with exclude should succeed");
    
    // Should not collect files containing "handlers" in path
    for file in &collected_files {
        let path_str = file.to_string_lossy();
        assert!(!path_str.contains("handlers"), "Excluded files should not be collected: {:?}", file);
    }
}

#[tokio::test]
async fn test_include_pattern_filtering() {
    let test_data = FileCollectionTestData::new().await;
    let mut collected_files = Vec::new();
    let include_pattern = Some("main".to_string());

    let result = collect_files_recursive_new(
        test_data.temp_dir.path(),
        &mut collected_files,
        &include_pattern,
        &None,
    ).await;

    assert!(result.is_ok(), "File collection with include should succeed");
    
    // Should only collect files containing "main" in path
    for file in &collected_files {
        let path_str = file.to_string_lossy();
        assert!(path_str.contains("main"), "Only included files should be collected: {:?}", file);
    }
}

#[tokio::test]
async fn test_directory_exclusion_logic() {
    let test_data = FileCollectionTestData::new().await;
    let mut collected_files = Vec::new();

    let result = collect_files_recursive_new(
        test_data.temp_dir.path(),
        &mut collected_files,
        &None,
        &None,
    ).await;

    assert!(result.is_ok(), "File collection should succeed");
    
    // Verify no files from excluded directories
    for file in &collected_files {
        let path_str = file.to_string_lossy();
        assert!(!path_str.contains("target"), "target directory should be excluded");
        assert!(!path_str.contains(".git"), ".git directory should be excluded");
        assert!(!path_str.contains("node_modules"), "node_modules directory should be excluded");
    }
}

#[tokio::test]
async fn test_source_file_detection() {
    // Test the is_source_file_new function
    assert!(is_source_file_new(&PathBuf::from("test.rs")));
    assert!(is_source_file_new(&PathBuf::from("test.js")));
    assert!(is_source_file_new(&PathBuf::from("test.ts")));
    assert!(is_source_file_new(&PathBuf::from("test.py")));
    assert!(is_source_file_new(&PathBuf::from("test.java")));
    
    assert!(!is_source_file_new(&PathBuf::from("README.md")));
    assert!(!is_source_file_new(&PathBuf::from("config.toml")));
    assert!(!is_source_file_new(&PathBuf::from("image.png")));
}

// Property-based tests for entropy validation

#[tokio::test]
async fn test_empty_directory_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let empty_subdir = temp_dir.path().join("empty");
    fs::create_dir(&empty_subdir).await.expect("Failed to create empty directory");
    
    let mut collected_files = Vec::new();
    let result = collect_files_recursive_new(
        &empty_subdir,
        &mut collected_files,
        &None,
        &None,
    ).await;

    assert!(result.is_ok(), "Empty directory should not cause errors");
    assert!(collected_files.is_empty(), "Empty directory should yield no files");
}

#[tokio::test]
async fn test_nonexistent_directory_error_handling() {
    let nonexistent_path = PathBuf::from("/nonexistent/directory");
    let mut collected_files = Vec::new();
    
    let result = collect_files_recursive_new(
        &nonexistent_path,
        &mut collected_files,
        &None,
        &None,
    ).await;

    assert!(result.is_err(), "Nonexistent directory should return error");
}

// Integration test for the complete workflow
#[tokio::test]
async fn test_complex_directory_structure_integration() {
    let test_data = FileCollectionTestData::new().await;
    
    // Create a more complex structure
    let complex_dirs = vec![
        test_data.temp_dir.path().join("src/deeply/nested/modules"),
        test_data.temp_dir.path().join("src/another/branch"),
    ];
    
    for dir in &complex_dirs {
        fs::create_dir_all(dir).await.expect("Failed to create complex directory");
        let test_file = dir.join("test.rs");
        fs::write(&test_file, "// Test content").await.expect("Failed to create test file");
    }
    
    let mut collected_files = Vec::new();
    let result = collect_files_recursive_new(
        test_data.temp_dir.path(),
        &mut collected_files,
        &None,
        &None,
    ).await;

    assert!(result.is_ok(), "Complex directory traversal should succeed");
    assert!(
        collected_files.len() >= test_data.expected_collected.len() + 2, 
        "Should collect files from nested directories"
    );
}

// GREEN PHASE PLACEHOLDER: These functions will be implemented during refactoring

/// NEW EXTRACTED FUNCTION: Main recursive collection with reduced complexity
/// Target complexity: ≤10 (A+ standard)
async fn collect_files_recursive_new(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    include: &Option<String>,
    exclude: &Option<String>,
) -> anyhow::Result<()> {
    // This is a placeholder - will be implemented in GREEN phase
    // to demonstrate the Extract Method pattern
    todo!("GREEN PHASE: Implement with ≤10 complexity using extracted helper functions")
}

/// NEW EXTRACTED FUNCTION: Check if path should be excluded
/// Target complexity: ≤3 (A+ standard)
fn should_exclude_path(path_str: &str, exclude_pattern: &Option<String>) -> bool {
    todo!("GREEN PHASE: Extract exclude logic with ≤3 complexity")
}

/// NEW EXTRACTED FUNCTION: Check if path should be included
/// Target complexity: ≤3 (A+ standard)  
fn should_include_path(path_str: &str, include_pattern: &Option<String>) -> bool {
    todo!("GREEN PHASE: Extract include logic with ≤3 complexity")
}

/// NEW EXTRACTED FUNCTION: Check if directory should be traversed
/// Target complexity: ≤5 (A+ standard)
fn should_traverse_directory(dir_name: &str) -> bool {
    todo!("GREEN PHASE: Extract directory filtering with ≤5 complexity")
}

/// NEW EXTRACTED FUNCTION: Process individual directory entry
/// Target complexity: ≤8 (A+ standard)
async fn process_directory_entry(
    entry_path: PathBuf,
    files: &mut Vec<PathBuf>,
    include: &Option<String>,
    exclude: &Option<String>,
) -> anyhow::Result<()> {
    todo!("GREEN PHASE: Extract entry processing with ≤8 complexity")
}

/// NEW EXTRACTED FUNCTION: Enhanced source file detection
/// Target complexity: ≤3 (A+ standard)
fn is_source_file_new(path: &Path) -> bool {
    todo!("GREEN PHASE: Extract source file detection with ≤3 complexity")
}

// Complexity validation tests - these should pass after GREEN phase
#[tokio::test] 
async fn test_complexity_targets_achieved() {
    // This test will validate our complexity reduction targets
    // It will be enabled after the GREEN phase implementation
    
    // Target: collect_files_recursive_new ≤10 complexity
    // Target: should_exclude_path ≤3 complexity
    // Target: should_include_path ≤3 complexity
    // Target: should_traverse_directory ≤5 complexity
    // Target: process_directory_entry ≤8 complexity
    // Target: is_source_file_new ≤3 complexity
    
    // TODO: Add complexity measurement assertions in GREEN phase
}

#[cfg(test)]
mod entropy_reduction_validation {
    use super::*;
    
    /// Validate that our refactoring reduces code entropy
    /// This will be implemented to measure entropy reduction
    #[tokio::test]
    async fn test_entropy_reduction_achieved() {
        // TODO: GREEN PHASE - Implement entropy measurement
        // Target: Demonstrate measurable entropy reduction
        // Success criteria: ≥50% reduction in function entropy
    }
}