#[cfg(test)]
mod check_handlers_tests {
    use super::*;

    #[test]
    fn test_format_violation_list_empty() {
        let issues: Vec<String> = vec![];
        let result = format_violation_list(&issues);
        assert!(result.is_empty() || result.trim().is_empty());
    }

    #[test]
    fn test_format_violation_list_single() {
        let issues = vec!["CB-001: test issue".to_string()];
        let result = format_violation_list(&issues);
        assert!(result.contains("CB-001"));
    }

    #[test]
    fn test_format_violation_list_multiple() {
        let issues = vec!["CB-001: issue 1".to_string(), "CB-002: issue 2".to_string()];
        let result = format_violation_list(&issues);
        assert!(result.contains("CB-001"));
        assert!(result.contains("CB-002"));
    }

    #[test]
    fn test_check_version_currency_current() {
        let check = check_version_currency(PMAT_VERSION);
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_check_version_currency_old() {
        let check = check_version_currency("1.0.0");
        assert!(check.status == CheckStatus::Warn || check.status == CheckStatus::Fail);
    }

    // GH-271 regression: long doc-comments pushing #[contract(...)] beyond the old
    // 10-line preceding window caused CB-1203 false-positives. Window is now 25.
    #[test]
    fn test_cb1203_contract_with_long_doc_comment_passes() {
        let temp = tempfile::tempdir().unwrap();
        let contracts_dir = temp.path().join("contracts");
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&contracts_dir).unwrap();
        std::fs::create_dir_all(&src_dir).unwrap();

        std::fs::write(
            contracts_dir.join("demo.yaml"),
            "equations:\n  my_fn:\n    preconditions:\n      - x > 0\n    postconditions:\n      - result >= 0\n",
        )
        .unwrap();

        // 15+ doc-comment lines between #[contract(...)] and `pub fn`.
        let rs_source = r#"use std::path::Path;

#[provable_contracts_macros::contract("demo.yaml", equation = "my_fn")]
/// Line 1 of a long doc comment describing the function in detail.
/// Line 2 of a long doc comment describing the function in detail.
/// Line 3 of a long doc comment describing the function in detail.
/// Line 4 of a long doc comment describing the function in detail.
/// Line 5 of a long doc comment describing the function in detail.
/// Line 6 of a long doc comment describing the function in detail.
/// Line 7 of a long doc comment describing the function in detail.
/// Line 8 of a long doc comment describing the function in detail.
/// Line 9 of a long doc comment describing the function in detail.
/// Line 10 of a long doc comment describing the function in detail.
/// Line 11 of a long doc comment describing the function in detail.
/// Line 12 of a long doc comment describing the function in detail.
/// Line 13 of a long doc comment describing the function in detail.
/// Line 14 of a long doc comment describing the function in detail.
/// Line 15 of a long doc comment describing the function in detail.
pub fn my_fn(x: i32) -> i32 { x }
"#;
        std::fs::write(src_dir.join("lib.rs"), rs_source).unwrap();

        let check = check_annotation_coverage(temp.path());
        assert_eq!(
            check.status,
            CheckStatus::Pass,
            "CB-1203 should accept #[contract] above long doc comment (GH-271); got: {}",
            check.message
        );
    }

    #[test]
    fn test_cb1203_missing_contract_reports_actionable_message() {
        let temp = tempfile::tempdir().unwrap();
        let contracts_dir = temp.path().join("contracts");
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&contracts_dir).unwrap();
        std::fs::create_dir_all(&src_dir).unwrap();

        std::fs::write(
            contracts_dir.join("demo.yaml"),
            "equations:\n  bare_fn:\n    preconditions:\n      - x > 0\n    postconditions:\n      - result >= 0\n",
        )
        .unwrap();

        // No #[contract], no #[requires]/#[ensures] — should fail.
        std::fs::write(
            src_dir.join("lib.rs"),
            "pub fn bare_fn(x: i32) -> i32 { x }\n",
        )
        .unwrap();

        let check = check_annotation_coverage(temp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(
            check.message.contains("missing #[contract(...)]"),
            "expected actionable message, got: {}",
            check.message
        );
    }

    // --- CB-1350 pure-helper characterization (extracted from
    // check_differential_obligations, which is git/filesystem-bound). ---

