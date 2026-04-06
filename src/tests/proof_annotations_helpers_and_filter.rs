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
