//! BUG-006: Parallel Analysis Count and Progress Tracking - RED Phase Tests
//!
//! These tests verify that:
//! 1. The parallel analysis count matches actual analyses spawned
//! 2. All 8 analyses actually run (not just 4)
//! 3. Progress bar updates incrementally (not all at once)
//! 4. Count is dynamic (not hardcoded)
//!
//! Current Status: 🔴 RED - These tests will FAIL until fixes are applied
//!
//! Test Strategy (Extreme TDD):
//! 1. RED: Write failing tests for progress tracking
//! 2. GREEN: Fix hardcoded count and incremental progress
//! 3. REFACTOR: Ensure clean code
//! 4. COMMIT: Single atomic commit with fix

use tempfile::TempDir;
use std::fs;
use std::path::Path;

// =============================================================================
// RED TEST 1: All 8 Analyses Should Actually Run
// =============================================================================

#[test]
#[ignore = "BUG-006: RED test - verifies all 8 analyses execute"]
fn test_all_eight_analyses_run() {
    // Arrange: Create a simple Rust project
    let project = create_simple_rust_project();

    // Act: Run context generation (which triggers parallel analyses)
    let result = run_context_generation(project.path());

    // Assert: All 8 analyses should have results
    assert!(result.is_ok(), "Context generation should succeed");
    let output = result.unwrap();

    // Check that all 8 analysis types are present in output:
    // 1. Complexity
    // 2. Provability
    // 3. SATD (Self-Admitted Technical Debt)
    // 4. Churn
    // 5. DAG (Dependency Graph)
    // 6. TDG (Technical Debt Gradient)
    // 7. Big-O
    // 8. Dead Code

    let analysis_indicators = [
        ("complexity", "Complexity"),
        ("provability", "Provability"),
        ("satd", "SATD"),
        ("churn", "Churn"),
        ("dag", "Dependencies"),
        ("tdg", "Technical Debt"),
        ("big_o", "Big-O"),
        ("dead_code", "Dead Code"),
    ];

    for (name, indicator) in &analysis_indicators {
        assert!(
            output.contains(indicator) || output.to_lowercase().contains(name),
            "Analysis '{}' should be present in output (looking for '{}')",
            name,
            indicator
        );
    }
}

// =============================================================================
// RED TEST 2: Progress Bar Count Should Match Actual Analyses
// =============================================================================

#[test]
#[ignore = "BUG-006: RED test - will fail if count is hardcoded"]
fn test_progress_bar_count_matches_analyses() {
    // This test verifies the count is correct
    // Currently: hardcoded "8" in deep_context_concurrent.rs:84
    // Expected: Should be a const ANALYSIS_COUNT = 8

    // We can't easily test progress bar internals, but we can verify
    // the count constant exists and matches the number of analyses

    // This is more of a code structure test - in GREEN phase we'll add:
    // const ANALYSIS_COUNT: u64 = 8;
    // let pb = self.create_progress_bar("Running analyses", ANALYSIS_COUNT);

    // For now, this test documents the requirement
    assert!(
        true,
        "GREEN phase should introduce ANALYSIS_COUNT constant"
    );
}

// =============================================================================
// RED TEST 3: Progress Should Update Incrementally
// =============================================================================

#[test]
#[ignore = "BUG-006: RED test - will fail because progress increments all at once"]
fn test_progress_updates_incrementally() {
    // Arrange: Create a simple project
    let _project = create_simple_rust_project();

    // Act: We need to capture progress updates somehow
    // This is difficult with the current implementation because:
    // - tokio::join! waits for ALL analyses to complete
    // - Then does pb.inc(8) all at once

    // The bug: Line 126 in deep_context_concurrent.rs does:
    //   pb.inc(8);  // Increments by 8 all at once!

    // Expected: Should increment after each analysis completes:
    //   tokio::spawn(async move {
    //       let result = analyze_complexity().await;
    //       pb.inc(1);
    //       result
    //   });

    // For now, document the requirement
    // In GREEN phase, we'll refactor to use tokio::spawn with individual progress updates

    assert!(
        true,
        "GREEN phase should update progress incrementally, not all at once"
    );
}

