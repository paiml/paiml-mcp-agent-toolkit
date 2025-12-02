// pmat repo-score module
// Repository health scoring system
//
// Specification: docs/specifications/repo-score-spec.md
// Design: docs/design/repo-score-implementation.md

pub mod aggregator;
pub mod bonus;
pub mod error;
pub mod models;
pub mod scorers;

pub use error::RepoScoreError;
pub use models::*;

#[cfg(test)]
mod integration_tests {
    use super::*;
    use aggregator::ScoreAggregator;
    use scorers::ScorerConfig;
    use std::fs;
    use tempfile::TempDir;

    fn create_realistic_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Create comprehensive README.md
        let readme_content = r#"# Test Repository

## Overview
This is a comprehensive test repository for the repo-score system.

## Installation
```bash
cargo build --release
```

## Usage
Run the tool with:
```bash
cargo run
```

## Contributing
Please submit pull requests with tests.

## License
MIT License
"#;
        fs::write(repo_path.join("README.md"), readme_content).unwrap();

        // Create .git/hooks/pre-commit (executable)
        let hooks_dir = repo_path.join(".git/hooks");
        fs::create_dir_all(&hooks_dir).unwrap();
        let precommit_path = hooks_dir.join("pre-commit");
        fs::write(&precommit_path, "#!/bin/bash\nmake test-fast\n").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&precommit_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&precommit_path, perms).unwrap();
        }

        // Create Makefile with all required targets
        let makefile_content = r#"
.PHONY: test-fast test lint coverage build

test-fast:
	cargo test --lib

test:
	cargo test

lint:
	cargo clippy -- -D warnings

coverage:
	cargo llvm-cov

build:
	cargo build --release
"#;
        fs::write(repo_path.join("Makefile"), makefile_content).unwrap();

        // Create GitHub Actions workflow
        let workflows_dir = repo_path.join(".github/workflows");
        fs::create_dir_all(&workflows_dir).unwrap();
        let ci_workflow = r#"
name: CI

on:
  push:
    branches: [ main, master ]
  pull_request:
    branches: [ main, master ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run tests
        run: cargo test
      - name: Run clippy
        run: cargo clippy -- -D warnings
"#;
        fs::write(workflows_dir.join("ci.yml"), ci_workflow).unwrap();

        // Create .pmat-gates.toml
        let pmat_config = r#"
[quality_gates]
max_complexity = 10
min_coverage = 80.0
max_duplicates = 5.0
"#;
        fs::write(repo_path.join(".pmat-gates.toml"), pmat_config).unwrap();

        // Create Cargo.toml with bonus features (proptest)
        let cargo_toml = r#"
[package]
name = "test-repo"
version = "0.1.0"

[dependencies]
proptest = "1.0"
"#;
        fs::write(repo_path.join("Cargo.toml"), cargo_toml).unwrap();

        // Create fuzz directory for fuzzing bonus
        fs::create_dir_all(repo_path.join("fuzz/fuzz_targets")).unwrap();

        temp_dir
    }

    #[tokio::test]
    async fn test_end_to_end_realistic_repository_scoring() {
        let temp_dir = create_realistic_repo();
        let repo_path = temp_dir.path();

        let aggregator = ScoreAggregator::new();
        let config = ScorerConfig::default();

        let result = aggregator.aggregate(repo_path, &config).await.unwrap();

        // Verify all categories scored
        assert!(result.categories.documentation.score > 0.0);
        assert!(result.categories.precommit_hooks.score > 0.0);
        assert!(result.categories.repository_hygiene.score > 0.0);
        assert!(result.categories.build_test_automation.score > 0.0);
        assert!(result.categories.continuous_integration.score > 0.0);
        assert!(result.categories.pmat_compliance.score > 0.0);

        // Verify high total score (should be excellent with all components)
        assert!(
            result.total_score >= 80.0,
            "Total score should be high: {}",
            result.total_score
        );

        // Verify grade is good (should be B+ or better without bonus points)
        assert!(matches!(
            result.grade,
            Grade::APlus | Grade::A | Grade::AMinus | Grade::BPlus
        ));

        // Verify metadata populated
        assert_eq!(result.metadata.repository_path, repo_path);
        assert_eq!(result.metadata.spec_version, "1.0.0");

        // Verify recommendations generated (even perfect repos have some suggestions)
        // Note: recommendations are only generated for failed/warning categories
        // A perfect repo might have no recommendations

        // Verify all subcategories present
        assert_eq!(result.categories.documentation.subcategories.len(), 2); // A1, A2
        assert_eq!(result.categories.precommit_hooks.subcategories.len(), 2); // B1, B2
        assert_eq!(result.categories.repository_hygiene.subcategories.len(), 3); // C1, C2, C3
        assert_eq!(
            result.categories.build_test_automation.subcategories.len(),
            3
        ); // D1, D2, D3
        assert_eq!(
            result.categories.continuous_integration.subcategories.len(),
            3
        ); // E1, E2, E3
        assert_eq!(result.categories.pmat_compliance.subcategories.len(), 2); // F1, F2
    }
}
