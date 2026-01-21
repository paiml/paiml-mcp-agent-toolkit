//\! Tests for rust tooling scorer
//\! Extracted for file health compliance (CB-040)

use super::*;

mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_scorer_creation() {
        let scorer = RustToolingScorer::new();
        assert_eq!(scorer.name(), "Rust Tooling & CI/CD");
        assert_eq!(scorer.max_points(), 130.0); // v2.0: 25 + 12 + 37 + 35 + 10 + 11 (profiles)
    }

    #[test]
    fn test_scorer_implements_trait() {
        let scorer = RustToolingScorer::new();
        let _trait_obj: &dyn Scorer = &scorer;
    }

    // v2.0 Workspace Lints Tests (RED phase - following EXTREME TDD)

    #[test]
    fn test_workspace_lints_no_cargo_toml() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let score = scorer.score_workspace_lints(temp_dir.path(), None).unwrap();
        assert_eq!(
            score, 0.0,
            "Should return 0 points when Cargo.toml doesn't exist"
        );
    }

    #[test]
    fn test_workspace_lints_no_workspace_section() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        // Create Cargo.toml without workspace lints
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        let score = scorer.score_workspace_lints(temp_dir.path(), None).unwrap();
        assert_eq!(
            score, 0.0,
            "Should return 0 points when no workspace lints configured"
        );
    }

    #[test]
    fn test_workspace_lints_rust_only() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[workspace.lints.rust]
rust_2018_idioms = { level = "warn", priority = -1 }
unreachable_pub = "warn"
"#,
        )
        .unwrap();

        let score = scorer.score_workspace_lints(temp_dir.path(), None).unwrap();
        assert_eq!(
            score, 9.0,
            "Should get 5pts (workspace lints) + 4pts (high-value: unreachable_pub)"
        );
    }

    #[test]
    fn test_workspace_lints_clippy_only() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[workspace.lints.clippy]
checked_conversions = "warn"
fallible_impl_from = "warn"
"#,
        )
        .unwrap();

        let score = scorer.score_workspace_lints(temp_dir.path(), None).unwrap();
        assert_eq!(score, 9.0, "Should get 5pts (workspace lints) + 4pts (high-value: checked_conversions, fallible_impl_from)");
    }

    #[test]
    fn test_workspace_lints_both_rust_and_clippy() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[workspace.lints.rust]
unsafe_op_in_unsafe_fn = "warn"
unused_lifetimes = "warn"

[workspace.lints.clippy]
checked_conversions = "warn"
"#,
        )
        .unwrap();

        let score = scorer.score_workspace_lints(temp_dir.path(), None).unwrap();
        assert_eq!(
            score, 9.0,
            "Should get 5pts (workspace lints) + 4pts (high-value lints)"
        );
    }

    #[test]
    fn test_workspace_lints_with_clippy_toml() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[workspace.lints.clippy]
checked_conversions = "warn"
"#,
        )
        .unwrap();

        let clippy_toml = temp_dir.path().join(".clippy.toml");
        std::fs::write(
            &clippy_toml,
            r#"
disallowed-methods = [
    { path = "std::option::Option::map_or", reason = "prefer map(..).unwrap_or(..)" },
]
"#,
        )
        .unwrap();

        let score = scorer.score_workspace_lints(temp_dir.path(), None).unwrap();
        assert_eq!(
            score, 12.0,
            "Should get 5pts + 4pts + 3pts (clippy.toml) = 12pts"
        );
    }

    #[test]
    fn test_workspace_lints_full_score() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        // Create Cargo.toml with workspace lints (like clap/tokio)
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[workspace.lints.rust]
rust_2018_idioms = { level = "warn", priority = -1 }
unreachable_pub = "warn"
unsafe_op_in_unsafe_fn = "warn"
unused_lifetimes = "warn"

[workspace.lints.clippy]
checked_conversions = "warn"
fallible_impl_from = "warn"
"#,
        )
        .unwrap();

        // Create .clippy.toml with disallowed-methods
        let clippy_toml = temp_dir.path().join(".clippy.toml");
        std::fs::write(
            &clippy_toml,
            r#"
allow-print-in-tests = true
disallowed-methods = [
    { path = "std::option::Option::map_or", reason = "prefer map(..).unwrap_or(..)" },
    { path = "std::iter::Iterator::for_each", reason = "prefer for loops" },
]
"#,
        )
        .unwrap();

        let score = scorer.score_workspace_lints(temp_dir.path(), None).unwrap();
        assert_eq!(score, 12.0, "Should get full 12 points: 5 + 4 + 3");
    }

    #[test]
    fn test_workspace_lints_no_high_value_lints() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[workspace.lints.clippy]
