#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for services/coverage_improvement/mod.rs
//! Targets: parse_coverage_percentage, CoverageImprovementConfig, CoverageImprovementReport,
//! IterationReport, extract_file_path_from_line, extract_files_from_json,
//! parse_and_score, generate_strategy_for_type, extract_public_functions,
//! generate_proptest_module

use crate::services::coverage_improvement::{
    CoverageImprovementConfig, CoverageImprovementReport, CoverageImprovementService,
    IterationReport,
};
use std::path::PathBuf;

// ==================== CoverageImprovementConfig ====================

#[test]
fn test_config_default() {
    let config = CoverageImprovementConfig::default();
    assert_eq!(config.target_coverage, 95.0);
    assert_eq!(config.max_iterations, 10);
    assert!(!config.fast_mode);
    assert_eq!(config.mutation_threshold, 80.0);
    assert!(config.focus_patterns.is_empty());
    assert!(config.exclude_patterns.is_empty());
    assert_eq!(config.project_path, PathBuf::from("."));
}

#[test]
fn test_config_custom() {
    let config = CoverageImprovementConfig {
        project_path: PathBuf::from("/tmp/test"),
        target_coverage: 90.0,
        max_iterations: 5,
        fast_mode: true,
        mutation_threshold: 70.0,
        focus_patterns: vec!["src/**".to_string()],
        exclude_patterns: vec!["tests/**".to_string()],
    };
    assert_eq!(config.target_coverage, 90.0);
    assert!(config.fast_mode);
    assert_eq!(config.focus_patterns.len(), 1);
    assert_eq!(config.exclude_patterns.len(), 1);
}

#[test]
fn test_config_clone() {
    let config = CoverageImprovementConfig::default();
    let cloned = config.clone();
    assert_eq!(cloned.target_coverage, config.target_coverage);
    assert_eq!(cloned.max_iterations, config.max_iterations);
}

// ==================== CoverageImprovementService construction ====================

#[test]
fn test_service_new() {
    let config = CoverageImprovementConfig::default();
    let svc = CoverageImprovementService::new(config);
    let _ = svc;
}

#[test]
fn test_service_with_custom_config() {
    let config = CoverageImprovementConfig {
        project_path: PathBuf::from("/nonexistent"),
        target_coverage: 80.0,
        max_iterations: 3,
        fast_mode: true,
        mutation_threshold: 50.0,
        focus_patterns: vec![],
        exclude_patterns: vec![],
    };
    let svc = CoverageImprovementService::new(config);
    let _ = svc;
}

// ==================== parse_coverage_percentage ====================

#[test]
fn test_parse_coverage_standard_output() {
    let output = r#"
some/file.rs                     100     50    50.00%      10      5    50.00%      80     40    50.00%
TOTAL                          10000   5000    50.00%    1000    500    50.00%    8000   4000    50.00%      0      0         -
"#;
    let result = CoverageImprovementService::parse_coverage_percentage(output);
    assert!(result.is_ok());
    assert!((result.unwrap() - 50.0).abs() < 0.01);
}

#[test]
fn test_parse_coverage_high_percentage() {
    let output =
        "TOTAL   336484  79572  76.35%  23872  4996  79.07%  234886  57001  75.73%  0  0  -\n";
    let result = CoverageImprovementService::parse_coverage_percentage(output);
    assert!(result.is_ok());
    assert!((result.unwrap() - 75.73).abs() < 0.01);
}

#[test]
fn test_parse_coverage_100_percent() {
    let output = "TOTAL   1000   0  100.00%  100  0  100.00%  800  0  100.00%  0  0  -\n";
    let result = CoverageImprovementService::parse_coverage_percentage(output);
    assert!(result.is_ok());
    assert!((result.unwrap() - 100.0).abs() < 0.01);
}

#[test]
fn test_parse_coverage_zero_percent() {
    let output = "TOTAL   1000   1000  0.00%  100  100  0.00%  800  800  0.00%  0  0  -\n";
    let result = CoverageImprovementService::parse_coverage_percentage(output);
    assert!(result.is_ok());
    assert!((result.unwrap() - 0.0).abs() < 0.01);
}

