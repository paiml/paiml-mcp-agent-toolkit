
    // EnforcementMetadata Tests

    #[test]
    fn test_enforcement_metadata_creation() {
        let metadata = EnforcementMetadata {
            enforcement_score: 8.5,
            requires_enforcement: true,
            estimated_fix_time: 1800,
            automation_confidence: 0.85,
            enforcement_priority: 3,
        };

        assert!((metadata.enforcement_score - 8.5).abs() < f64::EPSILON);
        assert!(metadata.requires_enforcement);
        assert_eq!(metadata.estimated_fix_time, 1800);
        assert!((metadata.automation_confidence - 0.85).abs() < f64::EPSILON);
        assert_eq!(metadata.enforcement_priority, 3);
    }

    #[test]
    fn test_enforcement_metadata_serialization() {
        let metadata = EnforcementMetadata {
            enforcement_score: 8.5,
            requires_enforcement: true,
            estimated_fix_time: 1800,
            automation_confidence: 0.85,
            enforcement_priority: 3,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: EnforcementMetadata = serde_json::from_str(&json).unwrap();

        assert!((deserialized.enforcement_score - metadata.enforcement_score).abs() < f64::EPSILON);
        assert_eq!(
            deserialized.requires_enforcement,
            metadata.requires_enforcement
        );
    }

    // RefactorChain and RefactorStep Tests

    #[test]
    fn test_refactor_step_creation() {
        let step = RefactorStep {
            id: "fix-unused".to_string(),
            lint: "unused_variable".to_string(),
            confidence: 0.95,
            impact: 5,
            description: "Remove unused variables".to_string(),
        };

        assert_eq!(step.id, "fix-unused");
        assert_eq!(step.lint, "unused_variable");
        assert!((step.confidence - 0.95).abs() < f64::EPSILON);
        assert_eq!(step.impact, 5);
    }

    #[test]
    fn test_refactor_chain_creation() {
        let chain = RefactorChain {
            id: "lint-hotspot-20240101-120000".to_string(),
            estimated_reduction: 15,
            automation_confidence: 0.88,
            steps: vec![
                RefactorStep {
                    id: "step-1".to_string(),
                    lint: "unused".to_string(),
                    confidence: 0.95,
                    impact: 10,
                    description: "Remove unused".to_string(),
                },
                RefactorStep {
                    id: "step-2".to_string(),
                    lint: "needless".to_string(),
                    confidence: 0.80,
                    impact: 5,
                    description: "Simplify needless".to_string(),
                },
            ],
        };

        assert_eq!(chain.steps.len(), 2);
        assert_eq!(chain.estimated_reduction, 15);
    }

    #[test]
    fn test_refactor_chain_serialization() {
        let chain = RefactorChain {
            id: "test-chain".to_string(),
            estimated_reduction: 10,
            automation_confidence: 0.9,
            steps: vec![],
        };

        let json = serde_json::to_string(&chain).unwrap();
        let deserialized: RefactorChain = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, chain.id);
        assert_eq!(deserialized.estimated_reduction, chain.estimated_reduction);
    }

    // QualityGateStatus and QualityViolation Tests

    #[test]
    fn test_quality_gate_status_passed() {
        let status = QualityGateStatus {
            passed: true,
            violations: vec![],
            blocking: false,
        };

        assert!(status.passed);
        assert!(status.violations.is_empty());
        assert!(!status.blocking);
    }

    #[test]
    fn test_quality_gate_status_failed_with_violations() {
        let status = QualityGateStatus {
            passed: false,
            violations: vec![QualityViolation {
                rule: "max_defect_density".to_string(),
                threshold: 5.0,
                actual: 10.0,
                severity: "blocking".to_string(),
            }],
            blocking: true,
        };

        assert!(!status.passed);
        assert_eq!(status.violations.len(), 1);
        assert!(status.blocking);
    }

    #[test]
    fn test_quality_violation_serialization() {
        let violation = QualityViolation {
            rule: "max_defect_density".to_string(),
            threshold: 5.0,
            actual: 10.0,
            severity: "blocking".to_string(),
        };

        let json = serde_json::to_string(&violation).unwrap();
        let deserialized: QualityViolation = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.rule, violation.rule);
        assert!((deserialized.threshold - violation.threshold).abs() < f64::EPSILON);
        assert!((deserialized.actual - violation.actual).abs() < f64::EPSILON);
    }

    // check_quality_gates Tests

    #[test]
    fn test_check_quality_gates_passes_below_threshold() {
        let hotspot = create_test_hotspot("src/main.rs", 5, 100, 2, 3);
        let status = check_quality_gates(&hotspot, 10.0);

        assert!(status.passed);
        assert!(status.violations.is_empty());
        assert!(!status.blocking);
    }

    #[test]
    fn test_check_quality_gates_fails_exceeds_density() {
        let hotspot = create_test_hotspot("src/main.rs", 100, 100, 50, 50);
        let status = check_quality_gates(&hotspot, 0.5);

        assert!(!status.passed);
        assert!(!status.violations.is_empty());
        assert!(status.blocking);

        let violation = &status.violations[0];
        assert_eq!(violation.rule, "max_defect_density");
        assert!((violation.threshold - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_check_quality_gates_warning_for_high_violations() {
        let mut hotspot = create_test_hotspot("src/main.rs", 60, 1000, 30, 30);
        hotspot.defect_density = 0.06; // Below max_density of 1.0
        hotspot.total_violations = 60;

        let status = check_quality_gates(&hotspot, 1.0);

        // Should have warning for max_single_file_violations
        let warning = status
            .violations
            .iter()
            .find(|v| v.rule == "max_single_file_violations");
        assert!(warning.is_some());
        assert_eq!(warning.unwrap().severity, "warning");
    }

    // calculate_enforcement_metadata Tests

    #[test]
    fn test_calculate_enforcement_metadata_low_density() {
        let hotspot = create_test_hotspot("src/main.rs", 5, 100, 2, 3);
        let metadata = calculate_enforcement_metadata(&hotspot, 0.7);

        assert!(metadata.enforcement_score < 10.0);
        assert!(!metadata.requires_enforcement); // Score < 7.0
        assert_eq!(metadata.estimated_fix_time, 5 * 300); // 5 violations * 300 seconds
    }

    #[test]
    fn test_calculate_enforcement_metadata_high_density() {
        let mut hotspot = create_test_hotspot("src/main.rs", 100, 100, 50, 50);
        hotspot.defect_density = 1.0;
        hotspot.top_lints = vec![("unused_variable".to_string(), 50)];

        let metadata = calculate_enforcement_metadata(&hotspot, 0.5);

        assert_eq!(metadata.enforcement_score, 10.0); // Capped at 10.0
        assert!(metadata.requires_enforcement);
        assert!((metadata.automation_confidence - 0.9).abs() < f64::EPSILON); // "unused" in lint
    }

    #[test]
    fn test_calculate_enforcement_metadata_with_redundant_lints() {
        let mut hotspot = create_test_hotspot("src/main.rs", 20, 100, 10, 10);
        hotspot.defect_density = 0.8;
        hotspot.top_lints = vec![("redundant_clone".to_string(), 10)];

        let metadata = calculate_enforcement_metadata(&hotspot, 0.7);

        assert!((metadata.automation_confidence - 0.9).abs() < f64::EPSILON); // "redundant" in lint
    }

    #[test]
    fn test_calculate_enforcement_metadata_without_auto_fixable() {
        let mut hotspot = create_test_hotspot("src/main.rs", 20, 100, 10, 10);
        hotspot.defect_density = 0.8;
        hotspot.top_lints = vec![("clippy::complexity".to_string(), 10)];

        let metadata = calculate_enforcement_metadata(&hotspot, 0.7);

        assert!((metadata.automation_confidence - 0.7).abs() < f64::EPSILON); // Default confidence
    }

    // generate_refactor_chain Tests

    #[test]
    fn test_generate_refactor_chain_with_unused_lints() {
        let mut hotspot = create_test_hotspot("src/main.rs", 10, 100, 5, 5);
        hotspot.top_lints = vec![
            ("unused_variable".to_string(), 5),
            ("unused_import".to_string(), 3),
        ];

        let chain = generate_refactor_chain(&hotspot, 0.7);

        assert_eq!(chain.steps.len(), 2);
        assert!((chain.steps[0].confidence - 0.95).abs() < f64::EPSILON);
        assert_eq!(chain.steps[0].description, "Remove unused code");
    }

    #[test]
    fn test_generate_refactor_chain_with_needless_lints() {
        let mut hotspot = create_test_hotspot("src/main.rs", 10, 100, 5, 5);
        hotspot.top_lints = vec![("needless_return".to_string(), 5)];

        let chain = generate_refactor_chain(&hotspot, 0.7);

        assert_eq!(chain.steps.len(), 1);
        assert!((chain.steps[0].confidence - 0.85).abs() < f64::EPSILON);
        assert_eq!(chain.steps[0].description, "Simplify needless patterns");
    }

    #[test]
    fn test_generate_refactor_chain_filters_by_confidence() {
        let mut hotspot = create_test_hotspot("src/main.rs", 10, 100, 5, 5);
        hotspot.top_lints = vec![
            ("unused_variable".to_string(), 5),  // confidence 0.95
            ("complex_lint".to_string(), 3),      // confidence 0.70
        ];

        let chain = generate_refactor_chain(&hotspot, 0.9);

        // Only unused_variable should pass the 0.9 threshold
        assert_eq!(chain.steps.len(), 1);
        assert!(chain.steps[0].lint.contains("unused"));
    }

    #[test]
    fn test_generate_refactor_chain_calculates_total_reduction() {
        let mut hotspot = create_test_hotspot("src/main.rs", 15, 100, 5, 10);
        hotspot.top_lints = vec![
            ("unused_variable".to_string(), 5),
            ("redundant_clone".to_string(), 3),
        ];

        let chain = generate_refactor_chain(&hotspot, 0.7);

        assert_eq!(chain.estimated_reduction, 8); // 5 + 3
    }

    // count_top_lints Tests

    #[test]
    fn test_count_top_lints_empty() {
        let violations: Vec<ViolationDetail> = vec![];
        let top_lints = count_top_lints(&violations);

        assert!(top_lints.is_empty());
    }

    #[test]
    fn test_count_top_lints_single_lint() {
        let violations = vec![
            create_test_violation("src/main.rs", 10, "unused_variable", "warning"),
            create_test_violation("src/main.rs", 20, "unused_variable", "warning"),
        ];

        let top_lints = count_top_lints(&violations);

        assert_eq!(top_lints.len(), 1);
        assert_eq!(top_lints[0].0, "unused_variable");
        assert_eq!(top_lints[0].1, 2);
    }

    #[test]
    fn test_count_top_lints_multiple_lints_sorted() {
        let violations = vec![
            create_test_violation("src/main.rs", 10, "lint_a", "warning"),
            create_test_violation("src/main.rs", 20, "lint_b", "warning"),
            create_test_violation("src/main.rs", 30, "lint_b", "warning"),
            create_test_violation("src/main.rs", 40, "lint_b", "warning"),
            create_test_violation("src/main.rs", 50, "lint_a", "warning"),
        ];

        let top_lints = count_top_lints(&violations);

        assert_eq!(top_lints.len(), 2);
        assert_eq!(top_lints[0].0, "lint_b"); // 3 occurrences
        assert_eq!(top_lints[0].1, 3);
        assert_eq!(top_lints[1].0, "lint_a"); // 2 occurrences
        assert_eq!(top_lints[1].1, 2);
    }

    #[test]
    fn test_count_top_lints_truncates_to_10() {
        let mut violations = vec![];
        for i in 0..15 {
            violations.push(create_test_violation(
                "src/main.rs",
                i as u32,
                &format!("lint_{i}"),
                "warning",
            ));
        }

        let top_lints = count_top_lints(&violations);

        assert_eq!(top_lints.len(), 10);
    }

    // update_severity_distribution Tests

    #[test]
    fn test_update_severity_distribution_error() {
        let mut dist = SeverityDistribution::default();
        update_severity_distribution(&mut dist, "error");

        assert_eq!(dist.error, 1);
        assert_eq!(dist.warning, 0);
        assert_eq!(dist.note, 0);
    }

    #[test]
    fn test_update_severity_distribution_warning() {
        let mut dist = SeverityDistribution::default();
        update_severity_distribution(&mut dist, "warning");

        assert_eq!(dist.error, 0);
        assert_eq!(dist.warning, 1);
        assert_eq!(dist.note, 0);
    }

    #[test]
    fn test_update_severity_distribution_unknown() {
        let mut dist = SeverityDistribution::default();
        update_severity_distribution(&mut dist, "unknown");

        assert_eq!(dist.error, 0);
        assert_eq!(dist.warning, 0);
        assert_eq!(dist.note, 1);
    }

    #[test]
    fn test_update_severity_distribution_multiple() {
        let mut dist = SeverityDistribution::default();
        update_severity_distribution(&mut dist, "error");
        update_severity_distribution(&mut dist, "error");
        update_severity_distribution(&mut dist, "warning");
        update_severity_distribution(&mut dist, "warning");
        update_severity_distribution(&mut dist, "warning");
        update_severity_distribution(&mut dist, "note");

        assert_eq!(dist.error, 2);
        assert_eq!(dist.warning, 3);
        assert_eq!(dist.note, 1);
    }

    // count_sloc Tests

    #[test]
    fn test_count_sloc_empty() {
        let content = "";
        assert_eq!(count_sloc(content), 0);
    }

    #[test]
    fn test_count_sloc_only_whitespace() {
        let content = "   \n\n   \n";
        assert_eq!(count_sloc(content), 0);
    }

    #[test]
    fn test_count_sloc_only_comments() {
        let content = "// comment 1\n// comment 2\n// comment 3";
        assert_eq!(count_sloc(content), 0);
    }

    #[test]
    fn test_count_sloc_mixed_content() {
        let content = r#"
fn main() {
    // This is a comment
    let x = 5;

    println!("Hello");
}
"#;
        // Should count: fn main(), let x = 5;, println!(), }
        // Empty line and comment line are excluded
        assert!(count_sloc(content) >= 3);
    }

    #[test]
    fn test_count_sloc_with_inline_comments() {
        let content = r#"let x = 5; // inline comment
let y = 10;"#;
        // Both lines have code, inline comments don't make a line "comment only"
        assert_eq!(count_sloc(content), 2);
    }

    // calculate_defect_density Tests

    #[test]
    fn test_calculate_defect_density_normal() {
        assert!((calculate_defect_density(10, 100) - 0.1).abs() < f64::EPSILON);
        assert!((calculate_defect_density(5, 50) - 0.1).abs() < f64::EPSILON);
        assert!((calculate_defect_density(0, 100) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_defect_density_zero_sloc() {
        assert!((calculate_defect_density(10, 0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_defect_density_high_density() {
        assert!((calculate_defect_density(200, 100) - 2.0).abs() < f64::EPSILON);
    }

    // calculate_total_violations Tests

    #[test]
    fn test_calculate_total_violations() {
        let metrics = FileMetrics {
            violations: HashMap::new(),
            severity_counts: SeverityDistribution {
                error: 5,
                warning: 10,
                suggestion: 3,
                note: 0,
            },
            sloc: 100,
            detailed_violations: vec![],
        };

        let total = calculate_total_violations(&metrics);
        assert_eq!(total, 18); // 5 + 10 + 3
    }

    #[test]
    fn test_calculate_total_violations_empty() {
        let metrics = FileMetrics {
            violations: HashMap::new(),
            severity_counts: SeverityDistribution::default(),
            sloc: 100,
            detailed_violations: vec![],
        };

        let total = calculate_total_violations(&metrics);
        assert_eq!(total, 0);
    }
