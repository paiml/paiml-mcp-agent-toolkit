//! Integration tests for context command with large file handling
//!
//! This module tests the context command with the new --include-large-files flag
//! to ensure proper integration and user experience.

#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use predicates::prelude::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_context_skips_large_files_by_default() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create a normal file
        fs::write(project_path.join("main.rs"), "fn main() {}\n").unwrap();

        // Create a large file (600KB)
        let large_content = "a".repeat(600_000);
        fs::write(project_path.join("large.js"), large_content).unwrap();

        // Run context command
        let mut cmd = Command::cargo_bin("pmat").unwrap();
        cmd.arg("context")
            .arg("-p")
            .arg(project_path)
            .assert()
            .success()
            .stderr(predicate::str::contains("Skipped:"))
            .stderr(predicate::str::contains("large.js"))
            .stderr(predicate::str::contains("large file >500KB"));
    }

    #[test]
    fn test_context_includes_large_files_with_flag() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create a normal file
        fs::write(project_path.join("main.rs"), "fn main() {}\n").unwrap();

        // Create a large file (600KB) with valid code
        let mut large_content = String::new();
        for i in 0..10_000 {
            large_content.push_str(&format!("function test{} () {{ return {}; }}\n", i, i));
        }
        fs::write(project_path.join("large.js"), large_content).unwrap();

        // Run context command with --include-large-files
        let mut cmd = Command::cargo_bin("pmat").unwrap();
        cmd.arg("context")
            .arg("-p")
            .arg(project_path)
            .arg("--include-large-files")
            .assert()
            .success()
            .stderr(predicate::str::contains("Skipped:").not());
    }

    #[test]
    fn test_context_progress_bars() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create some files
        fs::write(project_path.join("main.rs"), "fn main() {}\n").unwrap();
        fs::write(project_path.join("lib.rs"), "pub fn lib() {}\n").unwrap();

        // Run context command - should succeed
        let mut cmd = Command::cargo_bin("pmat").unwrap();
        cmd.arg("context")
            .arg("-p")
            .arg(project_path)
            .assert()
            .success();
    }

    #[test]
    fn test_context_help_shows_include_large_files() {
        let mut cmd = Command::cargo_bin("pmat").unwrap();
        cmd.arg("context")
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("--include-large-files"))
            .stdout(predicate::str::contains("Include large files (>500KB)"));
    }
}