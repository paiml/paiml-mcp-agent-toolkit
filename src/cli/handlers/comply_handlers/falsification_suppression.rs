#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod suppression_falsification {
    use super::super::comply_cb_detect::{SuppressionConfig, SuppressionRule};

    // Tests 071-085: Verify suppression rule matching

    #[test]
    fn tp_071_glob_pattern_matches() {
        // examples/** should match examples/demo.rs
        let mut config = SuppressionConfig::new();
        config.add_rule(SuppressionRule {
            check_ids: vec![],
            glob_pattern: Some("examples/**".to_string()),
            file: None,
            lines: None,
            expires: None,
            reason: "Example code".to_string(),
        });

        let result = config.should_suppress("CB-050-A", "examples/demo.rs", 10);
        assert!(
            result.suppressed,
            "FALSIFIED: examples/** should match examples/demo.rs"
        );
    }

    #[test]
    fn tn_072_glob_pattern_no_match() {
        // examples/** should NOT match src/lib.rs
        let mut config = SuppressionConfig::new();
        config.add_rule(SuppressionRule {
            check_ids: vec![],
            glob_pattern: Some("examples/**".to_string()),
            file: None,
            lines: None,
            expires: None,
            reason: "Example code".to_string(),
        });

        let result = config.should_suppress("CB-050-A", "src/lib.rs", 10);
        assert!(
            !result.suppressed,
            "FALSIFIED (FP): examples/** should NOT match src/lib.rs"
        );
    }

    #[test]
    fn tp_073_specific_file_matches() {
        // file = "src/lib.rs" matches src/lib.rs
        let mut config = SuppressionConfig::new();
        config.add_rule(SuppressionRule {
            check_ids: vec![],
            glob_pattern: None,
            file: Some("src/lib.rs".to_string()),
            lines: None,
            expires: None,
            reason: "Known issue".to_string(),
        });

        let result = config.should_suppress("CB-050-A", "src/lib.rs", 10);
        assert!(result.suppressed, "FALSIFIED: specific file should match");
    }

    #[test]
    fn tp_074_specific_line_matches() {
        // lines = [42] matches line 42
        let mut config = SuppressionConfig::new();
        config.add_rule(SuppressionRule {
            check_ids: vec![],
            glob_pattern: None,
            file: Some("src/lib.rs".to_string()),
            lines: Some(vec![42]),
            expires: None,
            reason: "Intentional stub".to_string(),
        });

        let result = config.should_suppress("CB-050-A", "src/lib.rs", 42);
        assert!(result.suppressed, "FALSIFIED: line 42 should match");
    }

    #[test]
    fn tn_075_specific_line_no_match() {
        // lines = [42] does NOT match line 43
        let mut config = SuppressionConfig::new();
        config.add_rule(SuppressionRule {
            check_ids: vec![],
            glob_pattern: None,
            file: Some("src/lib.rs".to_string()),
            lines: Some(vec![42]),
            expires: None,
            reason: "Intentional stub".to_string(),
        });

        let result = config.should_suppress("CB-050-A", "src/lib.rs", 43);
        assert!(
            !result.suppressed,
            "FALSIFIED (FP): line 43 should NOT match when only 42 is specified"
        );
    }

    #[test]
    fn tp_076_expired_suppression_ignored() {
        // expires = "2020-01-01" should not suppress in 2026
        let mut config = SuppressionConfig::new();
        config.add_rule(SuppressionRule {
            check_ids: vec![],
            glob_pattern: None,
            file: Some("src/lib.rs".to_string()),
            lines: None,
            expires: Some("2020-01-01".to_string()),
            reason: "Temporary".to_string(),
        });

        let result = config.should_suppress("CB-050-A", "src/lib.rs", 10);
        assert!(
            !result.suppressed,
            "FALSIFIED: expired suppression should be ignored"
        );
    }

    #[test]
    fn tp_077_future_expiry_still_active() {
        // expires = "2030-01-01" should still suppress
        let mut config = SuppressionConfig::new();
        config.add_rule(SuppressionRule {
            check_ids: vec![],
            glob_pattern: None,
            file: Some("src/lib.rs".to_string()),
            lines: None,
            expires: Some("2030-01-01".to_string()),
            reason: "Future expiry".to_string(),
        });

        let result = config.should_suppress("CB-050-A", "src/lib.rs", 10);
        assert!(
            result.suppressed,
            "FALSIFIED: future expiry should still suppress"
        );
    }

    #[test]
    fn tp_078_no_expiry_always_active() {
        // Missing expires field = never expires
        let mut config = SuppressionConfig::new();
        config.add_rule(SuppressionRule {
            check_ids: vec![],
            glob_pattern: None,
            file: Some("src/lib.rs".to_string()),
            lines: None,
            expires: None, // No expiry
            reason: "Permanent".to_string(),
        });

        let result = config.should_suppress("CB-050-A", "src/lib.rs", 10);
        assert!(
            result.suppressed,
            "FALSIFIED: no expiry should always suppress"
        );
    }

    #[test]
    fn tp_079_multiple_rules_or_logic() {
        // Multiple rules = suppress if ANY match
        let mut config = SuppressionConfig::new();
        config.add_rule(SuppressionRule {
            check_ids: vec![],
            glob_pattern: None,
            file: Some("src/a.rs".to_string()),
            lines: None,
            expires: None,
            reason: "Rule A".to_string(),
        });
        config.add_rule(SuppressionRule {
            check_ids: vec![],
            glob_pattern: None,
            file: Some("src/b.rs".to_string()),
            lines: None,
            expires: None,
            reason: "Rule B".to_string(),
        });

        let result_a = config.should_suppress("CB-050-A", "src/a.rs", 10);
        let result_b = config.should_suppress("CB-050-A", "src/b.rs", 10);
        assert!(result_a.suppressed, "FALSIFIED: rule A should match");
        assert!(result_b.suppressed, "FALSIFIED: rule B should match");
    }

    #[test]
    fn tp_080_reason_is_preserved() {
        // should_suppress returns the reason string
        let mut config = SuppressionConfig::new();
        config.add_rule(SuppressionRule {
            check_ids: vec![],
            glob_pattern: None,
            file: Some("src/lib.rs".to_string()),
            lines: None,
            expires: None,
            reason: "Known technical debt tracked in JIRA-123".to_string(),
        });

        let result = config.should_suppress("CB-050-A", "src/lib.rs", 10);
        assert!(result.suppressed);
        assert_eq!(
            result.reason,
            Some("Known technical debt tracked in JIRA-123".to_string()),
            "FALSIFIED: reason not preserved"
        );
    }

    #[test]
    fn edge_081_empty_suppressions() {
        // Empty config = nothing suppressed
        let config = SuppressionConfig::new();
        let result = config.should_suppress("CB-050-A", "src/lib.rs", 10);
        assert!(
            !result.suppressed,
            "FALSIFIED (FP): empty config should suppress nothing"
        );
    }

    #[test]
    fn edge_082_unknown_check_id() {
        // Suppression for specific check_ids = no effect on other checks
        let mut config = SuppressionConfig::new();
        config.add_rule(SuppressionRule {
            check_ids: vec!["CB-050-A".to_string()], // Only for CB-050-A
            glob_pattern: None,
            file: Some("src/lib.rs".to_string()),
            lines: None,
            expires: None,
            reason: "Only for todo!()".to_string(),
        });

        // Should suppress CB-050-A
        let result_a = config.should_suppress("CB-050-A", "src/lib.rs", 10);
        assert!(
            result_a.suppressed,
            "FALSIFIED: CB-050-A should be suppressed"
        );

        // Should NOT suppress CB-050-B (different check)
        let result_b = config.should_suppress("CB-050-B", "src/lib.rs", 10);
        assert!(
            !result_b.suppressed,
            "FALSIFIED (FP): CB-050-B should NOT be suppressed by CB-050-A rule"
        );
    }

    #[test]
    fn edge_083_glob_double_star() {
        // **/*.rs matches deeply nested files
        let mut config = SuppressionConfig::new();
        config.add_rule(SuppressionRule {
            check_ids: vec![],
            glob_pattern: Some("**/tests/*.rs".to_string()),
            file: None,
            lines: None,
            expires: None,
            reason: "Test files".to_string(),
        });

        let result = config.should_suppress("CB-050-A", "src/services/tests/mod.rs", 10);
        assert!(
            result.suppressed,
            "FALSIFIED: **/ should match deeply nested paths"
        );
    }

    #[test]
    fn edge_084_glob_single_star() {
        // *.rs only matches root level
        let mut config = SuppressionConfig::new();
        config.add_rule(SuppressionRule {
            check_ids: vec![],
            glob_pattern: Some("*.rs".to_string()),
            file: None,
            lines: None,
            expires: None,
            reason: "Root files".to_string(),
        });

        // Should match root level
        let result_root = config.should_suppress("CB-050-A", "lib.rs", 10);
        assert!(
            result_root.suppressed,
            "FALSIFIED: *.rs should match lib.rs"
        );

        // Should NOT match nested
        let result_nested = config.should_suppress("CB-050-A", "src/lib.rs", 10);
        assert!(
            !result_nested.suppressed,
            "FALSIFIED (FP): *.rs should NOT match src/lib.rs"
        );
    }

    #[test]
    fn edge_085_windows_path_separator() {
        // Should handle both / and \ in paths
        let mut config = SuppressionConfig::new();
        config.add_rule(SuppressionRule {
            check_ids: vec![],
            glob_pattern: None,
            file: Some("src/lib.rs".to_string()),
            lines: None,
            expires: None,
            reason: "Path test".to_string(),
        });

        // Windows-style path should be normalized
        let result = config.should_suppress("CB-050-A", "src\\lib.rs", 10);
        assert!(
            result.suppressed,
            "FALSIFIED: Windows path separators should be handled"
        );
    }
}
