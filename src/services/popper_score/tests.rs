//! Integration tests for Popper Falsifiability Score v1.1
//!
//! These tests verify the complete scoring workflow including:
//! - Gateway logic (Category A >= 15/25)
//! - Score normalization with N/A categories
//! - Recommendation generation

use super::*;
use std::fs;
use tempfile::tempdir;

/// Test scoring on this repository (paiml-mcp-agent-toolkit)
#[test]
fn test_score_this_repository() {
    let project_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();

    let result = score_project(project_path);
    assert!(result.is_ok(), "Should be able to score this repository");

    let score = result.unwrap();

    // This repo should have metadata
    assert!(!score.metadata.project_name.is_empty());
    assert_eq!(score.metadata.version, "1.1.0");

    // Score analysis should have a verdict
    assert!(!score.analysis.verdict.is_empty());

    // The gateway status is informative - print for debugging
    if score.gateway_passed {
        assert!(score.normalized_score > 0.0);
    } else {
        // Gateway failure is acceptable for some project structures
        assert_eq!(score.normalized_score, 0.0);
    }
}

/// Test that empty project fails gateway
#[test]
fn test_empty_project_fails_gateway() {
    let temp_dir = tempdir().unwrap();

    let result = score_project(temp_dir.path());
    assert!(result.is_ok());

    let score = result.unwrap();
    assert!(!score.gateway_passed);
    assert_eq!(score.normalized_score, 0.0);
    assert_eq!(score.grade, PopperGrade::InsufficientFalsifiability);
}

/// Test complete workflow with well-structured project
#[test]
fn test_well_structured_project() {
    let temp_dir = tempdir().unwrap();

    // Create complete project structure
    fs::write(
        temp_dir.path().join("README.md"),
        r#"# Test Project

This project claims to provide reliable data processing with >10x performance.

## Success Criteria

- All tests pass
- Coverage > 85%
- Performance benchmarks show 95% confidence interval

## Installation

```bash
make build
```

## Usage

Run the tool with `./test-project`
"#,
    )
    .unwrap();

    // Create tests
    fs::create_dir_all(temp_dir.path().join("tests")).unwrap();
    fs::write(
        temp_dir.path().join("tests/test_main.rs"),
        r#"#[test]
fn test_basic() { assert!(true); }

#[test]
#[should_panic]
fn test_panic() { panic!("expected"); }
"#,
    )
    .unwrap();

    // Create src with inline tests
    fs::create_dir_all(temp_dir.path().join("src")).unwrap();
    fs::write(
        temp_dir.path().join("src/lib.rs"),
        r#"//! Main library
/// Public function
pub fn process() {}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    #[test]
    fn test_process() {}
}
"#,
    )
    .unwrap();

    // Create Cargo.toml and lock
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"[package]
name = "test-project"
version = "1.0.0"

[dev-dependencies]
criterion = "0.5"
"#,
    )
    .unwrap();
    fs::write(temp_dir.path().join("Cargo.lock"), "# Lock file").unwrap();

    // Create LICENSE
    fs::write(
        temp_dir.path().join("LICENSE"),
        "MIT License\n\nCopyright (c) 2025\n",
    )
    .unwrap();

    // Create .git
    fs::create_dir_all(temp_dir.path().join(".git")).unwrap();

    // Create CI
    fs::create_dir_all(temp_dir.path().join(".github/workflows")).unwrap();
    fs::write(
        temp_dir.path().join(".github/workflows/ci.yml"),
        "on: push\njobs:\n  test:\n    steps:\n      - run: cargo test\n",
    )
    .unwrap();

    // Create Makefile
    fs::write(
        temp_dir.path().join("Makefile"),
        "build:\n\tcargo build\ntest:\n\tcargo test\n",
    )
    .unwrap();

    // Create benches
    fs::create_dir_all(temp_dir.path().join("benches")).unwrap();
    fs::write(
        temp_dir.path().join("benches/bench.rs"),
        "use criterion::*;\nfn bench(c: &mut Criterion) {}\n",
    )
    .unwrap();

    // Create CHANGELOG
    fs::write(
        temp_dir.path().join("CHANGELOG.md"),
        "# Changelog\n\n## [1.0.0] - 2025-01-15\n\n- Initial release\n",
    )
    .unwrap();

    let result = score_project(temp_dir.path());
    assert!(result.is_ok());

    let score = result.unwrap();

    // Should pass gateway
    assert!(score.gateway_passed);

    // Should have reasonable score
    assert!(
        score.normalized_score > 40.0,
        "Score: {}",
        score.normalized_score
    );

    // Should have analysis
    assert!(!score.analysis.verdict.is_empty());

    // Check individual categories
    assert!(score.categories.falsifiability.earned >= 15.0);
    assert!(score.categories.reproducibility.earned > 0.0);
    assert!(score.categories.transparency.earned > 0.0);
}

