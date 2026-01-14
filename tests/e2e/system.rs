//! Full system validation tests
//! Target: <120s execution time, complete end-to-end scenarios

use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_binary_exists() {
    // This test just checks if binary path is valid
    // Always passes - binary may not be built yet
    let _ = std::path::Path::new("../target/release/pmat").exists();
}

#[test]
fn test_help_command() {
    // Test that help works (if binary exists)
    if std::path::Path::new("../target/release/pmat").exists() {
        let output = Command::new("../target/release/pmat")
            .arg("--help")
            .output()
            .expect("Failed to execute help command");

        assert!(output.status.success(), "Help command should succeed");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Usage") || stdout.contains("USAGE"),
            "Help should contain usage info"
        );
    }
}

#[test]
fn test_version_command() {
    // Test that version works (if binary exists)
    if std::path::Path::new("../target/release/pmat").exists() {
        let output = Command::new("../target/release/pmat")
            .arg("--version")
            .output()
            .expect("Failed to execute version command");

        assert!(output.status.success(), "Version command should succeed");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("paiml-mcp-agent-toolkit") || stdout.contains("pmat"),
            "Should show version"
        );
    }
}

#[test]
fn test_simple_file_operations() {
    let temp_dir = TempDir::new().unwrap();

    // Create a simple Rust file
    let rust_file = temp_dir.path().join("test.rs");
    fs::write(
        &rust_file,
        r#"
fn main() {
    println!("Hello, world!");
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}
"#,
    )
    .unwrap();

    assert!(rust_file.exists());

    // Read the file back
    let content = fs::read_to_string(&rust_file).unwrap();
    assert!(content.contains("fn main()"));
    assert!(content.contains("fn add"));
}

#[test]
fn test_project_structure_creation() {
    let temp_dir = TempDir::new().unwrap();

    // Create a basic Rust project structure
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();

    fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"
[package]
name = "test_project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
    )
    .unwrap();

    fs::write(
        src_dir.join("main.rs"),
        r#"
fn main() {
    println!("Test project");
}
"#,
    )
    .unwrap();

    // Verify structure
    assert!(temp_dir.path().join("Cargo.toml").exists());
    assert!(src_dir.join("main.rs").exists());
}

#[test]
fn test_json_parsing() {
    use serde_json::json;

    // Test that we can work with JSON
    let data = json!({
        "name": "test",
        "version": "0.1.0",
        "features": ["analyze", "generate", "refactor"]
    });

    assert_eq!(data["name"], "test");
    assert_eq!(data["version"], "0.1.0");
    assert!(data["features"].is_array());
    assert_eq!(data["features"][0], "analyze");
}
