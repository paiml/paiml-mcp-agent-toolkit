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
