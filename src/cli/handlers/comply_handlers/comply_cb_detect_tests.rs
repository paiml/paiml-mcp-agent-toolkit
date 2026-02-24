
// =============================================================================
// TESTS
// =============================================================================

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_is_test_path() {
        assert!(is_test_path("src/tests/mod.rs"));
        assert!(is_test_path("src/foo_test.rs"));
        assert!(is_test_path("tests/integration.rs"));
        assert!(!is_test_path("src/lib.rs"));
        assert!(!is_test_path("src/services/context.rs"));
        assert!(!is_test_path(""));
    }

    #[test]
    fn test_is_marker_function() {
        assert!(is_marker_function("marker"));
        assert!(is_marker_function("type_marker"));
        assert!(is_marker_function("phantom_data"));
        assert!(is_marker_function("no_op"));
        assert!(is_marker_function("noop"));
        assert!(!is_marker_function("process"));
        assert!(!is_marker_function("handle_request"));
        assert!(!is_marker_function("marketer")); // not a marker
    }

    #[test]
    fn test_comment_detection() {
        // Rust comments
        assert!(COMMENT_PATTERN.is_match("// this is a comment"));
        assert!(COMMENT_PATTERN.is_match("/// doc comment"));
        assert!(COMMENT_PATTERN.is_match("//! inner doc"));
        assert!(COMMENT_PATTERN.is_match("   // indented comment"));
        assert!(!COMMENT_PATTERN.is_match("let x = 42; // inline"));

        // Python comments (separate pattern)
        assert!(PYTHON_COMMENT_PATTERN.is_match("# python comment"));
        assert!(!PYTHON_COMMENT_PATTERN.is_match("#[test]")); // attribute, not comment
    }

    // ========================================================================
    // WILD TESTS: Self-scan false positive detection
    // ========================================================================

    #[test]
    fn wild_string_literal_false_positive() {
        // From src/qdd/refactor.rs:190 - todo! inside string literal
        let code = r#"let result = code.replace("todo!(", "Ok(Default::default())");"#;
        let violations = detect_cb050_code_stubs_in_str(code);
        assert!(
            violations.is_empty(),
            "FALSE POSITIVE: Detected todo! in string literal: {:?}",
            violations
        );
    }

    #[test]
    fn wild_test_fixture_string_literal() {
        // From src/services/quality_proxy.rs - test fixture string
        let code = r#"content: Some("fn stub() { unimplemented!() }".to_string()),"#;
        let violations = detect_cb050_code_stubs_in_str(code);
        assert!(
            violations.is_empty(),
            "FALSE POSITIVE: Detected unimplemented! in test fixture string: {:?}",
            violations
        );
    }

    #[test]
    fn wild_actual_stub_detected() {
        // From src/services/languages/wasm.rs - actual stub
        let code = r#"
    fn _extract_wasm_functions(&mut self, _parser: &Parser) -> Result<(), String> {
        // TO BE IMPLEMENTED
        todo!("Extract function information from WASM module")
    }
"#;
        let violations = detect_cb050_code_stubs_in_str(code);
        assert!(
            !violations.is_empty(),
            "MISSED TRUE POSITIVE: Should detect todo!() in actual stub"
        );
        assert!(
            violations.iter().any(|(_, id, _)| *id == "CB-050-A"),
            "Wrong pattern ID for todo!()"
        );
    }

    #[test]
    fn wild_code_generation_string() {
        // From src/qdd/generator_ast.rs:75 - code generation string
        let code = r#"code.push_str("    todo!(\"Implementation needed\")\n");"#;
        let violations = detect_cb050_code_stubs_in_str(code);
        assert!(
            violations.is_empty(),
            "FALSE POSITIVE: Detected todo! in code generation string: {:?}",
            violations
        );
    }

    #[test]
    fn wild_string_pattern_matching() {
        // From src/qdd/refactor.rs:299 - pattern matching for todo!
        let code = r#"let todo_count = code.matches("todo!").count() as u32;"#;
        let violations = detect_cb050_code_stubs_in_str(code);
        assert!(
            violations.is_empty(),
            "FALSE POSITIVE: Detected todo! in pattern matching: {:?}",
            violations
        );
    }
}
