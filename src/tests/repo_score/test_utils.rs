// Test utilities for repo_score tests
// Shared fixtures and helper functions

use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Creates a temporary directory for testing
pub fn create_temp_repo() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

/// Creates a basic .git directory to simulate a git repository
pub fn init_git_repo(path: &Path) {
    let git_dir = path.join(".git");
    std::fs::create_dir_all(&git_dir).expect("Failed to create .git directory");

    // Create minimal git structure
    std::fs::create_dir_all(git_dir.join("objects")).unwrap();
    std::fs::create_dir_all(git_dir.join("refs/heads")).unwrap();
    std::fs::write(git_dir.join("HEAD"), "ref: refs/heads/master\n").unwrap();
}

/// Creates a README.md with given content
pub fn create_readme(path: &Path, content: &str) {
    std::fs::write(path.join("README.md"), content).expect("Failed to create README.md");
}

/// Creates a Makefile with given content
pub fn create_makefile(path: &Path, content: &str) {
    std::fs::write(path.join("Makefile"), content).expect("Failed to create Makefile");
}

/// Creates a .pmat-gates.toml config file
pub fn create_pmat_gates(path: &Path, content: &str) {
    std::fs::write(path.join(".pmat-gates.toml"), content)
        .expect("Failed to create .pmat-gates.toml");
}

/// Creates a pre-commit hook
pub fn create_precommit_hook(path: &Path, content: &str) {
    let hooks_dir = path.join(".git/hooks");
    std::fs::create_dir_all(&hooks_dir).expect("Failed to create hooks directory");

    let hook_path = hooks_dir.join("pre-commit");
    std::fs::write(&hook_path, content).expect("Failed to create pre-commit hook");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&hook_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&hook_path, perms).unwrap();
    }
}

/// Creates a GitHub Actions workflow
pub fn create_github_workflow(path: &Path, name: &str, content: &str) {
    let workflows_dir = path.join(".github/workflows");
    std::fs::create_dir_all(&workflows_dir).expect("Failed to create workflows directory");

    std::fs::write(workflows_dir.join(name), content)
        .expect("Failed to create workflow file");
}

/// Creates a cruft file (for hygiene testing)
pub fn create_cruft_file(path: &Path, filename: &str) {
    std::fs::write(path.join(filename), "cruft content")
        .expect("Failed to create cruft file");
}

/// Perfect README template (scores 20/20)
pub const PERFECT_README: &str = r#"
# Project Name

## Overview
This is a comprehensive test project demonstrating repository best practices.

## Installation

```bash
cargo install project-name
```

## Usage

```bash
project-name --help
project-name run
```

## Getting Started

1. Clone the repository
2. Run `make install`
3. Run `make test`

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Documentation

Full documentation available at https://docs.example.com
"#;

/// Minimal README (missing sections)
pub const MINIMAL_README: &str = r#"
# Project Name

This is a project.
"#;

/// Perfect Makefile template
pub const PERFECT_MAKEFILE: &str = r#"
.PHONY: help test test-fast lint coverage

help:
	@echo "Available targets:"
	@echo "  test-fast  - Run fast tests (<5 min)"
	@echo "  test       - Run all tests"
	@echo "  lint       - Run linters"
	@echo "  coverage   - Generate coverage report"

test-fast:
	cargo test --lib --bins

test:
	cargo test --workspace

lint:
	cargo clippy --all-targets -- -D warnings

coverage:
	cargo llvm-cov --all-features --workspace --html
"#;

/// Perfect .pmat-gates.toml
pub const PERFECT_PMAT_GATES: &str = r#"
[gates]
run_clippy = true
clippy_strict = true
run_tests = true
test_timeout = 300
check_coverage = true
min_coverage = 85.0
check_complexity = true
max_complexity = 10
"#;

/// Perfect pre-commit hook
pub const PERFECT_PRECOMMIT_HOOK: &str = r#"#!/usr/bin/env bash
set -e

echo "Running pre-commit checks..."

# Run lint
make lint

# Run fast tests
make test-fast

echo "✅ All checks passed!"
"#;

/// Perfect GitHub Actions CI workflow
pub const PERFECT_CI_WORKFLOW: &str = r#"
name: CI

on:
  push:
    branches: [main, master]
  pull_request:

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: make test
      - run: make lint
"#;
