//! CLI Integration Tests
//!
//! These tests spawn the `pmat` binary as a subprocess. They are ignored by default
//! because they depend on a built binary being present and are non-deterministic
//! under parallel test load (resource contention causes spurious failures).
//!
//! Run explicitly with: `cargo test --lib -- cli_integration_tests --ignored`

use std::process::Command;
use tempfile::TempDir;

/// Get path to pmat binary from the cargo build output directory
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
#[ignore] // E2E binary test — run with --ignored
fn test_scaffold_agent_dry_run() {
    let output = Command::new(get_pmat_binary())
        .args([
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
#[ignore] // E2E binary test — run with --ignored
fn test_scaffold_agent_list_templates() {
    let output = Command::new(get_pmat_binary())
        .args(["scaffold", "list-templates"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Available Agent Templates") || stderr.contains("Templates"));
}

#[test]
#[ignore] // E2E binary test — run with --ignored
fn test_scaffold_agent_invalid_template() {
    let output = Command::new(get_pmat_binary())
        .args([
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
#[ignore] // E2E binary test — run with --ignored
fn test_scaffold_agent_with_features() {
    let output = Command::new(get_pmat_binary())
        .args([
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
#[ignore] // E2E binary test — run with --ignored
fn test_scaffold_agent_quality_levels() {
    for quality in &["standard", "strict", "extreme"] {
        let output = Command::new(get_pmat_binary())
            .args([
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
#[ignore] // E2E binary test — run with --ignored
fn test_scaffold_wasm_dry_run() {
    let output = Command::new(get_pmat_binary())
        .args([
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
#[ignore] // E2E binary test — run with --ignored
fn test_scaffold_wasm_frameworks() {
    for framework in &["wasm-labs", "pure-wasm"] {
        let output = Command::new(get_pmat_binary())
            .args([
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
#[ignore] // E2E binary test — run with --ignored
fn test_scaffold_wasm_invalid_framework() {
    let output = Command::new(get_pmat_binary())
        .args([
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
#[ignore] // E2E binary test — run with --ignored
fn test_maintain_health_no_project() {
    let temp_dir = TempDir::new().unwrap();

    let output = Command::new(get_pmat_binary())
        .args(["maintain", "health", "--quick"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    // Should succeed but skip checks (no Cargo.toml)
    assert!(output.status.success());
}

#[test]
#[ignore] // E2E binary test — run with --ignored
fn test_maintain_health_quick_flag() {
    let output = Command::new(get_pmat_binary())
        .args(["maintain", "health", "--quick"])
        .current_dir(std::env::current_dir().unwrap())
        .output()
        .expect("Failed to execute command");

    // Quick mode should complete (may fail on health, but command runs)
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Health Report") || stderr.contains("Build"));
}

#[test]
#[ignore] // E2E binary test — run with --ignored
fn test_maintain_health_individual_checks() {
    let output = Command::new(get_pmat_binary())
        .args(["maintain", "health", "--check-build"])
        .current_dir(std::env::current_dir().unwrap())
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Health") || stderr.contains("Build"));
}

// Maintain Roadmap Tests

#[test]
#[ignore] // E2E binary test — run with --ignored
fn test_maintain_roadmap_missing_file() {
    let temp_dir = TempDir::new().unwrap();

    let output = Command::new(get_pmat_binary())
        .args(["maintain", "roadmap"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("ROADMAP.md"));
    assert!(stderr.contains("Suggestions") || stderr.contains("not found"));
}

#[test]
#[ignore] // E2E binary test — run with --ignored
fn test_maintain_roadmap_with_file() {
    let output = Command::new(get_pmat_binary())
        .args(["maintain", "roadmap"])
        .current_dir(std::env::current_dir().unwrap())
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

// Hooks Tests

#[test]
#[ignore] // E2E binary test — run with --ignored
fn test_hooks_status() {
    let temp_dir = TempDir::new().unwrap();

    Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to init git");

    let output = Command::new(get_pmat_binary())
        .args(["hooks", "status"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Hook") || stdout.contains("installed") || stdout.contains("not installed")
    );
}

// Version and Help Tests

#[test]
#[ignore] // E2E binary test — run with --ignored
fn test_version_flag() {
    let output = Command::new(get_pmat_binary())
        .args(["--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pmat") || stdout.contains("version"));
}

#[test]
#[ignore] // E2E binary test — run with --ignored
fn test_help_flag() {
    let output = Command::new(get_pmat_binary())
        .args(["--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage") || stdout.contains("USAGE"));
}

#[test]
#[ignore] // E2E binary test — run with --ignored
fn test_scaffold_help() {
    let output = Command::new(get_pmat_binary())
        .args(["scaffold", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("scaffold") || stdout.contains("agent"));
}

#[test]
#[ignore] // E2E binary test — run with --ignored
fn test_maintain_help() {
    let output = Command::new(get_pmat_binary())
        .args(["maintain", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("maintain") || stdout.contains("roadmap"));
}

// Error Message Quality Tests

#[test]
#[ignore] // E2E binary test — run with --ignored
fn test_error_messages_are_helpful() {
    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path().join("existing_dir");
    std::fs::create_dir(&test_path).unwrap();

    let output = Command::new(get_pmat_binary())
        .args([
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
#[ignore] // E2E binary test — run with --ignored
fn test_invalid_command_suggestions() {
    let output = Command::new(get_pmat_binary())
        .args(["scafold"]) // Typo
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("scaffold") || stderr.contains("Did you mean") || stderr.contains("help")
    );
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        #[ignore] // E2E binary test — run with --ignored
        fn test_scaffold_names_are_validated(name in "[a-z_][a-z0-9_]{0,20}") {
            let output = std::process::Command::new(super::get_pmat_binary())
                .args(["scaffold", "agent", "--name", &name, "--template", "basic", "--dry-run"])
                .output()
                .expect("Failed to execute");

            prop_assert!(output.status.success());
        }
    }
}
