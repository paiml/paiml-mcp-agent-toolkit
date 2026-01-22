            make_location("test.rs", 0, 100),
            make_annotation(
                PropertyType::DataRaceFreeze,
                VerificationMethod::BorrowChecker,
                ConfidenceLevel::High,
                vec![],
                None,
            ),
        )];
        let mut output = String::new();
        let result = format_property_stats(&annotations, &mut output);

        assert!(result.is_ok());
        assert!(output.contains("DataRaceFreeze"));
    }

    #[test]
    fn test_format_property_stats_termination() {
        let annotations = vec![(
            make_location("test.rs", 0, 100),
            make_annotation(
                PropertyType::Termination,
                VerificationMethod::FormalProof {
                    prover: "Coq".to_string(),
                },
                ConfidenceLevel::High,
                vec![],
                None,
            ),
        )];
        let mut output = String::new();
        let result = format_property_stats(&annotations, &mut output);

        assert!(result.is_ok());
        assert!(output.contains("Termination"));
    }

    #[test]
    fn test_format_property_stats_functional_correctness() {
        let annotations = vec![(
            make_location("test.rs", 0, 100),
            make_annotation(
                PropertyType::FunctionalCorrectness("spec_001".to_string()),
                VerificationMethod::FormalProof {
                    prover: "Coq".to_string(),
                },
                ConfidenceLevel::High,
                vec![],
                None,
            ),
        )];
        let mut output = String::new();
        let result = format_property_stats(&annotations, &mut output);

        assert!(result.is_ok());
        assert!(output.contains("FunctionalCorrectness"));
    }

    #[test]
    fn test_format_property_stats_resource_bounds() {
        let annotations = vec![(
            make_location("test.rs", 0, 100),
            make_annotation(
                PropertyType::ResourceBounds {
                    cpu: Some(1000),
                    memory: Some(1024),
                },
                VerificationMethod::StaticAnalysis {
                    tool: "resource-analyzer".to_string(),
                },
                ConfidenceLevel::Medium,
                vec![],
                None,
            ),
        )];
        let mut output = String::new();
        let result = format_property_stats(&annotations, &mut output);

        assert!(result.is_ok());
        assert!(output.contains("ResourceBounds"));
    }

    #[test]
    fn test_format_property_stats_all_types() {
        let annotations = vec![
            (
                make_location("test.rs", 0, 100),
                make_annotation(
                    PropertyType::MemorySafety,
                    VerificationMethod::BorrowChecker,
                    ConfidenceLevel::High,
                    vec![],
                    None,
                ),
            ),
            (
                make_location("test.rs", 100, 200),
                make_annotation(
                    PropertyType::ThreadSafety,
                    VerificationMethod::BorrowChecker,
                    ConfidenceLevel::High,
                    vec![],
                    None,
                ),
            ),
            (
                make_location("test.rs", 200, 300),
                make_annotation(
                    PropertyType::DataRaceFreeze,
                    VerificationMethod::BorrowChecker,
                    ConfidenceLevel::High,
                    vec![],
                    None,
                ),
            ),
            (
                make_location("test.rs", 300, 400),
                make_annotation(
                    PropertyType::Termination,
                    VerificationMethod::FormalProof {
                        prover: "Z3".to_string(),
                    },
                    ConfidenceLevel::Medium,
                    vec![],
                    None,
                ),
            ),
        ];
        let mut output = String::new();
        let result = format_property_stats(&annotations, &mut output);

        assert!(result.is_ok());
        assert!(output.contains("Properties Proven"));
    }

    // ========== group_by_file tests ==========

    #[test]
    fn test_group_by_file_empty() {
        let annotations: Vec<(Location, ProofAnnotation)> = vec![];
        let result = group_by_file(&annotations);

        assert!(result.is_empty());
    }

    #[test]
    fn test_group_by_file_single_file() {
        let annotations = vec![
            (
                make_location("test.rs", 100, 200),
                make_annotation(
                    PropertyType::MemorySafety,
                    VerificationMethod::BorrowChecker,
                    ConfidenceLevel::High,
                    vec![],
                    None,
                ),
            ),
            (
                make_location("test.rs", 0, 50),
                make_annotation(
                    PropertyType::ThreadSafety,
                    VerificationMethod::BorrowChecker,
                    ConfidenceLevel::High,
                    vec![],
                    None,
                ),
            ),
        ];
        let result = group_by_file(&annotations);

        assert_eq!(result.len(), 1);
        let proofs = result.get(&PathBuf::from("test.rs")).unwrap();
        assert_eq!(proofs.len(), 2);
        // Should be sorted by line number (0-50 comes before 100-200)
        assert_eq!(proofs[0].0.span.start.0, 0);
        assert_eq!(proofs[1].0.span.start.0, 100);
    }

    #[test]
    fn test_group_by_file_multiple_files() {
        let annotations = vec![
            (
                make_location("a.rs", 0, 100),
                make_annotation(
                    PropertyType::MemorySafety,
                    VerificationMethod::BorrowChecker,
                    ConfidenceLevel::High,
                    vec![],
                    None,
                ),
            ),
            (
                make_location("b.rs", 0, 100),
                make_annotation(
                    PropertyType::ThreadSafety,
                    VerificationMethod::BorrowChecker,
                    ConfidenceLevel::High,
                    vec![],
                    None,
                ),
            ),
            (
                make_location("a.rs", 200, 300),
                make_annotation(
                    PropertyType::Termination,
                    VerificationMethod::BorrowChecker,
                    ConfidenceLevel::Medium,
                    vec![],
                    None,
                ),
            ),
        ];
        let result = group_by_file(&annotations);

        assert_eq!(result.len(), 2);
        assert_eq!(result.get(&PathBuf::from("a.rs")).unwrap().len(), 2);
        assert_eq!(result.get(&PathBuf::from("b.rs")).unwrap().len(), 1);
    }

    #[test]
    fn test_group_by_file_sorting_correctness() {
        let annotations = vec![
            (
                make_location("test.rs", 300, 400),
                make_annotation(
                    PropertyType::MemorySafety,
                    VerificationMethod::BorrowChecker,
                    ConfidenceLevel::High,
                    vec![],
                    None,
                ),
            ),
            (
                make_location("test.rs", 100, 200),
                make_annotation(
                    PropertyType::ThreadSafety,
                    VerificationMethod::BorrowChecker,
                    ConfidenceLevel::High,
                    vec![],
                    None,
                ),
            ),
            (
                make_location("test.rs", 0, 50),
                make_annotation(
                    PropertyType::Termination,
                    VerificationMethod::BorrowChecker,
                    ConfidenceLevel::High,
                    vec![],
                    None,
                ),
            ),
            (
                make_location("test.rs", 200, 250),
                make_annotation(
                    PropertyType::DataRaceFreeze,
                    VerificationMethod::BorrowChecker,
                    ConfidenceLevel::High,
                    vec![],
                    None,
                ),
            ),
        ];
        let result = group_by_file(&annotations);

        let proofs = result.get(&PathBuf::from("test.rs")).unwrap();
        assert_eq!(proofs.len(), 4);
        // Verify sorted order
        assert_eq!(proofs[0].0.span.start.0, 0);
        assert_eq!(proofs[1].0.span.start.0, 100);
        assert_eq!(proofs[2].0.span.start.0, 200);
        assert_eq!(proofs[3].0.span.start.0, 300);
    }

    // ========== format_single_proof tests ==========

    #[test]
    fn test_format_single_proof_without_evidence() {
        let location = make_location("test.rs", 10, 50);
        let annotation = make_annotation(
            PropertyType::MemorySafety,
            VerificationMethod::BorrowChecker,
            ConfidenceLevel::High,
            vec![],
            None,
        );
        let mut output = String::new();

        let result = format_single_proof(&location, &annotation, &mut output, false);

        assert!(result.is_ok());
        assert!(output.contains("Position 10-50"));
        assert!(output.contains("**Property**:"));
        assert!(output.contains("**Method**:"));
        assert!(output.contains("**Tool**: test-tool v1.0.0"));
        assert!(output.contains("**Confidence**:"));
        assert!(output.contains("**Verified**:"));
        // Should NOT contain evidence when include_evidence is false
        assert!(!output.contains("**Evidence**:"));
    }

    #[test]
    fn test_format_single_proof_with_evidence() {
        let location = make_location("test.rs", 10, 50);
        let annotation = make_annotation(
            PropertyType::MemorySafety,
            VerificationMethod::BorrowChecker,
            ConfidenceLevel::High,
            vec![],
            Some("spec_001".to_string()),
        );
        let mut output = String::new();

        let result = format_single_proof(&location, &annotation, &mut output, true);

        assert!(result.is_ok());
        assert!(output.contains("**Evidence**:"));
        assert!(output.contains("**Specification ID**: spec_001"));
    }

    #[test]
    fn test_format_single_proof_with_assumptions() {
        let location = make_location("test.rs", 10, 50);
        let annotation = make_annotation(
            PropertyType::MemorySafety,
            VerificationMethod::BorrowChecker,
            ConfidenceLevel::High,
            vec![
                "Input is non-null".to_string(),
                "Buffer size is bounded".to_string(),
            ],
            None,
        );
        let mut output = String::new();

        let result = format_single_proof(&location, &annotation, &mut output, false);

        assert!(result.is_ok());
        assert!(output.contains("**Assumptions**:"));
        assert!(output.contains("- Input is non-null"));
        assert!(output.contains("- Buffer size is bounded"));
    }

    #[test]
    fn test_format_single_proof_empty_assumptions() {
        let location = make_location("test.rs", 10, 50);
        let annotation = make_annotation(
            PropertyType::MemorySafety,
            VerificationMethod::BorrowChecker,
            ConfidenceLevel::High,
            vec![],
            None,
        );
        let mut output = String::new();

        let result = format_single_proof(&location, &annotation, &mut output, false);

        assert!(result.is_ok());
        // Should not contain assumptions section when empty
        assert!(!output.contains("**Assumptions**:"));
    }

    #[test]
    fn test_format_single_proof_with_evidence_no_spec_id() {
        let location = make_location("test.rs", 10, 50);
        let annotation = make_annotation(
            PropertyType::MemorySafety,
            VerificationMethod::BorrowChecker,
            ConfidenceLevel::High,
            vec![],
            None, // No specification_id
        );
        let mut output = String::new();

        let result = format_single_proof(&location, &annotation, &mut output, true);

        assert!(result.is_ok());
        assert!(output.contains("**Evidence**:"));
        // Should NOT contain spec ID line when None
        assert!(!output.contains("**Specification ID**:"));
    }

    // ========== format_provability_summary tests ==========

    #[test]
    fn test_format_provability_summary_empty() {
        use crate::services::lightweight_provability_analyzer::ProofSummary;

        let summaries: Vec<ProofSummary> = vec![];
        let mut output = String::new();

        // Note: This will produce NaN percentages, but shouldn't panic
        let result = format_provability_summary(&summaries, &mut output, false);

        // The function should handle empty summaries (division by zero produces NaN)
        assert!(result.is_ok());
    }

    #[test]
