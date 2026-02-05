//! Extended tests for proof annotation formatter
//!
//! This module contains additional coverage tests for proof annotations.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod extended_coverage_tests {
    use super::super::{format_single_proof, generate_proof_sarif_rules, group_by_file};
    use crate::models::unified_ast::{
        BytePos, ConfidenceLevel, EvidenceType, Location, ProofAnnotation, PropertyType, Span,
        VerificationMethod,
    };
    use chrono::Utc;
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
        use super::super::format_method_stats;

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
        use super::super::format_property_stats;

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
        use super::super::format_property_stats;

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
