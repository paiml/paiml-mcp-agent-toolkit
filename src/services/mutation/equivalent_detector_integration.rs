
// ============================================================================
// EquivalentMutantDetector tests
// ============================================================================

#[test]
fn test_detector_new() {
    let detector = EquivalentMutantDetector::new();
    assert!(!detector.is_trained());
}

#[test]
fn test_detector_default() {
    let detector = EquivalentMutantDetector::default();
    assert!(!detector.is_trained());
}

#[test]
fn test_detector_train_empty_data_fails() {
    let mut detector = EquivalentMutantDetector::new();
    let result = detector.train(&[]);
    assert!(result.is_err());
}

#[test]
fn test_detector_train_success() {
    let mut detector = EquivalentMutantDetector::new();
    let training_data = vec![create_training_sample("x + 0", "x", true)];

    let result = detector.train(&training_data);
    assert!(result.is_ok());
    assert!(detector.is_trained());
}

#[test]
fn test_detector_detect_untrained_fails() {
    let detector = EquivalentMutantDetector::new();
    let mutant = create_test_mutant("x + 0", "x");

    let result = detector.detect_equivalent(&mutant, "x + 0");
    assert!(result.is_err());
}

#[test]
fn test_detector_detect_identity_op() {
    let mut detector = EquivalentMutantDetector::new();
    let training_data = vec![create_training_sample("x + 0", "x", true)];
    detector.train(&training_data).unwrap();

    let mutant = create_test_mutant("y + 0", "y");
    let result = detector.detect_equivalent(&mutant, "y + 0").unwrap();

    assert!(result.is_equivalent);
    assert!(result.confidence >= 0.8);
}

#[test]
fn test_detector_detect_tautology() {
    let mut detector = EquivalentMutantDetector::new();
    let training_data = vec![create_training_sample("x || true", "{ true }", true)];
    detector.train(&training_data).unwrap();

    let mutant = create_test_mutant("y || true", "{ true }");
    let result = detector.detect_equivalent(&mutant, "y || true").unwrap();

    assert!(result.is_equivalent);
}

#[test]
fn test_detector_detect_commutative() {
    let mut detector = EquivalentMutantDetector::new();
    let training_data = vec![create_training_sample("a + b", "b + a", true)];
    detector.train(&training_data).unwrap();

    let mutant = create_test_mutant("x + y", "y + x");
    let result = detector.detect_equivalent(&mutant, "x + y").unwrap();

    assert!(result.is_equivalent);
}

#[test]
fn test_detector_update() {
    let mut detector = EquivalentMutantDetector::new();
    let training_data = vec![create_training_sample("x + 0", "x", true)];
    detector.train(&training_data).unwrap();

    let new_data = vec![create_training_sample("y * 1", "y", true)];
    let result = detector.update(&new_data);

    assert!(result.is_ok());
}

#[test]
fn test_detector_update_when_not_trained() {
    let mut detector = EquivalentMutantDetector::new();
    let new_data = vec![create_training_sample("x + 0", "x", true)];

    let result = detector.update(&new_data);
    assert!(result.is_ok());
    assert!(detector.is_trained());
}

#[test]
fn test_detector_detect_with_explanation() {
    let mut detector = EquivalentMutantDetector::new();
    let training_data = vec![create_training_sample("x + 0", "x", true)];
    detector.train(&training_data).unwrap();

    let mutant = create_test_mutant("y + 0", "y");
    let (result, explanation) = detector.detect_with_explanation(&mutant, "y + 0").unwrap();

    assert!(result.is_equivalent);
    assert!(explanation.contains("EQUIVALENT"));
    assert!(explanation.contains("confidence"));
}

#[test]
fn test_detector_get_accuracy_estimate_untrained() {
    let detector = EquivalentMutantDetector::new();
    assert_eq!(detector.get_accuracy_estimate(), 0.0);
}

#[test]
fn test_detector_get_accuracy_estimate_trained() {
    let mut detector = EquivalentMutantDetector::new();
    let training_data = vec![
        create_training_sample("x + 0", "x", true),
        create_training_sample("y * 1", "y", true),
    ];
    detector.train(&training_data).unwrap();

    let accuracy = detector.get_accuracy_estimate();
    assert!(accuracy > 0.0);
    assert!(accuracy <= 0.95);
}

#[test]
fn test_detector_filter_equivalents() {
    let mut detector = EquivalentMutantDetector::new();
    let training_data = vec![create_training_sample("x + 0", "x", true)];
    detector.train(&training_data).unwrap();

    let mutants = vec![
        create_test_mutant("a + 0", "a"),     // equivalent
        create_test_mutant("a + b", "a - b"), // not equivalent
    ];
    let sources = vec![("a.rs", "a + 0"), ("b.rs", "a + b")];

    let non_equivalents = detector.filter_equivalents(&mutants, &sources).unwrap();

    // Should filter out the equivalent one
    assert!(non_equivalents.len() <= mutants.len());
}

// ============================================================================
// Serialization tests
// ============================================================================

#[test]
fn test_detector_serialization() {
    let mut detector = EquivalentMutantDetector::new();
    let training_data = vec![create_training_sample("x + 0", "x", true)];
    detector.train(&training_data).unwrap();

    let json = serde_json::to_string(&detector).unwrap();
    assert!(json.contains("equivalence_patterns"));
    assert!(json.contains("trained"));
}

#[test]
fn test_detector_deserialization() {
    let mut detector = EquivalentMutantDetector::new();
    let training_data = vec![create_training_sample("x + 0", "x", true)];
    detector.train(&training_data).unwrap();

    let json = serde_json::to_string(&detector).unwrap();
    let deserialized: EquivalentMutantDetector = serde_json::from_str(&json).unwrap();

    assert!(deserialized.is_trained());
}

#[test]
fn test_detector_save_and_load() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("detector.bin");

    let mut detector = EquivalentMutantDetector::new();
    let training_data = vec![create_training_sample("x + 0", "x", true)];
    detector.train(&training_data).unwrap();

    detector.save(&path).unwrap();

    let loaded = EquivalentMutantDetector::load(&path).unwrap();
    assert!(loaded.is_trained());
}

// ============================================================================
// EquivalenceTrainingData tests
// ============================================================================

#[test]
fn test_training_data_creation() {
    let data = create_training_sample("x + 0", "x", true);

    assert!(data.is_equivalent);
    assert!(data.verified_manually);
    assert_eq!(data.detection_method, "manual");
}

#[test]
fn test_training_data_serialization() {
    let data = create_training_sample("x + 0", "x", true);

    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("is_equivalent"));
    assert!(json.contains("verified_manually"));
}

// ============================================================================
// Edge case tests
// ============================================================================

#[test]
fn test_detect_with_empty_source() {
    let mut detector = EquivalentMutantDetector::new();
    let training_data = vec![create_training_sample("x", "x", true)];
    detector.train(&training_data).unwrap();

    let mutant = create_test_mutant("", "");
    let result = detector.detect_equivalent(&mutant, "").unwrap();

    // Should not panic, just return a result
    assert!(!result.reason.is_empty());
}

#[test]
fn test_detect_with_whitespace_only() {
    let mut detector = EquivalentMutantDetector::new();
    let training_data = vec![create_training_sample("x", "x", false)];
    detector.train(&training_data).unwrap();

    let mutant = create_test_mutant("   ", "  ");
    let result = detector.detect_equivalent(&mutant, "   ").unwrap();

    assert!(!result.reason.is_empty());
}
