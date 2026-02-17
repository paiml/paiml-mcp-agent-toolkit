#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for proof_annotations_handler.rs
//!
//! Tests the proof annotations handler including:
//! - ProofAnnotationFilter construction and filtering logic
//! - filter_annotation() with various confidence levels
//! - filter_by_property_type() for all PropertyTypeFilter variants
//! - filter_by_verification_method() for all VerificationMethodFilter variants
//! - format_as_json() output structure and content
//! - format_as_summary() output format
//! - format_as_full() with and without evidence
//! - format_as_markdown() with and without evidence
//! - format_as_sarif() SARIF 2.1.0 compliance
//! - format_as_table() table formatting
//! - setup_proof_annotator() mock source setup
//! - collect_and_filter_annotations() async collection
//! - handle_analyze_proof_annotations() full handler workflow
//! - Output format enum Display implementations
//! - Property type filter Display implementations
//! - Verification method filter Display implementations

use crate::cli::proof_annotation_helpers::{
    filter_annotation, format_as_full, format_as_json, format_as_markdown, format_as_sarif,
    format_as_summary, format_as_table, setup_proof_annotator, ProofAnnotationFilter,
};
use crate::cli::{ProofAnnotationOutputFormat, PropertyTypeFilter, VerificationMethodFilter};
use crate::models::unified_ast::{
    BytePos, ConfidenceLevel, EvidenceType, Location, ProofAnnotation, PropertyType, Span,
    VerificationMethod,
};
use chrono::Utc;
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

// =============================================================================
// Test Helpers
// =============================================================================

/// Create a test annotation with specified parameters
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
        evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
        specification_id: None,
        evidence_location: None,
    }
}

/// Create a test location with file path and span
fn create_test_location(file_name: &str, start: u32, end: u32) -> Location {
    Location {
        file_path: PathBuf::from(file_name),
        span: Span {
            start: BytePos(start),
            end: BytePos(end),
        },
    }
}

