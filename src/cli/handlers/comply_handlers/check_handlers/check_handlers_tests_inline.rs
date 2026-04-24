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
}
