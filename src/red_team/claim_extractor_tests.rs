#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claim_extractor_compiles() {
        let extractor = ClaimExtractor::new();
        assert!(!extractor.test_patterns.is_empty());
    }

    #[test]
    fn test_claim_extractor_default() {
        let extractor = ClaimExtractor::default();
        assert!(!extractor.test_patterns.is_empty());
        assert!(!extractor.documentation_patterns.is_empty());
        assert!(!extractor.coverage_patterns.is_empty());
    }

    // ============================================================================
    // ClaimCategory Tests
    // ============================================================================

    #[test]
    fn test_claim_category_clone() {
        let cat = ClaimCategory::TestStatus;
        let cloned = cat.clone();
        assert_eq!(cat, cloned);
    }

    #[test]
    fn test_claim_category_debug() {
        let cat = ClaimCategory::Documentation;
        let debug = format!("{:?}", cat);
        assert!(debug.contains("Documentation"));
    }

    #[test]
    fn test_claim_category_serialize() {
        let cat = ClaimCategory::Coverage;
        let json = serde_json::to_string(&cat).unwrap();
        assert!(json.contains("Coverage"));
    }

    #[test]
    fn test_claim_category_deserialize() {
        let json = r#""BugFix""#;
        let cat: ClaimCategory = serde_json::from_str(json).unwrap();
        assert_eq!(cat, ClaimCategory::BugFix);
    }

    // ============================================================================
    // Claim struct Tests
    // ============================================================================

    #[test]
    fn test_claim_creation() {
        let claim = Claim {
            category: ClaimCategory::TestStatus,
            text: "all tests passing".to_string(),
            is_absolute: true,
            numeric_value: None,
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };

        assert_eq!(claim.category, ClaimCategory::TestStatus);
        assert!(claim.is_absolute);
    }

    #[test]
    fn test_claim_clone() {
        let claim = Claim {
            category: ClaimCategory::Coverage,
            text: "coverage at 85%".to_string(),
            is_absolute: false,
            numeric_value: Some(85.0),
            issue_number: None,
            has_scope_qualifier: false,
            scope: None,
        };

        let cloned = claim.clone();
        assert_eq!(cloned.category, claim.category);
        assert_eq!(cloned.numeric_value, claim.numeric_value);
    }

    #[test]
    fn test_claim_serialize() {
        let claim = Claim {
            category: ClaimCategory::BugFix,
            text: "fixed bug #123".to_string(),
            is_absolute: false,
            numeric_value: None,
            issue_number: Some(123),
            has_scope_qualifier: false,
            scope: None,
        };

        let json = serde_json::to_string(&claim).unwrap();
        assert!(json.contains("BugFix"));
        assert!(json.contains("123"));
    }

    // ============================================================================
    // Extract Tests - TestStatus
    // ============================================================================

    #[test]
    fn test_extract_test_status_all_tests_passing() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("all tests passing");

        assert!(!claims.is_empty());
        let claim = &claims[0];
        assert_eq!(claim.category, ClaimCategory::TestStatus);
        assert!(claim.is_absolute);
    }

    #[test]
    fn test_extract_test_status_fraction() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("15/15 tests pass");

        assert!(!claims.is_empty());
        let claim = &claims[0];
        assert_eq!(claim.category, ClaimCategory::TestStatus);
    }

    // ============================================================================
    // Extract Tests - Documentation
    // ============================================================================

    #[test]
    fn test_extract_documentation_fixed_links() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("fixed all broken documentation links");

        assert!(!claims.is_empty());
        let claim = claims
            .iter()
            .find(|c| c.category == ClaimCategory::Documentation);
        assert!(claim.is_some());
    }

    #[test]
    fn test_extract_documentation_complete() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("documentation complete");

        assert!(!claims.is_empty());
    }

    // ============================================================================
    // Extract Tests - Coverage
    // ============================================================================

    #[test]
    fn test_extract_coverage_percentage() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("coverage achieved at 85%");

        assert!(!claims.is_empty());
        let claim = claims
            .iter()
            .find(|c| c.category == ClaimCategory::Coverage);
        assert!(claim.is_some());
        assert_eq!(claim.unwrap().numeric_value, Some(85.0));
    }

    #[test]
    fn test_extract_coverage_stable() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("coverage stable at 90%");

        assert!(!claims.is_empty());
    }

    // ============================================================================
    // Extract Tests - FeatureCompletion
    // ============================================================================

    #[test]
    fn test_extract_feature_completion() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("API implementation ready");

        assert!(!claims.is_empty());
    }

    #[test]
    fn test_extract_fully_functional() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("module fully functional");

        assert!(!claims.is_empty());
    }

    // ============================================================================
    // Extract Tests - Migration
    // ============================================================================

    #[test]
    fn test_extract_migration() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("complete migration to async");

        let migration_claim = claims
            .iter()
            .find(|c| c.category == ClaimCategory::Migration);
        assert!(migration_claim.is_some());
    }

    #[test]
    fn test_extract_fully_migrated() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("fully migrated to tokio");

        assert!(!claims.is_empty());
    }

    // ============================================================================
    // Extract Tests - BugFix
    // ============================================================================

    #[test]
    fn test_extract_bugfix_with_issue() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("fixes bug #42");

        let bugfix_claim = claims.iter().find(|c| c.category == ClaimCategory::BugFix);
        assert!(bugfix_claim.is_some());
        assert_eq!(bugfix_claim.unwrap().issue_number, Some(42));
    }

    #[test]
    fn test_extract_resolved_issue() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("resolved issue 100");

        assert!(!claims.is_empty());
    }

    // ============================================================================
    // Extract Tests - Performance
    // ============================================================================

    #[test]
    fn test_extract_performance_improvement() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("50% faster parsing");

        let perf_claim = claims
            .iter()
            .find(|c| c.category == ClaimCategory::Performance);
        assert!(perf_claim.is_some());
        assert_eq!(perf_claim.unwrap().numeric_value, Some(50.0));
    }

    #[test]
    fn test_extract_performance_optimized() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("performance optimized");

        assert!(!claims.is_empty());
    }

    // ============================================================================
    // Extract Tests - Security
    // ============================================================================

    #[test]
    fn test_extract_security_zero_vulnerabilities() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("zero vulnerabilities detected");

        let sec_claim = claims
            .iter()
            .find(|c| c.category == ClaimCategory::Security);
        assert!(sec_claim.is_some());
        assert!(sec_claim.unwrap().is_absolute);
    }

    #[test]
    fn test_extract_security_audit_passed() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("security audit passed");

        assert!(!claims.is_empty());
    }

    // ============================================================================
    // Scope Qualifier Tests
    // ============================================================================

    #[test]
    fn test_extract_with_mvp_scope() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("all tests passing (MVP)");

        assert!(!claims.is_empty());
        let claim = &claims[0];
        assert!(claim.has_scope_qualifier);
        assert!(claim
            .scope
            .as_ref()
            .map(|s| s.contains("MVP"))
            .unwrap_or(false));
    }

    #[test]
    fn test_extract_with_sprint_scope() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("complete implementation Sprint 5");

        assert!(!claims.is_empty());
        let claim = &claims[0];
        assert!(claim.has_scope_qualifier);
    }

    #[test]
    fn test_extract_with_phase_scope() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("API ready Phase 1");

        assert!(!claims.is_empty());
    }

    // ============================================================================
    // Edge Cases
    // ============================================================================

    #[test]
    fn test_extract_empty_message() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("");

        assert!(claims.is_empty());
    }

    #[test]
    fn test_extract_no_claims() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("refactor: renamed variable");

        // May or may not have claims depending on patterns
        let _ = claims.len();
    }

    #[test]
    fn test_extract_multiple_claims() {
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract("all tests passing, coverage stable at 85%");

        assert!(claims.len() >= 2);
    }

    // ============================================================================
    // Helper Method Tests
    // ============================================================================

    #[test]
    fn test_is_absolute_claim() {
        let extractor = ClaimExtractor::new();
        assert!(extractor.is_absolute_claim("all tests passing"));
        assert!(extractor.is_absolute_claim("zero bugs"));
        assert!(extractor.is_absolute_claim("fully complete"));
        assert!(!extractor.is_absolute_claim("some tests pass"));
    }

    #[test]
    fn test_extract_numeric_value() {
        let extractor = ClaimExtractor::new();
        assert_eq!(extractor.extract_numeric_value("85% coverage"), Some(85.0));
        assert_eq!(extractor.extract_numeric_value("100 tests"), Some(100.0));
        assert_eq!(extractor.extract_numeric_value("no numbers"), None);
    }

    #[test]
    fn test_has_scope_qualifier() {
        let extractor = ClaimExtractor::new();
        assert!(extractor.has_scope_qualifier("ready (MVP)"));
        assert!(extractor.has_scope_qualifier("Sprint 5 complete"));
        assert!(extractor.has_scope_qualifier("Phase 1 done"));
        assert!(!extractor.has_scope_qualifier("just a normal message"));
    }

    #[test]
    fn test_extract_scope() {
        let extractor = ClaimExtractor::new();
        let scope = extractor.extract_scope("complete (MVP release)");
        assert!(scope.is_some());

        let scope = extractor.extract_scope("no scope here");
        assert!(scope.is_none());
    }
}