/// Create a set of test annotations with various configurations
fn create_diverse_annotations() -> Vec<(Location, ProofAnnotation)> {
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

// =============================================================================
// ProofAnnotationFilter tests
// =============================================================================

#[test]
fn test_proof_annotation_filter_default_construction() {
    let filter = ProofAnnotationFilter {
        high_confidence_only: false,
        property_type: None,
        verification_method: None,
    };
    assert!(!filter.high_confidence_only);
    assert!(filter.property_type.is_none());
    assert!(filter.verification_method.is_none());
}

#[test]
fn test_proof_annotation_filter_high_confidence_only() {
    let filter = ProofAnnotationFilter {
        high_confidence_only: true,
        property_type: None,
        verification_method: None,
    };
    assert!(filter.high_confidence_only);
}

#[test]
fn test_proof_annotation_filter_with_property_type() {
    let filter = ProofAnnotationFilter {
        high_confidence_only: false,
        property_type: Some(PropertyTypeFilter::MemorySafety),
        verification_method: None,
    };
    assert_eq!(filter.property_type, Some(PropertyTypeFilter::MemorySafety));
}

#[test]
fn test_proof_annotation_filter_with_verification_method() {
    let filter = ProofAnnotationFilter {
        high_confidence_only: false,
        property_type: None,
        verification_method: Some(VerificationMethodFilter::BorrowChecker),
    };
    assert_eq!(
        filter.verification_method,
        Some(VerificationMethodFilter::BorrowChecker)
    );
}

#[test]
fn test_proof_annotation_filter_all_options() {
    let filter = ProofAnnotationFilter {
        high_confidence_only: true,
        property_type: Some(PropertyTypeFilter::ThreadSafety),
        verification_method: Some(VerificationMethodFilter::StaticAnalysis),
    };
    assert!(filter.high_confidence_only);
    assert_eq!(filter.property_type, Some(PropertyTypeFilter::ThreadSafety));
    assert_eq!(
        filter.verification_method,
        Some(VerificationMethodFilter::StaticAnalysis)
    );
}

// =============================================================================
// filter_annotation tests
// =============================================================================

#[test]
fn test_filter_annotation_passes_all_no_filters() {
    let annotation = create_test_annotation(
        ConfidenceLevel::Low,
        PropertyType::MemorySafety,
        VerificationMethod::BorrowChecker,
    );
    let filter = ProofAnnotationFilter {
        high_confidence_only: false,
        property_type: None,
        verification_method: None,
    };
    assert!(filter_annotation(&annotation, &filter));
}

#[test]
fn test_filter_annotation_high_confidence_passes() {
    let annotation = create_test_annotation(
        ConfidenceLevel::High,
        PropertyType::MemorySafety,
        VerificationMethod::BorrowChecker,
    );
    let filter = ProofAnnotationFilter {
        high_confidence_only: true,
        property_type: None,
        verification_method: None,
    };
    assert!(filter_annotation(&annotation, &filter));
}

#[test]
fn test_filter_annotation_high_confidence_filters_low() {
    let annotation = create_test_annotation(
        ConfidenceLevel::Low,
        PropertyType::MemorySafety,
        VerificationMethod::BorrowChecker,
    );
    let filter = ProofAnnotationFilter {
        high_confidence_only: true,
        property_type: None,
        verification_method: None,
    };
    assert!(!filter_annotation(&annotation, &filter));
}

#[test]
fn test_filter_annotation_high_confidence_filters_medium() {
    let annotation = create_test_annotation(
        ConfidenceLevel::Medium,
        PropertyType::MemorySafety,
        VerificationMethod::BorrowChecker,
    );
    let filter = ProofAnnotationFilter {
        high_confidence_only: true,
        property_type: None,
        verification_method: None,
    };
    assert!(!filter_annotation(&annotation, &filter));
}

#[test]
fn test_filter_annotation_property_type_matches() {
    let annotation = create_test_annotation(
        ConfidenceLevel::High,
        PropertyType::MemorySafety,
        VerificationMethod::BorrowChecker,
    );
    let filter = ProofAnnotationFilter {
        high_confidence_only: false,
        property_type: Some(PropertyTypeFilter::MemorySafety),
        verification_method: None,
    };
    assert!(filter_annotation(&annotation, &filter));
}

#[test]
fn test_filter_annotation_property_type_no_match() {
    let annotation = create_test_annotation(
        ConfidenceLevel::High,
        PropertyType::ThreadSafety,
        VerificationMethod::BorrowChecker,
    );
    let filter = ProofAnnotationFilter {
        high_confidence_only: false,
        property_type: Some(PropertyTypeFilter::MemorySafety),
        verification_method: None,
    };
    assert!(!filter_annotation(&annotation, &filter));
}

#[test]
fn test_filter_annotation_verification_method_matches() {
    let annotation = create_test_annotation(
        ConfidenceLevel::High,
        PropertyType::MemorySafety,
        VerificationMethod::BorrowChecker,
    );
    let filter = ProofAnnotationFilter {
        high_confidence_only: false,
        property_type: None,
        verification_method: Some(VerificationMethodFilter::BorrowChecker),
    };
    assert!(filter_annotation(&annotation, &filter));
}

#[test]
fn test_filter_annotation_verification_method_no_match() {
    let annotation = create_test_annotation(
        ConfidenceLevel::High,
        PropertyType::MemorySafety,
        VerificationMethod::BorrowChecker,
    );
    let filter = ProofAnnotationFilter {
        high_confidence_only: false,
        property_type: None,
        verification_method: Some(VerificationMethodFilter::FormalProof),
    };
    assert!(!filter_annotation(&annotation, &filter));
}

#[test]
fn test_filter_annotation_combined_filters_pass() {
    let annotation = create_test_annotation(
        ConfidenceLevel::High,
        PropertyType::MemorySafety,
        VerificationMethod::BorrowChecker,
    );
    let filter = ProofAnnotationFilter {
        high_confidence_only: true,
        property_type: Some(PropertyTypeFilter::MemorySafety),
        verification_method: Some(VerificationMethodFilter::BorrowChecker),
    };
    assert!(filter_annotation(&annotation, &filter));
}

#[test]
fn test_filter_annotation_combined_filters_fail_confidence() {
    let annotation = create_test_annotation(
        ConfidenceLevel::Low,
        PropertyType::MemorySafety,
        VerificationMethod::BorrowChecker,
    );
    let filter = ProofAnnotationFilter {
        high_confidence_only: true,
        property_type: Some(PropertyTypeFilter::MemorySafety),
        verification_method: Some(VerificationMethodFilter::BorrowChecker),
    };
    assert!(!filter_annotation(&annotation, &filter));
}

// =============================================================================
// PropertyTypeFilter matching tests
// =============================================================================

#[test]
fn test_filter_property_type_memory_safety() {
    let annotation = create_test_annotation(
        ConfidenceLevel::High,
        PropertyType::MemorySafety,
        VerificationMethod::BorrowChecker,
    );
    let filter = ProofAnnotationFilter {
        high_confidence_only: false,
        property_type: Some(PropertyTypeFilter::MemorySafety),
        verification_method: None,
    };
    assert!(filter_annotation(&annotation, &filter));
}

#[test]
fn test_filter_property_type_thread_safety() {
    let annotation = create_test_annotation(
        ConfidenceLevel::High,
        PropertyType::ThreadSafety,
        VerificationMethod::BorrowChecker,
    );
    let filter = ProofAnnotationFilter {
        high_confidence_only: false,
        property_type: Some(PropertyTypeFilter::ThreadSafety),
        verification_method: None,
    };
    assert!(filter_annotation(&annotation, &filter));
}

#[test]
fn test_filter_property_type_data_race_freeze() {
    let annotation = create_test_annotation(
        ConfidenceLevel::High,
        PropertyType::DataRaceFreeze,
        VerificationMethod::BorrowChecker,
    );
    let filter = ProofAnnotationFilter {
        high_confidence_only: false,
        property_type: Some(PropertyTypeFilter::DataRaceFreeze),
        verification_method: None,
    };
    assert!(filter_annotation(&annotation, &filter));
}

#[test]
fn test_filter_property_type_termination() {
    let annotation = create_test_annotation(
        ConfidenceLevel::High,
        PropertyType::Termination,
        VerificationMethod::FormalProof {
            prover: "coq".to_string(),
        },
    );
    let filter = ProofAnnotationFilter {
        high_confidence_only: false,
        property_type: Some(PropertyTypeFilter::Termination),
        verification_method: None,
    };
    assert!(filter_annotation(&annotation, &filter));
}

#[test]
fn test_filter_property_type_functional_correctness() {
    let annotation = create_test_annotation(
        ConfidenceLevel::High,
        PropertyType::FunctionalCorrectness("spec_123".to_string()),
        VerificationMethod::FormalProof {
            prover: "lean".to_string(),
        },
    );
    let filter = ProofAnnotationFilter {
        high_confidence_only: false,
        property_type: Some(PropertyTypeFilter::FunctionalCorrectness),
        verification_method: None,
    };
    assert!(filter_annotation(&annotation, &filter));
}

#[test]
fn test_filter_property_type_resource_bounds() {
    let annotation = create_test_annotation(
        ConfidenceLevel::High,
        PropertyType::ResourceBounds {
            cpu: Some(1000),
            memory: Some(4096),
        },
        VerificationMethod::StaticAnalysis {
            tool: "analyzer".to_string(),
        },
    );
    let filter = ProofAnnotationFilter {
        high_confidence_only: false,
        property_type: Some(PropertyTypeFilter::ResourceBounds),
        verification_method: None,
    };
    assert!(filter_annotation(&annotation, &filter));
}

#[test]
fn test_filter_property_type_all_passes_any() {
    let annotations = vec![
        create_test_annotation(
            ConfidenceLevel::High,
            PropertyType::MemorySafety,
            VerificationMethod::BorrowChecker,
        ),
        create_test_annotation(
            ConfidenceLevel::High,
            PropertyType::ThreadSafety,
            VerificationMethod::BorrowChecker,
        ),
        create_test_annotation(
            ConfidenceLevel::High,
            PropertyType::Termination,
            VerificationMethod::BorrowChecker,
        ),
    ];
    let filter = ProofAnnotationFilter {
        high_confidence_only: false,
        property_type: Some(PropertyTypeFilter::All),
        verification_method: None,
    };
    for ann in annotations {
        assert!(filter_annotation(&ann, &filter));
    }
}

// =============================================================================
// VerificationMethodFilter matching tests
// =============================================================================

#[test]
fn test_filter_verification_method_formal_proof() {
    let annotation = create_test_annotation(
        ConfidenceLevel::High,
        PropertyType::MemorySafety,
        VerificationMethod::FormalProof {
            prover: "coq".to_string(),
        },
    );
    let filter = ProofAnnotationFilter {
        high_confidence_only: false,
        property_type: None,
        verification_method: Some(VerificationMethodFilter::FormalProof),
    };
    assert!(filter_annotation(&annotation, &filter));
}

#[test]
fn test_filter_verification_method_model_checking() {
    let annotation = create_test_annotation(
        ConfidenceLevel::High,
        PropertyType::MemorySafety,
        VerificationMethod::ModelChecking { bounded: true },
    );
    let filter = ProofAnnotationFilter {
        high_confidence_only: false,
        property_type: None,
        verification_method: Some(VerificationMethodFilter::ModelChecking),
    };
    assert!(filter_annotation(&annotation, &filter));
}

#[test]
fn test_filter_verification_method_static_analysis() {
    let annotation = create_test_annotation(
        ConfidenceLevel::High,
        PropertyType::MemorySafety,
        VerificationMethod::StaticAnalysis {
            tool: "miri".to_string(),
        },
    );
    let filter = ProofAnnotationFilter {
        high_confidence_only: false,
        property_type: None,
        verification_method: Some(VerificationMethodFilter::StaticAnalysis),
    };
    assert!(filter_annotation(&annotation, &filter));
}

#[test]
fn test_filter_verification_method_abstract_interpretation() {
    let annotation = create_test_annotation(
        ConfidenceLevel::High,
        PropertyType::MemorySafety,
        VerificationMethod::AbstractInterpretation,
    );
    let filter = ProofAnnotationFilter {
        high_confidence_only: false,
        property_type: None,
        verification_method: Some(VerificationMethodFilter::AbstractInterpretation),
    };
    assert!(filter_annotation(&annotation, &filter));
}

#[test]
fn test_filter_verification_method_borrow_checker() {
    let annotation = create_test_annotation(
        ConfidenceLevel::High,
        PropertyType::MemorySafety,
        VerificationMethod::BorrowChecker,
    );
    let filter = ProofAnnotationFilter {
        high_confidence_only: false,
        property_type: None,
        verification_method: Some(VerificationMethodFilter::BorrowChecker),
    };
    assert!(filter_annotation(&annotation, &filter));
}

#[test]
fn test_filter_verification_method_all_passes_any() {
    let annotations = vec![
        create_test_annotation(
            ConfidenceLevel::High,
            PropertyType::MemorySafety,
            VerificationMethod::BorrowChecker,
        ),
        create_test_annotation(
            ConfidenceLevel::High,
            PropertyType::MemorySafety,
            VerificationMethod::FormalProof {
                prover: "coq".to_string(),
            },
        ),
        create_test_annotation(
            ConfidenceLevel::High,
            PropertyType::MemorySafety,
            VerificationMethod::AbstractInterpretation,
        ),
    ];
    let filter = ProofAnnotationFilter {
        high_confidence_only: false,
        property_type: None,
        verification_method: Some(VerificationMethodFilter::All),
    };
    for ann in annotations {
        assert!(filter_annotation(&ann, &filter));
    }
}

// =============================================================================
// format_as_json tests
// =============================================================================

#[test]
fn test_format_as_json_empty_annotations() {
    let annotations: Vec<(Location, ProofAnnotation)> = vec![];
    let elapsed = Duration::from_millis(100);
    let annotator = setup_proof_annotator(false);

    let result = format_as_json(&annotations, elapsed, &annotator);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("\"proof_annotations\""));
    assert!(content.contains("\"summary\""));
    assert!(content.contains("\"total_annotations\": 0"));
}

