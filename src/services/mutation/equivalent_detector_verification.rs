fn create_test_mutant(original: &str, mutated: &str) -> Mutant {
    Mutant {
        id: "test_mutant".to_string(),
        original_file: std::path::PathBuf::from("test.rs"),
        location: SourceLocation {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 10,
        },
        operator: MutationOperatorType::ArithmeticReplacement,
        mutated_source: mutated.to_string(),
        // `hash` (dedup key) and `status` were added to `Mutant`; a
        // freshly-built fixture has not been executed yet.
        // Derived from both sides: `Mutant` no longer stores the original
        // text, and callers pass distinct originals expecting distinct
        // mutants. A constant here would collapse them.
        hash: format!("{original}=>{mutated}"),
        status: crate::services::mutation::types::MutantStatus::Pending,
    }
}

fn create_training_sample(
    original: &str,
    mutated: &str,
    is_equivalent: bool,
) -> EquivalenceTrainingData {
    EquivalenceTrainingData {
        mutant: create_test_mutant(original, mutated),
        original_source: original.to_string(),
        is_equivalent,
        verified_manually: true,
        detection_method: "manual".to_string(),
    }
}

// ============================================================================
// levenshtein_distance tests
// ============================================================================

#[test]
fn test_levenshtein_distance_identical() {
    assert_eq!(levenshtein_distance("hello", "hello"), 0);
}

#[test]
fn test_levenshtein_distance_one_char_diff() {
    assert_eq!(levenshtein_distance("hello", "hallo"), 1);
}

#[test]
fn test_levenshtein_distance_insertion() {
    assert_eq!(levenshtein_distance("hello", "helloo"), 1);
}

#[test]
fn test_levenshtein_distance_deletion() {
    assert_eq!(levenshtein_distance("hello", "helo"), 1);
}

#[test]
fn test_levenshtein_distance_empty_first() {
    assert_eq!(levenshtein_distance("", "hello"), 5);
}

#[test]
fn test_levenshtein_distance_empty_second() {
    assert_eq!(levenshtein_distance("hello", ""), 5);
}

#[test]
fn test_levenshtein_distance_both_empty() {
    assert_eq!(levenshtein_distance("", ""), 0);
}

#[test]
fn test_levenshtein_distance_completely_different() {
    assert_eq!(levenshtein_distance("abc", "xyz"), 3);
}

// ============================================================================
// calculate_token_similarity tests
// ============================================================================

#[test]
fn test_token_similarity_identical() {
    let similarity = calculate_token_similarity("a + b", "a + b");
    assert_eq!(similarity, 1.0);
}

#[test]
fn test_token_similarity_partial() {
    let similarity = calculate_token_similarity("a + b", "a + c");
    assert!(similarity > 0.0 && similarity < 1.0);
}

#[test]
fn test_token_similarity_empty() {
    let similarity = calculate_token_similarity("", "");
    assert_eq!(similarity, 1.0);
}

#[test]
fn test_token_similarity_no_overlap() {
    let similarity = calculate_token_similarity("a b c", "x y z");
    assert_eq!(similarity, 0.0);
}

// ============================================================================
// detect_identity_operations tests
// ============================================================================

#[test]
fn test_detect_identity_add_zero() {
    assert!(detect_identity_operations("x + 0", "x"));
}

#[test]
fn test_detect_identity_mul_one() {
    assert!(detect_identity_operations("x * 1", "x"));
}

#[test]
fn test_detect_identity_sub_zero() {
    assert!(detect_identity_operations("x - 0", "x"));
}

#[test]
fn test_detect_identity_div_one() {
    assert!(detect_identity_operations("x / 1", "x"));
}

#[test]
fn test_detect_identity_mul_zero_simplifies() {
    assert!(detect_identity_operations("x * 0", "0"));
}

#[test]
fn test_detect_identity_no_identity() {
    assert!(!detect_identity_operations("x + y", "x - y"));
}

// ============================================================================
// detect_boolean_tautology tests
// ============================================================================

#[test]
fn test_detect_tautology_or_true() {
    assert!(detect_boolean_tautology("x || true", "{ true }"));
}

#[test]
fn test_detect_tautology_and_false() {
    assert!(detect_boolean_tautology("x && false", "{ false }"));
}

