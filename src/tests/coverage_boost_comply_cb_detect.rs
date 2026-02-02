//! Coverage boost tests for comply_cb_detect.rs
//!
//! Tests the CB-050 and CB-060 detection logic including:
//! - SATD manifestation type classification
//! - Severity escalation
//! - Suppression configuration
//! - CB-050 code stub detection (todo!(), unimplemented!(), etc.)
//! - CB-060 GPU quality detection (PTX/WGSL barrier divergence, shared memory)
//! - Pattern matching and false positive avoidance

use crate::cli::handlers::comply_handlers::comply_cb_detect::{
    classify_satd_by_pattern_id, classify_satd_manifestation, detect_cb050_code_stubs_in_str,
    detect_cb050_code_stubs_in_str_with_path, detect_ptx_barrier_divergence_in_str,
    detect_shared_memory_unbounded_in_str, detect_tiled_kernel_no_bounds_in_str,
    detect_wgsl_barrier_divergence_in_str, CbViolation, SATDManifestationType, Severity,
    SuppressionConfig, SuppressionResult, SuppressionRule,
};

// =============================================================================
// SATD Manifestation Type Tests
// =============================================================================

#[test]
fn test_satd_manifestation_type_equality() {
    assert_eq!(SATDManifestationType::Comment, SATDManifestationType::Comment);
    assert_eq!(SATDManifestationType::Code, SATDManifestationType::Code);
    assert_ne!(SATDManifestationType::Comment, SATDManifestationType::Code);
}

#[test]
fn test_satd_manifestation_type_clone() {
    let original = SATDManifestationType::Code;
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_satd_manifestation_type_copy() {
    let original = SATDManifestationType::Comment;
    let copied: SATDManifestationType = original;
    assert_eq!(original, copied);
}

#[test]
fn test_satd_manifestation_type_debug() {
    let comment = SATDManifestationType::Comment;
    let code = SATDManifestationType::Code;
    assert!(format!("{:?}", comment).contains("Comment"));
    assert!(format!("{:?}", code).contains("Code"));
}

#[test]
fn test_satd_manifestation_type_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(SATDManifestationType::Comment);
    set.insert(SATDManifestationType::Code);
    assert_eq!(set.len(), 2);
    // Re-insert should not change size
    set.insert(SATDManifestationType::Comment);
    assert_eq!(set.len(), 2);
}

// =============================================================================
// Severity Tests
// =============================================================================

#[test]
fn test_severity_equality() {
    assert_eq!(Severity::Low, Severity::Low);
    assert_eq!(Severity::Medium, Severity::Medium);
    assert_eq!(Severity::High, Severity::High);
    assert_eq!(Severity::Critical, Severity::Critical);
}

#[test]
fn test_severity_ordering() {
    assert!(Severity::Low < Severity::Medium);
    assert!(Severity::Medium < Severity::High);
    assert!(Severity::High < Severity::Critical);
    assert!(Severity::Low < Severity::Critical);
}