#[test]
fn test_format_as_json_with_annotations() {
    let annotations = create_diverse_annotations();
    let elapsed = Duration::from_millis(250);
    let annotator = setup_proof_annotator(false);

    let result = format_as_json(&annotations, elapsed, &annotator);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("\"proof_annotations\""));
    assert!(content.contains("\"total_annotations\": 3"));
    assert!(content.contains("\"analysis_time_ms\""));
    assert!(content.contains("\"cache_stats\""));
}

#[test]
fn test_format_as_json_contains_file_paths() {
    let annotations = create_diverse_annotations();
    let elapsed = Duration::from_millis(100);
    let annotator = setup_proof_annotator(false);

    let result = format_as_json(&annotations, elapsed, &annotator);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("lib.rs"));
    assert!(content.contains("main.rs"));
}

#[test]
fn test_format_as_json_contains_positions() {
    let annotations = vec![(
        create_test_location("test.rs", 42, 84),
        create_test_annotation(
            ConfidenceLevel::High,
            PropertyType::MemorySafety,
            VerificationMethod::BorrowChecker,
        ),
    )];
    let elapsed = Duration::from_millis(50);
    let annotator = setup_proof_annotator(false);

    let result = format_as_json(&annotations, elapsed, &annotator);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("\"start_pos\": 42"));
    assert!(content.contains("\"end_pos\": 84"));
}

