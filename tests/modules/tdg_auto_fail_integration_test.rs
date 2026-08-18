#![cfg(feature = "red-phase-tests")]
/// EXTREME TDD: Integration test for TDG auto-fail on critical defects
/// RED PHASE: This test should FAIL until binary is rebuilt with TDG auto-fail code
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_tdg_auto_fail_on_critical_defects() {
    // Arrange: Create temp directory with Rust file containing .unwrap()
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("has_defect.rs");

    fs::write(
        &test_file,
        r#"pub fn has_critical_defect() -> i32 {
    Some(42).unwrap() // This should trigger TDG auto-fail
}
"#,
    )
    .expect("Failed to write test file");

    // Get path to pmat binary
    let binary_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("target/release/pmat");

    // Act: Run TDG analysis
    let output = Command::new(&binary_path)
        .args([
            "analyze",
            "tdg",
            "--path",
            temp_dir.path().to_str().unwrap(),
            "--format",
            "table",
        ])
        .output()
        .expect("Failed to execute pmat");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}\n{}", stderr, stdout);

    // Assert: TDG should FAIL with critical defects
    // 1. Exit code should be non-zero (failure)
    assert!(
        !output.status.success(),
        "TDG should fail when critical defects are found. Exit code: {:?}\nOutput:\n{}",
        output.status.code(),
        combined
    );

    // 2. Output should mention TDG auto-fail
    assert!(
        combined.contains("TDG ANALYSIS FAILED") || combined.contains("TDG auto-fail"),
        "Output should mention TDG auto-fail. Actual output:\n{}",
        combined
    );

    // 3. Output should mention critical defects
    assert!(
        combined.contains("critical defect") || combined.contains("CRITICAL DEFECT"),
        "Output should mention critical defects. Actual output:\n{}",
        combined
    );

    // 4. Should suggest running analyze defects
    assert!(
        combined.contains("analyze defects"),
        "Output should suggest running 'pmat analyze defects'. Actual output:\n{}",
        combined
    );
}

#[test]
fn test_tdg_passes_on_clean_code() {
    // Arrange: Create temp directory with clean Rust code (no defects)
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("clean.rs");

    fs::write(
        &test_file,
        r#"pub fn clean_function() -> Result<i32, String> {
    Some(42).ok_or_else(|| "error".to_string())
}
"#,
    )
    .expect("Failed to write test file");

    // Get path to pmat binary
    let binary_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("target/release/pmat");

    // Act: Run TDG analysis
    let output = Command::new(&binary_path)
        .args([
            "analyze",
            "tdg",
            "--path",
            temp_dir.path().to_str().unwrap(),
            "--format",
            "table",
        ])
        .output()
        .expect("Failed to execute pmat");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}\n{}", stderr, stdout);

    // Assert: TDG should PASS (exit code 0)
    assert!(
        output.status.success(),
        "TDG should pass on clean code. Exit code: {:?}\nOutput:\n{}",
        output.status.code(),
        combined
    );

    // Should mention checking for defects
    assert!(
        combined.contains("No critical defects found")
            || combined.contains("TDG analysis complete"),
        "Output should indicate clean analysis. Actual output:\n{}",
        combined
    );
}
