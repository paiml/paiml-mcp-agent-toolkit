#![cfg_attr(coverage_nightly, coverage(off))]

use super::detection_cc001_cc002::{
    detect_cc001_function_clones, detect_cc002_api_divergence, normalize_signature,
};
use super::discovery::{discover_workspace_crates, read_cargo_deps, read_crate_name};
use super::types::*;
use crate::services::agent_context::FunctionEntry;
use std::collections::HashSet;
use std::path::PathBuf;

#[test]
fn test_discover_workspace_crates_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let crates = discover_workspace_crates(tmp.path(), None);
    // Should have at least the base crate
    assert_eq!(crates.len(), 1);
    assert!(!crates[0].name.is_empty());
}

#[test]
fn test_discover_from_cargo_workspace() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // Create workspace Cargo.toml
    std::fs::write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["crate-a", "crate-b"]
"#,
    )
    .unwrap();

    // Create member crates
    for name in &["crate-a", "crate-b"] {
        let dir = root.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("Cargo.toml"),
            format!("[package]\nname = \"{}\"\nversion = \"0.1.0\"\n", name),
        )
        .unwrap();
    }

    let crates = discover_workspace_crates(root, None);
    assert_eq!(crates.len(), 2);
    let names: HashSet<&str> = crates.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains("crate-a"));
    assert!(names.contains("crate-b"));
}