# Low-value lints only (no correctness/safety lints)
bool_assert_comparison = "allow"
"#,
        )
        .unwrap();

        let score = scorer.score_workspace_lints(temp_dir.path(), None).unwrap();
        assert_eq!(
            score, 5.0,
            "Should get 5pts (workspace section exists) but not 4pts (no high-value lints)"
        );
    }

    // CI/CD Integration Tests (v2.0 Phase 2)
    // Based on "Learn from Rust Giants" specification
    // Academic Foundation: Hilton 2016 ASE, Memon 2017 ICSE-SEIP

    #[test]
    fn test_ci_cd_full_score() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        // Create .github/workflows directory
        let workflows_dir = temp_dir.path().join(".github/workflows");
        std::fs::create_dir_all(&workflows_dir).unwrap();

        // Create ci.yml with multi-platform matrix (like clap/tokio)
        let ci_workflow = workflows_dir.join("ci.yml");
        std::fs::write(
            &ci_workflow,
            r#"
name: CI

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        features: [minimal, default, full]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - run: cargo test --features ${{ matrix.features }}
"#,
        )
        .unwrap();

        // Create audit.yml workflow (security)
        let audit_workflow = workflows_dir.join("audit.yml");
        std::fs::write(
            &audit_workflow,
            r#"
name: Security Audit

on:
  schedule:
    - cron: '0 0 * * *'

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo audit
"#,
        )
        .unwrap();

        // Create bench.yml workflow (benchmarks)
        let bench_workflow = workflows_dir.join("bench.yml");
        std::fs::write(
            &bench_workflow,
            r#"
name: Benchmarks

on: [push]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo bench
"#,
        )
        .unwrap();

        // Create justfile (Rust-native build automation)
        let justfile = temp_dir.path().join("justfile");
        std::fs::write(
            &justfile,
            r#"
# Build commands
build:
    cargo build --release

# Test commands
test:
    cargo test

# Lint commands
lint:
    cargo clippy -- -D warnings

# Benchmark commands
bench:
    cargo bench
"#,
        )
        .unwrap();

        let score = scorer
            .score_ci_cd_integration(temp_dir.path(), None)
            .unwrap();

        // Expected score:
        // Multi-platform: +6 (Linux+Windows+Mac)
        // Feature matrix: +4 (minimal, default, full)
        // CI workflow diversity: +6 (≥3 workflows: ci.yml, audit.yml, bench.yml)
        // Dedicated audit: +4 (audit.yml)
        // Dedicated benchmark: +3 (bench.yml)
        // Build automation (justfile): +5
        // Common targets: +3 (build, test, lint, bench all present)
        // Note: Separate workflows for stress/loom (+3) NOT counted (no stress.yml/loom.yml)
        // Total: 31 points
        assert_eq!(score, 31.0, "Should get 31 points for complete CI/CD setup");
    }

    #[test]
    fn test_ci_cd_multi_platform_only() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let workflows_dir = temp_dir.path().join(".github/workflows");
        std::fs::create_dir_all(&workflows_dir).unwrap();

        // Only multi-platform CI, no other workflows
        let ci_workflow = workflows_dir.join("ci.yml");
        std::fs::write(
            &ci_workflow,
            r#"
jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
"#,
        )
        .unwrap();

        let score = scorer
            .score_ci_cd_integration(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 6.0, "Should get 6pts for Linux+Windows+Mac");
    }

    #[test]
    fn test_ci_cd_feature_matrix() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let workflows_dir = temp_dir.path().join(".github/workflows");
        std::fs::create_dir_all(&workflows_dir).unwrap();

        let ci_workflow = workflows_dir.join("ci.yml");
        std::fs::write(
            &ci_workflow,
            r#"
jobs:
  test:
    strategy:
      matrix:
        features: [minimal, default, full]
"#,
        )
        .unwrap();

        let score = scorer
            .score_ci_cd_integration(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 4.0, "Should get 4pts for feature matrix testing");
    }

    #[test]
    fn test_ci_cd_workflow_counting() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let workflows_dir = temp_dir.path().join(".github/workflows");
        std::fs::create_dir_all(&workflows_dir).unwrap();

        // Create 3 workflows (ci, test, lint)
        std::fs::write(workflows_dir.join("ci.yml"), "name: CI").unwrap();
        std::fs::write(workflows_dir.join("test.yml"), "name: Test").unwrap();
        std::fs::write(workflows_dir.join("lint.yml"), "name: Lint").unwrap();

        let score = scorer
            .score_ci_cd_integration(temp_dir.path(), None)
            .unwrap();
        // +6 for ≥3 workflows + +2 for dedicated lint workflow = 8 total
        assert_eq!(
            score, 8.0,
            "Should get 8pts (6 for ≥3 workflows + 2 for lint workflow)"
        );
    }

    #[test]
    fn test_ci_cd_dedicated_audit_workflow() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let workflows_dir = temp_dir.path().join(".github/workflows");
        std::fs::create_dir_all(&workflows_dir).unwrap();

        let audit_workflow = workflows_dir.join("audit.yml");
        std::fs::write(
            &audit_workflow,
            r#"
name: Security Audit
jobs:
  audit:
    steps:
      - run: cargo audit
"#,
        )
        .unwrap();

        let score = scorer
            .score_ci_cd_integration(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 4.0, "Should get 4pts for dedicated audit workflow");
    }

    #[test]
    fn test_ci_cd_dedicated_benchmark_workflow() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let workflows_dir = temp_dir.path().join(".github/workflows");
        std::fs::create_dir_all(&workflows_dir).unwrap();

        let bench_workflow = workflows_dir.join("bench.yml");
        std::fs::write(
            &bench_workflow,
            r#"
name: Benchmarks
jobs:
  benchmark:
    steps:
      - run: cargo bench
"#,
        )
        .unwrap();

        let score = scorer
            .score_ci_cd_integration(temp_dir.path(), None)
            .unwrap();
        assert_eq!(
            score, 3.0,
            "Should get 3pts for dedicated benchmark workflow"
        );
    }

    #[test]
    fn test_ci_cd_justfile_detection() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        // Create justfile with common targets
        let justfile = temp_dir.path().join("justfile");
        std::fs::write(
            &justfile,
            r#"
build:
    cargo build

test:
    cargo test

lint:
    cargo clippy

bench:
    cargo bench
"#,
        )
        .unwrap();

        let score = scorer
            .score_ci_cd_integration(temp_dir.path(), None)
            .unwrap();
        assert_eq!(
            score, 8.0,
            "Should get 5pts for justfile + 3pts for common targets"
        );
    }

    #[test]
    fn test_ci_cd_makefile_detection() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        // Create Makefile (downgraded to 3pts per TPS review)
        let makefile = temp_dir.path().join("Makefile");
        std::fs::write(
            &makefile,
            r#"
build:
	cargo build

test:
	cargo test
"#,
        )
        .unwrap();

        let score = scorer
            .score_ci_cd_integration(temp_dir.path(), None)
            .unwrap();
        assert_eq!(
            score, 3.0,
            "Should get 3pts for Makefile (downgraded, Windows-problematic)"
        );
    }

    #[test]
    fn test_ci_cd_no_workflows() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let score = scorer
            .score_ci_cd_integration(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 0.0, "Should get 0pts with no CI/CD infrastructure");
    }

    #[test]
    fn test_ci_cd_partial_platform_coverage() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let workflows_dir = temp_dir.path().join(".github/workflows");
        std::fs::create_dir_all(&workflows_dir).unwrap();

        // Only Linux and Windows (no Mac)
        let ci_workflow = workflows_dir.join("ci.yml");
        std::fs::write(
            &ci_workflow,
            r#"
jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
"#,
        )
        .unwrap();

        let score = scorer
            .score_ci_cd_integration(temp_dir.path(), None)
            .unwrap();
        assert_eq!(
            score, 0.0,
            "Should get 0pts - all 3 platforms required (Linux+Windows+Mac)"
        );
    }

    // Advanced Metadata Tests (v2.0 Phase 3)
    // Based on "Learn from Rust Giants" specification
    // Academic Foundation: Aghajani 2019 ICSE, FSE 2022

    // docs.rs Metadata Tests (10pts total)

    #[test]
    fn test_docs_rs_metadata_full_score() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[package]
name = "test-crate"
version = "1.0.0"

[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs", "--generate-link-to-definition"]
"#,
        )
        .unwrap();

        let score = scorer
            .score_docs_rs_metadata(temp_dir.path(), None)
            .unwrap();
        // +5 for [package.metadata.docs.rs]
        // +3 for all-features = true
        // +2 for --generate-link-to-definition
        assert_eq!(
            score, 10.0,
            "Should get full 10 points for complete docs.rs config"
        );
    }

    #[test]
    fn test_docs_rs_metadata_basic() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[package]