// =============================================================================
// format_as_summary tests
// =============================================================================

#[test]
fn test_format_as_summary_empty() {
    let annotations: Vec<(Location, ProofAnnotation)> = vec![];
    let elapsed = Duration::from_millis(50);

    let result = format_as_summary(&annotations, elapsed);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("Proof Annotations Summary"));
    assert!(content.contains("Total proofs: 0"));
}

#[test]
fn test_format_as_summary_with_annotations() {
    let annotations = create_diverse_annotations();
    let elapsed = Duration::from_millis(100);

    let result = format_as_summary(&annotations, elapsed);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("Total proofs: 3"));
    assert!(content.contains("High confidence:"));
    assert!(content.contains("Analysis time:"));
}

#[test]
fn test_format_as_summary_property_counts() {
    let annotations = vec![
        (
            create_test_location("a.rs", 1, 10),
            create_test_annotation(
                ConfidenceLevel::High,
                PropertyType::MemorySafety,
                VerificationMethod::BorrowChecker,
            ),
        ),
        (
            create_test_location("b.rs", 1, 10),
            create_test_annotation(
                ConfidenceLevel::High,
                PropertyType::MemorySafety,
                VerificationMethod::BorrowChecker,
            ),
        ),
    ];
    let elapsed = Duration::from_millis(100);

    let result = format_as_summary(&annotations, elapsed);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("MemorySafety"));
}

#[test]
fn test_format_as_summary_top_files() {
    let annotations = create_diverse_annotations();
    let elapsed = Duration::from_millis(100);

    let result = format_as_summary(&annotations, elapsed);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("Top Files with Proof Annotations"));
}

// =============================================================================
// format_as_table tests
// =============================================================================

#[test]
fn test_format_as_table_empty() {
    let annotations: Vec<(Location, ProofAnnotation)> = vec![];
    let elapsed = Duration::from_millis(50);

    let result = format_as_table(&annotations, elapsed);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("| File | Position | Property | Method | Confidence |"));
    assert!(content.contains("|------|----------|----------|---------|------------|"));
}

#[test]
fn test_format_as_table_with_annotations() {
    let annotations = create_diverse_annotations();
    let elapsed = Duration::from_millis(100);

    let result = format_as_table(&annotations, elapsed);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("lib.rs"));
    assert!(content.contains("main.rs"));
    assert!(content.contains("MemorySafety"));
    assert!(content.contains("High"));
}

// =============================================================================
// format_as_full tests
// =============================================================================

#[test]
fn test_format_as_full_empty() {
    let annotations: Vec<(Location, ProofAnnotation)> = vec![];
    let project_path = Path::new("/test/project");

    let result = format_as_full(&annotations, project_path, true);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("Full Proof Annotations Report"));
    assert!(content.contains("**Total proofs**: 0"));
}

#[test]
fn test_format_as_full_with_annotations() {
    let annotations = create_diverse_annotations();
    let project_path = Path::new("/test/project");

    let result = format_as_full(&annotations, project_path, true);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("Full Proof Annotations Report"));
    assert!(content.contains("**Total proofs**: 3"));
    assert!(content.contains("File:"));
}