#[test]
fn test_discover_from_cargo_workspace_with_glob() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // Create workspace Cargo.toml with glob
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/*\"]\n",
    )
    .unwrap();

    // Create member crates under crates/
    let crates_dir = root.join("crates");
    std::fs::create_dir_all(&crates_dir).unwrap();
    for name in &["alpha", "beta"] {
        let dir = crates_dir.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("Cargo.toml"),
            format!("[package]\nname = \"{}\"\nversion = \"0.1.0\"\n", name),
        )
        .unwrap();
    }

    let crates = discover_workspace_crates(root, None);
    assert_eq!(crates.len(), 2);
    let names: HashSet<&str> = crates.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains("alpha"));
    assert!(names.contains("beta"));
}

#[test]
fn test_discover_explicit_crates() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // Create base Cargo.toml
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"base\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    // Create sibling crate
    let sibling = root.join("sibling");
    std::fs::create_dir_all(&sibling).unwrap();
    std::fs::write(
        sibling.join("Cargo.toml"),
        "[package]\nname = \"sibling\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    let explicit = vec![sibling];
    let crates = discover_workspace_crates(root, Some(&explicit));
    assert_eq!(crates.len(), 2);
    assert_eq!(crates[0].name, "base");
    assert_eq!(crates[1].name, "sibling");
}

#[test]
fn test_read_cargo_deps_parses_section() {
    let tmp = tempfile::tempdir().unwrap();
    let cargo_toml = tmp.path().join("Cargo.toml");
    std::fs::write(
        &cargo_toml,
        r#"[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = { version = "1", features = ["full"] }
anyhow = "1"

[dev-dependencies]
tempfile = "3"
"#,
    )
    .unwrap();

    let deps = read_cargo_deps(&cargo_toml);
    assert!(deps.contains(&"serde".to_string()));
    assert!(deps.contains(&"tokio".to_string()));
    assert!(deps.contains(&"anyhow".to_string()));
    assert!(deps.contains(&"tempfile".to_string()));
}

#[test]
fn test_read_crate_name() {
    let tmp = tempfile::tempdir().unwrap();
    let cargo_toml = tmp.path().join("Cargo.toml");
    std::fs::write(
        &cargo_toml,
        "[package]\nname = \"my-cool-crate\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    assert_eq!(
        read_crate_name(&cargo_toml),
        Some("my-cool-crate".to_string())
    );
}

#[test]
fn test_normalize_signature_strips_pub() {
    assert_eq!(
        normalize_signature("pub fn foo(x: i32) -> bool"),
        "fn foo(x: i32) -> bool"
    );
    assert_eq!(normalize_signature("pub async fn bar()"), "fn bar()");
    assert_eq!(normalize_signature("fn baz(s: &str)"), "fn baz(s: &str)");
}

#[test]
fn test_cc001_detects_identical_source() {
    let crate_a = CrateInfo {
        name: "crate_a".to_string(),
        path: PathBuf::from("/tmp/a"),
        cargo_deps: vec![],
    };
    let crate_b = CrateInfo {
        name: "crate_b".to_string(),
        path: PathBuf::from("/tmp/b"),
        cargo_deps: vec![],
    };

    let source = r#"pub fn silu_activation(input: &[f32], output: &mut [f32]) {
    assert_eq!(input.len(), output.len(), "Input and output lengths must match");
    for i in 0..input.len() {
        let x = input[i];
        let sigmoid = 1.0 / (1.0 + (-x).exp());
        output[i] = x * sigmoid;
    }
}"#
    .to_string();
    let func_a = make_test_func("silu_activation", &source, "src/a.rs");
    let func_b = make_test_func("silu_activation", &source, "src/b.rs");

    let crate_functions = vec![(crate_a, vec![func_a]), (crate_b, vec![func_b])];
    let det = default_detection_config();

    let findings = detect_cc001_function_clones(&crate_functions, 0.80, &det);
    assert!(
        !findings.is_empty(),
        "CC-001 should detect identical functions across crates"
    );
    assert_eq!(findings[0].rule, "CC-001");
}

#[test]
fn test_cc001_no_finding_within_same_crate() {
    let crate_a = CrateInfo {
        name: "crate_a".to_string(),
        path: PathBuf::from("/tmp/a"),
        cargo_deps: vec![],
    };

    let source = r#"pub fn silu_activation(input: &[f32], output: &mut [f32]) {
    assert_eq!(input.len(), output.len(), "Input and output lengths must match");
    for i in 0..input.len() {
        let x = input[i];
        let sigmoid = 1.0 / (1.0 + (-x).exp());
        output[i] = x * sigmoid;
    }
}"#
    .to_string();
    let func_a = make_test_func("silu_activation", &source, "src/a.rs");
    let func_b = make_test_func("silu_activation_v2", &source, "src/b.rs");

    let crate_functions = vec![(crate_a, vec![func_a, func_b])];
    let det = default_detection_config();

    let findings = detect_cc001_function_clones(&crate_functions, 0.80, &det);
    assert!(
        findings.is_empty(),
        "CC-001 should NOT flag duplicates within the same crate"
    );
}

#[test]
fn test_cc002_same_name_different_sig() {
    let crate_a = CrateInfo {
        name: "crate_a".to_string(),
        path: PathBuf::from("/tmp/a"),
        cargo_deps: vec![],
    };
    let crate_b = CrateInfo {
        name: "crate_b".to_string(),
        path: PathBuf::from("/tmp/b"),
        cargo_deps: vec!["crate_a".to_string()],
    };

    let func_a = make_test_func_with_sig(
        "rms_norm",
        "pub fn rms_norm(x: &[f32]) -> Vec<f32>",
        "src/a.rs",
    );
    let func_b = make_test_func_with_sig(
        "rms_norm",
        "pub fn rms_norm(x: &[f64], eps: f64) -> Vec<f64>",
        "src/b.rs",
    );

    let crate_functions = vec![(crate_a, vec![func_a]), (crate_b, vec![func_b])];
    let det = default_detection_config();

    let findings = detect_cc002_api_divergence(&crate_functions, &det);
    assert!(
        !findings.is_empty(),
        "CC-002 should detect divergent signatures"
    );
    assert_eq!(findings[0].rule, "CC-002");
}

#[test]
fn test_cc002_same_name_same_sig_no_finding() {
    let crate_a = CrateInfo {
        name: "crate_a".to_string(),
        path: PathBuf::from("/tmp/a"),
        cargo_deps: vec![],
    };
    let crate_b = CrateInfo {
        name: "crate_b".to_string(),
        path: PathBuf::from("/tmp/b"),
        cargo_deps: vec!["crate_a".to_string()],
    };

    let func_a = make_test_func_with_sig("gelu", "pub fn gelu(x: f32) -> f32", "src/a.rs");
    let func_b = make_test_func_with_sig("gelu", "pub fn gelu(x: f32) -> f32", "src/b.rs");

    let crate_functions = vec![(crate_a, vec![func_a]), (crate_b, vec![func_b])];
    let det = default_detection_config();

    let findings = detect_cc002_api_divergence(&crate_functions, &det);
    assert!(
        findings.is_empty(),
        "CC-002 should NOT flag identical signatures"
    );
}

// --- Test helpers ---

fn default_detection_config() -> DetectionConfig {
    DetectionConfig {
        excluded_functions: HashSet::new(),
        excluded_crate_pairs: HashSet::new(),
        min_body_lines: 3,
        min_tokens: 15,
        cc003_min_similarity: 0.5,
    }
}

fn make_test_func(name: &str, source: &str, file_path: &str) -> FunctionEntry {
    FunctionEntry {
        function_name: name.to_string(),
        signature: format!("fn {name}()"),
        source: source.to_string(),
        file_path: file_path.to_string(),
        doc_comment: None,
        definition_type: Default::default(),
        start_line: 1,
        end_line: 1,
        language: "Rust".to_string(),
        quality: Default::default(),
        checksum: String::new(),
        commit_count: 0,
        churn_score: 0.0,
        clone_count: 0,
        pattern_diversity: 0.0,
        fault_annotations: vec![],
    }
}

fn make_test_func_with_sig(name: &str, sig: &str, file_path: &str) -> FunctionEntry {
    let mut func = make_test_func(name, "// placeholder source", file_path);
    func.signature = sig.to_string();
    func
}
