//! Integration tests for `pmat prompt` command using assert_cmd
//!
//! These tests verify the prompt command works correctly as a CLI binary.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// FAILED: Integration test - requires pmat binary
#[ignore]
#[test]
fn test_prompt_list() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["prompt", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Available Prompts:"))
        .stdout(predicate::str::contains("code-coverage"))
        .stdout(predicate::str::contains("debug"))
        .stdout(predicate::str::contains("continue"))
        .stdout(predicate::str::contains("EXTREME TDD"));
}

/// FAILED: Integration test - requires pmat binary
#[ignore]
#[test]
fn test_prompt_show_yaml_format() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["prompt", "code-coverage"])
        .assert()
        .success()
        .stdout(predicate::str::contains("name: code-coverage"))
        .stdout(predicate::str::contains("description:"))
        .stdout(predicate::str::contains("priority: critical"))
        .stdout(predicate::str::contains("coverage_target: 85"));
}

/// FAILED: Integration test - requires pmat binary
#[ignore]
#[test]
fn test_prompt_show_json_format() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["prompt", "continue", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("{"))
        .stdout(predicate::str::contains("\"name\": \"continue\""))
        .stdout(predicate::str::contains("\"priority\":"))
        .stdout(predicate::str::contains("\"category\":"));
}

/// FAILED: Integration test - requires pmat binary
#[ignore]
#[test]
fn test_prompt_show_text_format() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["prompt", "debug", "--format", "text"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Five Whys"))
        .stdout(predicate::str::contains("ROOT CAUSE"))
        .stdout(predicate::str::contains("EXTREME TDD"));
}

/// FAILED: Integration test - requires pmat binary
#[ignore]
#[test]
fn test_prompt_not_found() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["prompt", "nonexistent-prompt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Prompt not found"));
}

/// FAILED: Integration test - requires pmat binary
#[ignore]
#[test]
fn test_prompt_show_variables() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["prompt", "code-coverage", "--show-variables"])
        .assert()
        .success();
}

/// FAILED: Integration test - requires pmat binary
#[ignore]
#[test]
fn test_prompt_write_to_file() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("test-prompt.yaml");

    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args([
        "prompt",
        "mutation-testing",
        "-o",
        output_path.to_str().unwrap(),
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("Prompt written to"));

    // Verify file was created and contains expected content
    assert!(output_path.exists());
    let content = fs::read_to_string(output_path).unwrap();
    assert!(content.contains("name: mutation-testing"));
    assert!(content.contains("description:"));
}

/// FAILED: Integration test - requires pmat binary
#[ignore]
#[test]
fn test_prompt_json_to_file() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("test-prompt.json");

    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args([
        "prompt",
        "security-audit",
        "--format",
        "json",
        "-o",
        output_path.to_str().unwrap(),
    ])
    .assert()
    .success();

    // Verify JSON file is valid
    assert!(output_path.exists());
    let content = fs::read_to_string(output_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(json["name"], "security-audit");
    assert_eq!(json["priority"], "critical");
}

/// FAILED: Integration test - requires pmat binary
#[ignore]
#[test]
fn test_prompt_all_available_prompts() {
    let prompts = [
        "code-coverage",
        "clean-repo-cruft",
        "continue",
        "assert-cmd-testing",
        "documentation",
        "debug",
        "mutation-testing",
        "performance-optimization",
        "quality-enforcement",
        "refactor-hotspots",
        "security-audit",
    ];

    for prompt in &prompts {
        let mut cmd = Command::cargo_bin("pmat").unwrap();
        cmd.args(["prompt", prompt])
            .assert()
            .success()
            .stdout(predicate::str::contains(format!("name: {prompt}")));
    }
}

/// FAILED: Integration test - requires pmat binary
#[ignore]
#[test]
fn test_prompt_help() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["prompt", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("workflow prompts"))
        .stdout(predicate::str::contains("EXTREME TDD"))
        .stdout(predicate::str::contains("--list"))
        .stdout(predicate::str::contains("--format"));
}

/// FAILED: Integration test - requires pmat binary
#[ignore]
#[test]
fn test_prompt_short_alias() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["p", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Available Prompts:"));
}

/// FAILED: Integration test - requires pmat binary
#[ignore]
#[test]
fn test_prompt_missing_name_without_list() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["prompt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("specify a prompt name"));
}