#[test]
fn test_severity_clone() {
    let original = Severity::High;
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_severity_copy() {
    let original = Severity::Medium;
    let copied: Severity = original;
    assert_eq!(original, copied);
}

#[test]
fn test_severity_debug() {
    assert!(format!("{:?}", Severity::Low).contains("Low"));
    assert!(format!("{:?}", Severity::Medium).contains("Medium"));
    assert!(format!("{:?}", Severity::High).contains("High"));
    assert!(format!("{:?}", Severity::Critical).contains("Critical"));
}

// =============================================================================
// SATD Severity Escalation Tests
// =============================================================================

#[test]
fn test_comment_manifestation_no_escalation() {
    let comment = SATDManifestationType::Comment;
    assert_eq!(comment.escalate_severity(Severity::Low), Severity::Low);
    assert_eq!(comment.escalate_severity(Severity::Medium), Severity::Medium);
    assert_eq!(comment.escalate_severity(Severity::High), Severity::High);
    assert_eq!(
        comment.escalate_severity(Severity::Critical),
        Severity::Critical
    );
}

#[test]
fn test_code_manifestation_escalation() {
    let code = SATDManifestationType::Code;
    assert_eq!(code.escalate_severity(Severity::Low), Severity::Medium);
    assert_eq!(code.escalate_severity(Severity::Medium), Severity::High);
    assert_eq!(code.escalate_severity(Severity::High), Severity::Critical);
    assert_eq!(code.escalate_severity(Severity::Critical), Severity::Critical);
}

// =============================================================================
// classify_satd_manifestation Tests
// =============================================================================

#[test]
fn test_classify_satd_todo_macro() {
    assert_eq!(
        classify_satd_manifestation("todo!()"),
        SATDManifestationType::Code
    );
    assert_eq!(
        classify_satd_manifestation("fn test() { todo!(); }"),
        SATDManifestationType::Code
    );
}

#[test]
fn test_classify_satd_unimplemented_macro() {
    assert_eq!(
        classify_satd_manifestation("unimplemented!()"),
        SATDManifestationType::Code
    );
    assert_eq!(
        classify_satd_manifestation("fn test() { unimplemented!(); }"),
        SATDManifestationType::Code
    );
}

#[test]
fn test_classify_satd_panic_not_implemented() {
    assert_eq!(
        classify_satd_manifestation(r#"panic!("not implemented")"#),
        SATDManifestationType::Code
    );
    assert_eq!(
        classify_satd_manifestation(r#"panic!("Not implemented yet")"#),
        SATDManifestationType::Code
    );
}

#[test]
fn test_classify_satd_python_not_implemented_error() {
    assert_eq!(
        classify_satd_manifestation("raise NotImplementedError"),
        SATDManifestationType::Code
    );
    assert_eq!(
        classify_satd_manifestation("raise NotImplementedError()"),
        SATDManifestationType::Code
    );
}

#[test]
fn test_classify_satd_function_marker() {
    assert_eq!(
        classify_satd_manifestation("fn stub() {}"),
        SATDManifestationType::Code
    );
}

#[test]
fn test_classify_satd_empty_body() {
    assert_eq!(
        classify_satd_manifestation("{}"),
        SATDManifestationType::Code
    );
    assert_eq!(
        classify_satd_manifestation("{ }"),
        SATDManifestationType::Code
    );
}

#[test]
fn test_classify_satd_comment_todo() {
    assert_eq!(
        classify_satd_manifestation("// TODO: fix this"),
        SATDManifestationType::Comment
    );
    assert_eq!(
        classify_satd_manifestation("/* FIXME: broken */"),
        SATDManifestationType::Comment
    );
}

#[test]
fn test_classify_satd_regular_comment() {
    assert_eq!(
        classify_satd_manifestation("// This is a regular comment"),
        SATDManifestationType::Comment
    );
}

// =============================================================================
// classify_satd_by_pattern_id Tests
// =============================================================================

#[test]
fn test_classify_by_pattern_id_cb050_a() {
    assert_eq!(
        classify_satd_by_pattern_id("CB-050-A"),
        SATDManifestationType::Code
    );
}

#[test]
fn test_classify_by_pattern_id_cb050_b() {
    assert_eq!(
        classify_satd_by_pattern_id("CB-050-B"),
        SATDManifestationType::Code
    );
}

#[test]
fn test_classify_by_pattern_id_cb050_c() {
    assert_eq!(
        classify_satd_by_pattern_id("CB-050-C"),
        SATDManifestationType::Code
    );
}

#[test]
fn test_classify_by_pattern_id_cb050_d() {
    assert_eq!(
        classify_satd_by_pattern_id("CB-050-D"),
        SATDManifestationType::Code
    );
}

#[test]
fn test_classify_by_pattern_id_cb050_e() {
    assert_eq!(
        classify_satd_by_pattern_id("CB-050-E"),
        SATDManifestationType::Code
    );
}

#[test]
fn test_classify_by_pattern_id_cb050_f() {
    assert_eq!(
        classify_satd_by_pattern_id("CB-050-F"),
        SATDManifestationType::Comment
    );
}

#[test]
fn test_classify_by_pattern_id_unknown() {
    assert_eq!(
        classify_satd_by_pattern_id("CB-999"),
        SATDManifestationType::Comment
    );
    assert_eq!(
        classify_satd_by_pattern_id("UNKNOWN"),
        SATDManifestationType::Comment
    );
}

// =============================================================================
// CbViolation Tests
// =============================================================================

#[test]
fn test_cb_violation_creation() {
    let violation = CbViolation {
        line: 42,
        pattern_id: "CB-050-A",
        description: "todo!() macro".to_string(),
        code_snippet: Some("todo!()".to_string()),
    };

    assert_eq!(violation.line, 42);
    assert_eq!(violation.pattern_id, "CB-050-A");
    assert!(violation.description.contains("todo!"));
    assert!(violation.code_snippet.is_some());
}

#[test]
fn test_cb_violation_equality() {
    let v1 = CbViolation {
        line: 10,
        pattern_id: "CB-050-A",
        description: "test".to_string(),
        code_snippet: None,
    };
    let v2 = CbViolation {
        line: 10,
        pattern_id: "CB-050-A",
        description: "test".to_string(),
        code_snippet: None,
    };
    let v3 = CbViolation {
        line: 20,
        pattern_id: "CB-050-A",
        description: "test".to_string(),
        code_snippet: None,
    };

    assert_eq!(v1, v2);
    assert_ne!(v1, v3);
}

#[test]
fn test_cb_violation_clone() {
    let original = CbViolation {
        line: 5,
        pattern_id: "CB-050-B",
        description: "unimplemented".to_string(),
        code_snippet: Some("code".to_string()),
    };
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_cb_violation_debug() {
    let violation = CbViolation {
        line: 1,
        pattern_id: "CB-050-A",
        description: "test".to_string(),
        code_snippet: None,
    };
    let debug_str = format!("{:?}", violation);
    assert!(debug_str.contains("CbViolation"));
    assert!(debug_str.contains("CB-050-A"));
}

// =============================================================================
// SuppressionRule Tests
// =============================================================================

#[test]
fn test_suppression_rule_creation() {
    let rule = SuppressionRule {
        check_ids: vec!["CB-050-A".to_string()],
        glob_pattern: Some("examples/**".to_string()),
        file: None,
        lines: None,
        expires: None,
        reason: "Example code".to_string(),
    };

    assert_eq!(rule.check_ids.len(), 1);
    assert!(rule.glob_pattern.is_some());
    assert!(rule.file.is_none());
    assert!(rule.lines.is_none());
    assert!(rule.expires.is_none());
}

#[test]
fn test_suppression_rule_with_all_fields() {
    let rule = SuppressionRule {
        check_ids: vec!["CB-050-A".to_string(), "CB-050-B".to_string()],
        glob_pattern: Some("**/test/**".to_string()),
        file: Some("src/lib.rs".to_string()),
        lines: Some(vec![10, 20, 30]),
        expires: Some("2027-12-31".to_string()),
        reason: "Known technical debt".to_string(),
    };

    assert_eq!(rule.check_ids.len(), 2);
    assert_eq!(rule.lines.as_ref().unwrap().len(), 3);
}

#[test]
fn test_suppression_rule_equality() {
    let r1 = SuppressionRule {
        check_ids: vec!["CB-050-A".to_string()],
        glob_pattern: None,
        file: Some("test.rs".to_string()),
        lines: None,
        expires: None,
        reason: "test".to_string(),
    };
    let r2 = r1.clone();
    assert_eq!(r1, r2);
}

// =============================================================================
// SuppressionConfig Tests
// =============================================================================

#[test]
fn test_suppression_config_new() {
    let config = SuppressionConfig::new();
    // Should be empty by default
    let result = config.should_suppress("CB-050-A", "src/lib.rs", 1);
    assert!(!result.suppressed);
}

#[test]
fn test_suppression_config_add_rule_file_exact() {
    let mut config = SuppressionConfig::new();
    config.add_rule(SuppressionRule {
        check_ids: vec!["CB-050-A".to_string()],
        glob_pattern: None,
        file: Some("src/lib.rs".to_string()),
        lines: None,
        expires: None,
        reason: "Known issue".to_string(),
    });

    // Should suppress matching file
    let result = config.should_suppress("CB-050-A", "src/lib.rs", 1);
    assert!(result.suppressed);
    assert!(result.reason.unwrap().contains("Known issue"));

    // Should not suppress different file
    let result = config.should_suppress("CB-050-A", "src/main.rs", 1);
    assert!(!result.suppressed);
}

#[test]
fn test_suppression_config_add_rule_glob() {
    let mut config = SuppressionConfig::new();
    config.add_rule(SuppressionRule {
        check_ids: vec!["CB-050-A".to_string()],
        glob_pattern: Some("examples/*.rs".to_string()),
        file: None,
        lines: None,
        expires: None,
        reason: "Example code".to_string(),
    });

    // Should suppress files matching glob
    let result = config.should_suppress("CB-050-A", "examples/demo.rs", 1);
    assert!(result.suppressed);

    // Should not suppress non-matching files
    let result = config.should_suppress("CB-050-A", "src/lib.rs", 1);
    assert!(!result.suppressed);
}

#[test]
fn test_suppression_config_line_specific() {
    let mut config = SuppressionConfig::new();
    config.add_rule(SuppressionRule {
        check_ids: vec!["CB-050-A".to_string()],
        glob_pattern: None,
        file: Some("src/lib.rs".to_string()),
        lines: Some(vec![10, 20]),
        expires: None,
        reason: "Specific lines".to_string(),
    });

    // Should suppress specific lines
    let result = config.should_suppress("CB-050-A", "src/lib.rs", 10);
    assert!(result.suppressed);

    let result = config.should_suppress("CB-050-A", "src/lib.rs", 20);
    assert!(result.suppressed);

    // Should not suppress other lines
    let result = config.should_suppress("CB-050-A", "src/lib.rs", 15);
    assert!(!result.suppressed);
}

#[test]
fn test_suppression_config_check_id_filter() {
    let mut config = SuppressionConfig::new();
    config.add_rule(SuppressionRule {
        check_ids: vec!["CB-050-A".to_string()],
        glob_pattern: None,
        file: None,
        lines: None,
        expires: None,
        reason: "All files".to_string(),
    });

    // Should suppress matching check
    let result = config.should_suppress("CB-050-A", "any.rs", 1);
    assert!(result.suppressed);

    // Should not suppress different check
    let result = config.should_suppress("CB-050-B", "any.rs", 1);
    assert!(!result.suppressed);
}

#[test]
fn test_suppression_config_expired_rule() {
    let mut config = SuppressionConfig::new();
    config.add_rule(SuppressionRule {
        check_ids: vec!["CB-050-A".to_string()],
        glob_pattern: None,
        file: None,
        lines: None,
        expires: Some("2020-01-01".to_string()), // Expired
        reason: "Old suppression".to_string(),
    });

    // Should not suppress due to expiry
    let result = config.should_suppress("CB-050-A", "any.rs", 1);
    assert!(!result.suppressed);
}

#[test]
fn test_suppression_config_future_expiry() {
    let mut config = SuppressionConfig::new();
    config.add_rule(SuppressionRule {
        check_ids: vec!["CB-050-A".to_string()],
        glob_pattern: None,
        file: None,
        lines: None,
        expires: Some("2099-12-31".to_string()), // Far future
        reason: "Future suppression".to_string(),
    });

    // Should suppress since not expired
    let result = config.should_suppress("CB-050-A", "any.rs", 1);
    assert!(result.suppressed);
}

#[test]
fn test_suppression_config_windows_path_normalization() {
    let mut config = SuppressionConfig::new();
    config.add_rule(SuppressionRule {
        check_ids: vec!["CB-050-A".to_string()],
        glob_pattern: None,
        file: Some("src/lib.rs".to_string()),
        lines: None,
        expires: None,
        reason: "test".to_string(),
    });

    // Windows-style path should be normalized
    let result = config.should_suppress("CB-050-A", "src\\lib.rs", 1);
    assert!(result.suppressed);
}

#[test]
fn test_suppression_config_empty_check_ids_matches_all() {
    let mut config = SuppressionConfig::new();
    config.add_rule(SuppressionRule {
        check_ids: vec![], // Empty = matches all
        glob_pattern: None,
        file: Some("suppress_all.rs".to_string()),
        lines: None,
        expires: None,
        reason: "Suppress all checks".to_string(),
    });

    let result = config.should_suppress("CB-050-A", "suppress_all.rs", 1);
    assert!(result.suppressed);

    let result = config.should_suppress("CB-060-A", "suppress_all.rs", 1);
    assert!(result.suppressed);
}

#[test]
fn test_suppression_result_fields() {
    let result = SuppressionResult {
        suppressed: true,
        reason: Some("Test reason".to_string()),
    };
    assert!(result.suppressed);
    assert_eq!(result.reason.unwrap(), "Test reason");
}

// =============================================================================
// CB-050 Code Stub Detection Tests
// =============================================================================

#[test]
fn test_detect_todo_macro() {
    let code = "fn test() { todo!(); }";
    let violations = detect_cb050_code_stubs_in_str(code);
    assert!(!violations.is_empty());
    assert_eq!(violations[0].1, "CB-050-A");
}

#[test]
fn test_detect_todo_with_message() {
    let code = r#"fn test() { todo!("implement this"); }"#;
    let violations = detect_cb050_code_stubs_in_str(code);
    assert!(!violations.is_empty());
    assert_eq!(violations[0].1, "CB-050-A");
}

#[test]
fn test_detect_unimplemented_macro() {
    let code = "fn test() { unimplemented!(); }";
    let violations = detect_cb050_code_stubs_in_str(code);
    assert!(!violations.is_empty());
    assert_eq!(violations[0].1, "CB-050-B");
}

#[test]
fn test_detect_panic_not_implemented() {
    let code = r#"fn test() { panic!("not implemented"); }"#;
    let violations = detect_cb050_code_stubs_in_str(code);
    assert!(!violations.is_empty());
    assert_eq!(violations[0].1, "CB-050-C");
}

#[test]
fn test_detect_panic_not_implemented_variant() {
    // The regex is case-insensitive for "not" via the pattern
    // panic!("not implemented yet") should match
    let code = r#"fn test() { panic!("not implemented yet"); }"#;
    let violations = detect_cb050_code_stubs_in_str(code);
    assert!(!violations.is_empty());
    assert_eq!(violations[0].1, "CB-050-C");
}

#[test]
fn test_detect_panic_not_implemented_caps_not_matched() {
    // Capital "Not" at start doesn't match the current regex pattern
    // This test documents the actual behavior
    let code = r#"fn test() { panic!("Not implemented"); }"#;
    let violations = detect_cb050_code_stubs_in_str(code);
    // Current pattern expects lowercase "not implemented"
    let cb050c = violations.iter().filter(|(_, id, _)| *id == "CB-050-C").count();
    assert_eq!(cb050c, 0, "Capital 'Not' doesn't match current pattern");
}

#[test]
fn test_detect_python_not_implemented_error() {
    let code = "def test():\n    raise NotImplementedError";
    let violations = detect_cb050_code_stubs_in_str(code);
    assert!(!violations.is_empty());
    assert_eq!(violations[0].1, "CB-050-E");
}

#[test]
fn test_detect_python_pass_stub() {
    let code = "def test():\n    pass # stub";
    let violations = detect_cb050_code_stubs_in_str(code);
    assert!(!violations.is_empty());
    assert_eq!(violations[0].1, "CB-050-F");
}

#[test]
fn test_detect_python_pass_todo() {
    let code = "def test():\n    pass # TODO";
    let violations = detect_cb050_code_stubs_in_str(code);
    assert!(!violations.is_empty());
    assert_eq!(violations[0].1, "CB-050-F");
}

#[test]
fn test_detect_empty_function_body() {
    let code = "fn empty_stub() {}";
    let violations = detect_cb050_code_stubs_in_str(code);
    assert!(!violations.is_empty());
    assert_eq!(violations[0].1, "CB-050-D");
}

#[test]
fn test_detect_empty_function_with_return_type() {
    let code = "fn stub() -> Result<(), Error> { }";
    let violations = detect_cb050_code_stubs_in_str(code);
    assert!(!violations.is_empty());
    assert_eq!(violations[0].1, "CB-050-D");
}

#[test]
fn test_skip_trait_default_methods() {
    let code = r#"
trait MyTrait {
    fn default_method() {}
}
"#;
    let violations = detect_cb050_code_stubs_in_str(code);
    // Should not flag trait defaults
    let cb050d = violations.iter().filter(|(_, id, _)| *id == "CB-050-D");
    assert_eq!(cb050d.count(), 0);
}

#[test]
fn test_skip_marker_functions() {
    let code = "fn marker() {}";
    let violations = detect_cb050_code_stubs_in_str(code);
    // Should not flag marker functions
    let cb050d = violations.iter().filter(|(_, id, _)| *id == "CB-050-D");
    assert_eq!(cb050d.count(), 0);
}

#[test]
fn test_skip_sentinel_function() {
    let code = "fn type_sentinel() {}";
    let violations = detect_cb050_code_stubs_in_str(code);
    let cb050d = violations.iter().filter(|(_, id, _)| *id == "CB-050-D");
    assert_eq!(cb050d.count(), 0);
}

#[test]
fn test_skip_phantom_function() {
    let code = "fn phantom_data() {}";
    let violations = detect_cb050_code_stubs_in_str(code);
    let cb050d = violations.iter().filter(|(_, id, _)| *id == "CB-050-D");
    assert_eq!(cb050d.count(), 0);
}

#[test]
fn test_skip_noop_function() {
    let code = "fn noop() {}";
    let violations = detect_cb050_code_stubs_in_str(code);
    let cb050d = violations.iter().filter(|(_, id, _)| *id == "CB-050-D");
    assert_eq!(cb050d.count(), 0);
}

#[test]
fn test_skip_dummy_function() {
    let code = "fn dummy() {}";
    let violations = detect_cb050_code_stubs_in_str(code);
    let cb050d = violations.iter().filter(|(_, id, _)| *id == "CB-050-D");
    assert_eq!(cb050d.count(), 0);
}

#[test]
fn test_skip_test_file_path() {
    let code = "fn stub() { todo!(); }";
    let violations = detect_cb050_code_stubs_in_str_with_path(code, "src/tests/mod.rs");
    // Should skip test files
    assert!(violations.is_empty());
}

#[test]
fn test_skip_tests_directory() {
    let code = "fn stub() { todo!(); }";
    let violations = detect_cb050_code_stubs_in_str_with_path(code, "tests/integration.rs");
    assert!(violations.is_empty());
}

#[test]
fn test_skip_test_suffix_file() {
    let code = "fn stub() { todo!(); }";
    let violations = detect_cb050_code_stubs_in_str_with_path(code, "src/foo_test.rs");
    assert!(violations.is_empty());
}

#[test]
fn test_detect_in_production_file() {
    let code = "fn stub() { todo!(); }";
    let violations = detect_cb050_code_stubs_in_str_with_path(code, "src/lib.rs");
    assert!(!violations.is_empty());
}

#[test]
fn test_skip_comment_lines() {
    let code = "// todo!() in comment\nfn real() {}";
    let violations = detect_cb050_code_stubs_in_str(code);
    // Should not detect todo in comment
    let cb050a = violations.iter().filter(|(_, id, _)| *id == "CB-050-A");
    assert_eq!(cb050a.count(), 0);
}

#[test]
fn test_skip_doc_comments() {
    let code = "/// Example: todo!()\nfn example() {}";
    let violations = detect_cb050_code_stubs_in_str(code);
    let cb050a = violations.iter().filter(|(_, id, _)| *id == "CB-050-A");
    assert_eq!(cb050a.count(), 0);
}

#[test]
fn test_skip_inner_doc_comments() {
    let code = "//! Module doc: todo!()\nfn test() {}";
    let violations = detect_cb050_code_stubs_in_str(code);
    let cb050a = violations.iter().filter(|(_, id, _)| *id == "CB-050-A");
    assert_eq!(cb050a.count(), 0);
}

#[test]
fn test_skip_python_comments() {
    let code = "# todo!() in python comment\ndef test(): pass";
    let violations = detect_cb050_code_stubs_in_str(code);
    let cb050a = violations.iter().filter(|(_, id, _)| *id == "CB-050-A");
    assert_eq!(cb050a.count(), 0);
}

#[test]
fn test_not_skip_rust_attributes() {
    // #[test] should not be skipped as a python comment
    let code = "#[test]\nfn test() { todo!(); }";
    let violations = detect_cb050_code_stubs_in_str(code);
    assert!(!violations.is_empty());
}

#[test]
fn test_line_numbers_correct() {
    let code = "fn line1() {}\nfn line2() { todo!(); }\nfn line3() {}";
    let violations = detect_cb050_code_stubs_in_str(code);
    assert!(!violations.is_empty());
    assert_eq!(violations[0].0, 2); // todo on line 2
}

#[test]
fn test_multiple_violations() {
    let code = r#"
fn stub1() { todo!(); }
fn stub2() { unimplemented!(); }
fn stub3() { panic!("not implemented"); }
"#;
    let violations = detect_cb050_code_stubs_in_str(code);
    assert!(violations.len() >= 3);
}

#[test]
fn test_no_false_positive_string_literal() {
    let code = r#"let s = "todo!() is not real code";"#;
    let violations = detect_cb050_code_stubs_in_str(code);
    let cb050a = violations.iter().filter(|(_, id, _)| *id == "CB-050-A");
    assert_eq!(cb050a.count(), 0);
}

#[test]
fn test_no_false_positive_code_generation() {
    let code = r#"code.push_str("todo!(\"test\")");"#;
    let violations = detect_cb050_code_stubs_in_str(code);
    let cb050a = violations.iter().filter(|(_, id, _)| *id == "CB-050-A");
    assert_eq!(cb050a.count(), 0);
}

// =============================================================================
// CB-060 PTX Barrier Divergence Tests
// =============================================================================

#[test]
fn test_ptx_barrier_divergence_detected() {
    let ptx = r#"
    @%p1 bra skip_label
    ld.shared.f32 %f1, [%r2]
    bar.sync 0
skip_label:
"#;
    let violations = detect_ptx_barrier_divergence_in_str(ptx);
    assert!(!violations.is_empty());
    assert_eq!(violations[0].1, "CB-060-A");
}

#[test]
fn test_ptx_no_divergence_no_barrier() {
    let ptx = r#"
    @%p1 bra skip_label
    ld.shared.f32 %f1, [%r2]
skip_label:
    st.global.f32 [%r3], %f1
"#;
    let violations = detect_ptx_barrier_divergence_in_str(ptx);
    // No barrier, so no divergence issue
    assert!(violations.is_empty());
}

#[test]
fn test_ptx_barrier_no_branch() {
    let ptx = r#"
    ld.shared.f32 %f1, [%r2]
    bar.sync 0
    st.global.f32 [%r3], %f1
"#;
    let violations = detect_ptx_barrier_divergence_in_str(ptx);
    // No branch, so no divergence issue
    assert!(violations.is_empty());
}

#[test]
fn test_ptx_branch_after_barrier_safe() {
    let ptx = r#"
    bar.sync 0
    @%p1 bra skip_label
skip_label:
"#;
    let violations = detect_ptx_barrier_divergence_in_str(ptx);
    // Branch AFTER barrier is safe
    assert!(violations.is_empty());
}

#[test]
fn test_ptx_skip_comments() {
    let ptx = r#"
// @%p1 bra divergent
// bar.sync 0
    ld.global.f32 %f1, [%r2]
"#;
    let violations = detect_ptx_barrier_divergence_in_str(ptx);
    assert!(violations.is_empty());
}

// =============================================================================
// CB-060 WGSL Barrier Divergence Tests
// =============================================================================

#[test]
fn test_wgsl_barrier_in_if_divergent() {
    let wgsl = r#"
fn main() {
    if (condition) {
        workgroupBarrier();
    }
}
"#;
    let violations = detect_wgsl_barrier_divergence_in_str(wgsl);
    assert!(!violations.is_empty());
    assert_eq!(violations[0].1, "CB-060-D");
}

#[test]
fn test_wgsl_barrier_outside_if_safe() {
    let wgsl = r#"
fn main() {
    if (condition) {
        // some work
    }
    workgroupBarrier();
}
"#;
    let violations = detect_wgsl_barrier_divergence_in_str(wgsl);
    assert!(violations.is_empty());
}

#[test]
fn test_wgsl_barrier_in_divergent_loop() {
    let wgsl = r#"
fn main() {
    for (var i = 0u; i < local_id.x; i++) {
        workgroupBarrier();
    }
}
"#;
    let violations = detect_wgsl_barrier_divergence_in_str(wgsl);
    assert!(!violations.is_empty());
    assert_eq!(violations[0].1, "CB-060-D");
}

#[test]
fn test_wgsl_no_barrier_safe() {
    let wgsl = r#"
fn main() {
    if (condition) {
        // no barrier
    }
}
"#;
    let violations = detect_wgsl_barrier_divergence_in_str(wgsl);
    assert!(violations.is_empty());
}

#[test]
fn test_wgsl_skip_comments() {
    let wgsl = r#"
// workgroupBarrier() in comment
fn main() {
    let x = 1;
}
"#;
    let violations = detect_wgsl_barrier_divergence_in_str(wgsl);
    assert!(violations.is_empty());
}

// =============================================================================
// CB-060 Shared Memory Unbounded Access Tests
// =============================================================================

#[test]
fn test_shared_memory_unbounded_detected() {
    let ptx = r#"
    ld.shared.f32 %f1, [%r2]
"#;
    let violations = detect_shared_memory_unbounded_in_str(ptx);
    assert!(!violations.is_empty());
    assert_eq!(violations[0].1, "CB-060-B");
}

#[test]
fn test_shared_memory_predicated_safe() {
    let ptx = r#"
    @%p1 ld.shared.f32 %f1, [%r2]
"#;
    let violations = detect_shared_memory_unbounded_in_str(ptx);
    // Predicated access is safe
    assert!(violations.is_empty());
}

#[test]
fn test_shared_memory_constant_offset_safe() {
    let ptx = r#"
    ld.shared.f32 %f1, [shared_mem + 128]
"#;
    let violations = detect_shared_memory_unbounded_in_str(ptx);
    // Constant offset is safe
    assert!(violations.is_empty());
}

#[test]
fn test_shared_memory_bounds_checked_safe() {
    let ptx = r#"
    setp.lt.u32 %p1, %r1, 256
    ld.shared.f32 %f1, [%r2]
"#;
    let violations = detect_shared_memory_unbounded_in_str(ptx);
    // Has bounds check before access
    assert!(violations.is_empty());
}

#[test]
fn test_shared_store_unbounded() {
    let ptx = r#"
    st.shared.f32 [%r2], %f1
"#;
    let violations = detect_shared_memory_unbounded_in_str(ptx);
    assert!(!violations.is_empty());
}

#[test]
fn test_shared_memory_skip_comments() {
    let ptx = r#"
// ld.shared.f32 %f1, [%r2]
    ld.global.f32 %f1, [%r2]
"#;
    let violations = detect_shared_memory_unbounded_in_str(ptx);
    assert!(violations.is_empty());
}

// =============================================================================
// CB-060 Tiled Kernel Bounds Check Tests
// =============================================================================

#[test]
fn test_tiled_kernel_no_bounds() {
    let code = r#"
fn kernel() {
    c[row * n + col] = result;
}
"#;
    let violations = detect_tiled_kernel_no_bounds_in_str(code);
    assert!(!violations.is_empty());
    assert_eq!(violations[0].1, "CB-060-C");
}

#[test]
fn test_tiled_kernel_with_proper_bounds() {
    let code = r#"
fn kernel() {
    if row < m && col < n {
        c[row * n + col] = result;
    }
}
"#;
    let violations = detect_tiled_kernel_no_bounds_in_str(code);
    // Has proper bounds check
    let cb060c: Vec<_> = violations
        .iter()
        .filter(|(_, id, desc)| *id == "CB-060-C" && desc.contains("without bounds"))
        .collect();
    assert!(cb060c.is_empty());
}

#[test]
fn test_tiled_kernel_partial_bounds() {
    let code = r#"
fn kernel() {
    if row < m {
        c[row * n + col] = result;
    }
}
"#;
    let violations = detect_tiled_kernel_no_bounds_in_str(code);
    // Should warn about partial bounds
    let partial: Vec<_> = violations
        .iter()
        .filter(|(_, _, desc)| desc.contains("Partial bounds"))
        .collect();
    assert!(!partial.is_empty());
}

#[test]
fn test_tiled_kernel_skip_string_literal() {
    let code = r#"
let kernel_src = "c[row * n + col] = result";
"#;
    let violations = detect_tiled_kernel_no_bounds_in_str(code);
    let cb060c: Vec<_> = violations
        .iter()
        .filter(|(_, id, _)| *id == "CB-060-C")
        .collect();
    assert!(cb060c.is_empty());
}

#[test]
fn test_tiled_kernel_skip_comments() {
    let code = r#"
// c[row * n + col] = result
fn kernel() {
    // actual work
}
"#;
    let violations = detect_tiled_kernel_no_bounds_in_str(code);
    let cb060c: Vec<_> = violations
        .iter()
        .filter(|(_, id, _)| *id == "CB-060-C")
        .collect();
    assert!(cb060c.is_empty());
}

#[test]
fn test_ptx_early_exit_detected() {
    let code = r#"
@%p1 bra exit
ld.shared.f32 %f1, [%r2]
bar.sync 0
exit:
"#;
    let violations = detect_tiled_kernel_no_bounds_in_str(code);
    let early_exit: Vec<_> = violations
        .iter()
        .filter(|(_, _, desc)| desc.contains("Early exit"))
        .collect();
    assert!(!early_exit.is_empty());
}

#[test]
fn test_wgsl_tiled_pattern_no_bounds() {
    let code = r#"
fn kernel() {
    a[global_id.x * size] = value;
}
"#;
    let violations = detect_tiled_kernel_no_bounds_in_str(code);
    let wgsl_tiled: Vec<_> = violations
        .iter()
        .filter(|(_, _, desc)| desc.contains("WGSL tiled"))
        .collect();
    assert!(!wgsl_tiled.is_empty());
}

// =============================================================================
// Edge Cases and Boundary Tests
// =============================================================================

#[test]
fn test_empty_input() {
    assert!(detect_cb050_code_stubs_in_str("").is_empty());
    assert!(detect_ptx_barrier_divergence_in_str("").is_empty());
    assert!(detect_wgsl_barrier_divergence_in_str("").is_empty());
    assert!(detect_shared_memory_unbounded_in_str("").is_empty());
    assert!(detect_tiled_kernel_no_bounds_in_str("").is_empty());
}

#[test]
fn test_whitespace_only() {
    assert!(detect_cb050_code_stubs_in_str("   \n\t\n  ").is_empty());
}

#[test]
fn test_empty_path() {
    let violations = detect_cb050_code_stubs_in_str_with_path("fn stub() {}", "");
    // Empty path is not a test path, so should detect violations
    assert!(!violations.is_empty());
}

#[test]
fn test_todo_with_spacing_variations() {
    // todo ! ( ) with spaces
    let code = "fn test() { todo ! ( ); }";
    let violations = detect_cb050_code_stubs_in_str(code);
    assert!(!violations.is_empty());
}

#[test]
fn test_doc_test_block_skipped() {
    let code = r#"
/// Example:
/// ```
/// todo!()
/// ```
fn example() {}
"#;
    let violations = detect_cb050_code_stubs_in_str(code);
    // Doc test blocks should be skipped
    let cb050a = violations.iter().filter(|(_, id, _)| *id == "CB-050-A");
    assert_eq!(cb050a.count(), 0);
}

#[test]
fn test_multiline_raw_string_handling() {
    // Test raw string detection behavior
    // The current implementation tracks raw strings starting with r#" and ending with "#
    // and skips lines while inside them
    let code = r###"let s = r#"
todo!()
"#;
fn normal() { let x = 1; }
"###;
    let violations = detect_cb050_code_stubs_in_str(code);
    // The todo!() is inside a raw string, should be skipped
    let cb050a = violations.iter().filter(|(_, id, _)| *id == "CB-050-A").count();
    assert_eq!(cb050a, 0, "todo!() inside raw string should be skipped");
}

#[test]
fn test_raw_string_on_same_line() {
    // When raw string starts and ends on same line, detection should work correctly
    let code = r###"let s = r#"todo!()"#;
fn stub() { todo!(); }
"###;
    let violations = detect_cb050_code_stubs_in_str(code);
    // The second todo!() is NOT in a raw string
    let cb050a: Vec<_> = violations.iter().filter(|(_, id, _)| *id == "CB-050-A").collect();
    assert!(!cb050a.is_empty(), "todo!() on line 2 should be detected");
}

#[test]
fn test_underscore_function_is_marker() {
    let code = "fn _() {}";
    let violations = detect_cb050_code_stubs_in_str(code);
    let cb050d = violations.iter().filter(|(_, id, _)| *id == "CB-050-D");
    assert_eq!(cb050d.count(), 0);
}

#[test]
fn test_marker_prefix() {
    let code = "fn marker_type() {}";
    let violations = detect_cb050_code_stubs_in_str(code);
    let cb050d = violations.iter().filter(|(_, id, _)| *id == "CB-050-D");
    assert_eq!(cb050d.count(), 0);
}

#[test]
fn test_suppression_config_default() {
    let config = SuppressionConfig::default();
    let result = config.should_suppress("CB-050-A", "test.rs", 1);
    assert!(!result.suppressed);
}

#[test]
fn test_suppression_glob_recursive() {
    let mut config = SuppressionConfig::new();
    config.add_rule(SuppressionRule {
        check_ids: vec![],
        glob_pattern: Some("**/examples/**".to_string()),
        file: None,
        lines: None,
        expires: None,
        reason: "Examples".to_string(),
    });

    let result = config.should_suppress("CB-050-A", "deep/nested/examples/test.rs", 1);
    assert!(result.suppressed);
}

#[test]
fn test_suppression_file_suffix_match() {
    let mut config = SuppressionConfig::new();
    config.add_rule(SuppressionRule {
        check_ids: vec![],
        glob_pattern: None,
        file: Some("lib.rs".to_string()), // Just the filename
        lines: None,
        expires: None,
        reason: "Match by suffix".to_string(),
    });

    // Should match because path ends with the file pattern
    let result = config.should_suppress("CB-050-A", "src/lib.rs", 1);
    assert!(result.suppressed);
}

#[test]
fn test_invalid_glob_pattern_graceful() {
    let mut config = SuppressionConfig::new();
    config.add_rule(SuppressionRule {
        check_ids: vec![],
        glob_pattern: Some("[invalid".to_string()), // Invalid glob
        file: Some("fallback.rs".to_string()),
        lines: None,
        expires: None,
        reason: "Invalid glob".to_string(),
    });

    // Should still work via file match
    let result = config.should_suppress("CB-050-A", "fallback.rs", 1);
    assert!(result.suppressed);
}
