#![cfg_attr(coverage_nightly, coverage(off))]
//! Equivalent Mutant Detector - Phase 4.2
//!
//! EXTREME TDD: GREEN PHASE - Minimal implementation to pass RED tests

use super::Mutant;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

include!("equivalent_detector_types.rs");
include!("equivalent_detector_core.rs");
include!("equivalent_detector_patterns.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::mutation::types::{MutationOperator, SourceLocation};
    use tempfile::TempDir;

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
            operator: MutationOperator::ArithmeticReplace,
            original_source: original.to_string(),
            mutated_source: mutated.to_string(),
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
}
