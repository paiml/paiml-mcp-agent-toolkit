// Coverage tests for proof annotations handler - Part 1: Helpers and format tests
// This file is include\!()'d into proof_annotations_handler.rs coverage_tests module.
// Do not add module-level attributes or standalone use statements.

    use super::*;
    use crate::cli::proof_annotation_helpers::setup_proof_annotator;
    use crate::models::unified_ast::{
        BytePos, ConfidenceLevel, EvidenceType, ProofAnnotation, PropertyType, Span,
        VerificationMethod,
    };
    use chrono::Utc;
    use std::time::Duration;
    use tempfile::TempDir;
    use uuid::Uuid;

    /// Create a test annotation with specified confidence level
    fn create_test_annotation(
        confidence: ConfidenceLevel,
        property: PropertyType,
        method: VerificationMethod,
    ) -> ProofAnnotation {
        ProofAnnotation {
            annotation_id: Uuid::new_v4(),
            property_proven: property,
            method,
            confidence_level: confidence,
            date_verified: Utc::now(),
            tool_name: "test_tool".to_string(),
            tool_version: "1.0.0".to_string(),
            assumptions: vec![],
            evidence_type: EvidenceType::CheckerOutput("test output".to_string()),
            specification_id: None,
        }
    }

    /// Create a test location with a file path
    fn create_test_location(file_name: &str, start: u32, end: u32) -> Location {
        Location {
            file_path: PathBuf::from(file_name),
            span: Span {
                start: BytePos(start),
                end: BytePos(end),
            },
        }
    }

    /// Create test annotations for testing
    fn create_test_annotations() -> Vec<(Location, ProofAnnotation)> {
        vec![
            (
                create_test_location("src/lib.rs", 10, 50),
                create_test_annotation(
                    ConfidenceLevel::High,
                    PropertyType::MemorySafety,
                    VerificationMethod::BorrowChecker,
                ),
            ),
            (
                create_test_location("src/lib.rs", 60, 100),
                create_test_annotation(
                    ConfidenceLevel::Medium,
                    PropertyType::ThreadSafety,
                    VerificationMethod::StaticAnalysis {
                        tool: "miri".to_string(),
                    },
                ),
            ),
            (
                create_test_location("src/main.rs", 5, 20),
                create_test_annotation(
                    ConfidenceLevel::Low,
                    PropertyType::Termination,
                    VerificationMethod::FormalProof {
                        prover: "coq".to_string(),
                    },
                ),
            ),
        ]
    }

    // ==================== format_proof_annotations tests ====================

    #[test]
    fn test_format_proof_annotations_json() {
        let annotations = create_test_annotations();
        let elapsed = Duration::from_millis(100);
        let annotator = setup_proof_annotator(false);
        let project_path = Path::new("/test/project");

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
        assert!(content.contains("proof_annotations"));
        assert!(content.contains("summary"));
        assert!(content.contains("total_annotations"));
        assert!(content.contains("3")); // 3 annotations
    }

    #[test]
    fn test_format_proof_annotations_summary() {
        let annotations = create_test_annotations();
        let elapsed = Duration::from_millis(500);
        let annotator = setup_proof_annotator(false);
        let project_path = Path::new("/test/project");

        let result = format_proof_annotations(
            ProofAnnotationOutputFormat::Summary,
            &annotations,
            elapsed,
            &annotator,
            project_path,
            false,
        );

        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("Proof Annotations Summary"));
        assert!(content.contains("Total proofs:"));
    }

    #[test]
    fn test_format_proof_annotations_full() {
        let annotations = create_test_annotations();
        let elapsed = Duration::from_millis(200);
        let annotator = setup_proof_annotator(false);
        let project_path = Path::new("/test/project");

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
        assert!(content.contains("Full Proof Annotations Report"));
        assert!(content.contains("Project:"));
        assert!(content.contains("Evidence"));
    }

    #[test]
    fn test_format_proof_annotations_full_without_evidence() {
        let annotations = create_test_annotations();
        let elapsed = Duration::from_millis(200);
        let annotator = setup_proof_annotator(false);
        let project_path = Path::new("/test/project");

        let result = format_proof_annotations(
            ProofAnnotationOutputFormat::Full,
            &annotations,
            elapsed,
            &annotator,
            project_path,
            false,
        );

        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("Full Proof Annotations Report"));
        // Evidence should not be included when include_evidence is false
    }

    #[test]
    fn test_format_proof_annotations_markdown() {
        let annotations = create_test_annotations();
        let elapsed = Duration::from_millis(300);
        let annotator = setup_proof_annotator(false);
        let project_path = Path::new("/test/project");

        let result = format_proof_annotations(
            ProofAnnotationOutputFormat::Markdown,
            &annotations,
            elapsed,
            &annotator,
            project_path,
            true,
        );

        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("# Proof Annotations Analysis"));
        assert!(content.contains("Summary Statistics"));
        assert!(content.contains("Detailed Proofs"));
    }

    #[test]
    fn test_format_proof_annotations_markdown_without_evidence() {
        let annotations = create_test_annotations();
        let elapsed = Duration::from_millis(300);
        let annotator = setup_proof_annotator(false);
        let project_path = Path::new("/test/project");

        let result = format_proof_annotations(
            ProofAnnotationOutputFormat::Markdown,
            &annotations,
            elapsed,
            &annotator,
            project_path,
            false,
        );

        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("# Proof Annotations Analysis"));
        assert!(content.contains("Summary Statistics"));
        // Should not contain detailed proofs section when include_evidence is false
        assert!(!content.contains("Detailed Proofs"));
    }

    #[test]
    fn test_format_proof_annotations_sarif() {
        let annotations = create_test_annotations();
        let elapsed = Duration::from_millis(400);
        let annotator = setup_proof_annotator(false);
        let project_path = Path::new("/test/project");

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
        assert!(content.contains("version"));
        assert!(content.contains("2.1.0"));
        assert!(content.contains("paiml-proof-annotator"));
        assert!(content.contains("results"));
        assert!(content.contains("ruleId"));
    }

    #[test]
    fn test_format_proof_annotations_empty() {
        let annotations: Vec<(Location, ProofAnnotation)> = vec![];
        let elapsed = Duration::from_millis(10);
        let annotator = setup_proof_annotator(false);
        let project_path = Path::new("/test/project");

        // Test all formats with empty annotations
        let json_result = format_proof_annotations(
            ProofAnnotationOutputFormat::Json,
            &annotations,
            elapsed,
            &annotator,
            project_path,
            true,
        );
        assert!(json_result.is_ok());

        let summary_result = format_proof_annotations(
            ProofAnnotationOutputFormat::Summary,
            &annotations,
            elapsed,
            &annotator,
            project_path,
            true,
        );
        assert!(summary_result.is_ok());

        let full_result = format_proof_annotations(
            ProofAnnotationOutputFormat::Full,
            &annotations,
            elapsed,
            &annotator,
            project_path,
            true,
        );
        assert!(full_result.is_ok());

        let markdown_result = format_proof_annotations(
            ProofAnnotationOutputFormat::Markdown,
            &annotations,
            elapsed,
            &annotator,
            project_path,
            true,
        );
        assert!(markdown_result.is_ok());

        let sarif_result = format_proof_annotations(
            ProofAnnotationOutputFormat::Sarif,
            &annotations,
            elapsed,
            &annotator,
            project_path,
            true,
        );
        assert!(sarif_result.is_ok());
    }

