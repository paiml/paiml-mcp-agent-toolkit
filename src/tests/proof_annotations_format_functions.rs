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
