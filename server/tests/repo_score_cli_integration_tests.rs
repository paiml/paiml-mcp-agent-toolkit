//! Integration tests for `pmat repo-score` CLI command using assert_cmd
//!
//! RED PHASE: These tests MUST fail initially to prove EXTREME TDD methodology

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_repo_score_help() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["repo-score", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Calculate repository health score"))
        .stdout(predicate::str::contains("--path"))
        .stdout(predicate::str::contains("--format"));
}

#[test]
fn test_repo_score_basic_execution() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Create minimal valid repository
    fs::write(repo_path.join("README.md"), "# Test Project\n\n## Overview\nTest").unwrap();

    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["repo-score", "--path", repo_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Repository Health Score"))
        .stdout(predicate::str::contains("Total Score:"))
        .stdout(predicate::str::contains("Grade:"));
}

#[test]
fn test_repo_score_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    fs::write(repo_path.join("README.md"), "# Test\n").unwrap();

    let mut cmd = Command::cargo_bin("pmat").unwrap();
    let output = cmd
        .args(["repo-score", "--path", repo_path.to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify valid JSON
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Output should be valid JSON");

    assert!(json["total_score"].is_number());
    assert!(json["final_score"].is_number());
    assert!(json["grade"].is_string());
}

#[test]
fn test_repo_score_text_output() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    fs::write(repo_path.join("README.md"), "# Test\n").unwrap();

    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["repo-score", "--path", repo_path.to_str().unwrap(), "--format", "text"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Total Score:"))
        .stdout(predicate::str::contains("Grade:"))
        .stdout(predicate::str::contains("Documentation:"))
        .stdout(predicate::str::contains("Pre-commit Hooks:"));
}

#[test]
fn test_repo_score_markdown_output() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    fs::write(repo_path.join("README.md"), "# Test\n").unwrap();

    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["repo-score", "--path", repo_path.to_str().unwrap(), "--format", "markdown"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Repository Health Score"))
        .stdout(predicate::str::contains("## Summary"))
        .stdout(predicate::str::contains("| Category |"));
}

#[test]
fn test_repo_score_current_directory() {
    // Should work without --path argument (uses current directory)
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["repo-score"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Repository Health Score"));
}

#[test]
fn test_repo_score_nonexistent_path() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["repo-score", "--path", "/nonexistent/path/that/does/not/exist"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("does not exist")));
}

#[test]
fn test_repo_score_with_verbose() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    fs::write(repo_path.join("README.md"), "# Test\n").unwrap();

    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["repo-score", "--path", repo_path.to_str().unwrap(), "--verbose"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Repository Health Score"));
}

#[test]
fn test_repo_score_shows_categories() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    fs::write(repo_path.join("README.md"), "# Test\n").unwrap();

    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["repo-score", "--path", repo_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Documentation"))
        .stdout(predicate::str::contains("Pre-commit Hooks"))
        .stdout(predicate::str::contains("Repository Hygiene"))
        .stdout(predicate::str::contains("Build/Test Automation"))
        .stdout(predicate::str::contains("Continuous Integration"));
}

#[test]
fn test_repo_score_shows_grade() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    fs::write(repo_path.join("README.md"), "# Test\n").unwrap();

    let mut cmd = Command::cargo_bin("pmat").unwrap();
    let output = cmd
        .args(["repo-score", "--path", repo_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should show a grade line (with any spacing)
    assert!(stdout.contains("Grade:"));
    // Verify at least one grade letter appears
    assert!(
        stdout.contains(" A+") ||
        stdout.contains(" A") ||
        stdout.contains(" A-") ||
        stdout.contains(" B+") ||
        stdout.contains(" B") ||
        stdout.contains(" C") ||
        stdout.contains(" D") ||
        stdout.contains(" F")
    );
}
