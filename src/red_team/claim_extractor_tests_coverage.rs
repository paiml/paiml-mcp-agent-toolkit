#[cfg(test)]
mod coverage_instrumented_tests {
    use super::*;

    // ====================================================================
    // ClaimExtractor construction
    // ====================================================================

    #[test]
    fn test_ci_extractor_new_all_patterns_populated() {
        let ext = ClaimExtractor::new();
        assert!(!ext.test_patterns.is_empty());
        assert!(!ext.documentation_patterns.is_empty());
        assert!(!ext.coverage_patterns.is_empty());
        assert!(!ext.completion_patterns.is_empty());
        assert!(!ext.migration_patterns.is_empty());
        assert!(!ext.bugfix_patterns.is_empty());
        assert!(!ext.performance_patterns.is_empty());
        assert!(!ext.security_patterns.is_empty());
        assert!(!ext.absolute_keywords.is_empty());
        assert!(!ext.scope_patterns.is_empty());
    }

    #[test]
    fn test_ci_extractor_default_equals_new() {
        let d = ClaimExtractor::default();
        let n = ClaimExtractor::new();
        assert_eq!(d.test_patterns.len(), n.test_patterns.len());
        assert_eq!(d.absolute_keywords.len(), n.absolute_keywords.len());
    }

    // ====================================================================
    // Category: TestStatus
    // ====================================================================

    #[test]
    fn test_ci_test_status_every_test_succeed() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract("every test succeeds in CI");
        let test_claims: Vec<_> = claims
            .iter()
            .filter(|c| c.category == ClaimCategory::TestStatus)
            .collect();
        assert!(!test_claims.is_empty());
        assert!(test_claims[0].is_absolute); // "every" is absolute
    }

    #[test]
    fn test_ci_test_status_complete_test_coverage() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract("complete test coverage achieved");
        assert!(claims
            .iter()
            .any(|c| c.category == ClaimCategory::TestStatus));
    }

    // ====================================================================
    // Category: Documentation
    // ====================================================================

    #[test]
    fn test_ci_documentation_all_examples_work() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract("all examples work now");
        let doc_claims: Vec<_> = claims
            .iter()
            .filter(|c| c.category == ClaimCategory::Documentation)
            .collect();
        assert!(!doc_claims.is_empty());
        assert!(doc_claims[0].is_absolute); // "all" is absolute
    }

    #[test]
    fn test_ci_documentation_fixed_broken_docs() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract("fixed broken docs");
        assert!(claims
            .iter()
            .any(|c| c.category == ClaimCategory::Documentation));
    }

    // ====================================================================
    // Category: Coverage
    // ====================================================================

    #[test]
    fn test_ci_coverage_percentage_format() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract("92% coverage reached");
        let cov = claims
            .iter()
            .find(|c| c.category == ClaimCategory::Coverage);
        assert!(cov.is_some());
        assert_eq!(cov.unwrap().numeric_value, Some(92.0));
    }

    // ====================================================================
    // Category: Migration
    // ====================================================================

    #[test]
    fn test_ci_migration_deprecated_removed() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract("deprecated callbacks removed");
        let mig = claims
            .iter()
            .find(|c| c.category == ClaimCategory::Migration);
        assert!(mig.is_some());
    }

    // ====================================================================
    // Category: FeatureCompletion
    // ====================================================================

    #[test]
    fn test_ci_feature_completion_complete_keyword() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract("complete refactoring of parser");
        // Should find FeatureCompletion (may also find Migration if "complete" overlaps)
        assert!(claims
            .iter()
            .any(|c| c.category == ClaimCategory::FeatureCompletion
                || c.category == ClaimCategory::Migration));
    }

    // ====================================================================
    // Category: BugFix
    // ====================================================================

    #[test]
    fn test_ci_bugfix_resolved_issue_hash() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract("resolved #77");
        let bf = claims.iter().find(|c| c.category == ClaimCategory::BugFix);
        assert!(bf.is_some());
        assert_eq!(bf.unwrap().issue_number, Some(77));
    }

    #[test]
    fn test_ci_bugfix_bug_fixed_no_number() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract("bug fixed in parser");
        let bf = claims.iter().find(|c| c.category == ClaimCategory::BugFix);
        assert!(bf.is_some());
    }

    // ====================================================================
    // Category: Performance
    // ====================================================================

    #[test]
    fn test_ci_performance_reduced_memory() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract("reduced memory by 30%");
        let perf = claims
            .iter()
            .find(|c| c.category == ClaimCategory::Performance);
        assert!(perf.is_some());
        assert_eq!(perf.unwrap().numeric_value, Some(30.0));
    }

    // ====================================================================
    // Category: Security
    // ====================================================================

    #[test]
    fn test_ci_security_all_deps_updated() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract("all deps updated");
        let sec = claims
            .iter()
            .find(|c| c.category == ClaimCategory::Security);
        assert!(sec.is_some());
        assert!(sec.unwrap().is_absolute); // "all"
    }

    // ====================================================================
    // Helper methods
    // ====================================================================

    #[test]
    fn test_ci_is_absolute_entirely() {
        let ext = ClaimExtractor::new();
        assert!(ext.is_absolute_claim("entirely rewritten"));
        assert!(!ext.is_absolute_claim("mostly done"));
    }

    #[test]
    fn test_ci_extract_numeric_value_none() {
        let ext = ClaimExtractor::new();
        assert!(ext.extract_numeric_value("no digits here").is_none());
    }

    #[test]
    fn test_ci_extract_numeric_value_first_number() {
        let ext = ClaimExtractor::new();
        assert_eq!(ext.extract_numeric_value("upgraded to v3"), Some(3.0));
    }

    #[test]
    fn test_ci_scope_alpha_beta() {
        let ext = ClaimExtractor::new();
        assert!(ext.has_scope_qualifier("Alpha release"));
        assert!(ext.has_scope_qualifier("Beta version"));
        let scope = ext.extract_scope("all tests pass (Beta launch)");
        assert!(scope.is_some());
        assert!(scope.unwrap().contains("Beta"));
    }

    // ====================================================================
    // Edge cases
    // ====================================================================

    #[test]
    fn test_ci_multiple_categories_in_one_message() {
        let ext = ClaimExtractor::new();
        let claims = ext.extract(
            "all tests passing, 95% coverage, fixed bug #10, performance optimized, zero vulnerabilities"
        );
        // Should extract at least 4 distinct categories
        let cats: std::collections::HashSet<_> =
            claims.iter().map(|c| format!("{:?}", c.category)).collect();
        assert!(cats.len() >= 4, "Expected >=4 categories, got {:?}", cats);
    }

    #[test]
    fn test_ci_claim_serialization_roundtrip() {
        let claim = Claim {
            category: ClaimCategory::Performance,
            text: "50% faster".to_string(),
            is_absolute: false,
            numeric_value: Some(50.0),
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };
        let json = serde_json::to_string(&claim).unwrap();
        let deserialized: Claim = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.category, ClaimCategory::Performance);
        assert_eq!(deserialized.numeric_value, Some(50.0));
    }

    #[test]
    fn test_ci_category_all_variants_deserialize() {
        for variant in [
            "\"TestStatus\"",
            "\"Documentation\"",
            "\"Coverage\"",
            "\"FeatureCompletion\"",
            "\"Migration\"",
            "\"BugFix\"",
            "\"Performance\"",
            "\"Security\"",
        ] {
            let cat: ClaimCategory = serde_json::from_str(variant).unwrap();
            let _ = format!("{:?}", cat); // exercises Debug
        }
    }
}