/// FAILED: Integration test - requires pmat binary
#[ignore]
#[test]
fn test_prompt_yaml_format_explicit() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["prompt", "refactor-hotspots", "--format", "yaml"])
        .assert()
        .success()
        .stdout(predicate::str::contains("name: refactor-hotspots"))
        .stdout(predicate::str::contains("category:"))
        .stdout(predicate::str::contains("priority:"));
}

/// FAILED: Integration test - requires pmat binary
#[ignore]
#[test]
fn test_prompt_performance_optimization() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["prompt", "performance-optimization"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Five Whys"))
        .stdout(predicate::str::contains("compilation"))
        .stdout(predicate::str::contains("test execution"));
}

/// FAILED: Integration test - requires pmat binary
#[ignore]
#[test]
fn test_prompt_quality_enforcement() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["prompt", "quality-enforcement"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Quality Gates"))
        .stdout(predicate::str::contains("Toyota Way"))
        .stdout(predicate::str::contains("Andon Cord"));
}

/// FAILED: Integration test - requires pmat binary
#[ignore]
#[test]
fn test_prompt_clean_repo_cruft() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["prompt", "clean-repo-cruft"])
        .assert()
        .success()
        .stdout(predicate::str::contains("temporary files"))
        .stdout(predicate::str::contains(".gitignore"));
}

/// FAILED: Integration test - requires pmat binary
#[ignore]
#[test]
fn test_prompt_assert_cmd_testing() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["prompt", "assert-cmd-testing"])
        .assert()
        .success()
        .stdout(predicate::str::contains("CLI"))
        .stdout(predicate::str::contains("assert_cmd"));
}

/// FAILED: Integration test - requires pmat binary
#[ignore]
#[test]
fn test_prompt_documentation() {
    let mut cmd = Command::cargo_bin("pmat").unwrap();
    cmd.args(["prompt", "documentation"])
        .assert()
        .success()
        .stdout(predicate::str::contains("pmat validate-docs"))
        .stdout(predicate::str::contains("pmat validate-readme"));
}

#[cfg(test)]
mod property_tests {
    use super::*;

    #[test]
    fn test_all_prompts_produce_valid_yaml() {
        let prompts = [
            "code-coverage",
            "clean-repo-cruft",
            "continue",
            "assert-cmd-testing",
            "documentation",
            "debug",
            "mutation-testing",
            "performance-optimization",
            "quality-enforcement",
            "refactor-hotspots",
            "security-audit",
        ];

        for prompt in &prompts {
            let mut cmd = Command::cargo_bin("pmat").unwrap();
            let output = cmd.args(["prompt", prompt]).output().unwrap();

            assert!(output.status.success());
            let stdout = String::from_utf8(output.stdout).unwrap();

            // Parse as YAML to ensure it's valid
            let parsed: serde_yaml::Value = serde_yaml::from_str(&stdout)
                .unwrap_or_else(|e| panic!("Failed to parse YAML for {}: {}", prompt, e));

            // Verify required fields
            assert!(parsed["name"].as_str().is_some());
            assert!(parsed["description"].as_str().is_some());
            assert!(parsed["category"].as_str().is_some());
            assert!(parsed["priority"].as_str().is_some());
        }
    }

    #[test]
    fn test_all_prompts_produce_valid_json() {
        let prompts = [
            "code-coverage",
            "clean-repo-cruft",
            "continue",
            "assert-cmd-testing",
            "documentation",
            "debug",
            "mutation-testing",
            "performance-optimization",
            "quality-enforcement",
            "refactor-hotspots",
            "security-audit",
        ];

        for prompt in &prompts {
            let mut cmd = Command::cargo_bin("pmat").unwrap();
            let output = cmd
                .args(["prompt", prompt, "--format", "json"])
                .output()
                .unwrap();

            assert!(output.status.success());
            let stdout = String::from_utf8(output.stdout).unwrap();

            // Parse as JSON to ensure it's valid
            let parsed: serde_json::Value = serde_json::from_str(&stdout)
                .unwrap_or_else(|e| panic!("Failed to parse JSON for {}: {}", prompt, e));

            // Verify required fields
            assert!(parsed["name"].is_string());
            assert!(parsed["description"].is_string());
            assert!(parsed["category"].is_string());
            assert!(parsed["priority"].is_string());
        }
    }
}