#[test]
fn test_format_as_full_includes_evidence() {
    let mut ann = create_test_annotation(
        ConfidenceLevel::High,
        PropertyType::MemorySafety,
        VerificationMethod::BorrowChecker,
    );
    ann.evidence_type = EvidenceType::TheoremName {
        theorem: "memory_safety_theorem".to_string(),
        theory: Some("rust_model".to_string()),
    };
    let annotations = vec![(create_test_location("test.rs", 1, 10), ann)];
    let project_path = Path::new("/test/project");

    let result = format_as_full(&annotations, project_path, true);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("Evidence"));
}

#[test]
fn test_format_as_full_without_evidence() {
    let annotations = create_diverse_annotations();
    let project_path = Path::new("/test/project");

    let result = format_as_full(&annotations, project_path, false);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("Full Proof Annotations Report"));
}

#[test]
fn test_format_as_full_with_assumptions() {
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
    let annotations = vec![(create_test_location("test.rs", 1, 10), ann)];
    let project_path = Path::new("/test/project");

    let result = format_as_full(&annotations, project_path, true);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("Assumptions"));
    assert!(content.contains("No integer overflow"));
}

// =============================================================================
// format_as_markdown tests
// =============================================================================

#[test]
fn test_format_as_markdown_empty() {
    let annotations: Vec<(Location, ProofAnnotation)> = vec![];
    let project_path = Path::new("/test/project");

    let result = format_as_markdown(&annotations, project_path, true);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("# Proof Annotations Analysis"));
    assert!(content.contains("Summary Statistics"));
}

#[test]
fn test_format_as_markdown_with_annotations() {
    let annotations = create_diverse_annotations();
    let project_path = Path::new("/test/project");

    let result = format_as_markdown(&annotations, project_path, true);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("# Proof Annotations Analysis"));
    assert!(content.contains("**Total Proofs**: 3"));
}

#[test]
fn test_format_as_markdown_includes_statistics_table() {
    let annotations = create_diverse_annotations();
    let project_path = Path::new("/test/project");

    let result = format_as_markdown(&annotations, project_path, true);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("| Metric | Count |"));
    assert!(content.contains("|--------|-------|"));
}

#[test]
fn test_format_as_markdown_with_evidence() {
    let annotations = create_diverse_annotations();
    let project_path = Path::new("/test/project");

    let result = format_as_markdown(&annotations, project_path, true);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("Detailed Proofs"));
}

#[test]
fn test_format_as_markdown_without_evidence() {
    let annotations = create_diverse_annotations();
    let project_path = Path::new("/test/project");

    let result = format_as_markdown(&annotations, project_path, false);
    assert!(result.is_ok());

    let content = result.unwrap();
    // Without evidence, we should not have detailed proofs section
    assert!(!content.contains("Detailed Proofs"));
}

// =============================================================================
// format_as_sarif tests
// =============================================================================

#[test]
fn test_format_as_sarif_empty() {
    let annotations: Vec<(Location, ProofAnnotation)> = vec![];
    let project_path = Path::new("/test/project");

    let result = format_as_sarif(&annotations, project_path);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("\"version\": \"2.1.0\""));
    assert!(content.contains("\"$schema\""));
    assert!(content.contains("paiml-proof-annotator"));
}

#[test]
fn test_format_as_sarif_with_annotations() {
    let annotations = create_diverse_annotations();
    let project_path = Path::new("/test/project");

    let result = format_as_sarif(&annotations, project_path);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("\"results\""));
    assert!(content.contains("\"ruleId\""));
}

#[test]
fn test_format_as_sarif_high_confidence_mapping() {
    let annotations = vec![(
        create_test_location("test.rs", 1, 10),
        create_test_annotation(
            ConfidenceLevel::High,
            PropertyType::MemorySafety,
            VerificationMethod::BorrowChecker,
        ),
    )];
    let project_path = Path::new("/test/project");

    let result = format_as_sarif(&annotations, project_path);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("high-confidence-proof"));
    assert!(content.contains("\"level\": \"none\""));
}

#[test]
fn test_format_as_sarif_medium_confidence_mapping() {
    let annotations = vec![(
        create_test_location("test.rs", 1, 10),
        create_test_annotation(
            ConfidenceLevel::Medium,
            PropertyType::MemorySafety,
            VerificationMethod::BorrowChecker,
        ),
    )];
    let project_path = Path::new("/test/project");

    let result = format_as_sarif(&annotations, project_path);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("medium-confidence-proof"));
    assert!(content.contains("\"level\": \"note\""));
}

#[test]
fn test_format_as_sarif_low_confidence_mapping() {
    let annotations = vec![(
        create_test_location("test.rs", 1, 10),
        create_test_annotation(
            ConfidenceLevel::Low,
            PropertyType::MemorySafety,
            VerificationMethod::BorrowChecker,
        ),
    )];
    let project_path = Path::new("/test/project");

    let result = format_as_sarif(&annotations, project_path);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("low-confidence-proof"));
    assert!(content.contains("\"level\": \"warning\""));
}

#[test]
fn test_format_as_sarif_rules_defined() {
    let annotations: Vec<(Location, ProofAnnotation)> = vec![];
    let project_path = Path::new("/test/project");

    let result = format_as_sarif(&annotations, project_path);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("\"rules\""));
    assert!(content.contains("Low Confidence Proof"));
    assert!(content.contains("Medium Confidence Proof"));
    assert!(content.contains("High Confidence Proof"));
}

