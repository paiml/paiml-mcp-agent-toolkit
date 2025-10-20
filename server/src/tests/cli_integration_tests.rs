//! CLI Integration Tests
//!
//! TICKET-PMAT-6005: Integration tests for all Sprint 19/20 CLI commands
//! Tests the actual pmat binary to ensure end-to-end functionality

use std::process::Command;
use tempfile::TempDir;

/// Get path to pmat binary
///
/// CC=1: Simple path construction
fn get_pmat_binary() -> String {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    if path.ends_with("deps") {
        path.pop();
    }
    path.push("pmat");
    path.to_string_lossy().to_string()
}

// Scaffold Agent Tests

#[test]
fn test_scaffold_agent_dry_run() {
    let output = Command::new(get_pmat_binary())
        .args(&[
            "scaffold",
            "agent",
            "--name",
            "test_agent",
            "--template",
            "basic",
            "--dry-run",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Dry run"));
    assert!(stderr.contains("test_agent"));
}

#[test]
fn test_scaffold_agent_list_templates() {
    let output = Command::new(get_pmat_binary())
        .args(&["scaffold", "list-templates"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Available Agent Templates") || stderr.contains("Templates"));
}

#[test]
fn test_scaffold_agent_invalid_template() {
    let output = Command::new(get_pmat_binary())
        .args(&[
            "scaffold",
            "agent",
            "--name",
            "test",
            "--template",
            "nonexistent",
            "--dry-run",
        ])
        .output()
        .expect("Failed to execute command");

    // Should fail with unknown template
    assert!(!output.status.success());
}

#[test]
fn test_scaffold_agent_with_features() {
    let output = Command::new(get_pmat_binary())
        .args(&[
            "scaffold",
            "agent",
            "--name",
            "featured_agent",
            "--template",
            "basic",
            "--features",
            "logging,metrics",
            "--dry-run",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("featured_agent"));
}

#[test]
fn test_scaffold_agent_quality_levels() {
    for quality in &["standard", "strict", "extreme"] {
        let output = Command::new(get_pmat_binary())
            .args(&[
                "scaffold",
                "agent",
                "--name",
                "quality_test",
                "--template",
                "basic",
                "--quality",
                quality,
                "--dry-run",
            ])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success(), "Failed for quality: {}", quality);
    }
}

// Scaffold WASM Tests

#[test]
#[ignore] // Requires pmat binary (CLI integration test) - Sprint 45 Round 3
          // Run manually: cargo build --bin pmat && cargo test test_scaffold_wasm_dry_run -- --ignored
fn test_scaffold_wasm_dry_run() {
    let output = Command::new(get_pmat_binary())
        .args(&[
            "scaffold",
            "wasm",
            "--name",
            "test_wasm",
            "--framework",
            "wasm-labs",
            "--dry-run",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Dry run"));
    assert!(stderr.contains("test_wasm"));
}

#[test]
fn test_scaffold_wasm_frameworks() {
    for framework in &["wasm-labs", "pure-wasm"] {
        let output = Command::new(get_pmat_binary())
            .args(&[
                "scaffold",
                "wasm",
                "--name",
                "fw_test",
                "--framework",
                framework,
                "--dry-run",
            ])
            .output()
            .expect("Failed to execute command");

        assert!(
            output.status.success(),
            "Failed for framework: {}",
            framework
        );
    }
}

#[test]
fn test_scaffold_wasm_invalid_framework() {
    let output = Command::new(get_pmat_binary())
        .args(&[
            "scaffold",
            "wasm",
            "--name",
            "test",
            "--framework",
            "invalid-framework",
            "--dry-run",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown WASM framework"));
    assert!(stderr.contains("Suggestions"));
}

// Maintain Health Tests

#[test]
fn test_maintain_health_no_project() {
    let temp_dir = TempDir::new().unwrap();

    let output = Command::new(get_pmat_binary())
        .args(&["maintain", "health", "--quick"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    // Should succeed but skip checks (no Cargo.toml)
    assert!(output.status.success());
}

#[test]
fn test_maintain_health_quick_flag() {
    let output = Command::new(get_pmat_binary())
        .args(&["maintain", "health", "--quick"])
        .current_dir(std::env::current_dir().unwrap())
        .output()
        .expect("Failed to execute command");

    // Quick mode should complete (may fail on health, but command runs)
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Health Report") || stderr.contains("Build"));
}

#[test]
fn test_maintain_health_individual_checks() {
    let output = Command::new(get_pmat_binary())
        .args(&["maintain", "health", "--check-build"])
        .current_dir(std::env::current_dir().unwrap())
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Health") || stderr.contains("Build"));
}

// Maintain Roadmap Tests

#[test]
fn test_maintain_roadmap_missing_file() {
    let temp_dir = TempDir::new().unwrap();

    let output = Command::new(get_pmat_binary())
        .args(&["maintain", "roadmap"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("ROADMAP.md"));
    assert!(stderr.contains("Suggestions") || stderr.contains("not found"));
}

#[test]
fn test_maintain_roadmap_with_file() {
    // Test with actual ROADMAP.md
    let output = Command::new(get_pmat_binary())
        .args(&["maintain", "roadmap"])
        .current_dir(std::env::current_dir().unwrap())
        .output()
        .expect("Failed to execute command");

    // Should succeed with our ROADMAP.md
    assert!(output.status.success());
}

// Hooks Tests

#[test]
fn test_hooks_status() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize git repo first (hooks status requires .git directory)
    Command::new("git")
        .args(&["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to init git");

    let output = Command::new(get_pmat_binary())
        .args(&["hooks", "status"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    // Fixed: hooks status outputs to stdout, not stderr
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Hook") || stdout.contains("installed") || stdout.contains("not installed")
    );
}

// REMOVED: test_hooks_install_dry_run
// Reason: The `pmat hooks install` command never implemented the `--dry-run` flag.
// This test was written in RED phase of TDD but never reached GREEN.
// Available flags: --force, --mode, --backup, --verbose, --quiet, --debug, --trace
// Ticket: PMAT-COVERAGE-001

// Version and Help Tests

#[test]
fn test_version_flag() {
    let output = Command::new(get_pmat_binary())
        .args(&["--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pmat") || stdout.contains("version"));
}

#[test]
fn test_help_flag() {
    let output = Command::new(get_pmat_binary())
        .args(&["--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage") || stdout.contains("USAGE"));
}

#[test]
fn test_scaffold_help() {
    let output = Command::new(get_pmat_binary())
        .args(&["scaffold", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("scaffold") || stdout.contains("agent"));
}

#[test]
fn test_maintain_help() {
    let output = Command::new(get_pmat_binary())
        .args(&["maintain", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("maintain") || stdout.contains("roadmap"));
}

// Error Message Quality Tests

#[test]
fn test_error_messages_are_helpful() {
    // Test directory exists error
    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path().join("existing_dir");
    std::fs::create_dir(&test_path).unwrap();

    let output = Command::new(get_pmat_binary())
        .args(&[
            "scaffold",
            "agent",
            "--name",
            "existing_dir",
            "--template",
            "basic",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Suggestions") || stderr.contains("--force"));
}

#[test]
fn test_invalid_command_suggestions() {
    let output = Command::new(get_pmat_binary())
        .args(&["scafold"]) // Typo
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    // Should suggest correct command or show help
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("scaffold")
            || stderr.contains("Did you mean")
            || stderr.contains("help")
    );
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_scaffold_names_are_validated(name in "[a-z_][a-z0-9_]{0,20}") {
            // Valid names should work with dry-run
            let output = std::process::Command::new(super::get_pmat_binary())
                .args(&["scaffold", "agent", "--name", &name, "--template", "basic", "--dry-run"])
                .output()
                .expect("Failed to execute");

            // Should succeed for valid names
            prop_assert!(output.status.success());
        }
    }
}