name = "test-crate"

[package.metadata.docs.rs]
features = ["std"]
"#,
        )
        .unwrap();

        let score = scorer
            .score_docs_rs_metadata(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 5.0, "Should get 5pts for basic docs.rs metadata");
    }

    #[test]
    fn test_docs_rs_no_metadata() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(&cargo_toml, "[package]\nname = \"test\"").unwrap();

        let score = scorer
            .score_docs_rs_metadata(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 0.0, "Should get 0pts with no docs.rs metadata");
    }

    // Workspace Organization Tests (13pts total)

    #[test]
    fn test_workspace_organization_full_score() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[workspace]
members = ["crate-a", "crate-b"]
resolver = "2"

[workspace.dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }

[workspace.package]
version = "1.0.0"
edition = "2021"
license = "MIT"
authors = ["Test Author"]
"#,
        )
        .unwrap();

        let score = scorer
            .score_workspace_organization(temp_dir.path(), None)
            .unwrap();
        // +6 for [workspace] section
        // +3 for resolver = "2"
        // +2 for [workspace.dependencies]
        // +2 for [workspace.package]
        assert_eq!(
            score, 13.0,
            "Should get full 13 points for complete workspace config"
        );
    }

    #[test]
    fn test_workspace_organization_basic() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[workspace]
members = ["crate-a"]
"#,
        )
        .unwrap();

        let score = scorer
            .score_workspace_organization(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 6.0, "Should get 6pts for basic workspace");
    }

    #[test]
    fn test_workspace_organization_with_resolver() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[workspace]
members = ["crate-a"]
resolver = "2"
"#,
        )
        .unwrap();

        let score = scorer
            .score_workspace_organization(temp_dir.path(), None)
            .unwrap();
        assert_eq!(
            score, 9.0,
            "Should get 9pts (6 for workspace + 3 for resolver)"
        );
    }

    #[test]
    fn test_workspace_organization_no_workspace() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(&cargo_toml, "[package]\nname = \"single-crate\"").unwrap();

        let score = scorer
            .score_workspace_organization(temp_dir.path(), None)
            .unwrap();
        assert_eq!(
            score, 0.0,
            "Should get 0pts for single-crate project (no workspace)"
        );
    }

    // Release Automation Tests (12pts total)

    #[test]
    fn test_release_automation_full_score() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[package]
name = "test-crate"

[workspace]
members = ["crate-a", "crate-b"]

[package.metadata.release]
shared-version = true
tag-name = "v{{version}}"
pre-release-replacements = [
  {file="CHANGELOG.md", search="Unreleased", replace="{{version}}", min=1},
]
"#,
        )
        .unwrap();

        // Create post-release workflow
        let workflows_dir = temp_dir.path().join(".github/workflows");
        std::fs::create_dir_all(&workflows_dir).unwrap();
        std::fs::write(
            workflows_dir.join("post-release.yml"),
            r#"
name: Post-Release
on:
  release:
    types: [published]
"#,
        )
        .unwrap();

        let score = scorer
            .score_release_automation(temp_dir.path(), None)
            .unwrap();
        // +5 for [package.metadata.release]
        // +3 for CHANGELOG.md automation (pre-release-replacements)
        // +2 for shared-version (workspace version sync)
        // +2 for post-release.yml workflow
        assert_eq!(
            score, 12.0,
            "Should get full 12 points for complete release automation"
        );
    }

    #[test]
    fn test_release_automation_basic_metadata() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[package.metadata.release]
tag-name = "v{{version}}"
"#,
        )
        .unwrap();

        let score = scorer
            .score_release_automation(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 5.0, "Should get 5pts for basic release metadata");
    }

    #[test]
    fn test_release_automation_changelog_only() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[package.metadata.release]
pre-release-replacements = [
  {file="CHANGELOG.md", search="Unreleased", replace="{{version}}"},
]
"#,
        )
        .unwrap();

        let score = scorer
            .score_release_automation(temp_dir.path(), None)
            .unwrap();
        assert_eq!(
            score, 8.0,
            "Should get 8pts (5 for metadata + 3 for changelog automation)"
        );
    }

    #[test]
    fn test_release_automation_no_metadata() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(&cargo_toml, "[package]\nname = \"test\"").unwrap();

        let score = scorer
            .score_release_automation(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 0.0, "Should get 0pts with no release automation");
    }

    // MSRV Tracking Tests (10pts total)

    #[test]
    fn test_msrv_tracking_full_score() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        // Create Cargo.toml with rust-version
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[package]
name = "test-crate"
rust-version = "1.74"
"#,
        )
        .unwrap();

        // Create README with MSRV documentation
        let readme = temp_dir.path().join("README.md");
        std::fs::write(&readme, "MSRV: 1.74").unwrap();

        // Create CI workflow testing MSRV
        let workflows_dir = temp_dir.path().join(".github/workflows");
        std::fs::create_dir_all(&workflows_dir).unwrap();
        std::fs::write(
            workflows_dir.join("ci.yml"),
            r#"
jobs:
  test:
    strategy:
      matrix:
        rust: [1.74, stable]
"#,
        )
        .unwrap();

        let score = scorer.score_msrv_tracking(temp_dir.path(), None).unwrap();
        // +5 for rust-version field
        // +3 for CI testing MSRV
        // +2 for README documentation
        assert_eq!(
            score, 10.0,
            "Should get full 10 points for complete MSRV tracking"
        );
    }

    #[test]
    fn test_msrv_tracking_cargo_toml_only() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[package]
rust-version = "1.68"
"#,
        )
        .unwrap();

        let score = scorer.score_msrv_tracking(temp_dir.path(), None).unwrap();
        assert_eq!(score, 5.0, "Should get 5pts for rust-version field only");
    }

    #[test]
    fn test_msrv_tracking_with_ci() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(&cargo_toml, "[package]\nrust-version = \"1.70\"").unwrap();

        let workflows_dir = temp_dir.path().join(".github/workflows");
        std::fs::create_dir_all(&workflows_dir).unwrap();
        std::fs::write(workflows_dir.join("msrv.yml"), "rust: [1.70, stable]").unwrap();

        let score = scorer.score_msrv_tracking(temp_dir.path(), None).unwrap();
        assert_eq!(score, 8.0, "Should get 8pts (5 for field + 3 for CI)");
    }

    #[test]
    fn test_msrv_tracking_no_rust_version() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(&cargo_toml, "[package]\nname = \"test\"").unwrap();

        let score = scorer.score_msrv_tracking(temp_dir.path(), None).unwrap();
        assert_eq!(score, 0.0, "Should get 0pts without rust-version field");
    }

    // Release Profile Optimization Tests (11pts total)

    #[test]
    fn test_release_profiles_full_score() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[package]
