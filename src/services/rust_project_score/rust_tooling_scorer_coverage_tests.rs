//! Coverage tests for rust tooling scorer
//! Extracted for file health compliance (CB-040)

use super::super::*;
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

    fs::write(workflows_dir.join("spell-check.yml"), "name: Spell Check").expect("Failed to write");

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
    fs::write(xtask_dir.join("Cargo.toml"), "[package]\nname=\"xtask\"").expect("Failed to write");

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
    fs::write(temp_dir.path().join("rustfmt.toml"), "max_width = 100").expect("Failed to write");

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
    fs::write(temp_dir.path().join(".rustfmt.toml"), "max_width = 100").expect("Failed to write");

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
