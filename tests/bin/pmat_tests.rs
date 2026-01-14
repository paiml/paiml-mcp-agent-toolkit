//! Tests for the pmat binary

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_pmat_version() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("pmat"));
}

#[test]
fn test_pmat_help() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("Commands:"));
}

#[test]
fn test_pmat_analyze_help() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(&["analyze", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("analyze"));
}

#[test]
fn test_pmat_generate_help() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(&["generate", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("generate"));
}

#[test]
fn test_pmat_demo_help() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(&["demo", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("demo"));
}

#[test]
fn test_pmat_context_help() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(&["context", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("context"));
}

#[test]
fn test_pmat_refactor_help() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(&["refactor", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("refactor"));
}

#[test]
fn test_pmat_validate_help() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(&["validate", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("validate"));
}

#[test]
fn test_pmat_mcp_protocol() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("--mcp");
    
    // Write a minimal initialize request
    cmd.write_stdin(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#);
    
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Check for JSON-RPC response
    assert!(stdout.contains("jsonrpc"));
    assert!(stdout.contains("2.0"));
}

#[test]
fn test_pmat_http_protocol() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(&["--protocol", "http"]);
    
    // HTTP mode should fail without a port or other config
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("http").or(predicate::str::contains("HTTP")));
}

#[test]
fn test_pmat_invalid_command() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.arg("nonexistent");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized").or(predicate::str::contains("invalid")));
}

#[test]
fn test_pmat_verbose_flag() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(&["-v", "--version"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("pmat"));
}

#[test]
fn test_pmat_multiple_verbose() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(&["-vvv", "--version"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("pmat"));
}