    #[test]
    fn test_cb1350_collect_affected_bindings_matches_staged() {
        let index = serde_json::json!({
            "src/foo.rs": ["bind_a", "bind_b"],
            "src/bar.rs": ["bind_c"],
        });
        let obj = index.as_object().unwrap();
        let staged = vec!["src/foo.rs".to_string()];
        let (affected, total) = cb1350_collect_affected_bindings(obj, &staged);
        assert_eq!(total, 3, "total counts all bindings regardless of staging");
        assert_eq!(affected, vec!["bind_a".to_string(), "bind_b".to_string()]);
    }

    #[test]
    fn test_cb1350_collect_affected_bindings_object_form_and_no_match() {
        let index = serde_json::json!({
            "src/foo.rs": [{"name": "bind_obj"}, "bind_str"],
        });
        let obj = index.as_object().unwrap();
        // No staged file matches -> nothing affected, but total still counted.
        let (affected, total) =
            cb1350_collect_affected_bindings(obj, &["src/other.rs".to_string()]);
        assert!(affected.is_empty());
        assert_eq!(total, 2);
        // Substring match -> extracts both object-form and string-form names.
        let (affected2, _) = cb1350_collect_affected_bindings(obj, &["foo.rs".to_string()]);
        assert_eq!(
            affected2,
            vec!["bind_obj".to_string(), "bind_str".to_string()]
        );
    }

    #[test]
    fn test_cb1350_count_verified() {
        let verdicts = serde_json::json!({
            "bind_a": "pass",
            "bind_b": "fail",
            "bind_c": "pass",
        });
        let affected = vec![
            "bind_a".to_string(),
            "bind_b".to_string(),
            "bind_c".to_string(),
        ];
        assert_eq!(cb1350_count_verified(&affected, &verdicts), 2);
        // Empty verdicts -> nothing verified.
        assert_eq!(cb1350_count_verified(&affected, &serde_json::json!({})), 0);
    }

    // --- refresh-bindings pure-parser characterization (extracted from
    // handle_refresh_bindings, which is filesystem-bound). ---

    #[test]
    fn test_refresh_parse_binding_lines_name_and_source() {
        let content = "\
- name: my_func
  source_file: src/foo.rs
- name: other_func
  source_file: src/bar.rs
";
        let pairs = refresh_parse_binding_lines(content);
        assert_eq!(
            pairs,
            vec![
                ("src/foo.rs".to_string(), "my_func".to_string()),
                ("src/bar.rs".to_string(), "other_func".to_string()),
            ]
        );
    }

    #[test]
    fn test_refresh_parse_binding_lines_function_format_and_quotes() {
        // `function:` is the binding name (pv format); quotes are stripped.
        let content = "\
function: \"pv_func\"
source_file: 'src/pv.rs'
";
        let pairs = refresh_parse_binding_lines(content);
        assert_eq!(pairs, vec![("src/pv.rs".to_string(), "pv_func".to_string())]);
    }

    #[test]
    fn test_refresh_parse_contract_source_files() {
        let content = "\
source_file: src/contract.rs
file: src/other.rs
- src/listed.rs
ignored: value
";
        let pairs = refresh_parse_contract_source_files(content, "mycontract");
        assert_eq!(
            pairs,
            vec![
                ("src/contract.rs".to_string(), "mycontract".to_string()),
                ("src/other.rs".to_string(), "mycontract".to_string()),
                ("src/listed.rs".to_string(), "mycontract".to_string()),
            ]
        );
    }

    // --- CB-1307 pure WASM-export scanner characterization (extracted from
    // check_wasm_ffi_contracts). The heuristic exits a #[wasm_bindgen] block at
    // the first non-pub/non-comment/non-let line, so only the first method of
    // an impl is counted — this quirk is locked here. ---
    #[test]
    fn test_scan_wasm_exports() {
        let src = "\
#[wasm_bindgen]
pub struct Widget {
    val: i32,
}

#[wasm_bindgen]
impl Widget {
    /// Documented constructor.
    pub fn new(v: i32) -> Widget {
        Widget { val: v }
    }
}
";
        let c = scan_wasm_exports(src);
        // struct Widget (undocumented) + fn new (documented, no Result return).
        assert_eq!(c.total_exports, 2);
        assert_eq!(c.undocumented, 1, "the struct has no doc comment");
        assert_eq!(c.unwrap_in_export, 0);
        assert_eq!(c.no_result_return, 1, "new() returns Widget, not Result");
    }
}
