#![cfg_attr(coverage_nightly, coverage(off))]

use super::baseline::ratchet_threshold;
use super::detection_cc003_cc004::detect_cc003_primitive_upstream;
use super::detection_cc005::detect_cc005_example_duplication;
use super::discovery::parse_workspace_members_with_globs;
use super::helpers::{
    build_report, is_crate_pair_excluded, is_excluded_function, parse_rules_filter,
};
use super::types::*;
use crate::models::comply_config::PmatYamlConfig;
use crate::services::agent_context::FunctionEntry;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

// Re-use the test helpers from the main tests module
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
        linked_definition: None,
    }
}

#[test]
fn test_cc003_finding_when_dep_reimplements() {
    let crate_a = CrateInfo {
        name: "trueno".to_string(),
        path: PathBuf::from("/tmp/trueno"),
        cargo_deps: vec![],
    };
    let crate_b = CrateInfo {
        name: "aprender".to_string(),
        path: PathBuf::from("/tmp/aprender"),
        cargo_deps: vec!["trueno".to_string()],
    };

    // Use longer, realistic function bodies so MinHash signatures are computed
    let src_a = r#"pub fn f16_to_f32(input: &[u16], output: &mut [f32]) {
    assert_eq!(input.len(), output.len(), "Length mismatch");
    for i in 0..input.len() {
        let bits = input[i];
        let sign = (bits >> 15) & 1;
        let exponent = (bits >> 10) & 0x1F;
        let mantissa = bits & 0x3FF;
        let f32_bits = (sign as u32) << 31 | (exponent as u32 + 112) << 23 | (mantissa as u32) << 13;
        output[i] = f32::from_bits(f32_bits);
    }
}"#;
    let src_b = r#"pub fn f16_to_f32(input: &[u16], output: &mut [f32]) {
    assert_eq!(input.len(), output.len(), "Length mismatch");
    for idx in 0..input.len() {
        let raw = input[idx];
        let sign = (raw >> 15) & 1;
        let exponent = (raw >> 10) & 0x1F;
        let mantissa = raw & 0x3FF;
        let f32_bits = (sign as u32) << 31 | (exponent as u32 + 112) << 23 | (mantissa as u32) << 13;
        output[idx] = f32::from_bits(f32_bits);
    }
}"#;
    let func_a = make_test_func("f16_to_f32", src_a, "src/conv.rs");
    let func_b = make_test_func("f16_to_f32", src_b, "src/quant.rs");

    let crate_functions = vec![(crate_a, vec![func_a]), (crate_b, vec![func_b])];
    let det = default_detection_config();

    let findings = detect_cc003_primitive_upstream(&crate_functions, &det);
    assert!(
        !findings.is_empty(),
        "CC-003 should detect reimplementation of upstream function"
    );
    assert_eq!(findings[0].rule, "CC-003");
    assert!(
        findings[0].similarity.is_some(),
        "CC-003 should now include similarity score from MinHash"
    );
}

#[test]
fn test_cc003_no_finding_when_no_dep() {
    let crate_a = CrateInfo {
        name: "trueno".to_string(),
        path: PathBuf::from("/tmp/trueno"),
        cargo_deps: vec![],
    };
    let crate_b = CrateInfo {
        name: "unrelated".to_string(),
        path: PathBuf::from("/tmp/unrelated"),
        cargo_deps: vec!["serde".to_string()],
    };

    let src_a = r#"pub fn f16_to_f32(input: &[u16], output: &mut [f32]) {
    assert_eq!(input.len(), output.len(), "Length mismatch");
    for i in 0..input.len() {
        let bits = input[i];
        let sign = (bits >> 15) & 1;
        let exponent = (bits >> 10) & 0x1F;
        output[i] = f32::from_bits((sign as u32) << 31 | (exponent as u32) << 23);
    }
}"#;
    let src_b = r#"pub fn f16_to_f32(input: &[u16], output: &mut [f32]) {
    assert_eq!(input.len(), output.len(), "Length mismatch");
    for idx in 0..input.len() {
        let raw = input[idx];
        let sign = (raw >> 15) & 1;
        let exponent = (raw >> 10) & 0x1F;
        output[idx] = f32::from_bits((sign as u32) << 31 | (exponent as u32) << 23);
    }
}"#;
    let func_a = make_test_func("f16_to_f32", src_a, "src/conv.rs");
    let func_b = make_test_func("f16_to_f32", src_b, "src/quant.rs");

    let crate_functions = vec![(crate_a, vec![func_a]), (crate_b, vec![func_b])];
    let det = default_detection_config();

    let findings = detect_cc003_primitive_upstream(&crate_functions, &det);
    assert!(
        findings.is_empty(),
        "CC-003 should NOT flag when no dependency relationship"
    );
}