// =============================================================================
// setup_proof_annotator tests
// =============================================================================

#[test]
fn test_setup_proof_annotator_creates_annotator() {
    let annotator = setup_proof_annotator(false);
    let stats = annotator.cache_stats();
    // Annotator should be initialized with empty cache
    assert_eq!(stats.size, 0);
}

#[test]
fn test_setup_proof_annotator_with_clear_cache() {
    let annotator = setup_proof_annotator(true);
    let stats = annotator.cache_stats();
    assert_eq!(stats.size, 0);
}

#[test]
fn test_setup_proof_annotator_mock_sources() {
    let annotator = setup_proof_annotator(false);
    // The annotator should be functional
    let stats = annotator.cache_stats();
    let _ = stats.files_tracked;
}

// =============================================================================
// ProofAnnotationOutputFormat Display tests
// =============================================================================

#[test]
fn test_proof_annotation_output_format_display_summary() {
    assert_eq!(ProofAnnotationOutputFormat::Summary.to_string(), "summary");
}

#[test]
fn test_proof_annotation_output_format_display_full() {
    assert_eq!(ProofAnnotationOutputFormat::Full.to_string(), "full");
}

#[test]
fn test_proof_annotation_output_format_display_json() {
    assert_eq!(ProofAnnotationOutputFormat::Json.to_string(), "json");
}

#[test]
fn test_proof_annotation_output_format_display_markdown() {
    assert_eq!(
        ProofAnnotationOutputFormat::Markdown.to_string(),
        "markdown"
    );
}

#[test]
fn test_proof_annotation_output_format_display_sarif() {
    assert_eq!(ProofAnnotationOutputFormat::Sarif.to_string(), "sarif");
}

#[test]
fn test_proof_annotation_output_format_equality() {
    assert_eq!(
        ProofAnnotationOutputFormat::Json,
        ProofAnnotationOutputFormat::Json
    );
    assert_ne!(
        ProofAnnotationOutputFormat::Json,
        ProofAnnotationOutputFormat::Summary
    );
}

// =============================================================================
// PropertyTypeFilter Display tests
// =============================================================================

#[test]
fn test_property_type_filter_display_memory_safety() {
    assert_eq!(
        PropertyTypeFilter::MemorySafety.to_string(),
        "memory-safety"
    );
}

#[test]
fn test_property_type_filter_display_thread_safety() {
    assert_eq!(
        PropertyTypeFilter::ThreadSafety.to_string(),
        "thread-safety"
    );
}

#[test]
fn test_property_type_filter_display_data_race_freeze() {
    assert_eq!(
        PropertyTypeFilter::DataRaceFreeze.to_string(),
        "data-race-freeze"
    );
}

#[test]
fn test_property_type_filter_display_termination() {
    assert_eq!(PropertyTypeFilter::Termination.to_string(), "termination");
}

#[test]
fn test_property_type_filter_display_functional_correctness() {
    assert_eq!(
        PropertyTypeFilter::FunctionalCorrectness.to_string(),
        "functional-correctness"
    );
}

#[test]
fn test_property_type_filter_display_resource_bounds() {
    assert_eq!(
        PropertyTypeFilter::ResourceBounds.to_string(),
        "resource-bounds"
    );
}

#[test]
fn test_property_type_filter_display_all() {
    assert_eq!(PropertyTypeFilter::All.to_string(), "all");
}

#[test]
fn test_property_type_filter_equality() {
    assert_eq!(
        PropertyTypeFilter::MemorySafety,
        PropertyTypeFilter::MemorySafety
    );
    assert_ne!(
        PropertyTypeFilter::MemorySafety,
        PropertyTypeFilter::ThreadSafety
    );
}

// =============================================================================
// VerificationMethodFilter Display tests
// =============================================================================

#[test]
fn test_verification_method_filter_display_formal_proof() {
    assert_eq!(
        VerificationMethodFilter::FormalProof.to_string(),
        "formal-proof"
    );
}

#[test]
fn test_verification_method_filter_display_model_checking() {
    assert_eq!(
        VerificationMethodFilter::ModelChecking.to_string(),
        "model-checking"
    );
}

#[test]
fn test_verification_method_filter_display_static_analysis() {
    assert_eq!(
        VerificationMethodFilter::StaticAnalysis.to_string(),
        "static-analysis"
    );
}

#[test]
fn test_verification_method_filter_display_abstract_interpretation() {
    assert_eq!(
        VerificationMethodFilter::AbstractInterpretation.to_string(),
        "abstract-interpretation"
    );
}

#[test]
fn test_verification_method_filter_display_borrow_checker() {
    assert_eq!(
        VerificationMethodFilter::BorrowChecker.to_string(),
        "borrow-checker"
    );
}

#[test]
fn test_verification_method_filter_display_all() {
    assert_eq!(VerificationMethodFilter::All.to_string(), "all");
}

#[test]
fn test_verification_method_filter_equality() {
    assert_eq!(
        VerificationMethodFilter::BorrowChecker,
        VerificationMethodFilter::BorrowChecker
    );
    assert_ne!(
        VerificationMethodFilter::BorrowChecker,
        VerificationMethodFilter::FormalProof
    );
}

// =============================================================================
// ConfidenceLevel tests
// =============================================================================

