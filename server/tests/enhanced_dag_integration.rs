use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
#[cfg_attr(feature = "skip-slow-tests", ignore)]
fn test_enhanced_dag_analysis() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("enhanced-dag.mmd");

    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args([
        "analyze",
        "dag",
        "--enhanced",
        "--include-duplicates",
        "--include-dead-code",
        "--show-complexity",
        "-o",
        output_path.to_str().unwrap(),
    ]);

    cmd.assert().success().stderr(predicate::str::contains(
        "Enhanced dependency graph written to",
    ));

    // Verify the file was created
    assert!(output_path.exists());

    // Check the content
    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("Graph Statistics"));
    assert!(content.contains("Analysis timestamp"));
}

#[test]
#[cfg_attr(feature = "skip-slow-tests", ignore)]
fn test_enhanced_analysis_backward_compatibility() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("regular-dag.mmd");

    // Test that regular DAG analysis still works without enhanced flags
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args([
        "analyze",
        "dag",
        "--dag-type",
        "full-dependency",
        "--show-complexity",
        "-o",
        output_path.to_str().unwrap(),
    ]);

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Dependency graph written to"));

    // Verify the file was created
    assert!(output_path.exists());

    // Check that it's a valid Mermaid graph
    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("graph") || content.contains("%%"));
}

#[test]
#[cfg_attr(feature = "skip-slow-tests", ignore)]
fn test_enhanced_flags_combinations() {
    let temp_dir = TempDir::new().unwrap();

    // Test with only duplicate detection
    let output_path = temp_dir.path().join("duplicates-only.mmd");
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args([
        "analyze",
        "dag",
        "--enhanced",
        "--include-duplicates",
        "-o",
        output_path.to_str().unwrap(),
    ]);
    cmd.assert().success();

    // Test with only dead code detection
    let output_path = temp_dir.path().join("dead-code-only.mmd");
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args([
        "analyze",
        "dag",
        "--enhanced",
        "--include-dead-code",
        "-o",
        output_path.to_str().unwrap(),
    ]);
    cmd.assert().success();

    // Test enhanced mode without additional analyses
    let output_path = temp_dir.path().join("enhanced-only.mmd");
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args([
        "analyze",
        "dag",
        "--enhanced",
        "-o",
        output_path.to_str().unwrap(),
    ]);
    cmd.assert().success();
}

#[test]
#[cfg_attr(feature = "skip-slow-tests", ignore)]
fn test_enhanced_dag_help() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["analyze", "dag", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--enhanced"))
        .stdout(predicate::str::contains("--include-duplicates"))
        .stdout(predicate::str::contains("--include-dead-code"))
        .stdout(predicate::str::contains(
            "Use enhanced vectorized analysis engine",
        ))
        .stdout(predicate::str::contains(
            "Include duplicate detection analysis",
        ))
        .stdout(predicate::str::contains("Include dead code analysis"));
}
