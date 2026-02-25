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
