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
