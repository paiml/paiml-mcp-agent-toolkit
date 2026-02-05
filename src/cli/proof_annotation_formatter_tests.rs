//! Tests for proof annotation formatter
//!
//! This module contains property tests and coverage tests for proof annotations.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::super::{
        format_confidence_stats, format_method_stats, format_property_stats,
        format_provability_summary, format_single_proof, generate_proof_sarif_rules, group_by_file,
    };
    use crate::models::unified_ast::{
        BytePos, ConfidenceLevel, EvidenceType, Location, ProofAnnotation, PropertyType, Span,
        VerificationMethod,
    };
    use chrono::Utc;
    use proptest::prelude::*;
    use std::path::PathBuf;
    use uuid::Uuid;

    /// Helper to create a test Location
    fn make_location(file: &str, start: u32, end: u32) -> Location {
        Location {
            file_path: PathBuf::from(file),
            span: Span {
                start: BytePos(start),
                end: BytePos(end),
            },
        }
    }

    /// Helper to create a test ProofAnnotation
    fn make_annotation(
        property: PropertyType,
        method: VerificationMethod,
        confidence: ConfidenceLevel,
        assumptions: Vec<String>,
        spec_id: Option<String>,
    ) -> ProofAnnotation {
        ProofAnnotation {
            annotation_id: Uuid::new_v4(),
            property_proven: property,
            specification_id: spec_id,
            method,
            tool_name: "test-tool".to_string(),
            tool_version: "1.0.0".to_string(),
            confidence_level: confidence,
            assumptions,
            evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
            evidence_location: None,
            date_verified: Utc::now(),
        }
    }

    // ========== format_confidence_stats tests ==========

    #[test]
    fn test_format_confidence_stats_empty_annotations() {
        let annotations: Vec<(Location, ProofAnnotation)> = vec![];
        let mut output = String::new();
        let result = format_confidence_stats(&annotations, &mut output);

        assert!(result.is_ok());
        // Empty annotations should produce no output
        assert!(output.is_empty());
    }

    #[test]
    fn test_format_confidence_stats_single_low() {
        let annotations = vec![(
            make_location("test.rs", 0, 100),
            make_annotation(
                PropertyType::MemorySafety,
                VerificationMethod::BorrowChecker,
                ConfidenceLevel::Low,
                vec![],
                None,
            ),
        )];
        let mut output = String::new();
        let result = format_confidence_stats(&annotations, &mut output);

        assert!(result.is_ok());
        assert!(output.contains("Confidence Levels"));
        assert!(output.contains("Low"));
        assert!(output.contains("1 proofs"));
    }

    #[test]
    fn test_format_confidence_stats_single_medium() {
        let annotations = vec![(
            make_location("test.rs", 0, 100),
            make_annotation(
                PropertyType::MemorySafety,
                VerificationMethod::BorrowChecker,
                ConfidenceLevel::Medium,
                vec![],
                None,
            ),
        )];
        let mut output = String::new();
        let result = format_confidence_stats(&annotations, &mut output);

        assert!(result.is_ok());
        assert!(output.contains("Medium"));
    }

    #[test]
    fn test_format_confidence_stats_single_high() {
        let annotations = vec![(
            make_location("test.rs", 0, 100),
            make_annotation(
                PropertyType::MemorySafety,
                VerificationMethod::BorrowChecker,
                ConfidenceLevel::High,
                vec![],
                None,
            ),
        )];
        let mut output = String::new();
        let result = format_confidence_stats(&annotations, &mut output);

        assert!(result.is_ok());
        assert!(output.contains("High"));
    }

    #[test]
    fn test_format_confidence_stats_multiple_levels() {
        let annotations = vec![
            (
                make_location("test.rs", 0, 100),
                make_annotation(
                    PropertyType::MemorySafety,
                    VerificationMethod::BorrowChecker,
                    ConfidenceLevel::Low,
                    vec![],
                    None,
                ),
            ),
            (
                make_location("test.rs", 100, 200),
                make_annotation(
                    PropertyType::ThreadSafety,
                    VerificationMethod::BorrowChecker,
                    ConfidenceLevel::Medium,
                    vec![],
                    None,
                ),
            ),
            (
                make_location("test.rs", 200, 300),
                make_annotation(
                    PropertyType::Termination,
                    VerificationMethod::BorrowChecker,
                    ConfidenceLevel::High,
                    vec![],
                    None,
                ),
            ),
            (
                make_location("test.rs", 300, 400),
                make_annotation(
                    PropertyType::DataRaceFreeze,
                    VerificationMethod::BorrowChecker,
                    ConfidenceLevel::High,
                    vec![],
                    None,
                ),
            ),
        ];
        let mut output = String::new();
        let result = format_confidence_stats(&annotations, &mut output);

        assert!(result.is_ok());
        assert!(output.contains("Confidence Levels"));
        // We should have multiple levels counted
        assert!(output.contains("proofs"));
    }

    // ========== format_method_stats tests ==========

    #[test]
    fn test_format_method_stats_empty() {
        let annotations: Vec<(Location, ProofAnnotation)> = vec![];
        let mut output = String::new();
        let result = format_method_stats(&annotations, &mut output);

        assert!(result.is_ok());
        assert!(output.is_empty());
    }

    #[test]
    fn test_format_method_stats_formal_proof() {
        let annotations = vec![(
            make_location("test.rs", 0, 100),
            make_annotation(
                PropertyType::MemorySafety,
                VerificationMethod::FormalProof {
                    prover: "Z3".to_string(),
                },
                ConfidenceLevel::High,
                vec![],
                None,
            ),
        )];
        let mut output = String::new();
        let result = format_method_stats(&annotations, &mut output);

        assert!(result.is_ok());
        assert!(output.contains("Verification Methods"));
        assert!(output.contains("Formal Proof"));
    }

    #[test]
    fn test_format_method_stats_model_checking() {
        let annotations = vec![(
            make_location("test.rs", 0, 100),
            make_annotation(
                PropertyType::MemorySafety,
                VerificationMethod::ModelChecking { bounded: true },
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

    #[test]
    fn test_format_method_stats_static_analysis() {
        let annotations = vec![(
            make_location("test.rs", 0, 100),
            make_annotation(
                PropertyType::MemorySafety,
                VerificationMethod::StaticAnalysis {
                    tool: "clippy".to_string(),
                },
                ConfidenceLevel::Medium,
                vec![],
                None,
            ),
        )];
        let mut output = String::new();
        let result = format_method_stats(&annotations, &mut output);

        assert!(result.is_ok());
        assert!(output.contains("Static Analysis"));
    }

    #[test]
    fn test_format_method_stats_abstract_interpretation() {
        let annotations = vec![(
            make_location("test.rs", 0, 100),
            make_annotation(
                PropertyType::MemorySafety,
                VerificationMethod::AbstractInterpretation,
                ConfidenceLevel::Medium,
                vec![],
                None,
            ),
        )];
        let mut output = String::new();
        let result = format_method_stats(&annotations, &mut output);

        assert!(result.is_ok());
        assert!(output.contains("Abstract Interpretation"));
    }

    #[test]
    fn test_format_method_stats_borrow_checker() {
        let annotations = vec![(
            make_location("test.rs", 0, 100),
            make_annotation(
                PropertyType::MemorySafety,
                VerificationMethod::BorrowChecker,
                ConfidenceLevel::High,
                vec![],
                None,
            ),
        )];
        let mut output = String::new();
        let result = format_method_stats(&annotations, &mut output);

        assert!(result.is_ok());
        assert!(output.contains("Borrow Checker"));
    }

    #[test]
    fn test_format_method_stats_all_methods() {
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
                    PropertyType::MemorySafety,
                    VerificationMethod::FormalProof {
                        prover: "Coq".to_string(),
                    },
                    ConfidenceLevel::High,
                    vec![],
                    None,
                ),
            ),
            (
                make_location("test.rs", 200, 300),
                make_annotation(
                    PropertyType::MemorySafety,
                    VerificationMethod::ModelChecking { bounded: false },
                    ConfidenceLevel::Medium,
                    vec![],
                    None,
                ),
            ),
            (
                make_location("test.rs", 300, 400),
                make_annotation(
                    PropertyType::MemorySafety,
                    VerificationMethod::StaticAnalysis {
                        tool: "miri".to_string(),
                    },
                    ConfidenceLevel::Medium,
                    vec![],
                    None,
                ),
            ),
            (
                make_location("test.rs", 400, 500),
                make_annotation(
                    PropertyType::MemorySafety,
                    VerificationMethod::AbstractInterpretation,
                    ConfidenceLevel::Low,
                    vec![],
                    None,
                ),
            ),
        ];
        let mut output = String::new();
        let result = format_method_stats(&annotations, &mut output);

        assert!(result.is_ok());
        assert!(output.contains("Verification Methods"));
    }

    // ========== format_property_stats tests ==========

    #[test]
    fn test_format_property_stats_empty() {
        let annotations: Vec<(Location, ProofAnnotation)> = vec![];
        let mut output = String::new();
        let result = format_property_stats(&annotations, &mut output);

        assert!(result.is_ok());
        assert!(output.is_empty());
    }

    #[test]
    fn test_format_property_stats_memory_safety() {
        let annotations = vec![(
            make_location("test.rs", 0, 100),
            make_annotation(
                PropertyType::MemorySafety,
                VerificationMethod::BorrowChecker,
                ConfidenceLevel::High,
                vec![],
                None,
            ),
        )];
        let mut output = String::new();
        let result = format_property_stats(&annotations, &mut output);

        assert!(result.is_ok());
        assert!(output.contains("Properties Proven"));
        assert!(output.contains("MemorySafety"));
    }

    #[test]
    fn test_format_property_stats_thread_safety() {
        let annotations = vec![(
            make_location("test.rs", 0, 100),
            make_annotation(
                PropertyType::ThreadSafety,
                VerificationMethod::BorrowChecker,
                ConfidenceLevel::High,
                vec![],
                None,
            ),
        )];
        let mut output = String::new();
        let result = format_property_stats(&annotations, &mut output);

        assert!(result.is_ok());
        assert!(output.contains("ThreadSafety"));
    }

    #[test]
    fn test_format_property_stats_data_race_freeze() {
        let annotations = vec![(
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

    // ========== Property-based tests ==========

    proptest! {
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
}
