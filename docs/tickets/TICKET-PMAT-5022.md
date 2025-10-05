# TICKET-PMAT-5022: GitHub Actions Workflow Generator

**Status**: GREEN
**Priority**: P0
**Complexity**: 6
**Estimated Time**: 2 hours
**Dependencies**: TICKET-PMAT-5020 (gate executor), TICKET-PMAT-5021 (hook integration)
**Sprint**: Sprint 18 - Quality Gate Automation

## Objective

Generate GitHub Actions workflows that run quality gates on push/PR. This provides CI/CD integration for automated quality enforcement, complementing local pre-commit hooks with cloud-based verification.

## Success Criteria

- [ ] Generate .github/workflows/quality.yml file
- [ ] Run full quality gates (clippy, tests, coverage, complexity)
- [ ] Support matrix builds (multiple Rust versions)
- [ ] Upload coverage reports
- [ ] Cache cargo dependencies for speed
- [ ] All quality gates pass (complexity <10, coverage >80%, no SATD)

## Test Strategy

### Unit Tests
- [ ] `test_generate_workflow` - Basic workflow generation
- [ ] `test_workflow_yaml_valid` - Valid YAML structure
- [ ] `test_workflow_includes_coverage` - Coverage upload
- [ ] `test_workflow_matrix_builds` - Multi-version support
- [ ] `test_install_workflow` - File creation

### Property Tests
- [ ] Property: Generated YAML is always valid
- [ ] Property: All jobs have required steps

### Integration Tests
- [ ] `integration_workflow_installation` - Install to project
- [ ] `integration_yaml_parsing` - Parse with serde_yaml

## Quality Gates

- [ ] Cyclomatic complexity <10 for all functions
- [ ] Cognitive complexity <15 for all functions
- [ ] Line coverage >80%
- [ ] Branch coverage >80%
- [ ] 0 SATD violations
- [ ] 0 clippy warnings
- [ ] All tests pass

## Implementation Plan

### Phase 1: Workflow Configuration

```rust
// server/src/scaffold/ci.rs

use serde::{Deserialize, Serialize};

/// GitHub Actions workflow configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    /// Workflow name
    pub name: String,
    /// Rust versions to test
    pub rust_versions: Vec<String>,
    /// Run coverage
    pub enable_coverage: bool,
    /// Run complexity checks
    pub enable_complexity: bool,
    /// Upload coverage to codecov
    pub upload_coverage: bool,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            name: "Quality Gates".to_string(),
            rust_versions: vec!["stable".to_string()],
            enable_coverage: true,
            enable_complexity: true,
            upload_coverage: false, // Requires codecov token
        }
    }
}
```

### Phase 2: Workflow Generation

```rust
/// Generate GitHub Actions workflow for quality gates
///
/// # Complexity
/// - Time: O(1)
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
```

### Phase 3: Installation

```rust
/// Install GitHub Actions workflow to project
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 2
pub fn install_github_workflow(project_dir: &Path, config: &WorkflowConfig) -> Result<()> {
    use std::fs;

    // Create .github/workflows directory
    let workflows_dir = project_dir.join(".github/workflows");
    fs::create_dir_all(&workflows_dir)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Generate and write workflow file
    let workflow = generate_github_workflow(config);
    let workflow_path = workflows_dir.join("quality.yml");
    fs::write(&workflow_path, workflow)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    Ok(())
}
```

### Phase 4: Integration with Scaffold

```rust
// server/src/scaffold/mod.rs (update)

impl ScaffoldEngine {
    pub fn scaffold(&self, config: ScaffoldConfig) -> Result<PathBuf> {
        // ... existing code ...

        // Install GitHub Actions workflow
        use crate::scaffold::ci::{install_github_workflow, WorkflowConfig};
        let workflow_config = WorkflowConfig::default();
        install_github_workflow(&project_dir, &workflow_config)?;

        Ok(project_dir)
    }
}
```

### Phase 5: Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[test]
    fn test_workflow_no_coverage() {
        let config = WorkflowConfig {
            enable_coverage: false,
            ..Default::default()
        };
        let workflow = generate_github_workflow(&config);

        assert!(!workflow.contains("cargo-llvm-cov"));
    }

    #[test]
    fn test_workflow_includes_complexity() {
        let config = WorkflowConfig {
            enable_complexity: true,
            ..Default::default()
        };
        let workflow = generate_github_workflow(&config);

        assert!(workflow.contains("pmat analyze"));
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
        use serde_yaml;

        let config = WorkflowConfig::default();
        let workflow = generate_github_workflow(&config);

        // Verify YAML is parseable
        let parsed: Result<serde_yaml::Value, _> = serde_yaml::from_str(&workflow);
        assert!(parsed.is_ok(), "Generated YAML should be valid");
    }

    #[test]
    fn test_workflow_config_serialization() {
        let config = WorkflowConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: WorkflowConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.name, deserialized.name);
        assert_eq!(config.rust_versions, deserialized.rust_versions);
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
            upload_coverage: false,
        };

        install_github_workflow(temp_dir.path(), &config).unwrap();

        let workflow_path = temp_dir.path().join(".github/workflows/quality.yml");
        assert!(workflow_path.exists());

        let content = std::fs::read_to_string(&workflow_path).unwrap();
        assert!(content.contains("Test Workflow"));
        assert!(content.contains("1.70"));
    }
}
```

## Complexity Analysis

Functions with complexity:
- `generate_github_workflow`: CC=3 (conditional coverage/complexity steps)
- `install_github_workflow`: CC=2 (create dir + write file)

All functions under CC=10 threshold ✓

## Verification Commands

```bash
# Run tests
cargo test --lib scaffold::ci

# Generate project with workflow
cargo run --bin pmat -- scaffold agent my-test-agent
cat my-test-agent/.github/workflows/quality.yml

# Validate YAML
yamllint my-test-agent/.github/workflows/quality.yml
```

## Files to Create/Modify

### New Files
- `server/src/scaffold/ci.rs` - CI/CD workflow generation

### Modified Files
- `server/src/scaffold/mod.rs` - Add ci module, integrate workflow generation
- `server/src/scaffold/mod.rs` - Update scaffold() to install workflow

## Risk Assessment

**Low Risk:**
- Static file generation
- Standard GitHub Actions syntax
- Well-tested YAML structure

**Mitigation:**
- YAML validation in tests
- Integration test on real project
- Caching to minimize CI time

## Notes

This ticket completes the CI/CD automation story:

**Local Development:**
- Pre-commit hooks (PMAT-5021) run fast checks (<30s)
- Developer productivity maintained

**CI/CD:**
- GitHub Actions runs full quality suite
- Coverage reports uploaded
- Complexity analysis performed
- Multi-version matrix builds

**Quality Enforcement:**
1. **Local**: Fast hooks block bad commits
2. **CI**: Full gates verify on push/PR
3. **Automation**: Roadmap auto-updates on merge (PMAT-5013)

**Integration Points:**
- Uses gate executor (PMAT-5020)
- Complements hooks (PMAT-5021)
- Leverages .pmat-gates.toml config

**TDD Cycle Duration**: Estimated 2 hours for RED → GREEN → REFACTOR
