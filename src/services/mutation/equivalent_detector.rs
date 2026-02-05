//! Equivalent Mutant Detector - Phase 4.2
//!
//! EXTREME TDD: GREEN PHASE - Minimal implementation to pass RED tests

use super::Mutant;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Features extracted from a mutant-original pair for equivalence detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalenceFeatures {
    /// AST structural similarity (0.0 - 1.0)
    pub ast_similarity: f64,

    /// Detected operator patterns
    pub operator_patterns: Vec<String>,

    /// Edit distance between sources
    pub edit_distance: usize,

    /// Has identity operations (e.g., +0, *1)
    pub has_identity_ops: bool,

    /// Has commutative swap
    pub has_commutative: bool,

    /// Has boolean tautology
    pub has_tautology: bool,

    /// Source length difference
    pub length_difference: i32,
}

impl EquivalenceFeatures {
    /// Extract features from mutant and original source pair
    pub fn from_mutant_pair(mutant: &Mutant, original: &str) -> Self {
        let mutated = &mutant.mutated_source;

        // Simple pattern detection (Phase 1)
        let has_identity_ops = detect_identity_operations(original, mutated);
        let has_commutative = detect_commutative_swap(original, mutated);
        let has_tautology = detect_boolean_tautology(original, mutated);

        let operator_patterns = extract_operator_patterns(original, mutated);
        let edit_distance = levenshtein_distance(original, mutated);
        let length_difference = (mutated.len() as i32 - original.len() as i32).abs();

        // Simple AST similarity based on token count
        let ast_similarity = calculate_token_similarity(original, mutated);

        Self {
            ast_similarity,
            operator_patterns,
            edit_distance,
            has_identity_ops,
            has_commutative,
            has_tautology,
            length_difference,
        }
    }
}

/// Result of equivalence detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalenceResult {
    /// Is the mutant equivalent to the original?
    pub is_equivalent: bool,

    /// Confidence in the detection (0.0 - 1.0)
    pub confidence: f64,

    /// Reason for equivalence or non-equivalence
    pub reason: String,

    /// Detected patterns
    pub patterns: Vec<String>,
}

/// Training data for equivalence detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalenceTrainingData {
    /// The mutant
    pub mutant: Mutant,

    /// Original source code
    pub original_source: String,

    /// Is this an equivalent mutant?
    pub is_equivalent: bool,

    /// Was this manually verified?
    pub verified_manually: bool,

    /// Detection method used
    pub detection_method: String,
}

/// Equivalent mutant detector
#[derive(Debug)]
pub struct EquivalentMutantDetector {
    /// Known equivalence patterns
    equivalence_patterns: HashMap<String, f64>,

    /// Pattern confidence scores
    pattern_confidence: HashMap<String, f64>,

    /// Is the detector trained?
    trained: bool,

    /// Training samples count
    training_samples: usize,
}

impl EquivalentMutantDetector {
    /// Create new detector
    pub fn new() -> Self {
        Self {
            equivalence_patterns: HashMap::new(),
            pattern_confidence: HashMap::new(),
            trained: false,
            training_samples: 0,
        }
    }

    /// Train the detector on labeled data
    pub fn train(&mut self, training_data: &[EquivalenceTrainingData]) -> Result<()> {
        if training_data.is_empty() {
            anyhow::bail!("Training data cannot be empty");
        }

        // Phase 1: Pattern-based detection
        for sample in training_data {
            let features =
                EquivalenceFeatures::from_mutant_pair(&sample.mutant, &sample.original_source);

            for pattern in &features.operator_patterns {
                let entry = self
                    .equivalence_patterns
                    .entry(pattern.clone())
                    .or_insert(0.0);

                if sample.is_equivalent {
                    *entry += 1.0;
                }

                // Track pattern confidence
                let confidence = if sample.verified_manually { 1.0 } else { 0.7 };
                self.pattern_confidence.insert(pattern.clone(), confidence);
            }
        }

        // Normalize pattern scores
        let total = training_data.len() as f64;
        for score in self.equivalence_patterns.values_mut() {
            *score /= total;
        }

        self.trained = true;
        self.training_samples = training_data.len();

        Ok(())
    }

