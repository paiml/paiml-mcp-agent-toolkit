//! CI/CD workflow generation for TICKET-PMAT-5022
//!
//! Generates GitHub Actions workflows for quality gate automation.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// GitHub Actions workflow configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowConfig {
    /// Workflow name
    pub name: String,
    /// Rust versions to test
    pub rust_versions: Vec<String>,
    /// Run coverage
    pub enable_coverage: bool,
    /// Run complexity checks
    pub enable_complexity: bool,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            name: "Quality Gates".to_string(),
            rust_versions: vec!["stable".to_string()],
            enable_coverage: true,
            enable_complexity: true,
        }
    }
}

/// Generate GitHub Actions workflow for quality gates
///
/// # Complexity
/// - Time: O(n) where n is number of rust versions
/// - Cyclomatic: 3
pub fn generate_github_workflow(config: &WorkflowConfig) -> String {
    let rust_versions = config
        .rust_versions
        .iter()
        .map(|v| format!("          - {}", v))
        .collect::<Vec<_>>()
        .join("\n");

    let coverage_step = if config.enable_coverage {
        r#"
      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov

      - name: Run coverage
        run: cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

      - name: Upload coverage
        if: matrix.rust == 'stable'
        uses: codecov/codecov-action@v3
        with:
          files: lcov.info
          fail_ci_if_error: false
"#
    } else {
        ""
    };

    let complexity_step = if config.enable_complexity {
        r#"
      - name: Check complexity
        if: matrix.rust == 'stable'
        run: |
          if command -v pmat &> /dev/null; then
            pmat analyze --format json > complexity.json
            echo "Complexity analysis complete"
          fi
"#
    } else {
        ""
    };

    format!(
        r#"name: {}

on:
  push:
    branches: [ main, master ]
  pull_request:
    branches: [ main, master ]

env:
  CARGO_TERM_COLOR: always

jobs:
  quality-gates:
    name: Quality Gates
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust:
{}

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{{{ matrix.rust }}}}
          components: clippy

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{{{ runner.os }}}}-cargo-registry-${{{{ hashFiles('**/Cargo.lock') }}}}

      - name: Cache cargo index
        uses: actions/cache@v3
        with:
          path: ~/.cargo/git
          key: ${{{{ runner.os }}}}-cargo-index-${{{{ hashFiles('**/Cargo.lock') }}}}

      - name: Cache target directory
        uses: actions/cache@v3
        with:
          path: target
          key: ${{{{ runner.os }}}}-target-${{{{ matrix.rust }}}}-${{{{ hashFiles('**/Cargo.lock') }}}}

      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Run tests
        run: cargo test --all-features
{}{}
      - name: Build
        run: cargo build --release
"#,
        config.name, rust_versions, coverage_step, complexity_step
    )
}