#[test]
fn test_detect_tautology_double_negation() {
    assert!(detect_boolean_tautology("!!x", "x"));
}

#[test]
fn test_detect_tautology_no_tautology() {
    assert!(!detect_boolean_tautology("x && y", "y && x"));
}

// ============================================================================
// detect_commutative_swap tests
// ============================================================================

#[test]
fn test_detect_commutative_addition() {
    assert!(detect_commutative_swap("a + b", "b + a"));
}

#[test]
fn test_detect_commutative_multiplication() {
    assert!(detect_commutative_swap("x * y", "y * x"));
}

#[test]
fn test_detect_commutative_no_swap() {
    assert!(!detect_commutative_swap("a + b", "a - b"));
}

#[test]
fn test_detect_commutative_different_length() {
    assert!(!detect_commutative_swap("a + b", "a + b + c"));
}

// ============================================================================
// is_commutative_op tests
// ============================================================================

#[test]
fn test_is_commutative_addition() {
    assert!(is_commutative_op("+"));
}

#[test]
fn test_is_commutative_multiplication() {
    assert!(is_commutative_op("*"));
}

#[test]
fn test_is_commutative_and() {
    assert!(is_commutative_op("&&"));
}

#[test]
fn test_is_commutative_or() {
    assert!(is_commutative_op("||"));
}

#[test]
fn test_is_not_commutative_subtraction() {
    assert!(!is_commutative_op("-"));
}

#[test]
fn test_is_not_commutative_division() {
    assert!(!is_commutative_op("/"));
}

// ============================================================================
// extract_operator_patterns tests
// ============================================================================

#[test]
fn test_extract_pattern_add_zero() {
    let patterns = extract_operator_patterns("x + 0", "x");
    assert!(patterns.contains(&"add_zero_identity".to_string()));
}

#[test]
fn test_extract_pattern_mul_one() {
    let patterns = extract_operator_patterns("x * 1", "x");
    assert!(patterns.contains(&"mul_one_identity".to_string()));
}

#[test]
fn test_extract_pattern_or_true() {
    let patterns = extract_operator_patterns("x || true", "true");
    assert!(patterns.contains(&"or_true_tautology".to_string()));
}

#[test]
fn test_extract_pattern_double_negation() {
    let patterns = extract_operator_patterns("!!x", "x");
    assert!(patterns.contains(&"double_negation".to_string()));
}

#[test]
fn test_extract_pattern_associative() {
    let patterns = extract_operator_patterns("(a + b) + c", "(a + b) + c");
    assert!(patterns.contains(&"associative_grouping".to_string()));
}

// ============================================================================
// EquivalenceFeatures tests
// ============================================================================

#[test]
fn test_equivalence_features_from_mutant_pair() {
    let mutant = create_test_mutant("x + 0", "x");
    let features = EquivalenceFeatures::from_mutant_pair(&mutant, "x + 0");

    assert!(features.has_identity_ops);
    assert!(!features.operator_patterns.is_empty());
}

#[test]
fn test_equivalence_features_edit_distance() {
    let mutant = create_test_mutant("abc", "xyz");
    let features = EquivalenceFeatures::from_mutant_pair(&mutant, "abc");

    assert_eq!(features.edit_distance, 3);
}

#[test]
fn test_equivalence_features_length_difference() {
    let mutant = create_test_mutant("hello", "hi");
    let features = EquivalenceFeatures::from_mutant_pair(&mutant, "hello");

    assert_eq!(features.length_difference, 3);
}

// ============================================================================
// EquivalenceResult tests
// ============================================================================

#[test]
fn test_equivalence_result_creation() {
    let result = EquivalenceResult {
        is_equivalent: true,
        confidence: 0.9,
        reason: "Test reason".to_string(),
        patterns: vec!["pattern1".to_string()],
    };

    assert!(result.is_equivalent);
    assert_eq!(result.confidence, 0.9);
}

#[test]
fn test_equivalence_result_serialization() {
    let result = EquivalenceResult {
        is_equivalent: false,
        confidence: 0.5,
        reason: "Not equivalent".to_string(),
        patterns: vec![],
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("is_equivalent"));
    assert!(json.contains("confidence"));
}