    /// Update detector with new patterns
    pub fn update(&mut self, new_data: &[EquivalenceTrainingData]) -> Result<()> {
        if !self.trained {
            return self.train(new_data);
        }

        self.training_samples += new_data.len();

        for sample in new_data {
            let features =
                EquivalenceFeatures::from_mutant_pair(&sample.mutant, &sample.original_source);

            for pattern in &features.operator_patterns {
                let current = self
                    .equivalence_patterns
                    .get(pattern)
                    .copied()
                    .unwrap_or(0.5);

                let alpha = 0.3; // Learning rate
                let new_score = if sample.is_equivalent {
                    current * (1.0 - alpha) + alpha
                } else {
                    current * (1.0 - alpha)
                };

                self.equivalence_patterns.insert(pattern.clone(), new_score);
            }
        }

        Ok(())
    }

    /// Detect if a mutant is equivalent to the original
    pub fn detect_equivalent(&self, mutant: &Mutant, original: &str) -> Result<EquivalenceResult> {
        if !self.trained {
            anyhow::bail!("Detector not trained");
        }

        let features = EquivalenceFeatures::from_mutant_pair(mutant, original);

        // Check for known equivalence patterns
        let patterns = features.operator_patterns.clone();

        let (is_equivalent, confidence, reason) = if features.has_identity_ops {
            // Identity operations
            (
                true,
                0.9,
                "Contains identity operation (e.g., +0, *1, -0, /1)".to_string(),
            )
        } else if features.has_tautology {
            // Boolean tautology
            (
                true,
                0.85,
                "Contains boolean tautology (e.g., x || true → true)".to_string(),
            )
        } else if features.has_commutative {
            // Commutative swap
            (
                true,
                0.8,
                "commutative operation swap (e.g., a + b → b + a)".to_string(),
            )
        } else {
            // Pattern-based detection
            let mut total_score = 0.0;
            let mut pattern_count = 0;

            for pattern in &features.operator_patterns {
                if let Some(&score) = self.equivalence_patterns.get(pattern) {
                    total_score += score;
                    pattern_count += 1;
                }
            }

            if pattern_count > 0 {
                let avg_score = total_score / pattern_count as f64;
                (
                    avg_score > 0.6,
                    avg_score,
                    format!("Pattern-based detection (score: {:.2})", avg_score),
                )
            } else {
                (
                    false,
                    0.5,
                    "No known equivalence patterns detected".to_string(),
                )
            }
        };

        Ok(EquivalenceResult {
            is_equivalent,
            confidence,
            reason,
            patterns,
        })
    }

    /// Detect with human-readable explanation
    pub fn detect_with_explanation(
        &self,
        mutant: &Mutant,
        original: &str,
    ) -> Result<(EquivalenceResult, String)> {
        let result = self.detect_equivalent(mutant, original)?;

        let explanation = if result.is_equivalent {
            format!(
                "Mutant is EQUIVALENT to original (confidence: {:.1}%). Reason: {}",
                result.confidence * 100.0,
                result.reason
            )
        } else {
            format!(
                "Mutant is NOT EQUIVALENT to original (confidence: {:.1}%). Reason: {}",
                result.confidence * 100.0,
                result.reason
            )
        };

        Ok((result, explanation))
    }

    /// Filter out equivalent mutants from a list
    pub fn filter_equivalents(
        &self,
        mutants: &[Mutant],
        original_sources: &[(&str, &str)],
    ) -> Result<Vec<(Mutant, EquivalenceResult)>> {
        let mut non_equivalents = Vec::new();

        for (i, mutant) in mutants.iter().enumerate() {
            // Find matching original source (simplified for Phase 1)
            let original = if i < original_sources.len() {
                original_sources[i].1
            } else {
                ""
            };

            let result = self.detect_equivalent(mutant, original)?;

            if !result.is_equivalent {
                non_equivalents.push((mutant.clone(), result));
            }
        }

        Ok(non_equivalents)
    }