name = "test-crate"

[profile.release]
lto = true
codegen-units = 1
panic = "abort"

[profile.dev]
panic = "abort"
"#,
        )
        .unwrap();

        let score = scorer
            .score_release_profiles(temp_dir.path(), None)
            .unwrap();
        // +4 for LTO in release
        // +3 for codegen-units = 1
        // +2 for panic = "abort" in release
        // +2 for panic = "abort" in dev
        assert_eq!(
            score, 11.0,
            "Should get full 11 points for optimized release profiles"
        );
    }

    #[test]
    fn test_release_profiles_release_only() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
"#,
        )
        .unwrap();

        let score = scorer
            .score_release_profiles(temp_dir.path(), None)
            .unwrap();
        assert_eq!(
            score, 9.0,
            "Should get 9pts (4+3+2 for release profile only)"
        );
    }

    #[test]
    fn test_release_profiles_lto_penalty() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[profile.release]
lto = true

[profile.dev]
lto = true
"#,
        )
        .unwrap();

        let score = scorer
            .score_release_profiles(temp_dir.path(), None)
            .unwrap();
        // +4 for LTO in release
        // -3 penalty for LTO in dev (slows TDD)
        assert_eq!(score, 1.0, "Should get 1pt (4 - 3 penalty for LTO in dev)");
    }

    #[test]
    fn test_release_profiles_lto_in_test_penalty() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[profile.release]
lto = true

[profile.test]
lto = true
"#,
        )
        .unwrap();

        let score = scorer
            .score_release_profiles(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 1.0, "Should get 1pt (4 - 3 penalty for LTO in test)");
    }

    #[test]
    fn test_release_profiles_no_optimizations() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(&cargo_toml, "[package]\nname = \"test\"").unwrap();

        let score = scorer
            .score_release_profiles(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 0.0, "Should get 0pts with no profile optimizations");
    }

    #[test]
    fn test_release_profiles_partial() {
        let scorer = RustToolingScorer::new();
        let temp_dir = TempDir::new().unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"
[profile.release]
lto = "thin"
"#,
        )
        .unwrap();

        let score = scorer
            .score_release_profiles(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 4.0, "Should get 4pts for LTO (thin counts)");
    }
}