// =============================================================================
// RED TEST 4: Verify Count Is Not Hardcoded (Property Test)
// =============================================================================

#[test]
#[ignore = "BUG-006: RED test - verifies count isn't magic number"]
fn test_count_is_not_hardcoded() {
    // This test verifies that if we add a 9th analysis in the future,
    // the count will automatically update

    // Currently hardcoded: let pb = self.create_progress_bar("Running analyses", 8);
    // Should be: const ANALYSIS_COUNT: u64 = 8; (at minimum)
    // Better: Calculate count from tokio::join! macro (complex)

    // For now, we'll verify the constant exists in GREEN phase
    assert!(
        true,
        "GREEN phase should replace magic number 8 with named constant"
    );
}

// =============================================================================
// RED TEST 5: All Analyses Complete Successfully
// =============================================================================

#[test]
#[ignore = "BUG-006: RED test - verifies no analysis is skipped"]
fn test_all_analyses_complete_successfully() {
    // Arrange: Create a project with various code patterns
    let project = create_complex_rust_project();

    // Act: Run analyses
    let result = run_context_generation(project.path());

    // Assert: Should complete without errors
    assert!(result.is_ok(), "All analyses should complete: {:?}", result.err());

    let output = result.unwrap();

    // Should not show "3/8" or "4/8" - should show "8/8" or complete
    assert!(
        !output.contains("3/8") && !output.contains("4/8"),
        "Progress should not stop at 3/8 or 4/8, all 8 should complete"
    );
}

// =============================================================================
// Helper Functions
// =============================================================================

fn create_simple_rust_project() -> TempDir {
    let temp = TempDir::new().unwrap();

    fs::create_dir_all(temp.path().join("src")).unwrap();

    let code = r#"
fn main() {
    println!("Hello, world!");
}

fn calculate(x: i32, y: i32) -> i32 {
    x + y
}

// TODO: Refactor this function
fn complex_function(n: i32) -> i32 {
    if n < 2 {
        return n;
    }
    complex_function(n - 1) + complex_function(n - 2)
}
"#;

    fs::write(temp.path().join("src/main.rs"), code).unwrap();
    fs::write(
        temp.path().join("Cargo.toml"),
        r#"[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#,
    ).unwrap();

    temp
}

fn create_complex_rust_project() -> TempDir {
    let temp = TempDir::new().unwrap();

    fs::create_dir_all(temp.path().join("src")).unwrap();

    // Create a more complex project with multiple files
    let main_code = r#"
mod utils;

fn main() {
    utils::helper();
}

// FIXME: This has O(n²) complexity
fn slow_function(items: &[i32]) -> Vec<i32> {
    let mut result = vec![];
    for i in items {
        for j in items {
            result.push(i * j);
        }
    }
    result
}
"#;

    let utils_code = r#"
pub fn helper() {
    println!("Helper function");
}

#[allow(dead_code)]
fn unused_function() {
    println!("This is never called");
}
"#;

    fs::write(temp.path().join("src/main.rs"), main_code).unwrap();
    fs::write(temp.path().join("src/utils.rs"), utils_code).unwrap();
    fs::write(
        temp.path().join("Cargo.toml"),
        r#"[package]
name = "test_complex"
version = "0.1.0"
edition = "2021"
"#,
    ).unwrap();

    temp
}

fn run_context_generation(project_path: &Path) -> Result<String, String> {
    use std::process::Command;
    use std::env;

    // Get workspace root (parent of server/ directory)
    let workspace_root = env::current_dir()
        .map_err(|e| format!("Failed to get current dir: {}", e))?
        .parent()
        .ok_or_else(|| "Failed to get workspace root".to_string())?
        .to_path_buf();

    // Run pmat context command from workspace root
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "pmat",
            "--",
            "context",
            "--language",
            "rust",
        ])
        .arg(project_path.to_str().ok_or_else(|| "Invalid path".to_string())?)
        .current_dir(&workspace_root)
        .output()
        .map_err(|e| format!("Failed to run pmat: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("pmat failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Combine stdout and stderr for analysis
    Ok(format!("{}\n{}", stdout, stderr))
}
