// Coverage tests for proof annotations handler - Part 3: SARIF, property, verification, and display tests
// This file is include\!()'d into proof_annotations_handler.rs coverage_tests module.
// Do not add module-level attributes or standalone use statements.

    // ==================== Confidence level SARIF mapping tests ====================

    #[test]
    fn test_sarif_confidence_level_mapping() {
        // Test that SARIF output correctly maps confidence levels to rule IDs and levels
        let annotations = vec![
            (
                create_test_location("test.rs", 1, 10),
                create_test_annotation(
                    ConfidenceLevel::High,
                    PropertyType::MemorySafety,
                    VerificationMethod::BorrowChecker,
                ),
            ),
            (
                create_test_location("test.rs", 11, 20),
                create_test_annotation(
                    ConfidenceLevel::Medium,
                    PropertyType::ThreadSafety,
                    VerificationMethod::StaticAnalysis {
                        tool: "test".to_string(),
                    },
                ),
            ),
            (
                create_test_location("test.rs", 21, 30),
                create_test_annotation(
                    ConfidenceLevel::Low,
                    PropertyType::Termination,
                    VerificationMethod::AbstractInterpretation,
                ),
            ),
        ];

        let elapsed = Duration::from_millis(100);
        let annotator = setup_proof_annotator(false);
        let project_path = Path::new("/test");

        let result = format_proof_annotations(
            ProofAnnotationOutputFormat::Sarif,
            &annotations,
            elapsed,
            &annotator,
            project_path,
            true,
        );

        assert!(result.is_ok());
        let content = result.unwrap();

        // Verify confidence level mappings
        assert!(content.contains("high-confidence-proof"));
        assert!(content.contains("medium-confidence-proof"));
        assert!(content.contains("low-confidence-proof"));
        assert!(content.contains("\"level\": \"none\"")); // High confidence
        assert!(content.contains("\"level\": \"note\"")); // Medium confidence
        assert!(content.contains("\"level\": \"warning\"")); // Low confidence
    }

    // ==================== Property type tests ====================

    #[test]
    fn test_format_with_functional_correctness_property() {
        let annotations = vec![(create_test_location("test.rs", 1, 10), {
            let mut ann = create_test_annotation(
                ConfidenceLevel::High,
                PropertyType::FunctionalCorrectness("spec_123".to_string()),
                VerificationMethod::FormalProof {
                    prover: "lean".to_string(),
                },
            );
            ann.specification_id = Some("spec_123".to_string());
            ann
        })];

        let elapsed = Duration::from_millis(100);
        let annotator = setup_proof_annotator(false);
        let project_path = Path::new("/test");

        let result = format_proof_annotations(
            ProofAnnotationOutputFormat::Full,
            &annotations,
            elapsed,
            &annotator,
            project_path,
            true,
        );

        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("FunctionalCorrectness"));
        assert!(content.contains("Specification ID"));
        assert!(content.contains("spec_123"));
    }

    #[test]
    fn test_format_with_resource_bounds_property() {
        let annotations = vec![(
            create_test_location("test.rs", 1, 10),
            create_test_annotation(
                ConfidenceLevel::Medium,
                PropertyType::ResourceBounds {
                    resource: "memory".to_string(),
                    bound: "O(n)".to_string(),
                },
                VerificationMethod::StaticAnalysis {
                    tool: "resource_analyzer".to_string(),
                },
            ),
        )];

        let elapsed = Duration::from_millis(100);
        let annotator = setup_proof_annotator(false);
        let project_path = Path::new("/test");

        let result = format_proof_annotations(
            ProofAnnotationOutputFormat::Json,
            &annotations,
            elapsed,
            &annotator,
            project_path,
            true,
        );

        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("ResourceBounds"));
    }

    // ==================== Verification method tests ====================

    #[test]
    fn test_format_with_model_checking_method() {
        let annotations = vec![(
            create_test_location("test.rs", 1, 10),
            create_test_annotation(
                ConfidenceLevel::High,
                PropertyType::MemorySafety,
                VerificationMethod::ModelChecking { bounded: true },
            ),
        )];

        let elapsed = Duration::from_millis(100);
        let annotator = setup_proof_annotator(false);
        let project_path = Path::new("/test");

        let result = format_proof_annotations(
            ProofAnnotationOutputFormat::Full,
            &annotations,
            elapsed,
            &annotator,
            project_path,
            true,
        );

        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("ModelChecking"));
    }

    // ==================== Annotations with assumptions tests ====================

    #[test]
    fn test_format_with_assumptions() {
        let annotations = vec![(create_test_location("test.rs", 1, 10), {
            let mut ann = create_test_annotation(
                ConfidenceLevel::Medium,
                PropertyType::MemorySafety,
                VerificationMethod::StaticAnalysis {
                    tool: "test".to_string(),
                },
            );
            ann.assumptions = vec![
                "No integer overflow".to_string(),
                "Valid input data".to_string(),
            ];
            ann
        })];

        let elapsed = Duration::from_millis(100);
        let annotator = setup_proof_annotator(false);
        let project_path = Path::new("/test");

        let result = format_proof_annotations(
            ProofAnnotationOutputFormat::Full,
            &annotations,
            elapsed,
            &annotator,
            project_path,
            true,
        );

        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("Assumptions"));
        assert!(content.contains("No integer overflow"));
        assert!(content.contains("Valid input data"));
    }

    // ==================== Multiple files tests ====================

    #[test]
    fn test_format_with_multiple_files() {
        let annotations = vec![
            (
                create_test_location("src/lib.rs", 1, 10),
                create_test_annotation(
                    ConfidenceLevel::High,
                    PropertyType::MemorySafety,
                    VerificationMethod::BorrowChecker,
                ),
            ),
            (
                create_test_location("src/main.rs", 5, 15),
                create_test_annotation(
                    ConfidenceLevel::Medium,
                    PropertyType::ThreadSafety,
                    VerificationMethod::StaticAnalysis {
                        tool: "test".to_string(),
                    },
                ),
            ),
            (
                create_test_location("tests/integration.rs", 20, 50),
                create_test_annotation(
                    ConfidenceLevel::Low,
                    PropertyType::Termination,
                    VerificationMethod::AbstractInterpretation,
                ),
            ),
        ];

        let elapsed = Duration::from_millis(100);
        let annotator = setup_proof_annotator(false);
        let project_path = Path::new("/test");

        let result = format_proof_annotations(
            ProofAnnotationOutputFormat::Full,
            &annotations,
            elapsed,
            &annotator,
            project_path,
            true,
        );

        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("lib.rs"));
        assert!(content.contains("main.rs"));
        assert!(content.contains("integration.rs"));
    }

    // ==================== Output format enum coverage tests ====================

    #[test]
    fn test_proof_annotation_output_format_display() {
        assert_eq!(ProofAnnotationOutputFormat::Summary.to_string(), "summary");
        assert_eq!(ProofAnnotationOutputFormat::Full.to_string(), "full");
        assert_eq!(ProofAnnotationOutputFormat::Json.to_string(), "json");
        assert_eq!(
            ProofAnnotationOutputFormat::Markdown.to_string(),
            "markdown"
        );
        assert_eq!(ProofAnnotationOutputFormat::Sarif.to_string(), "sarif");
    }

    #[test]
    fn test_property_type_filter_display() {
        assert_eq!(
            PropertyTypeFilter::MemorySafety.to_string(),
            "memory-safety"
        );
        assert_eq!(
            PropertyTypeFilter::ThreadSafety.to_string(),
            "thread-safety"
        );
        assert_eq!(
            PropertyTypeFilter::DataRaceFreeze.to_string(),
            "data-race-freeze"
        );
        assert_eq!(PropertyTypeFilter::Termination.to_string(), "termination");
        assert_eq!(
            PropertyTypeFilter::FunctionalCorrectness.to_string(),
            "functional-correctness"
        );
        assert_eq!(
            PropertyTypeFilter::ResourceBounds.to_string(),
            "resource-bounds"
        );
        assert_eq!(PropertyTypeFilter::All.to_string(), "all");
    }

    #[test]
    fn test_verification_method_filter_display() {
        assert_eq!(
            VerificationMethodFilter::FormalProof.to_string(),
            "formal-proof"
        );
        assert_eq!(
            VerificationMethodFilter::ModelChecking.to_string(),
            "model-checking"
        );
        assert_eq!(
            VerificationMethodFilter::StaticAnalysis.to_string(),
            "static-analysis"
        );
        assert_eq!(
            VerificationMethodFilter::AbstractInterpretation.to_string(),
            "abstract-interpretation"
        );
        assert_eq!(
            VerificationMethodFilter::BorrowChecker.to_string(),
            "borrow-checker"
        );
        assert_eq!(VerificationMethodFilter::All.to_string(), "all");
    }
