//! SATD Detector Tests - Part 1: Core tests and debt classification
//! Extracted for file health compliance (CB-040)

use super::*;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::fs;
    use tempfile::TempDir;

    // Helper function to create a test technical debt item
    fn create_test_debt(category: DebtCategory, severity: Severity) -> TechnicalDebt {
        TechnicalDebt {
            category,
            severity,
            text: "Test debt".to_string(),
            file: PathBuf::from("test.rs"),
            line: 42,
            column: 10,
            context_hash: [0; 16],
        }
    }

    #[test]
    fn test_debt_category_as_str() {
        assert_eq!(DebtCategory::Design.as_str(), "Design");
        assert_eq!(DebtCategory::Defect.as_str(), "Defect");
        assert_eq!(DebtCategory::Requirement.as_str(), "Requirement");
        assert_eq!(DebtCategory::Test.as_str(), "Test");
        assert_eq!(DebtCategory::Performance.as_str(), "Performance");
        assert_eq!(DebtCategory::Security.as_str(), "Security");
    }

    #[test]
    fn test_debt_category_display() {
        assert_eq!(format!("{}", DebtCategory::Design), "Design");
        assert_eq!(format!("{}", DebtCategory::Defect), "Defect");
        assert_eq!(format!("{}", DebtCategory::Requirement), "Requirement");
        assert_eq!(format!("{}", DebtCategory::Test), "Test");
        assert_eq!(format!("{}", DebtCategory::Performance), "Performance");
        assert_eq!(format!("{}", DebtCategory::Security), "Security");
    }

    #[test]
    fn test_severity_escalate() {
        assert_eq!(Severity::Low.escalate(), Severity::Medium);
        assert_eq!(Severity::Medium.escalate(), Severity::High);
        assert_eq!(Severity::High.escalate(), Severity::Critical);
        assert_eq!(Severity::Critical.escalate(), Severity::Critical);
    }

    #[test]
    fn test_severity_reduce() {
        assert_eq!(Severity::Critical.reduce(), Severity::High);
        assert_eq!(Severity::High.reduce(), Severity::Medium);
        assert_eq!(Severity::Medium.reduce(), Severity::Low);
        assert_eq!(Severity::Low.reduce(), Severity::Low);
    }

    #[test]
    fn test_debt_classifier_new() {
        let classifier = DebtClassifier::new();
        assert!(!classifier.patterns.is_empty());
        // Should have at least 10 patterns based on the implementation
        assert!(classifier.patterns.len() >= 10);
    }

    #[test]
    fn test_debt_classifier_default() {
        let _classifier = DebtClassifier::default();
        // Should not panic
    }

    #[test]
    fn test_pattern_classification() {
        let classifier = DebtClassifier::new();

        // Test various patterns
        assert_eq!(
            classifier.classify_comment("// TODO: implement error handling"),
            Some((DebtCategory::Requirement, Severity::Low))
        );

        assert_eq!(
            classifier.classify_comment("// SECURITY: potential SQL injection"),
            Some((DebtCategory::Security, Severity::Critical))
        );

        assert_eq!(
            classifier.classify_comment("// FIXME: broken logic here"),
            Some((DebtCategory::Defect, Severity::High))
        );

        assert_eq!(
            classifier.classify_comment("// HACK: ugly workaround"),
            Some((DebtCategory::Design, Severity::Medium))
        );

        assert_eq!(
            classifier.classify_comment("// BUG: memory leak"),
            Some((DebtCategory::Defect, Severity::High))
        );

        assert_eq!(
            classifier.classify_comment("// KLUDGE: temporary fix"),
            Some((DebtCategory::Design, Severity::Medium))
        );

        assert_eq!(
            classifier.classify_comment("// SMELL: code duplication"),
            Some((DebtCategory::Design, Severity::Medium))
        );

        assert_eq!(
            classifier.classify_comment("// performance issue here"),
            Some((DebtCategory::Performance, Severity::Medium))
        );

        assert_eq!(
            classifier.classify_comment("// test is disabled"),
            Some((DebtCategory::Test, Severity::Medium))
        );

        assert_eq!(
            classifier.classify_comment("// technical debt: refactor needed"),
            Some((DebtCategory::Design, Severity::Medium))
        );

        assert_eq!(
            classifier.classify_comment("// code smell: long method"),
            Some((DebtCategory::Design, Severity::Medium))
        );

        assert_eq!(
            classifier.classify_comment("// workaround for library issue"),
            Some((DebtCategory::Design, Severity::Low))
        );

        assert_eq!(
            classifier.classify_comment("// optimize this loop"),
            Some((DebtCategory::Performance, Severity::Low))
        );

        assert_eq!(
            classifier.classify_comment("// slow algorithm"),
            Some((DebtCategory::Performance, Severity::Low))
        );

        // Test case insensitivity
        assert_eq!(
            classifier.classify_comment("// todo: add validation"),
            Some((DebtCategory::Requirement, Severity::Low))
        );

        assert_eq!(
            classifier.classify_comment("// VULN: XSS possible"),
            Some((DebtCategory::Security, Severity::Critical))
        );

        assert_eq!(
            classifier.classify_comment("// CVE-2021-1234: patch needed"),
            Some((DebtCategory::Security, Severity::Critical))
        );

        // Test non-matching comment
        assert_eq!(
            classifier.classify_comment("// Just a regular comment"),
            None
        );

        assert_eq!(
            classifier.classify_comment("// This is documentation"),
            None
        );
    }

    #[test]
    fn test_adjust_severity() {
        let classifier = DebtClassifier::new();

        // Test security function context
        let security_context = AstContext {
            node_type: AstNodeType::SecurityFunction,
            parent_function: "validate_input".to_string(),
            complexity: 10,
            siblings_count: 2,
            nesting_depth: 1,
            surrounding_statements: vec![],
        };
        assert_eq!(
            classifier.adjust_severity(Severity::Low, &security_context),
            Severity::Medium
        );
        assert_eq!(
            classifier.adjust_severity(Severity::High, &security_context),
            Severity::Critical
        );

        // Test data validation context
        let validation_context = AstContext {
            node_type: AstNodeType::DataValidation,
            parent_function: "check_data".to_string(),
            complexity: 5,
            siblings_count: 1,
            nesting_depth: 2,
            surrounding_statements: vec![],
        };
        assert_eq!(
            classifier.adjust_severity(Severity::Medium, &validation_context),
            Severity::High
        );

        // Test test function context
        let test_context = AstContext {
            node_type: AstNodeType::TestFunction,
            parent_function: "test_feature".to_string(),
            complexity: 3,
            siblings_count: 5,
            nesting_depth: 1,
            surrounding_statements: vec![],
        };
        assert_eq!(
            classifier.adjust_severity(Severity::High, &test_context),
            Severity::Medium
        );

        // Test mock implementation context
        let mock_context = AstContext {
            node_type: AstNodeType::MockImplementation,
            parent_function: "mock_service".to_string(),
            complexity: 2,
            siblings_count: 1,
            nesting_depth: 1,
            surrounding_statements: vec![],
        };
        assert_eq!(
            classifier.adjust_severity(Severity::Critical, &mock_context),
            Severity::High
        );

        // Test high complexity regular context
        let complex_context = AstContext {
            node_type: AstNodeType::Regular,
            parent_function: "complex_function".to_string(),
            complexity: 25,
            siblings_count: 3,
            nesting_depth: 4,
            surrounding_statements: vec![],
        };
        assert_eq!(
            classifier.adjust_severity(Severity::Low, &complex_context),
            Severity::Medium
        );

        // Test regular context with low complexity
        let simple_context = AstContext {
            node_type: AstNodeType::Regular,
            parent_function: "simple_function".to_string(),
            complexity: 5,
            siblings_count: 2,
            nesting_depth: 1,
            surrounding_statements: vec![],
        };
        assert_eq!(
            classifier.adjust_severity(Severity::Medium, &simple_context),
            Severity::Medium
        );
    }

    #[test]
    fn test_satd_detector_new() {
        let detector = SATDDetector::new();
        // Should initialize with classifier
        assert!(!detector.patterns.is_empty());
    }

    #[test]
    fn test_satd_detector_default() {
        let _detector = SATDDetector::default();
        // Should not panic
    }

    #[test]
    fn test_extract_comment_content() {
        let detector = SATDDetector::new();

        // Test Rust/C++ style comments
        assert_eq!(
            detector
                .extract_comment_content("    // TODO: fix this")
                .expect("internal error"),
            Some("TODO: fix this".to_string())
        );

        // Test Python/Shell style comments
        assert_eq!(
            detector
                .extract_comment_content("    # FIXME: broken")
                .expect("internal error"),
            Some("FIXME: broken".to_string())
        );

        // Test multi-line comment style
        assert_eq!(
            detector
                .extract_comment_content("/* TODO: implement */")
                .expect("internal error"),
            Some("TODO: implement".to_string())
        );

        // Test HTML/XML comments
        assert_eq!(
            detector
                .extract_comment_content("<!-- HACK: workaround -->")
                .expect("internal error"),
            Some("HACK: workaround".to_string())
        );

        // Test no comment
        assert_eq!(
            detector
                .extract_comment_content("let x = 42;")
                .expect("internal error"),
            None
        );

        // Test empty line
        assert_eq!(
            detector
                .extract_comment_content("")
                .expect("internal error"),
            None
        );

        // Test line with only whitespace
        assert_eq!(
            detector
                .extract_comment_content("    ")
                .expect("internal error"),
            None
        );

        // Test very long line (should return error)
        let long_line = "a".repeat(11000);
        assert!(detector.extract_comment_content(&long_line).is_err());
    }

    #[test]
    fn test_find_comment_column() {
        let detector = SATDDetector::new();

        assert_eq!(detector.find_comment_column("    // comment"), 5);
        assert_eq!(detector.find_comment_column("# python comment"), 1);
        assert_eq!(detector.find_comment_column("code; /* comment */"), 7);
        assert_eq!(detector.find_comment_column("<!-- html comment -->"), 1);
        assert_eq!(detector.find_comment_column("no comment here"), 1);
    }

    #[test]
    fn test_context_hash_stability() {
        let detector = SATDDetector::new();

        let hash1 = detector.hash_context(Path::new("test.rs"), 42, "TODO: fix this");
        let hash2 = detector.hash_context(Path::new("test.rs"), 42, "TODO: fix this");

        assert_eq!(hash1, hash2, "Context hashes should be deterministic");

        let hash3 = detector.hash_context(Path::new("test.rs"), 43, "TODO: fix this");
        assert_ne!(
            hash1, hash3,
            "Different line numbers should produce different hashes"
        );

        let hash4 = detector.hash_context(Path::new("other.rs"), 42, "TODO: fix this");
        assert_ne!(
            hash1, hash4,
            "Different files should produce different hashes"
        );

        let hash5 = detector.hash_context(Path::new("test.rs"), 42, "FIXME: fix this");
        assert_ne!(
            hash1, hash5,
            "Different content should produce different hashes"
        );
    }
}