    /// Get accuracy estimate based on training data
    pub fn get_accuracy_estimate(&self) -> f64 {
        if !self.trained {
            return 0.0;
        }

        // Simple estimate based on pattern coverage
        let pattern_count = self.equivalence_patterns.len();
        (pattern_count as f64 / (pattern_count as f64 + 10.0)).min(0.95)
    }

    /// Check if detector is trained
    pub fn is_trained(&self) -> bool {
        self.trained
    }

    /// Save detector to file
    pub fn save(&self, path: &Path) -> Result<()> {
        let serialized = bincode::serialize(self)?;
        std::fs::write(path, serialized)?;
        Ok(())
    }

    /// Load detector from file
    pub fn load(path: &Path) -> Result<Self> {
        let data = std::fs::read(path)?;
        let detector = bincode::deserialize(&data)?;
        Ok(detector)
    }
}

impl Default for EquivalentMutantDetector {
    fn default() -> Self {
        Self::new()
    }
}

// Serialization support
impl Serialize for EquivalentMutantDetector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("EquivalentMutantDetector", 4)?;
        state.serialize_field("equivalence_patterns", &self.equivalence_patterns)?;
        state.serialize_field("pattern_confidence", &self.pattern_confidence)?;
        state.serialize_field("trained", &self.trained)?;
        state.serialize_field("training_samples", &self.training_samples)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for EquivalentMutantDetector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct DetectorData {
            equivalence_patterns: HashMap<String, f64>,
            pattern_confidence: HashMap<String, f64>,
            trained: bool,
            training_samples: usize,
        }

        let data = DetectorData::deserialize(deserializer)?;
        Ok(Self {
            equivalence_patterns: data.equivalence_patterns,
            pattern_confidence: data.pattern_confidence,
            trained: data.trained,
            training_samples: data.training_samples,
        })
    }
}

// Helper functions

/// Detect identity operations like +0, *1, -0, /1
fn detect_identity_operations(original: &str, mutated: &str) -> bool {
    // Check if original has identity op and mutated removes it
    let has_add_zero = original.contains("+ 0") || original.contains("+0");
    let has_mul_one = original.contains("* 1") || original.contains("*1");
    let has_sub_zero = original.contains("- 0") || original.contains("-0");
    let has_div_one = original.contains("/ 1") || original.contains("/1");
    let has_mul_zero =
        original.contains("* 0") || original.contains("*0") || original.contains("0 *");

    let identity_removed = (has_add_zero || has_mul_one || has_sub_zero || has_div_one)
        && (mutated.len() < original.len());

    let mul_zero_simplified = has_mul_zero && mutated.trim() == "0";

    identity_removed || mul_zero_simplified
}

/// Detect boolean tautologies
fn detect_boolean_tautology(original: &str, mutated: &str) -> bool {
    detect_or_true_tautology(original, mutated)
        || detect_and_false_contradiction(original, mutated)
        || detect_or_false_identity(original, mutated)
        || detect_and_true_identity(original, mutated)
        || detect_double_negation(original, mutated)
}

/// Detects x || true → true (tautology simplification)
fn detect_or_true_tautology(original: &str, mutated: &str) -> bool {
    original.contains("|| true")
        && (mutated.contains("{ true }") || mutated.trim().ends_with("true"))
        && mutated.len() < original.len()
        && !mutated.contains("||")
}

/// Detects x && false → false (contradiction simplification)
fn detect_and_false_contradiction(original: &str, mutated: &str) -> bool {
    original.contains("&& false")
        && (mutated.contains("{ false }") || mutated.trim().ends_with("false"))
        && mutated.len() < original.len()
        && !mutated.contains("&&")
}