#[test]
fn test_cc005_no_examples_dir() {
    let crate_a = CrateInfo {
        name: "crate_a".to_string(),
        path: PathBuf::from("/tmp/nonexistent_crate_path"),
        cargo_deps: vec![],
    };

    let crate_functions = vec![(crate_a, vec![])];

    let findings = detect_cc005_example_duplication(&crate_functions, 0.80);
    assert!(
        findings.is_empty(),
        "CC-005 should gracefully skip missing examples/"
    );
}

#[test]
fn test_build_report_summary() {
    let findings = vec![
        CrossCrateFinding {
            rule: "CC-001".to_string(),
            severity: CcSeverity::Error,
            crate_a: "a".to_string(),
            crate_b: "b".to_string(),
            function_a: "f".to_string(),
            function_b: "f".to_string(),
            file_a: "a.rs".to_string(),
            file_b: "b.rs".to_string(),
            similarity: Some(0.95),
            recommendation: "Consolidate".to_string(),
        },
        CrossCrateFinding {
            rule: "CC-002".to_string(),
            severity: CcSeverity::Warning,
            crate_a: "a".to_string(),
            crate_b: "c".to_string(),
            function_a: "g".to_string(),
            function_b: "g".to_string(),
            file_a: "a.rs".to_string(),
            file_b: "c.rs".to_string(),
            similarity: None,
            recommendation: "Align signatures".to_string(),
        },
        CrossCrateFinding {
            rule: "CC-004".to_string(),
            severity: CcSeverity::Advisory,
            crate_a: "a".to_string(),
            crate_b: "b".to_string(),
            function_a: "h".to_string(),
            function_b: "h".to_string(),
            file_a: "a.rs".to_string(),
            file_b: "b.rs".to_string(),
            similarity: None,
            recommendation: "Review".to_string(),
        },
    ];

    let report = build_report(
        findings,
        vec!["a".to_string(), "b".to_string(), "c".to_string()],
    );
    assert_eq!(report.summary.total_findings, 3);
    assert_eq!(report.summary.errors, 1);
    assert_eq!(report.summary.warnings, 1);
    assert_eq!(report.summary.advisories, 1);
    assert_eq!(report.summary.rules_triggered["CC-001"], 1);
    assert_eq!(report.summary.rules_triggered["CC-002"], 1);
}

#[test]
fn test_parse_rules_filter() {
    assert!(parse_rules_filter(None).is_none());

    let set = parse_rules_filter(Some("cc001,cc003")).unwrap();
    assert!(set.contains("cc001"));
    assert!(set.contains("cc003"));
    assert!(!set.contains("cc002"));

    let set2 = parse_rules_filter(Some(" CC001 , CC002 ")).unwrap();
    assert!(set2.contains("cc001"));
    assert!(set2.contains("cc002"));
}

#[test]
fn test_excluded_function_combines_hardcoded_and_config() {
    let config = DetectionConfig {
        excluded_functions: HashSet::from(["my_accessor".to_string()]),
        excluded_crate_pairs: HashSet::new(),
        min_body_lines: 3,
        min_tokens: 15,
        cc003_min_similarity: 0.5,
    };
    // Hardcoded exclusion
    assert!(is_excluded_function("shape", &config));
    assert!(is_excluded_function("default", &config));
    // Config exclusion
    assert!(is_excluded_function("my_accessor", &config));
    // Not excluded
    assert!(!is_excluded_function("silu_activation", &config));
}

#[test]
fn test_crate_pair_excluded() {
    let excluded = HashSet::from([("trueno".to_string(), "aprender".to_string())]);
    assert!(is_crate_pair_excluded("trueno", "aprender", &excluded));
    assert!(is_crate_pair_excluded("aprender", "trueno", &excluded));
    assert!(!is_crate_pair_excluded("trueno", "realizar", &excluded));
}

#[test]
fn test_baseline_ratchet_passes_when_equal() {
    let baseline = CrossCrateBaseline {
        version: "1.0".to_string(),
        generated: "2026-02-20".to_string(),
        rule_counts: HashMap::from([("CC-001".to_string(), 10)]),
        total_findings: 10,
    };
    let report = CrossCrateReport {
        findings: Vec::new(),
        summary: CrossCrateSummary {
            total_findings: 10,
            errors: 0,
            warnings: 10,
            advisories: 0,
            rules_triggered: HashMap::from([("CC-001".to_string(), 10)]),
        },
        crates_analyzed: vec!["a".to_string()],
    };
    let violations = baseline.check_ratchet(&report);
    assert!(violations.is_empty(), "Same counts should pass ratchet");
}

