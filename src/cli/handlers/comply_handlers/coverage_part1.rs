// Coverage tests for comply handlers
// Extracted for file health compliance (CB-040)

use super::super::*;
use std::fs;
use tempfile::TempDir;

    // Test Fixture Helpers

    /// Create a temporary directory with basic PMAT structure
    fn create_temp_project() -> TempDir {
        TempDir::new().expect("Failed to create temp dir")
    }

    /// Create a project with .pmat directory and project.toml
    fn create_pmat_project(version: &str) -> TempDir {
        let temp = create_temp_project();
        let pmat_dir = temp.path().join(".pmat");
        fs::create_dir_all(&pmat_dir).expect("Failed to create .pmat dir");

        let config = format!(
            r#"[pmat]
version = "{}"
auto_update = false
"#,
            version
        );
        fs::write(pmat_dir.join("project.toml"), config).expect("Failed to write project.toml");
        temp
    }

    /// Create a project with .pmat-metrics.toml
    fn create_project_with_metrics(version: &str) -> TempDir {
        let temp = create_pmat_project(version);
        let metrics_content = r#"
[thresholds]
lint = 30000
test-fast = 300000
"#;
        fs::write(temp.path().join(".pmat-metrics.toml"), metrics_content)
            .expect("Failed to write metrics");
        temp
    }

    /// Create a git repository structure
    fn create_git_repo() -> TempDir {
        let temp = create_temp_project();
        let hooks_dir = temp.path().join(".git").join("hooks");
        fs::create_dir_all(&hooks_dir).expect("Failed to create .git/hooks");
        temp
    }

    /// Create a Rust project with Cargo.toml
    fn create_rust_project(with_msrv: bool, with_lock: bool) -> TempDir {
        let temp = create_temp_project();
        let cargo_content = if with_msrv {
            r#"[package]
name = "test"
version = "0.1.0"
rust-version = "1.75"
"#
        } else {
            r#"[package]
name = "test"
version = "0.1.0"
"#
        };
        fs::write(temp.path().join("Cargo.toml"), cargo_content)
            .expect("Failed to write Cargo.toml");
        if with_lock {
            fs::write(temp.path().join("Cargo.lock"), "# lock file")
                .expect("Failed to write Cargo.lock");
        }
        temp
    }

    // Type and struct tests (ProjectConfig, ComplianceReport, etc.)
    include!("coverage_part1_types.rs");

    // Function check tests (versions_behind, currency, config, hooks, etc.)
    include!("coverage_part1_checks.rs");