#[test]
fn test_parse_coverage_no_total_line() {
    let output = "some random output without TOTAL\n";
    let result = CoverageImprovementService::parse_coverage_percentage(output);
    assert!(result.is_err());
}

#[test]
fn test_parse_coverage_empty_output() {
    let result = CoverageImprovementService::parse_coverage_percentage("");
    assert!(result.is_err());
}

#[test]
fn test_parse_coverage_total_no_percentages() {
    let output = "TOTAL   1000   500\n";
    let result = CoverageImprovementService::parse_coverage_percentage(output);
    assert!(result.is_err());
}

#[test]
fn test_parse_coverage_single_percentage() {
    let output = "TOTAL   1000   200  80.00%\n";
    let result = CoverageImprovementService::parse_coverage_percentage(output);
    assert!(result.is_ok());
    assert!((result.unwrap() - 80.0).abs() < 0.01);
}

#[test]
fn test_parse_coverage_with_leading_whitespace() {
    let output = "   TOTAL   5000   1000  80.00%  500  100  80.00%  4000  800  80.00%  0  0  -\n";
    let result = CoverageImprovementService::parse_coverage_percentage(output);
    assert!(result.is_ok());
}

// ==================== IterationReport ====================

#[test]
fn test_iteration_report_serde_roundtrip() {
    let report = IterationReport {
        iteration: 1,
        files_targeted: vec![PathBuf::from("src/main.rs"), PathBuf::from("src/lib.rs")],
        tests_generated: 5,
        coverage_gain: 2.5,
        mutation_score: 85.0,
    };
    let json = serde_json::to_string(&report).unwrap();
    let deserialized: IterationReport = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.iteration, 1);
    assert_eq!(deserialized.files_targeted.len(), 2);
    assert_eq!(deserialized.tests_generated, 5);
    assert!((deserialized.coverage_gain - 2.5).abs() < f64::EPSILON);
    assert!((deserialized.mutation_score - 85.0).abs() < f64::EPSILON);
}

#[test]
fn test_iteration_report_clone() {
    let report = IterationReport {
        iteration: 3,
        files_targeted: vec![],
        tests_generated: 0,
        coverage_gain: 0.0,
        mutation_score: 0.0,
    };
    let cloned = report.clone();
    assert_eq!(cloned.iteration, report.iteration);
}

// ==================== CoverageImprovementReport ====================

#[test]
fn test_coverage_improvement_report_serde() {
    let report = CoverageImprovementReport {
        baseline_coverage: 70.0,
        target_coverage: 95.0,
        final_coverage: 85.0,
        iterations: vec![IterationReport {
            iteration: 1,
            files_targeted: vec![PathBuf::from("a.rs")],
            tests_generated: 3,
            coverage_gain: 15.0,
            mutation_score: 90.0,
        }],
        success: false,
        stop_reason: "Max iterations reached".to_string(),
    };
    let json = serde_json::to_string(&report).unwrap();
    let back: CoverageImprovementReport = serde_json::from_str(&json).unwrap();
    assert!((back.baseline_coverage - 70.0).abs() < f64::EPSILON);
    assert!(!back.success);
    assert_eq!(back.iterations.len(), 1);
}

#[test]
fn test_coverage_improvement_report_success() {
    let report = CoverageImprovementReport {
        baseline_coverage: 90.0,
        target_coverage: 95.0,
        final_coverage: 96.0,
        iterations: vec![],
        success: true,
        stop_reason: "Already at target coverage".to_string(),
    };
    assert!(report.success);
    assert!(report.final_coverage >= report.target_coverage);
}

#[test]
fn test_coverage_improvement_report_clone() {
    let report = CoverageImprovementReport {
        baseline_coverage: 50.0,
        target_coverage: 95.0,
        final_coverage: 50.0,
        iterations: vec![],
        success: false,
        stop_reason: "test".to_string(),
    };
    let cloned = report.clone();
    assert_eq!(cloned.stop_reason, report.stop_reason);
}

// ==================== extract_file_path_from_line (via service) ====================

#[test]
fn test_extract_file_path_rs() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let path = svc.extract_file_path_from_line("  src/main.rs  42  violations");
    assert_eq!(path, Some(PathBuf::from("src/main.rs")));
}