mod coverage_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_project() -> TempDir {
        TempDir::new().expect("Failed to create temp dir")
    }

    fn create_cargo_toml(temp_dir: &TempDir, content: &str) {
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        fs::write(&cargo_toml, content).expect("Failed to write Cargo.toml");
    }

    // Default and Trait Implementation Tests

    #[test]
    fn test_rust_tooling_scorer_default() {
        let scorer = RustToolingScorer::default();
        assert_eq!(scorer.name(), "Rust Tooling & CI/CD");
        assert_eq!(scorer.max_points(), 130.0);
    }

    #[test]
    fn test_rust_tooling_scorer_new() {
        let scorer = RustToolingScorer::new();
        assert_eq!(scorer.name, "Rust Tooling & CI/CD".to_string());
        assert_eq!(scorer.max_points, 130.0);
    }

    // VulnerabilityCount Tests

    #[test]
    fn test_vulnerability_count_default() {
        let counts = VulnerabilityCount::default();
        assert_eq!(counts.critical, 0);
        assert_eq!(counts.high, 0);
        assert_eq!(counts.medium, 0);
        assert_eq!(counts.low, 0);
    }

    // parse_audit_json Tests

    #[test]
    fn test_parse_audit_json_empty_string() {
        let scorer = RustToolingScorer::new();
        let counts = scorer.parse_audit_json("");
        assert_eq!(counts.critical, 0);
        assert_eq!(counts.high, 0);
        assert_eq!(counts.medium, 0);
        assert_eq!(counts.low, 0);
    }

    #[test]
    fn test_parse_audit_json_invalid_json() {
        let scorer = RustToolingScorer::new();
        let counts = scorer.parse_audit_json("not valid json {{{");
        assert_eq!(counts.critical, 0);
        assert_eq!(counts.high, 0);
    }

    #[test]
    fn test_parse_audit_json_no_vulnerabilities() {
        let scorer = RustToolingScorer::new();
        let json = r#"{"vulnerabilities": {"list": []}}"#;
        let counts = scorer.parse_audit_json(json);
        assert_eq!(counts.critical, 0);
        assert_eq!(counts.high, 0);
    }

    #[test]
    fn test_parse_audit_json_single_critical() {
        let scorer = RustToolingScorer::new();
        let json = r#"{"vulnerabilities": {"list": [{"advisory": {"severity": "critical"}}]}}"#;
        let counts = scorer.parse_audit_json(json);
        assert_eq!(counts.critical, 1);
        assert_eq!(counts.high, 0);
        assert_eq!(counts.medium, 0);
        assert_eq!(counts.low, 0);
    }

    #[test]
    fn test_parse_audit_json_single_high() {
        let scorer = RustToolingScorer::new();
        let json = r#"{"vulnerabilities": {"list": [{"advisory": {"severity": "high"}}]}}"#;
        let counts = scorer.parse_audit_json(json);
        assert_eq!(counts.critical, 0);
        assert_eq!(counts.high, 1);
        assert_eq!(counts.medium, 0);
        assert_eq!(counts.low, 0);
    }

    #[test]
    fn test_parse_audit_json_single_medium() {
        let scorer = RustToolingScorer::new();
        let json = r#"{"vulnerabilities": {"list": [{"advisory": {"severity": "medium"}}]}}"#;
        let counts = scorer.parse_audit_json(json);
        assert_eq!(counts.critical, 0);
        assert_eq!(counts.high, 0);
        assert_eq!(counts.medium, 1);
        assert_eq!(counts.low, 0);
    }

    #[test]
    fn test_parse_audit_json_single_low() {
        let scorer = RustToolingScorer::new();
        let json = r#"{"vulnerabilities": {"list": [{"advisory": {"severity": "low"}}]}}"#;
        let counts = scorer.parse_audit_json(json);
        assert_eq!(counts.critical, 0);
        assert_eq!(counts.high, 0);
        assert_eq!(counts.medium, 0);
        assert_eq!(counts.low, 1);
    }

    #[test]
    fn test_parse_audit_json_mixed_severities() {
        let scorer = RustToolingScorer::new();
        let json = r#"{
            "vulnerabilities": {
                "list": [
                    {"advisory": {"severity": "CRITICAL"}},
                    {"advisory": {"severity": "HIGH"}},
                    {"advisory": {"severity": "MEDIUM"}},
                    {"advisory": {"severity": "LOW"}},
                    {"advisory": {"severity": "Critical"}},
                    {"advisory": {"severity": "Unknown"}}
                ]
            }
        }"#;
        let counts = scorer.parse_audit_json(json);
        assert_eq!(counts.critical, 2); // CRITICAL and Critical
        assert_eq!(counts.high, 1);
        assert_eq!(counts.medium, 1);
        assert_eq!(counts.low, 1);
        // Unknown severity ignored
    }

    #[test]
    fn test_parse_audit_json_missing_advisory() {
        let scorer = RustToolingScorer::new();
        let json = r#"{"vulnerabilities": {"list": [{"package": "some-crate"}]}}"#;
        let counts = scorer.parse_audit_json(json);
        assert_eq!(counts.critical, 0);
        assert_eq!(counts.high, 0);
    }

    #[test]
    fn test_parse_audit_json_missing_severity() {
        let scorer = RustToolingScorer::new();
        let json = r#"{"vulnerabilities": {"list": [{"advisory": {"id": "RUSTSEC-2021-0001"}}]}}"#;
        let counts = scorer.parse_audit_json(json);
        assert_eq!(counts.critical, 0);
        assert_eq!(counts.high, 0);
    }

    #[test]
    fn test_parse_audit_json_missing_vulnerabilities_key() {
        let scorer = RustToolingScorer::new();
        let json = r#"{"database": {"name": "RustSec Advisory Database"}}"#;
        let counts = scorer.parse_audit_json(json);
        assert_eq!(counts.critical, 0);
    }

    #[test]
    fn test_parse_audit_json_missing_list_key() {
        let scorer = RustToolingScorer::new();
        let json = r#"{"vulnerabilities": {"count": 0}}"#;
        let counts = scorer.parse_audit_json(json);
        assert_eq!(counts.critical, 0);
    }

    // score_cargo_deny Tests

    #[test]
    fn test_score_cargo_deny_with_deny_toml() {
        let temp_dir = create_temp_project();
        fs::write(
            temp_dir.path().join("deny.toml"),
            "[licenses]\nunlicensed = \"deny\"",
        )
        .expect("Failed to write deny.toml");

        let scorer = RustToolingScorer::new();
        let score = scorer.score_cargo_deny(temp_dir.path()).unwrap();
        assert_eq!(score, 3.0, "Should get full 3pts for having deny.toml");
    }

    #[test]
    fn test_score_cargo_deny_without_deny_toml() {
        let temp_dir = create_temp_project();

        let scorer = RustToolingScorer::new();
        let score = scorer.score_cargo_deny(temp_dir.path()).unwrap();
        assert_eq!(score, 0.0, "Should get 0pts without deny.toml");
    }

    // score_clippy Tests (Limited - requires cargo)

    #[test]
    fn test_score_clippy_no_cargo_toml() {
        let temp_dir = create_temp_project();
        let scorer = RustToolingScorer::new();

        let result = scorer.score_clippy(temp_dir.path());
        assert!(result.is_err());
        match result {
            Err(ScorerError::InvalidProject(msg)) => {
                assert!(msg.contains("No Cargo.toml"));
            }
            _ => panic!("Expected InvalidProject error"),
        }
    }

    // score_workspace_lints Tests - Additional Coverage

    #[test]
    fn test_workspace_lints_from_cache() {
        let temp_dir = create_temp_project();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        let cargo_toml_content = r#"
[workspace.lints.rust]
unsafe_op_in_unsafe_fn = "warn"
"#;
        fs::write(&cargo_toml_path, cargo_toml_content).expect("Failed to write");

        // Create cache
        let mut cache = FileCache::new();
        cache.insert(cargo_toml_path.clone(), cargo_toml_content.to_string());

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_workspace_lints(temp_dir.path(), Some(&cache))
            .unwrap();
        assert_eq!(score, 9.0, "Should work with cache");
    }

    #[test]
    fn test_workspace_lints_cache_missing_cargo_toml() {
        let temp_dir = create_temp_project();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        fs::write(&cargo_toml_path, "[package]\nname = \"test\"").expect("Failed to write");

        // Create empty cache (Cargo.toml not in cache)
        let cache = FileCache::new();

        let scorer = RustToolingScorer::new();
        let result = scorer.score_workspace_lints(temp_dir.path(), Some(&cache));
        assert!(result.is_err(), "Should error when Cargo.toml not in cache");
    }

    #[test]
    fn test_workspace_lints_clippy_toml_from_cache() {
        let temp_dir = create_temp_project();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let clippy_toml_path = temp_dir.path().join(".clippy.toml");

        let cargo_content = "[workspace.lints.clippy]\nchecked_conversions = \"warn\"";
        let clippy_content = "disallowed-methods = []";

        fs::write(&cargo_toml_path, cargo_content).expect("Failed to write");
        fs::write(&clippy_toml_path, clippy_content).expect("Failed to write");

        // Create cache with both files
        let mut cache = FileCache::new();
        cache.insert(cargo_toml_path.clone(), cargo_content.to_string());
        cache.insert(clippy_toml_path.clone(), clippy_content.to_string());

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_workspace_lints(temp_dir.path(), Some(&cache))
            .unwrap();
        assert_eq!(
            score, 12.0,
            "Should get full score with both files in cache"
        );
    }

    #[test]
    fn test_workspace_lints_clippy_toml_not_in_cache() {
        let temp_dir = create_temp_project();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let clippy_toml_path = temp_dir.path().join(".clippy.toml");

        let cargo_content = "[workspace.lints.clippy]\nchecked_conversions = \"warn\"";
        let clippy_content = "disallowed-methods = []";

        fs::write(&cargo_toml_path, cargo_content).expect("Failed to write");
        fs::write(&clippy_toml_path, clippy_content).expect("Failed to write");

        // Create cache WITHOUT clippy.toml
        let mut cache = FileCache::new();
        cache.insert(cargo_toml_path.clone(), cargo_content.to_string());

        let scorer = RustToolingScorer::new();
        let result = scorer.score_workspace_lints(temp_dir.path(), Some(&cache));
        assert!(
            result.is_err(),
            "Should error when .clippy.toml not in cache"
        );
    }

    // score_ci_cd_integration Tests - Additional Coverage

    #[test]
    fn test_ci_cd_stress_loom_workflows() {
        let temp_dir = create_temp_project();
        let workflows_dir = temp_dir.path().join(".github/workflows");
        fs::create_dir_all(&workflows_dir).expect("Failed to create dir");

        // Create stress test workflow
        fs::write(workflows_dir.join("stress.yml"), "name: Stress Test").expect("Failed to write");
        // Need at least 2 more for the workflow count bonus
        fs::write(workflows_dir.join("ci.yml"), "name: CI").expect("Failed to write");
        fs::write(workflows_dir.join("test.yml"), "name: Test").expect("Failed to write");

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_ci_cd_integration(temp_dir.path(), None)
            .unwrap();
        // +3 for stress workflow + +6 for >=3 workflows
        assert_eq!(
            score, 9.0,
            "Should get 9pts for stress workflow + 3+ workflows"
        );
    }

    #[test]
    fn test_ci_cd_loom_workflow() {
        let temp_dir = create_temp_project();
        let workflows_dir = temp_dir.path().join(".github/workflows");
        fs::create_dir_all(&workflows_dir).expect("Failed to create dir");

        fs::write(workflows_dir.join("loom.yml"), "name: Loom Test").expect("Failed to write");

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_ci_cd_integration(temp_dir.path(), None)
            .unwrap();
        // +3 for loom workflow
        assert_eq!(score, 3.0, "Should get 3pts for loom workflow");
    }

    #[test]
    fn test_ci_cd_security_workflow_filename() {
        let temp_dir = create_temp_project();
        let workflows_dir = temp_dir.path().join(".github/workflows");
        fs::create_dir_all(&workflows_dir).expect("Failed to create dir");

        fs::write(workflows_dir.join("security.yml"), "name: Security").expect("Failed to write");

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_ci_cd_integration(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 4.0, "Should get 4pts for security workflow");
    }

    #[test]
    fn test_ci_cd_benchmark_workflow_filename() {
        let temp_dir = create_temp_project();
        let workflows_dir = temp_dir.path().join(".github/workflows");
        fs::create_dir_all(&workflows_dir).expect("Failed to create dir");

        fs::write(workflows_dir.join("benchmark.yml"), "name: Benchmark").expect("Failed to write");

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_ci_cd_integration(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 3.0, "Should get 3pts for benchmark workflow");
    }

    #[test]
    fn test_ci_cd_clippy_workflow_filename() {
        let temp_dir = create_temp_project();
        let workflows_dir = temp_dir.path().join(".github/workflows");
        fs::create_dir_all(&workflows_dir).expect("Failed to create dir");

        fs::write(workflows_dir.join("clippy.yml"), "name: Clippy").expect("Failed to write");

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_ci_cd_integration(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 2.0, "Should get 2pts for clippy workflow");
    }

    #[test]
    fn test_ci_cd_spell_workflow_filename() {
        let temp_dir = create_temp_project();
        let workflows_dir = temp_dir.path().join(".github/workflows");
        fs::create_dir_all(&workflows_dir).expect("Failed to create dir");

        fs::write(workflows_dir.join("spell-check.yml"), "name: Spell Check")
            .expect("Failed to write");

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_ci_cd_integration(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 2.0, "Should get 2pts for spell-check workflow");
    }

    #[test]
    fn test_ci_cd_cargo_xtask() {
        let temp_dir = create_temp_project();
        let xtask_dir = temp_dir.path().join("xtask");
        fs::create_dir_all(&xtask_dir).expect("Failed to create dir");
        fs::write(xtask_dir.join("Cargo.toml"), "[package]\nname=\"xtask\"")
            .expect("Failed to write");

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_ci_cd_integration(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 5.0, "Should get 5pts for cargo-xtask");
    }

    #[test]
    fn test_ci_cd_justfile_missing_targets() {
        let temp_dir = create_temp_project();
        fs::write(temp_dir.path().join("justfile"), "# Empty justfile").expect("Failed to write");

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_ci_cd_integration(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 5.0, "Should get 5pts for justfile but no targets");
    }

    #[test]
    fn test_ci_cd_makefile_with_all_targets() {
        let temp_dir = create_temp_project();
        fs::write(
            temp_dir.path().join("Makefile"),
            r#"
build:
	cargo build

test:
	cargo test

lint:
	cargo clippy

bench:
	cargo bench
"#,
        )
        .expect("Failed to write");

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_ci_cd_integration(temp_dir.path(), None)
            .unwrap();
        assert_eq!(
            score, 6.0,
            "Should get 6pts (3 for Makefile + 3 for targets)"
        );
    }

    #[test]
    fn test_ci_cd_makefile_with_clippy_target() {
        let temp_dir = create_temp_project();
        fs::write(
            temp_dir.path().join("Makefile"),
            r#"
build:
	cargo build

test:
	cargo test

clippy:
	cargo clippy

bench:
	cargo bench
"#,
        )
        .expect("Failed to write");

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_ci_cd_integration(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 6.0, "Should get 6pts (clippy: counts as lint)");
    }

    #[test]
    fn test_ci_cd_yaml_extension() {
        let temp_dir = create_temp_project();
        let workflows_dir = temp_dir.path().join(".github/workflows");
        fs::create_dir_all(&workflows_dir).expect("Failed to create dir");

        // Use .yaml instead of .yml
        fs::write(workflows_dir.join("ci.yaml"), "name: CI").expect("Failed to write");
        fs::write(workflows_dir.join("test.yaml"), "name: Test").expect("Failed to write");
        fs::write(workflows_dir.join("lint.yaml"), "name: Lint").expect("Failed to write");

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_ci_cd_integration(temp_dir.path(), None)
            .unwrap();
        // +6 for >=3 workflows + +2 for lint workflow
        assert_eq!(score, 8.0, "Should work with .yaml extension");
    }

    #[test]
    fn test_ci_cd_ubuntu_variant() {
        let temp_dir = create_temp_project();
        let workflows_dir = temp_dir.path().join(".github/workflows");
        fs::create_dir_all(&workflows_dir).expect("Failed to create dir");

        // Use ubuntu-22.04 instead of ubuntu-latest
        fs::write(
            workflows_dir.join("ci.yml"),
            r#"
jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-22.04, windows-2022, macos-13]
"#,
        )
        .expect("Failed to write");

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_ci_cd_integration(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 6.0, "Should detect platform variants");
    }

    // score_docs_rs_metadata Tests - Additional Coverage

    #[test]
    fn test_docs_rs_metadata_no_cargo_toml() {
        let temp_dir = create_temp_project();

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_docs_rs_metadata(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 0.0, "Should get 0pts without Cargo.toml");
    }

    #[test]
    fn test_docs_rs_metadata_with_cache() {
        let temp_dir = create_temp_project();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        let content = r#"
[package.metadata.docs.rs]
all-features = true
"#;
        fs::write(&cargo_toml_path, content).expect("Failed to write");

        let mut cache = FileCache::new();
        cache.insert(cargo_toml_path.clone(), content.to_string());

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_docs_rs_metadata(temp_dir.path(), Some(&cache))
            .unwrap();
        assert_eq!(score, 8.0, "Should work with cache (5 + 3)");
    }

    // score_workspace_organization Tests - Additional Coverage

    #[test]
    fn test_workspace_organization_no_cargo_toml() {
        let temp_dir = create_temp_project();

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_workspace_organization(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 0.0, "Should get 0pts without Cargo.toml");
    }

    #[test]
    fn test_workspace_organization_single_quote_resolver() {
        let temp_dir = create_temp_project();
        create_cargo_toml(
            &temp_dir,
            r#"
[workspace]
members = ["crate-a"]
resolver = '2'
"#,
        );

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_workspace_organization(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 9.0, "Should detect single-quoted resolver");
    }

    #[test]
    fn test_workspace_organization_with_cache() {
        let temp_dir = create_temp_project();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        let content = r#"
[workspace]
members = ["crate-a"]
"#;
        fs::write(&cargo_toml_path, content).expect("Failed to write");

        let mut cache = FileCache::new();
        cache.insert(cargo_toml_path.clone(), content.to_string());

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_workspace_organization(temp_dir.path(), Some(&cache))
            .unwrap();
        assert_eq!(score, 6.0, "Should work with cache");
    }

    // score_release_automation Tests - Additional Coverage

    #[test]
    fn test_release_automation_no_cargo_toml() {
        let temp_dir = create_temp_project();

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_release_automation(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 0.0, "Should get 0pts without Cargo.toml");
    }

    #[test]
    fn test_release_automation_post_release_only() {
        let temp_dir = create_temp_project();
        create_cargo_toml(&temp_dir, "[package]\nname = \"test\"");

        let workflows_dir = temp_dir.path().join(".github/workflows");
        fs::create_dir_all(&workflows_dir).expect("Failed to create dir");
        fs::write(workflows_dir.join("post-release.yml"), "name: Post-Release")
            .expect("Failed to write");

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_release_automation(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 2.0, "Should get 2pts for post-release workflow only");
    }

    #[test]
    fn test_release_automation_with_cache() {
        let temp_dir = create_temp_project();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        let content = r#"
[package.metadata.release]
tag-name = "v{{version}}"
"#;
        fs::write(&cargo_toml_path, content).expect("Failed to write");

        let mut cache = FileCache::new();
        cache.insert(cargo_toml_path.clone(), content.to_string());

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_release_automation(temp_dir.path(), Some(&cache))
            .unwrap();
        assert_eq!(score, 5.0, "Should work with cache");
    }

    // score_msrv_tracking Tests - Additional Coverage

    #[test]
    fn test_msrv_tracking_no_cargo_toml() {
        let temp_dir = create_temp_project();

        let scorer = RustToolingScorer::new();
        let score = scorer.score_msrv_tracking(temp_dir.path(), None).unwrap();
        assert_eq!(score, 0.0, "Should get 0pts without Cargo.toml");
    }

    #[test]
    fn test_msrv_tracking_three_part_version() {
        let temp_dir = create_temp_project();
        create_cargo_toml(
            &temp_dir,
            r#"
[package]
rust-version = "1.74.0"
"#,
        );

        let workflows_dir = temp_dir.path().join(".github/workflows");
        fs::create_dir_all(&workflows_dir).expect("Failed to create dir");
        fs::write(workflows_dir.join("ci.yml"), "rust: [1.74, stable]").expect("Failed to write");

        let scorer = RustToolingScorer::new();
        let score = scorer.score_msrv_tracking(temp_dir.path(), None).unwrap();
        assert_eq!(score, 8.0, "Should parse 3-part version correctly");
    }

    #[test]
    fn test_msrv_tracking_readme_msrv_lowercase() {
        let temp_dir = create_temp_project();
        create_cargo_toml(&temp_dir, "[package]\nrust-version = \"1.70\"");
        fs::write(temp_dir.path().join("README.md"), "msrv: 1.70").expect("Failed to write");

        let scorer = RustToolingScorer::new();
        let score = scorer.score_msrv_tracking(temp_dir.path(), None).unwrap();
        assert_eq!(score, 7.0, "Should detect lowercase MSRV in README");
    }

    #[test]
    fn test_msrv_tracking_with_cache() {
        let temp_dir = create_temp_project();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        let content = "[package]\nrust-version = \"1.68\"";
        fs::write(&cargo_toml_path, content).expect("Failed to write");

        let mut cache = FileCache::new();
        cache.insert(cargo_toml_path.clone(), content.to_string());

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_msrv_tracking(temp_dir.path(), Some(&cache))
            .unwrap();
        assert_eq!(score, 5.0, "Should work with cache");
    }

    // score_release_profiles Tests - Additional Coverage

    #[test]
    fn test_release_profiles_no_cargo_toml() {
        let temp_dir = create_temp_project();

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_release_profiles(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 0.0, "Should get 0pts without Cargo.toml");
    }

    #[test]
    fn test_release_profiles_lto_fat() {
        let temp_dir = create_temp_project();
        create_cargo_toml(
            &temp_dir,
            r#"
[profile.release]
lto = "fat"
"#,
        );

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_release_profiles(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 4.0, "Should get 4pts for fat LTO");
    }

    #[test]
    fn test_release_profiles_panic_single_quotes() {
        let temp_dir = create_temp_project();
        create_cargo_toml(
            &temp_dir,
            r#"
[profile.release]
panic = 'abort'

[profile.dev]
panic = 'abort'
"#,
        );

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_release_profiles(temp_dir.path(), None)
            .unwrap();
        assert_eq!(score, 4.0, "Should detect single-quoted panic values");
    }

    #[test]
    fn test_release_profiles_lto_single_quotes() {
        let temp_dir = create_temp_project();
        create_cargo_toml(
            &temp_dir,
            r#"
[profile.release]
lto = 'fat'

[profile.dev]
lto = 'thin'
"#,
        );

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_release_profiles(temp_dir.path(), None)
            .unwrap();
        // +4 for release LTO, -3 for dev LTO = 1
        assert_eq!(score, 1.0, "Should detect single-quoted LTO values");
    }

    #[test]
    fn test_release_profiles_with_cache() {
        let temp_dir = create_temp_project();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        let content = r#"
[profile.release]
lto = true
"#;
        fs::write(&cargo_toml_path, content).expect("Failed to write");

        let mut cache = FileCache::new();
        cache.insert(cargo_toml_path.clone(), content.to_string());

        let scorer = RustToolingScorer::new();
        let score = scorer
            .score_release_profiles(temp_dir.path(), Some(&cache))
            .unwrap();
        assert_eq!(score, 4.0, "Should work with cache");
    }

    // score_internal Tests

    #[test]
    fn test_score_internal_no_cargo_toml() {
        let temp_dir = create_temp_project();
        let scorer = RustToolingScorer::new();

        let result = scorer.score_internal(temp_dir.path(), ScoringMode::Fast, None);
        assert!(result.is_err());
        match result {
            Err(ScorerError::InvalidProject(msg)) => {
                assert!(msg.contains("No Cargo.toml"));
            }
            _ => panic!("Expected InvalidProject error"),
        }
    }

    #[test]
    fn test_score_internal_fast_mode_rustfmt_toml_exists() {
        let temp_dir = create_temp_project();
        create_cargo_toml(&temp_dir, "[package]\nname = \"test\"");
        fs::write(temp_dir.path().join("rustfmt.toml"), "max_width = 100")
            .expect("Failed to write");

        let scorer = RustToolingScorer::new();
        let result = scorer
            .score_internal(temp_dir.path(), ScoringMode::Fast, None)
            .unwrap();

        // Fast mode with rustfmt.toml should give 3 pts for rustfmt instead of 2.5
        // Total: 5 (clippy) + 3 (rustfmt) + 3.5 (audit) + 0 (deny) + ... = varies
        assert!(result.earned > 0.0);
    }

    #[test]
    fn test_score_internal_fast_mode_dot_rustfmt_toml() {
        let temp_dir = create_temp_project();
        create_cargo_toml(&temp_dir, "[package]\nname = \"test\"");
        fs::write(temp_dir.path().join(".rustfmt.toml"), "max_width = 100")
            .expect("Failed to write");

        let scorer = RustToolingScorer::new();
        let result = scorer
            .score_internal(temp_dir.path(), ScoringMode::Fast, None)
            .unwrap();

        assert!(result.earned > 0.0);
    }

    // Scorer Trait Tests

    #[test]
    fn test_scorer_score_method() {
        let temp_dir = create_temp_project();
        create_cargo_toml(&temp_dir, "[package]\nname = \"test\"");

        let scorer = RustToolingScorer::new();
        let result = scorer.score(temp_dir.path()).unwrap();

        assert!(result.earned >= 0.0);
        // Max score depends on scoring configuration
        assert!(result.max > 0.0, "Max score should be positive");
    }

    #[test]
    fn test_scorer_score_with_mode_method() {
        let temp_dir = create_temp_project();
        create_cargo_toml(&temp_dir, "[package]\nname = \"test\"");

        let scorer = RustToolingScorer::new();
        let result = scorer
            .score_with_mode(temp_dir.path(), ScoringMode::Fast)
            .unwrap();

        assert!(result.earned >= 0.0);
    }

    #[test]
    fn test_scorer_score_with_cache_method() {
        let temp_dir = create_temp_project();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let content = "[package]\nname = \"test\"";
        fs::write(&cargo_toml_path, content).expect("Failed to write");

        let mut cache = FileCache::new();
        cache.insert(cargo_toml_path.clone(), content.to_string());

        let scorer = RustToolingScorer::new();
        let result = scorer
            .score_with_cache(temp_dir.path(), ScoringMode::Fast, Some(&cache))
            .unwrap();

        assert!(result.earned >= 0.0);
    }

    // recommendations Tests

    #[test]
    fn test_recommendations_basic() {
        let temp_dir = create_temp_project();
        create_cargo_toml(&temp_dir, "[package]\nname = \"test\"");

        let scorer = RustToolingScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        // Should always have clippy, fmt, and audit recommendations
        assert!(recommendations.iter().any(|r| r.contains("clippy")));
        assert!(recommendations.iter().any(|r| r.contains("fmt")));
        assert!(recommendations.iter().any(|r| r.contains("audit")));
    }

    #[test]
    fn test_recommendations_with_deny_toml() {
        let temp_dir = create_temp_project();
        create_cargo_toml(&temp_dir, "[package]\nname = \"test\"");
        fs::write(temp_dir.path().join("deny.toml"), "[licenses]").expect("Failed to write");

        let scorer = RustToolingScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        // Should NOT recommend deny.toml if it already exists
        assert!(!recommendations.iter().any(|r| r.contains("deny.toml")));
    }

    #[test]
    fn test_recommendations_without_deny_toml() {
        let temp_dir = create_temp_project();
        create_cargo_toml(&temp_dir, "[package]\nname = \"test\"");

        let scorer = RustToolingScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        // Should recommend deny.toml if missing
        assert!(recommendations.iter().any(|r| r.contains("deny.toml")));
    }

    #[test]
    fn test_recommendations_no_workspace_lints() {
        let temp_dir = create_temp_project();
        create_cargo_toml(&temp_dir, "[package]\nname = \"test\"");

        let scorer = RustToolingScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        // Should recommend workspace lints
        assert!(recommendations
            .iter()
            .any(|r| r.contains("workspace.lints")));
    }

    #[test]
    fn test_recommendations_with_workspace_lints() {
        let temp_dir = create_temp_project();
        create_cargo_toml(
            &temp_dir,
            r#"
[package]
name = "test"

[workspace.lints.rust]
unsafe_op_in_unsafe_fn = "warn"
"#,
        );

        let scorer = RustToolingScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        // Should NOT recommend workspace lints if present
        assert!(!recommendations
            .iter()
            .any(|r| r.contains("[workspace.lints") && !r.contains("high-value lint")));
    }

    #[test]
    fn test_recommendations_no_clippy_toml() {
        let temp_dir = create_temp_project();
        create_cargo_toml(&temp_dir, "[package]\nname = \"test\"");

        let scorer = RustToolingScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        // Should recommend .clippy.toml
        assert!(recommendations.iter().any(|r| r.contains(".clippy.toml")));
    }

    #[test]
    fn test_recommendations_with_clippy_toml() {
        let temp_dir = create_temp_project();
        create_cargo_toml(&temp_dir, "[package]\nname = \"test\"");
        fs::write(
            temp_dir.path().join(".clippy.toml"),
            "disallowed-methods = []",
        )
        .expect("Failed to write");

        let scorer = RustToolingScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        // Should NOT recommend .clippy.toml if it exists
        assert!(!recommendations
            .iter()
            .any(|r| r.contains("Create .clippy.toml")));
    }

    #[test]
    fn test_recommendations_no_cargo_toml() {
        let temp_dir = create_temp_project();

        let scorer = RustToolingScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        // Should still return some recommendations
        assert!(!recommendations.is_empty());
    }
}