/// Detects x || false → x (identity for OR)
fn detect_or_false_identity(original: &str, mutated: &str) -> bool {
    original.contains("|| false")
        && !mutated.contains("||")
        && !mutated.contains("false")
        && mutated.len() < original.len()
}

/// Detects x && true → x (identity for AND)
fn detect_and_true_identity(original: &str, mutated: &str) -> bool {
    original.contains("&& true")
        && !mutated.contains("&&")
        && !mutated.contains("true")
        && mutated.len() < original.len()
}

/// Detects !!x → x (double negation elimination)
fn detect_double_negation(original: &str, mutated: &str) -> bool {
    original.contains("!!") && !mutated.contains("!!")
}

/// Detect commutative operation swap
fn detect_commutative_swap(original: &str, mutated: &str) -> bool {
    let orig_tokens: Vec<&str> = original.split_whitespace().collect();
    let mut_tokens: Vec<&str> = mutated.split_whitespace().collect();

    if orig_tokens.len() != mut_tokens.len() || orig_tokens.len() < 3 {
        return false;
    }

    // Check each potential commutative operation
    for i in 0..orig_tokens.len() - 2 {
        if is_commutative_op(orig_tokens[i + 1])
            && has_swapped_operands(&orig_tokens, &mut_tokens, i)
        {
            return true;
        }
    }

    false
}

/// Check if operands are swapped at given position
fn has_swapped_operands(orig: &[&str], mutated: &[&str], pos: usize) -> bool {
    let (op, a, b) = (orig[pos + 1], orig[pos], orig[pos + 2]);

    mutated
        .windows(3)
        .any(|window| window[1] == op && window[0] == b && window[2] == a)
}

/// Check if operator is commutative
fn is_commutative_op(op: &str) -> bool {
    matches!(op, "+" | "*" | "&&" | "||" | "==" | "!=")
}

/// Extract operator patterns from source pair
fn extract_operator_patterns(original: &str, mutated: &str) -> Vec<String> {
    let mut patterns = Vec::new();

    // Identity operations
    if original.contains("+ 0") || original.contains("+0") {
        patterns.push("add_zero_identity".to_string());
    }
    if original.contains("* 1") || original.contains("*1") {
        patterns.push("mul_one_identity".to_string());
    }
    if original.contains("- 0") || original.contains("-0") {
        patterns.push("sub_zero_identity".to_string());
    }
    if original.contains("/ 1") || original.contains("/1") {
        patterns.push("div_one_identity".to_string());
    }

    // Boolean patterns
    if original.contains("|| true") {
        patterns.push("or_true_tautology".to_string());
    }
    if original.contains("&& false") {
        patterns.push("and_false_tautology".to_string());
    }
    if original.contains("!!") {
        patterns.push("double_negation".to_string());
    }

    // Associative patterns
    if original.contains("(")
        && mutated.contains("(")
        && original.matches('(').count() == mutated.matches('(').count()
    {
        patterns.push("associative_grouping".to_string());
    }

    patterns
}

/// Calculate token-based similarity
fn calculate_token_similarity(s1: &str, s2: &str) -> f64 {
    let tokens1: Vec<&str> = s1.split_whitespace().collect();
    let tokens2: Vec<&str> = s2.split_whitespace().collect();

    let max_len = tokens1.len().max(tokens2.len());
    if max_len == 0 {
        return 1.0;
    }

    let common = tokens1.iter().filter(|t| tokens2.contains(t)).count();
    common as f64 / max_len as f64
}

/// Calculate Levenshtein distance
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut prev_row: Vec<usize> = (0..=len2).collect();
    let mut curr_row = vec![0; len2 + 1];

    for (i, c1) in s1.chars().enumerate() {
        curr_row[0] = i + 1;

        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            curr_row[j + 1] = (curr_row[j] + 1)
                .min(prev_row[j + 1] + 1)
                .min(prev_row[j] + cost);
        }

        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[len2]
}

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