/// Install GitHub Actions workflow to project
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 2
pub fn install_github_workflow(project_dir: &Path, config: &WorkflowConfig) -> std::io::Result<()> {
    use std::fs;

    // Create .github/workflows directory
    let workflows_dir = project_dir.join(".github/workflows");
    fs::create_dir_all(&workflows_dir)?;

    // Generate and write workflow file
    let workflow = generate_github_workflow(config);
    let workflow_path = workflows_dir.join("quality.yml");
    fs::write(&workflow_path, workflow)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_config_default() {
        let config = WorkflowConfig::default();

        assert_eq!(config.name, "Quality Gates");
        assert_eq!(config.rust_versions, vec!["stable"]);
        assert!(config.enable_coverage);
        assert!(config.enable_complexity);
    }

    #[test]
    fn test_generate_workflow_basic() {
        let config = WorkflowConfig::default();
        let workflow = generate_github_workflow(&config);

        assert!(workflow.contains("name: Quality Gates"));
        assert!(workflow.contains("on:"));
        assert!(workflow.contains("push:"));
        assert!(workflow.contains("pull_request:"));
    }

    #[test]
    fn test_workflow_includes_clippy() {
        let config = WorkflowConfig::default();
        let workflow = generate_github_workflow(&config);

        assert!(workflow.contains("cargo clippy"));
        assert!(workflow.contains("-D warnings"));
    }

    #[test]
    fn test_workflow_includes_tests() {
        let config = WorkflowConfig::default();
        let workflow = generate_github_workflow(&config);

        assert!(workflow.contains("cargo test"));
        assert!(workflow.contains("--all-features"));
    }

    #[test]
    fn test_workflow_includes_coverage() {
        let config = WorkflowConfig {
            enable_coverage: true,
            ..Default::default()
        };
        let workflow = generate_github_workflow(&config);

        assert!(workflow.contains("cargo-llvm-cov"));
        assert!(workflow.contains("codecov"));
        assert!(workflow.contains("lcov.info"));
    }

    #[test]
    fn test_workflow_no_coverage() {
        let config = WorkflowConfig {
            enable_coverage: false,
            ..Default::default()
        };
        let workflow = generate_github_workflow(&config);

        assert!(!workflow.contains("cargo-llvm-cov"));
        assert!(!workflow.contains("codecov"));
    }

    #[test]
    fn test_workflow_includes_complexity() {
        let config = WorkflowConfig {
            enable_complexity: true,
            ..Default::default()
        };
        let workflow = generate_github_workflow(&config);

        assert!(workflow.contains("pmat analyze"));
        assert!(workflow.contains("complexity.json"));
    }

    #[test]
    fn test_workflow_no_complexity() {
        let config = WorkflowConfig {
            enable_complexity: false,
            ..Default::default()
        };
        let workflow = generate_github_workflow(&config);

        assert!(!workflow.contains("pmat analyze"));
    }

    #[test]
    fn test_workflow_matrix_builds() {
        let config = WorkflowConfig {
            rust_versions: vec!["stable".to_string(), "beta".to_string()],
            ..Default::default()
        };
        let workflow = generate_github_workflow(&config);

        assert!(workflow.contains("- stable"));
        assert!(workflow.contains("- beta"));
        assert!(workflow.contains("matrix:"));
    }

    #[test]
    fn test_workflow_includes_caching() {
        let config = WorkflowConfig::default();
        let workflow = generate_github_workflow(&config);

        assert!(workflow.contains("actions/cache"));
        assert!(workflow.contains("~/.cargo/registry"));
        assert!(workflow.contains("~/.cargo/git"));
        assert!(workflow.contains("target"));
    }

    #[test]
    fn test_install_workflow() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let config = WorkflowConfig::default();

        install_github_workflow(temp_dir.path(), &config).unwrap();

        let workflow_path = temp_dir.path().join(".github/workflows/quality.yml");
        assert!(workflow_path.exists());

        let content = std::fs::read_to_string(&workflow_path).unwrap();
        assert!(content.contains("Quality Gates"));
    }

    #[test]
    fn test_workflow_yaml_valid() {
        let config = WorkflowConfig::default();
        let workflow = generate_github_workflow(&config);

        // Verify YAML is parseable
        let parsed: Result<serde_yaml::Value, _> = serde_yaml::from_str(&workflow);
        assert!(parsed.is_ok(), "Generated YAML should be valid: {:?}", parsed.err());
    }

    #[test]
    fn test_workflow_config_serialization() {
        let config = WorkflowConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: WorkflowConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.name, deserialized.name);
        assert_eq!(config.rust_versions, deserialized.rust_versions);
        assert_eq!(config.enable_coverage, deserialized.enable_coverage);
    }

    #[test]
    fn test_workflow_custom_name() {
        let config = WorkflowConfig {
            name: "Custom CI".to_string(),
            ..Default::default()
        };
        let workflow = generate_github_workflow(&config);

        assert!(workflow.contains("name: Custom CI"));
    }

    #[test]
    fn test_workflow_multiple_versions() {
        let config = WorkflowConfig {
            rust_versions: vec![
                "stable".to_string(),
                "beta".to_string(),
                "1.70".to_string(),
            ],
            ..Default::default()
        };
        let workflow = generate_github_workflow(&config);

        assert!(workflow.contains("- stable"));
        assert!(workflow.contains("- beta"));
        assert!(workflow.contains("- 1.70"));
    }

    #[test]
    #[ignore] // Integration test
    fn integration_workflow_installation() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let config = WorkflowConfig {
            name: "Test Workflow".to_string(),
            rust_versions: vec!["stable".to_string(), "1.70".to_string()],
            enable_coverage: true,
            enable_complexity: true,
        };

        install_github_workflow(temp_dir.path(), &config).unwrap();

        let workflow_path = temp_dir.path().join(".github/workflows/quality.yml");
        assert!(workflow_path.exists());

        let content = std::fs::read_to_string(&workflow_path).unwrap();
        assert!(content.contains("Test Workflow"));
        assert!(content.contains("1.70"));
    }
}
