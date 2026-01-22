        #[test]
        fn prop_format_confidence_stats_never_panics(
            num_annotations in 0usize..20
        ) {
            let annotations: Vec<(Location, ProofAnnotation)> = (0..num_annotations)
                .map(|i| {
                    let confidence = match i % 3 {
                        0 => ConfidenceLevel::Low,
                        1 => ConfidenceLevel::Medium,
                        _ => ConfidenceLevel::High,
                    };
                    (
                        make_location("test.rs", i as u32 * 100, (i as u32 + 1) * 100),
                        make_annotation(
                            PropertyType::MemorySafety,
                            VerificationMethod::BorrowChecker,
                            confidence,
                            vec![],
                            None,
                        ),
                    )
                })
                .collect();

            let mut output = String::new();
            let result = format_confidence_stats(&annotations, &mut output);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn prop_format_method_stats_never_panics(
            num_annotations in 0usize..20
        ) {
            let methods = [
                VerificationMethod::BorrowChecker,
                VerificationMethod::AbstractInterpretation,
                VerificationMethod::FormalProof { prover: "Z3".to_string() },
                VerificationMethod::ModelChecking { bounded: true },
                VerificationMethod::StaticAnalysis { tool: "clippy".to_string() },
            ];

            let annotations: Vec<(Location, ProofAnnotation)> = (0..num_annotations)
                .map(|i| {
                    (
                        make_location("test.rs", i as u32 * 100, (i as u32 + 1) * 100),
                        make_annotation(
                            PropertyType::MemorySafety,
                            methods[i % methods.len()].clone(),
                            ConfidenceLevel::High,
                            vec![],
                            None,
                        ),
                    )
                })
                .collect();

            let mut output = String::new();
            let result = format_method_stats(&annotations, &mut output);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn prop_group_by_file_preserves_count(
            num_annotations in 0usize..50
        ) {
            let files = ["a.rs", "b.rs", "c.rs", "d.rs"];
            let annotations: Vec<(Location, ProofAnnotation)> = (0..num_annotations)
                .map(|i| {
                    (
                        make_location(files[i % files.len()], i as u32 * 10, (i as u32 + 1) * 10),
                        make_annotation(
                            PropertyType::MemorySafety,
                            VerificationMethod::BorrowChecker,
                            ConfidenceLevel::High,
                            vec![],
                            None,
                        ),
                    )
                })
                .collect();

            let result = group_by_file(&annotations);
            let total: usize = result.values().map(|v| v.len()).sum();
            prop_assert_eq!(total, num_annotations);
        }

        #[test]
        fn prop_group_by_file_sorted_by_line(
            num_per_file in 1usize..20
        ) {
            let annotations: Vec<(Location, ProofAnnotation)> = (0..num_per_file)
                .map(|i| {
                    // Insert in reverse order to test sorting
                    let start = ((num_per_file - i) as u32) * 100;
                    (
                        make_location("test.rs", start, start + 50),
                        make_annotation(
                            PropertyType::MemorySafety,
                            VerificationMethod::BorrowChecker,
                            ConfidenceLevel::High,
                            vec![],
                            None,
                        ),
                    )
                })
                .collect();

            let result = group_by_file(&annotations);
            let proofs = result.get(&PathBuf::from("test.rs")).unwrap();

            // Verify sorted order
            for i in 1..proofs.len() {
                prop_assert!(proofs[i - 1].0.span.start.0 <= proofs[i].0.span.start.0);
            }
        }

        #[test]
        fn prop_sarif_rules_ids_unique(_dummy in 0u32..1) {
            let rules = generate_proof_sarif_rules();
            let ids: Vec<_> = rules.iter().map(|r| r["id"].as_str().unwrap()).collect();
            let unique_ids: std::collections::HashSet<_> = ids.iter().collect();
            prop_assert_eq!(ids.len(), unique_ids.len());
        }

        #[test]
        fn prop_provability_summary_percentages_valid(
            scores in proptest::collection::vec(0.0f64..=1.0, 1..20)
        ) {
            use crate::services::lightweight_provability_analyzer::ProofSummary;

            let summaries: Vec<ProofSummary> = scores
                .iter()
                .map(|&s| ProofSummary {
                    provability_score: s,
                    verified_properties: vec![],
                    analysis_time_us: 1000,
                    version: 1,
                })
                .collect();

            let mut output = String::new();
            let result = format_provability_summary(&summaries, &mut output, false);

            prop_assert!(result.is_ok());
            // Output should contain valid percentage values (no NaN for non-empty input)
            prop_assert!(output.contains("Total Functions"));
        }
    }

    // ========== Evidence type coverage tests ==========

    #[test]
    fn test_format_proof_evidence_proof_script_reference() {
        let location = make_location("test.rs", 0, 100);
        let mut annotation = make_annotation(
            PropertyType::MemorySafety,
            VerificationMethod::BorrowChecker,
            ConfidenceLevel::High,
            vec![],
            Some("SPEC-001".to_string()),
        );
        annotation.evidence_type = EvidenceType::ProofScriptReference {
            uri: "file://proofs/safety.v".to_string(),
        };

        let mut output = String::new();
        let result = format_single_proof(&location, &annotation, &mut output, true);

        assert!(result.is_ok());
        assert!(output.contains("ProofScriptReference"));
    }

    #[test]
    fn test_format_proof_evidence_theorem_name() {
        let location = make_location("test.rs", 0, 100);
        let mut annotation = make_annotation(
            PropertyType::MemorySafety,
            VerificationMethod::BorrowChecker,
            ConfidenceLevel::High,
            vec![],
            None,
        );
        annotation.evidence_type = EvidenceType::TheoremName {
            theorem: "memory_safety_theorem".to_string(),
            theory: Some("LinearTypes".to_string()),
        };

        let mut output = String::new();
        let result = format_single_proof(&location, &annotation, &mut output, true);

        assert!(result.is_ok());
        assert!(output.contains("TheoremName"));
    }

    #[test]
    fn test_format_proof_evidence_static_analysis_report() {
        let location = make_location("test.rs", 0, 100);
        let mut annotation = make_annotation(
            PropertyType::MemorySafety,
            VerificationMethod::BorrowChecker,
            ConfidenceLevel::High,
            vec![],
            None,
        );
        annotation.evidence_type = EvidenceType::StaticAnalysisReport {
            report_id: "clippy-report-12345".to_string(),
        };

        let mut output = String::new();
        let result = format_single_proof(&location, &annotation, &mut output, true);

        assert!(result.is_ok());
        assert!(output.contains("StaticAnalysisReport"));
    }

    #[test]
    fn test_format_proof_evidence_certificate_hash() {
        let location = make_location("test.rs", 0, 100);
        let mut annotation = make_annotation(
            PropertyType::MemorySafety,
            VerificationMethod::BorrowChecker,
            ConfidenceLevel::High,
            vec![],
            None,
        );
        annotation.evidence_type = EvidenceType::CertificateHash {
            hash: "sha256:abc123def456".to_string(),
            algorithm: "SHA-256".to_string(),
        };

        let mut output = String::new();
        let result = format_single_proof(&location, &annotation, &mut output, true);

        assert!(result.is_ok());
        assert!(output.contains("CertificateHash"));
    }

    // ========== Model checking bounded variants ==========

    #[test]
    fn test_format_method_stats_model_checking_unbounded() {
        let annotations = vec![(
            make_location("test.rs", 0, 100),
            make_annotation(
                PropertyType::MemorySafety,
                VerificationMethod::ModelChecking { bounded: false },
                ConfidenceLevel::High,
                vec![],
                None,
            ),
        )];
        let mut output = String::new();
        let result = format_method_stats(&annotations, &mut output);

        assert!(result.is_ok());
        assert!(output.contains("Model Checking"));
    }

    // ========== Resource bounds edge cases ==========

    #[test]
    fn test_format_property_stats_resource_bounds_partial() {
        // Only CPU specified
        let annotations = vec![(
            make_location("test.rs", 0, 100),
            make_annotation(
                PropertyType::ResourceBounds {
                    cpu: Some(1000),
                    memory: None,
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
    fn test_format_property_stats_resource_bounds_memory_only() {
        // Only memory specified
        let annotations = vec![(
            make_location("test.rs", 0, 100),
            make_annotation(
                PropertyType::ResourceBounds {
                    cpu: None,
                    memory: Some(4096),
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
}
