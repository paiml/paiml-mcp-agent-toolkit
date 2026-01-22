mod coverage_tests {
    use super::*;
    use crate::models::unified_ast::{
        BytePos, ConfidenceLevel, EvidenceType, PropertyType, Span, VerificationMethod,
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