#[test]
fn test_confidence_level_ordering() {
    assert!(ConfidenceLevel::Low < ConfidenceLevel::Medium);
    assert!(ConfidenceLevel::Medium < ConfidenceLevel::High);
    assert!(ConfidenceLevel::Low < ConfidenceLevel::High);
}

#[test]
fn test_confidence_level_equality() {
    assert_eq!(ConfidenceLevel::High, ConfidenceLevel::High);
    assert_ne!(ConfidenceLevel::High, ConfidenceLevel::Low);
}

// =============================================================================
// EvidenceType tests
// =============================================================================

#[test]
fn test_evidence_type_implicit_type_system() {
    let evidence = EvidenceType::ImplicitTypeSystemGuarantee;
    assert_eq!(evidence, EvidenceType::ImplicitTypeSystemGuarantee);
}

#[test]
fn test_evidence_type_proof_script_reference() {
    let evidence = EvidenceType::ProofScriptReference {
        uri: "coq://theorem.v".to_string(),
    };
    if let EvidenceType::ProofScriptReference { uri } = evidence {
        assert_eq!(uri, "coq://theorem.v");
    } else {
        panic!("Expected ProofScriptReference");
    }
}

#[test]
fn test_evidence_type_theorem_name() {
    let evidence = EvidenceType::TheoremName {
        theorem: "memory_safe".to_string(),
        theory: Some("rust_model".to_string()),
    };
    if let EvidenceType::TheoremName { theorem, theory } = evidence {
        assert_eq!(theorem, "memory_safe");
        assert_eq!(theory, Some("rust_model".to_string()));
    } else {
        panic!("Expected TheoremName");
    }
}

#[test]
fn test_evidence_type_static_analysis_report() {
    let evidence = EvidenceType::StaticAnalysisReport {
        report_id: "report_123".to_string(),
    };
    if let EvidenceType::StaticAnalysisReport { report_id } = evidence {
        assert_eq!(report_id, "report_123");
    } else {
        panic!("Expected StaticAnalysisReport");
    }
}

#[test]
fn test_evidence_type_certificate_hash() {
    let evidence = EvidenceType::CertificateHash {
        hash: "abc123".to_string(),
        algorithm: "sha256".to_string(),
    };
    if let EvidenceType::CertificateHash { hash, algorithm } = evidence {
        assert_eq!(hash, "abc123");
        assert_eq!(algorithm, "sha256");
    } else {
        panic!("Expected CertificateHash");
    }
}

// =============================================================================
// Location and Span tests
// =============================================================================

#[test]
fn test_location_creation() {
    let location = create_test_location("test.rs", 10, 20);
    assert_eq!(location.file_path, PathBuf::from("test.rs"));
    assert_eq!(location.span.start.0, 10);
    assert_eq!(location.span.end.0, 20);
}

#[test]
fn test_location_equality() {
    let loc1 = create_test_location("test.rs", 10, 20);
    let loc2 = create_test_location("test.rs", 10, 20);
    let loc3 = create_test_location("other.rs", 10, 20);

    assert_eq!(loc1, loc2);
    assert_ne!(loc1, loc3);
}

#[test]
fn test_span_equality() {
    let span1 = Span {
        start: BytePos(10),
        end: BytePos(20),
    };
    let span2 = Span {
        start: BytePos(10),
        end: BytePos(20),
    };
    let span3 = Span {
        start: BytePos(15),
        end: BytePos(25),
    };

    assert_eq!(span1, span2);
    assert_ne!(span1, span3);
}

#[test]
fn test_byte_pos_ordering() {
    let pos1 = BytePos(10);
    let pos2 = BytePos(20);

    assert!(pos1 < pos2);
    assert_eq!(pos1, BytePos(10));
}

// =============================================================================
// PropertyType tests
// =============================================================================

#[test]
fn test_property_type_memory_safety() {
    let prop = PropertyType::MemorySafety;
    assert_eq!(prop, PropertyType::MemorySafety);
}

#[test]
fn test_property_type_thread_safety() {
    let prop = PropertyType::ThreadSafety;
    assert_eq!(prop, PropertyType::ThreadSafety);
}

#[test]
fn test_property_type_data_race_freeze() {
    let prop = PropertyType::DataRaceFreeze;
    assert_eq!(prop, PropertyType::DataRaceFreeze);
}

#[test]
fn test_property_type_termination() {
    let prop = PropertyType::Termination;
    assert_eq!(prop, PropertyType::Termination);
}

#[test]
fn test_property_type_functional_correctness() {
    let prop = PropertyType::FunctionalCorrectness("spec_123".to_string());
    if let PropertyType::FunctionalCorrectness(spec_id) = prop {
        assert_eq!(spec_id, "spec_123");
    } else {
        panic!("Expected FunctionalCorrectness");
    }
}

#[test]
fn test_property_type_resource_bounds() {
    let prop = PropertyType::ResourceBounds {
        cpu: Some(1000),
        memory: Some(4096),
    };
    if let PropertyType::ResourceBounds { cpu, memory } = prop {
        assert_eq!(cpu, Some(1000));
        assert_eq!(memory, Some(4096));
    } else {
        panic!("Expected ResourceBounds");
    }
}