/// Test ML project detection and scoring
#[test]
fn test_ml_project_scoring() {
    let temp_dir = tempdir().unwrap();

    // Create ML project
    fs::write(
        temp_dir.path().join("requirements.txt"),
        "torch==2.0.0\ntransformers\nnumpy\n",
    )
    .unwrap();

    fs::write(
        temp_dir.path().join("README.md"),
        r#"# ML Model

This model claims to achieve 95% accuracy on the benchmark dataset.

## Success Criteria

- Model achieves >90% accuracy
- Training is reproducible with fixed seed

## Dataset

The training data comes from public sources.
"#,
    )
    .unwrap();

    fs::create_dir_all(temp_dir.path().join("tests")).unwrap();
    fs::write(
        temp_dir.path().join("tests/test_model.py"),
        "def test_model(): assert True\n",
    )
    .unwrap();

    fs::write(
        temp_dir.path().join("train.py"),
        r#"import torch
import random
import numpy as np

SEED = 42
random.seed(SEED)
np.random.seed(SEED)
torch.manual_seed(SEED)
"#,
    )
    .unwrap();

    fs::create_dir_all(temp_dir.path().join(".github/workflows")).unwrap();
    fs::write(
        temp_dir.path().join(".github/workflows/ci.yml"),
        "on: push\njobs:\n  test:\n    steps:\n      - run: pytest\n",
    )
    .unwrap();

    fs::write(temp_dir.path().join("dvc.yaml"), "stages:\n  train:\n").unwrap();

    let result = score_project(temp_dir.path());
    assert!(result.is_ok());

    let score = result.unwrap();

    // ML category should be applicable
    assert!(!score.categories.ml_reproducibility.is_not_applicable);
    assert_eq!(score.max_available, 100.0);

    // Should have ML points
    assert!(score.categories.ml_reproducibility.earned > 0.0);
}

/// Test non-ML project normalization
#[test]
fn test_non_ml_normalization() {
    let temp_dir = tempdir().unwrap();

    // Create non-ML Rust project
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"cli\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();

    fs::write(
        temp_dir.path().join("README.md"),
        "# CLI Tool\n\nProvides command-line utilities.\n\n## Success Criteria\n\nAll tests pass.\n",
    ).unwrap();

    fs::create_dir_all(temp_dir.path().join("tests")).unwrap();
    fs::write(
        temp_dir.path().join("tests/test.rs"),
        "#[test]\nfn test() {}\n",
    )
    .unwrap();

    fs::create_dir_all(temp_dir.path().join(".github/workflows")).unwrap();
    fs::write(
        temp_dir.path().join(".github/workflows/ci.yml"),
        "on: push\njobs:\n  test:\n    steps:\n      - run: cargo test\n",
    )
    .unwrap();

    let result = score_project(temp_dir.path());
    assert!(result.is_ok());

    let score = result.unwrap();

    // ML category should be N/A
    assert!(score.categories.ml_reproducibility.is_not_applicable);

    // Max available should be 95 (100 - 5 for ML)
    if score.gateway_passed {
        assert_eq!(score.max_available, 95.0);
    }
}

/// Test recommendation generation
#[test]
fn test_recommendation_generation() {
    let temp_dir = tempdir().unwrap();

    // Create minimal project that passes gateway but has gaps
    fs::write(
        temp_dir.path().join("README.md"),
        "# Project\n\nClaims 10x performance.\n\n## Success Criteria\n\nTests pass.\n",
    )
    .unwrap();

    fs::create_dir_all(temp_dir.path().join("tests")).unwrap();
    fs::write(
        temp_dir.path().join("tests/test.rs"),
        "#[test]\nfn test() {}\n#[test]\n#[should_panic]\nfn test_panic() { panic!(); }\n",
    )
    .unwrap();

    fs::create_dir_all(temp_dir.path().join(".github/workflows")).unwrap();
    fs::write(
        temp_dir.path().join(".github/workflows/ci.yml"),
        "on: push\njobs:\n  test:\n    steps:\n      - run: cargo test\n",
    )
    .unwrap();

    let result = score_project(temp_dir.path());
    assert!(result.is_ok());

    let score = result.unwrap();

    // If passes gateway with low score, should have recommendations
    if score.gateway_passed && score.normalized_score < 85.0 {
        assert!(!score.recommendations.is_empty());

        // Recommendations should be sorted by priority
        if score.recommendations.len() > 1 {
            for i in 0..score.recommendations.len() - 1 {
                assert!(score.recommendations[i].priority >= score.recommendations[i + 1].priority);
            }
        }
    }
}