#[test]
fn test_baseline_ratchet_passes_when_decreased() {
    let baseline = CrossCrateBaseline {
        version: "1.0".to_string(),
        generated: "2026-02-20".to_string(),
        rule_counts: HashMap::from([("CC-001".to_string(), 10)]),
        total_findings: 10,
    };
    let report = CrossCrateReport {
        findings: Vec::new(),
        summary: CrossCrateSummary {
            total_findings: 5,
            errors: 0,
            warnings: 5,
            advisories: 0,
            rules_triggered: HashMap::from([("CC-001".to_string(), 5)]),
        },
        crates_analyzed: vec!["a".to_string()],
    };
    let violations = baseline.check_ratchet(&report);
    assert!(
        violations.is_empty(),
        "Decreased counts should pass ratchet"
    );
}

#[test]
fn test_baseline_ratchet_fails_when_increased() {
    let baseline = CrossCrateBaseline {
        version: "1.0".to_string(),
        generated: "2026-02-20".to_string(),
        rule_counts: HashMap::from([("CC-001".to_string(), 10)]),
        total_findings: 10,
    };
    let report = CrossCrateReport {
        findings: Vec::new(),
        summary: CrossCrateSummary {
            total_findings: 15,
            errors: 0,
            warnings: 15,
            advisories: 0,
            rules_triggered: HashMap::from([("CC-001".to_string(), 15)]),
        },
        crates_analyzed: vec!["a".to_string()],
    };
    let violations = baseline.check_ratchet(&report);
    assert!(
        !violations.is_empty(),
        "Increased counts should fail ratchet"
    );
    assert_eq!(violations[0].0, "CC-001");
    assert_eq!(violations[0].1, 10);
    assert_eq!(violations[0].2, 15);
}

#[test]
fn test_baseline_ratchet_tolerates_minhash_jitter() {
    // CC-001 has 25% tolerance: baseline=100, threshold=125
    let baseline = CrossCrateBaseline {
        version: "1.0".to_string(),
        generated: "2026-02-20".to_string(),
        rule_counts: HashMap::from([("CC-001".to_string(), 100)]),
        total_findings: 100,
    };
    let report = CrossCrateReport {
        findings: Vec::new(),
        summary: CrossCrateSummary {
            total_findings: 120,
            errors: 0,
            warnings: 120,
            advisories: 0,
            rules_triggered: HashMap::from([("CC-001".to_string(), 120)]),
        },
        crates_analyzed: vec!["a".to_string()],
    };
    let violations = baseline.check_ratchet(&report);
    assert!(
        violations.is_empty(),
        "20% increase in CC-001 should be within tolerance"
    );
}

#[test]
fn test_baseline_ratchet_cc002_exact() {
    // CC-002 is deterministic — no tolerance
    let baseline = CrossCrateBaseline {
        version: "1.0".to_string(),
        generated: "2026-02-20".to_string(),
        rule_counts: HashMap::from([("CC-002".to_string(), 100)]),
        total_findings: 100,
    };
    let report = CrossCrateReport {
        findings: Vec::new(),
        summary: CrossCrateSummary {
            total_findings: 101,
            errors: 0,
            warnings: 101,
            advisories: 0,
            rules_triggered: HashMap::from([("CC-002".to_string(), 101)]),
        },
        crates_analyzed: vec!["a".to_string()],
    };
    let violations = baseline.check_ratchet(&report);
    assert!(
        !violations.is_empty(),
        "CC-002 should fail on +1 increase (no tolerance)"
    );
}

#[test]
fn test_ratchet_threshold_function() {
    // MinHash-based rules get 25% tolerance
    assert_eq!(ratchet_threshold("CC-001", 100), 125);
    assert_eq!(ratchet_threshold("CC-003", 52), 65);
    assert_eq!(ratchet_threshold("CC-005", 60), 75);
    // Deterministic rules get exact comparison
    assert_eq!(ratchet_threshold("CC-002", 100), 100);
    assert_eq!(ratchet_threshold("CC-004", 50), 50);
}

#[test]
fn test_cross_crate_config_yaml_parsing() {
    let yaml = r#"
cross_crate:
  excluded_functions: [shape, dim, duration]
  excluded_crate_pairs: ["trueno:aprender"]
  min_body_lines: 5
  min_tokens: 20
  cc003_min_similarity: 0.6
"#;
    let config: PmatYamlConfig = serde_yaml_ng::from_str(yaml).unwrap();
    assert_eq!(config.cross_crate.excluded_functions.len(), 3);
    assert_eq!(config.cross_crate.excluded_crate_pairs.len(), 1);
    assert_eq!(config.cross_crate.min_body_lines, 5);
    assert_eq!(config.cross_crate.min_tokens, 20);
    assert!((config.cross_crate.cc003_min_similarity - 0.6).abs() < 0.001);
}

#[test]
fn test_parse_workspace_members_with_globs_literal() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // Create member directories
    for name in &["core", "utils"] {
        let dir = root.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();
    }

    let content = "[workspace]\nmembers = [\"core\", \"utils\"]\n";
    let members = parse_workspace_members_with_globs(content, root);
    assert_eq!(members.len(), 2);
}