#[test]
fn test_extract_file_path_toml() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let path = svc.extract_file_path_from_line("Cargo.toml has issues");
    assert_eq!(path, Some(PathBuf::from("Cargo.toml")));
}

#[test]
fn test_extract_file_path_md() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let path = svc.extract_file_path_from_line("README.md is outdated");
    assert_eq!(path, Some(PathBuf::from("README.md")));
}

#[test]
fn test_extract_file_path_no_match() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let path = svc.extract_file_path_from_line("no file paths here");
    assert!(path.is_none());
}

#[test]
fn test_extract_file_path_empty() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let path = svc.extract_file_path_from_line("");
    assert!(path.is_none());
}

// ==================== extract_files_from_json ====================

#[test]
fn test_extract_files_from_json_object_with_file() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let json: serde_json::Value = serde_json::json!({
        "file": "src/main.rs",
        "violations": 5
    });
    let mut scores = std::collections::HashMap::new();
    svc.extract_files_from_json(&json, &mut scores, 0.4);
    assert!(scores.contains_key(&PathBuf::from("src/main.rs")));
    assert!((scores[&PathBuf::from("src/main.rs")] - 0.4).abs() < f64::EPSILON);
}

#[test]
fn test_extract_files_from_json_nested_array() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let json: serde_json::Value = serde_json::json!([
        {"path": "a.rs"},
        {"file_path": "b.rs"},
        {"file": "c.rs"}
    ]);
    let mut scores = std::collections::HashMap::new();
    svc.extract_files_from_json(&json, &mut scores, 0.3);
    assert_eq!(scores.len(), 3);
}

#[test]
fn test_extract_files_from_json_deep_nesting() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let json: serde_json::Value = serde_json::json!({
        "results": {
            "files": [
                {"file": "deep/nested.rs"}
            ]
        }
    });
    let mut scores = std::collections::HashMap::new();
    svc.extract_files_from_json(&json, &mut scores, 1.0);
    assert!(scores.contains_key(&PathBuf::from("deep/nested.rs")));
}

#[test]
fn test_extract_files_from_json_no_files() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let json: serde_json::Value = serde_json::json!({"count": 42});
    let mut scores = std::collections::HashMap::new();
    svc.extract_files_from_json(&json, &mut scores, 0.5);
    assert!(scores.is_empty());
}

#[test]
fn test_extract_files_from_json_duplicate_accumulates() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let json: serde_json::Value = serde_json::json!([
        {"file": "same.rs"},
        {"file": "same.rs"},
        {"file": "same.rs"}
    ]);
    let mut scores = std::collections::HashMap::new();
    svc.extract_files_from_json(&json, &mut scores, 0.1);
    assert_eq!(scores.len(), 1);
    assert!((scores[&PathBuf::from("same.rs")] - 0.3).abs() < 0.001);
}

// ==================== parse_and_score ====================

#[test]
fn test_parse_and_score_json() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let input = r#"[{"file": "a.rs"}, {"file": "b.rs"}]"#;
    let mut scores = std::collections::HashMap::new();
    svc.parse_and_score(input, &mut scores, 0.5).unwrap();
    assert_eq!(scores.len(), 2);
}

#[test]
fn test_parse_and_score_text_fallback() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let input = "src/main.rs has 5 violations\nsrc/lib.rs has 3 violations\n";
    let mut scores = std::collections::HashMap::new();
    svc.parse_and_score(input, &mut scores, 0.3).unwrap();
    assert!(scores.len() >= 2);
}

#[test]
fn test_parse_and_score_empty() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let mut scores = std::collections::HashMap::new();
    svc.parse_and_score("{}", &mut scores, 0.5).unwrap();
    assert!(scores.is_empty());
}

// ==================== extract_public_functions ====================

#[test]
fn test_extract_public_functions() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let code = r#"
/// Hello.
pub fn hello() {}
fn private_fn() {}
/// World.
pub fn world(x: i32) -> bool { true }
"#;
    let syntax_tree = syn::parse_file(code).unwrap();
    let funcs = svc.extract_public_functions(&syntax_tree);
    assert_eq!(funcs.len(), 2);
}

