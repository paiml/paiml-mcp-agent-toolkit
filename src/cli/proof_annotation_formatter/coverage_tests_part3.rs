    fn test_format_provability_summary_single_high() {
        use crate::services::lightweight_provability_analyzer::ProofSummary;

        let summaries = vec![ProofSummary {
            provability_score: 0.9, // High
            verified_properties: vec![],
            analysis_time_us: 1000,
            version: 1,
        }];
        let mut output = String::new();

        let result = format_provability_summary(&summaries, &mut output, false);

        assert!(result.is_ok());
        assert!(output.contains("Provability Analysis Summary"));
        assert!(output.contains("**Total Functions**: 1"));
        assert!(output.contains("**High Provability"));
    }

    #[test]
    fn test_format_provability_summary_single_medium() {
        use crate::services::lightweight_provability_analyzer::ProofSummary;

        let summaries = vec![ProofSummary {
            provability_score: 0.6, // Medium (50-79%)
            verified_properties: vec![],
            analysis_time_us: 1000,
            version: 1,
        }];
        let mut output = String::new();

        let result = format_provability_summary(&summaries, &mut output, false);

        assert!(result.is_ok());
        assert!(output.contains("**Medium Provability"));
    }

    #[test]
    fn test_format_provability_summary_single_low() {
        use crate::services::lightweight_provability_analyzer::ProofSummary;

        let summaries = vec![ProofSummary {
            provability_score: 0.3, // Low (<50%)
            verified_properties: vec![],
            analysis_time_us: 1000,
            version: 1,
        }];
        let mut output = String::new();

        let result = format_provability_summary(&summaries, &mut output, false);

        assert!(result.is_ok());
        assert!(output.contains("**Low Provability"));
    }

    #[test]
    fn test_format_provability_summary_mixed_scores() {
        use crate::services::lightweight_provability_analyzer::ProofSummary;

        let summaries = vec![
            ProofSummary {
                provability_score: 0.9,  // High
                verified_properties: vec![],
                analysis_time_us: 1000,
                version: 1,
            },
            ProofSummary {
                provability_score: 0.85, // High
                verified_properties: vec![],
                analysis_time_us: 1000,
                version: 1,
            },
            ProofSummary {
                provability_score: 0.6,  // Medium
                verified_properties: vec![],
                analysis_time_us: 1000,
                version: 1,
            },
            ProofSummary {
                provability_score: 0.3,  // Low
                verified_properties: vec![],
                analysis_time_us: 1000,
                version: 1,
            },
            ProofSummary {
                provability_score: 0.1,  // Low
                verified_properties: vec![],
                analysis_time_us: 1000,
                version: 1,
            },
        ];
        let mut output = String::new();

        let result = format_provability_summary(&summaries, &mut output, false);

        assert!(result.is_ok());
        assert!(output.contains("**Total Functions**: 5"));
        assert!(output.contains("**Average Score**:"));
    }

    #[test]
    fn test_format_provability_summary_boundary_scores() {
        use crate::services::lightweight_provability_analyzer::ProofSummary;

        // Test exact boundary values
        let summaries = vec![
            ProofSummary {
                provability_score: 0.8, // Exactly at high threshold
                verified_properties: vec![],
                analysis_time_us: 1000,
                version: 1,
            },
            ProofSummary {
                provability_score: 0.5, // Exactly at medium threshold
                verified_properties: vec![],
                analysis_time_us: 1000,
                version: 1,
            },
            ProofSummary {
                provability_score: 0.0, // Zero score
                verified_properties: vec![],
                analysis_time_us: 1000,
                version: 1,
            },
            ProofSummary {
                provability_score: 1.0, // Perfect score
                verified_properties: vec![],
                analysis_time_us: 1000,
                version: 1,
            },
        ];
        let mut output = String::new();

        let result = format_provability_summary(&summaries, &mut output, false);

        assert!(result.is_ok());
        assert!(output.contains("**Total Functions**: 4"));
    }

    // ========== generate_proof_sarif_rules tests ==========

    #[test]
    fn test_generate_proof_sarif_rules_count() {
        let rules = generate_proof_sarif_rules();
        assert_eq!(rules.len(), 4);
    }

    #[test]
    fn test_generate_proof_sarif_rules_low_confidence() {
        let rules = generate_proof_sarif_rules();
        let low_rule = rules.iter().find(|r| r["id"] == "low-confidence-proof");

        assert!(low_rule.is_some());
        let rule = low_rule.unwrap();
        assert_eq!(rule["name"], "Low Confidence Proof");
        assert_eq!(rule["defaultConfiguration"]["level"], "warning");
    }

    #[test]
    fn test_generate_proof_sarif_rules_medium_confidence() {
        let rules = generate_proof_sarif_rules();
        let medium_rule = rules.iter().find(|r| r["id"] == "medium-confidence-proof");

        assert!(medium_rule.is_some());
        let rule = medium_rule.unwrap();
        assert_eq!(rule["name"], "Medium Confidence Proof");
        assert_eq!(rule["defaultConfiguration"]["level"], "note");
    }

    #[test]
    fn test_generate_proof_sarif_rules_high_confidence() {
        let rules = generate_proof_sarif_rules();
        let high_rule = rules.iter().find(|r| r["id"] == "high-confidence-proof");

        assert!(high_rule.is_some());
        let rule = high_rule.unwrap();
        assert_eq!(rule["name"], "High Confidence Proof");
        assert_eq!(rule["defaultConfiguration"]["level"], "none");
    }

    #[test]
    fn test_generate_proof_sarif_rules_unverified() {
        let rules = generate_proof_sarif_rules();
        let unverified_rule = rules.iter().find(|r| r["id"] == "unverified-property");

        assert!(unverified_rule.is_some());
        let rule = unverified_rule.unwrap();
        assert_eq!(rule["name"], "Unverified Safety Property");
        assert_eq!(rule["defaultConfiguration"]["level"], "note");
    }

    #[test]
    fn test_generate_proof_sarif_rules_structure() {
        let rules = generate_proof_sarif_rules();

        for rule in &rules {
            // Every rule must have these fields
            assert!(rule.get("id").is_some());
            assert!(rule.get("name").is_some());
            assert!(rule.get("shortDescription").is_some());
            assert!(rule.get("fullDescription").is_some());
            assert!(rule.get("defaultConfiguration").is_some());

            // shortDescription and fullDescription must have text
            assert!(rule["shortDescription"].get("text").is_some());
            assert!(rule["fullDescription"].get("text").is_some());

            // defaultConfiguration must have level
            assert!(rule["defaultConfiguration"].get("level").is_some());
        }
    }

    // ========== Private function coverage via integration ==========

    #[test]
    fn test_format_proof_header_integration() {
        // format_proof_header is called via format_single_proof
        let location = make_location("src/main.rs", 42, 100);
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
        assert!(output.contains("### Position 42-100"));
    }

    #[test]
    fn test_format_proof_metadata_integration() {
        // format_proof_metadata is called via format_single_proof
        let location = make_location("test.rs", 0, 100);
        let annotation = make_annotation(
            PropertyType::ThreadSafety,
            VerificationMethod::FormalProof {
                prover: "Lean4".to_string(),
            },
            ConfidenceLevel::Medium,
            vec![],
            None,
        );
        let mut output = String::new();

        let result = format_single_proof(&location, &annotation, &mut output, false);

        assert!(result.is_ok());
        assert!(output.contains("ThreadSafety"));
        assert!(output.contains("FormalProof"));
        assert!(output.contains("Medium"));
    }

    #[test]
    fn test_format_proof_evidence_integration() {
        // format_proof_evidence is called via format_single_proof when include_evidence=true
        let location = make_location("test.rs", 0, 100);
        let mut annotation = make_annotation(
            PropertyType::MemorySafety,
            VerificationMethod::BorrowChecker,
            ConfidenceLevel::High,
            vec![],
            Some("SPEC-001".to_string()),
        );
        // Use a different evidence type
        annotation.evidence_type = EvidenceType::ProofScriptReference {
            uri: "file://proofs/memory_safety.v".to_string(),
        };

        let mut output = String::new();

        let result = format_single_proof(&location, &annotation, &mut output, true);

        assert!(result.is_ok());
        assert!(output.contains("**Evidence**:"));
        assert!(output.contains("**Specification ID**: SPEC-001"));
    }

    // ========== Edge case tests ==========

    #[test]
    fn test_format_with_special_characters_in_tool_name() {
        let location = make_location("test.rs", 0, 100);
        let mut annotation = make_annotation(
            PropertyType::MemorySafety,
            VerificationMethod::BorrowChecker,
            ConfidenceLevel::High,
            vec![],
            None,
        );
        annotation.tool_name = "tool-with-dashes_and_underscores".to_string();
        annotation.tool_version = "1.0.0-beta+build.123".to_string();

        let mut output = String::new();
        let result = format_single_proof(&location, &annotation, &mut output, false);

        assert!(result.is_ok());
        assert!(output.contains("tool-with-dashes_and_underscores"));
        assert!(output.contains("1.0.0-beta+build.123"));
    }

    #[test]
    fn test_format_with_unicode_in_assumptions() {
        let location = make_location("test.rs", 0, 100);
        let annotation = make_annotation(
            PropertyType::MemorySafety,
            VerificationMethod::BorrowChecker,
            ConfidenceLevel::High,
            vec![
                "x > 0 (positive)".to_string(),
                "value is not null".to_string(),
            ],
            None,
        );

        let mut output = String::new();
        let result = format_single_proof(&location, &annotation, &mut output, false);

        assert!(result.is_ok());
        assert!(output.contains("x > 0"));
    }

    #[test]
    fn test_format_with_empty_string_assumption() {
        let location = make_location("test.rs", 0, 100);
        let annotation = make_annotation(
            PropertyType::MemorySafety,
            VerificationMethod::BorrowChecker,
            ConfidenceLevel::High,
            vec!["".to_string(), "valid assumption".to_string()],
            None,
        );

        let mut output = String::new();
        let result = format_single_proof(&location, &annotation, &mut output, false);

        assert!(result.is_ok());
        assert!(output.contains("**Assumptions**:"));
    }

    #[test]
    fn test_format_with_long_file_path() {
        let long_path = "src/very/deeply/nested/directory/structure/that/goes/on/forever/file.rs";
        let location = make_location(long_path, 0, 100);
        let annotations = vec![(
            location,
            make_annotation(
                PropertyType::MemorySafety,
                VerificationMethod::BorrowChecker,
                ConfidenceLevel::High,
                vec![],
                None,
            ),
        )];

        let result = group_by_file(&annotations);
        assert!(result.contains_key(&PathBuf::from(long_path)));
    }

    #[test]
    fn test_format_with_zero_span() {
        let location = make_location("test.rs", 0, 0);
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
        assert!(output.contains("Position 0-0"));
    }

    #[test]
    fn test_format_with_large_span_values() {
        let location = make_location("test.rs", u32::MAX - 100, u32::MAX);
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
    }

    // ========== Property-based tests ==========

    proptest! {