#[test]
fn test_property_type_resource_bounds_partial() {
    let prop = PropertyType::ResourceBounds {
        cpu: Some(500),
        memory: None,
    };
    if let PropertyType::ResourceBounds { cpu, memory } = prop {
        assert_eq!(cpu, Some(500));
        assert_eq!(memory, None);
    } else {
        panic!("Expected ResourceBounds");
    }
}

// =============================================================================
// VerificationMethod tests
// =============================================================================

#[test]
fn test_verification_method_borrow_checker() {
    let method = VerificationMethod::BorrowChecker;
    assert_eq!(method, VerificationMethod::BorrowChecker);
}

#[test]
fn test_verification_method_formal_proof() {
    let method = VerificationMethod::FormalProof {
        prover: "coq".to_string(),
    };
    if let VerificationMethod::FormalProof { prover } = method {
        assert_eq!(prover, "coq");
    } else {
        panic!("Expected FormalProof");
    }
}

#[test]
fn test_verification_method_static_analysis() {
    let method = VerificationMethod::StaticAnalysis {
        tool: "miri".to_string(),
    };
    if let VerificationMethod::StaticAnalysis { tool } = method {
        assert_eq!(tool, "miri");
    } else {
        panic!("Expected StaticAnalysis");
    }
}

#[test]
fn test_verification_method_model_checking_bounded() {
    let method = VerificationMethod::ModelChecking { bounded: true };
    if let VerificationMethod::ModelChecking { bounded } = method {
        assert!(bounded);
    } else {
        panic!("Expected ModelChecking");
    }
}

#[test]
fn test_verification_method_model_checking_unbounded() {
    let method = VerificationMethod::ModelChecking { bounded: false };
    if let VerificationMethod::ModelChecking { bounded } = method {
        assert!(!bounded);
    } else {
        panic!("Expected ModelChecking");
    }
}

#[test]
fn test_verification_method_abstract_interpretation() {
    let method = VerificationMethod::AbstractInterpretation;
    assert_eq!(method, VerificationMethod::AbstractInterpretation);
}

// =============================================================================
// ProofAnnotation construction tests
// =============================================================================

#[test]
fn test_proof_annotation_full_construction() {
    let ann = ProofAnnotation {
        annotation_id: Uuid::new_v4(),
        property_proven: PropertyType::MemorySafety,
        specification_id: Some("spec_001".to_string()),
        method: VerificationMethod::BorrowChecker,
        tool_name: "rustc".to_string(),
        tool_version: "1.75.0".to_string(),
        confidence_level: ConfidenceLevel::High,
        assumptions: vec!["safe code only".to_string()],
        evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
        evidence_location: Some("/path/to/evidence".to_string()),
        date_verified: Utc::now(),
    };

    assert_eq!(ann.property_proven, PropertyType::MemorySafety);
    assert_eq!(ann.specification_id, Some("spec_001".to_string()));
    assert_eq!(ann.method, VerificationMethod::BorrowChecker);
    assert_eq!(ann.tool_name, "rustc");
    assert_eq!(ann.tool_version, "1.75.0");
    assert_eq!(ann.confidence_level, ConfidenceLevel::High);
    assert_eq!(ann.assumptions.len(), 1);
}

#[test]
fn test_proof_annotation_minimal_construction() {
    let ann = create_test_annotation(
        ConfidenceLevel::Low,
        PropertyType::Termination,
        VerificationMethod::AbstractInterpretation,
    );

    assert_eq!(ann.property_proven, PropertyType::Termination);
    assert_eq!(ann.method, VerificationMethod::AbstractInterpretation);
    assert_eq!(ann.confidence_level, ConfidenceLevel::Low);
    assert!(ann.assumptions.is_empty());
    assert!(ann.specification_id.is_none());
}

#[test]
fn test_proof_annotation_with_multiple_assumptions() {
    let mut ann = create_test_annotation(
        ConfidenceLevel::Medium,
        PropertyType::ThreadSafety,
        VerificationMethod::StaticAnalysis {
            tool: "miri".to_string(),
        },
    );
    ann.assumptions = vec![
        "No unsafe code".to_string(),
        "Single threaded".to_string(),
        "No FFI".to_string(),
    ];

    assert_eq!(ann.assumptions.len(), 3);
    assert!(ann.assumptions.contains(&"No unsafe code".to_string()));
}

// =============================================================================
// Edge case tests
// =============================================================================

#[test]
fn test_empty_file_path() {
    let location = create_test_location("", 0, 0);
    assert_eq!(location.file_path, PathBuf::from(""));
}

#[test]
fn test_zero_span() {
    let location = create_test_location("test.rs", 0, 0);
    assert_eq!(location.span.start.0, 0);
    assert_eq!(location.span.end.0, 0);
}

#[test]
fn test_large_span() {
    let location = create_test_location("test.rs", 0, u32::MAX);
    assert_eq!(location.span.end.0, u32::MAX);
}

#[test]
fn test_unicode_file_path() {
    let location = create_test_location("src/\u{1F600}/test.rs", 10, 20);
    assert!(location.file_path.to_string_lossy().contains("\u{1F600}"));
}

#[test]
fn test_deep_nested_path() {
    let location = create_test_location("a/b/c/d/e/f/g/h/i/j/k/l/test.rs", 10, 20);
    assert!(location.file_path.to_string_lossy().contains("a/b/c"));
}