#[test]
fn test_extract_public_functions_none() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let code = "fn private() {}\nfn also_private() {}";
    let syntax_tree = syn::parse_file(code).unwrap();
    let funcs = svc.extract_public_functions(&syntax_tree);
    assert_eq!(funcs.len(), 0);
}

#[test]
fn test_extract_public_functions_empty() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let code = "";
    let syntax_tree = syn::parse_file(code).unwrap();
    let funcs = svc.extract_public_functions(&syntax_tree);
    assert_eq!(funcs.len(), 0);
}

// ==================== generate_strategy_for_type ====================

#[test]
fn test_generate_strategy_i32() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let ty: syn::Type = syn::parse_str("i32").unwrap();
    let strategy = svc.generate_strategy_for_type(&ty);
    assert!(strategy.contains("i32"));
}

#[test]
fn test_generate_strategy_string() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let ty: syn::Type = syn::parse_str("String").unwrap();
    let strategy = svc.generate_strategy_for_type(&ty);
    assert!(strategy.contains(".*"));
}

#[test]
fn test_generate_strategy_bool() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let ty: syn::Type = syn::parse_str("bool").unwrap();
    let strategy = svc.generate_strategy_for_type(&ty);
    assert!(strategy.contains("bool"));
}

#[test]
fn test_generate_strategy_pathbuf() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let ty: syn::Type = syn::parse_str("PathBuf").unwrap();
    let strategy = svc.generate_strategy_for_type(&ty);
    assert!(!strategy.is_empty());
}

#[test]
fn test_generate_strategy_vec() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let ty: syn::Type = syn::parse_str("Vec<i32>").unwrap();
    let strategy = svc.generate_strategy_for_type(&ty);
    assert!(strategy.contains("vec"));
}

#[test]
fn test_generate_strategy_option() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let ty: syn::Type = syn::parse_str("Option<i32>").unwrap();
    let strategy = svc.generate_strategy_for_type(&ty);
    assert!(strategy.contains("option"));
}

#[test]
fn test_generate_strategy_reference() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let ty: syn::Type = syn::parse_str("&str").unwrap();
    let strategy = svc.generate_strategy_for_type(&ty);
    assert!(!strategy.is_empty());
}

#[test]
fn test_generate_strategy_all_numeric_types() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    for type_name in &[
        "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "usize", "isize", "f32", "f64",
    ] {
        let ty: syn::Type = syn::parse_str(type_name).unwrap();
        let strategy = svc.generate_strategy_for_type(&ty);
        assert!(
            strategy.contains(type_name),
            "Strategy for {type_name} should reference the type"
        );
    }
}

#[test]
fn test_generate_strategy_unknown_type() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let ty: syn::Type = syn::parse_str("MyCustomType").unwrap();
    let strategy = svc.generate_strategy_for_type(&ty);
    // Should fall back to i32
    assert!(strategy.contains("i32"));
}

// ==================== generate_proptest_module ====================

#[test]
fn test_generate_proptest_module_no_params() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let code = "pub fn hello() {}";
    let syntax_tree = syn::parse_file(code).unwrap();
    let funcs = svc.extract_public_functions(&syntax_tree);

    let module = svc
        .generate_proptest_module(&PathBuf::from("test.rs"), &funcs)
        .unwrap();
    assert!(module.contains("proptest_hello"));
    assert!(module.contains("let _result = hello()"));
}

#[test]
fn test_generate_proptest_module_with_params() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let code = "pub fn add(x: i32, y: i32) -> i32 { x + y }";
    let syntax_tree = syn::parse_file(code).unwrap();
    let funcs = svc.extract_public_functions(&syntax_tree);

    let module = svc
        .generate_proptest_module(&PathBuf::from("math.rs"), &funcs)
        .unwrap();
    assert!(module.contains("proptest_add"));
    assert!(module.contains("proptest!"));
    assert!(module.contains("x in"));
    assert!(module.contains("y in"));
}

#[test]
fn test_generate_proptest_module_empty() {
    let svc = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let module = svc
        .generate_proptest_module(&PathBuf::from("empty.rs"), &[])
        .unwrap();
    assert!(module.contains("Auto-generated"));
    // No test functions
    assert!(!module.contains("proptest_"));
}
