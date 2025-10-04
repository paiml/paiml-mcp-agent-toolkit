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
            let features = EquivalenceFeatures::from_mutant_pair(
                &sample.mutant,
                &sample.original_source,
            );

            for pattern in &features.operator_patterns {
                let entry = self.equivalence_patterns
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
            let features = EquivalenceFeatures::from_mutant_pair(
                &sample.mutant,
                &sample.original_source,
            );

            for pattern in &features.operator_patterns {
                let current = self.equivalence_patterns
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
    let has_mul_zero = original.contains("* 0") || original.contains("*0")
        || original.contains("0 *");

    let identity_removed = (has_add_zero || has_mul_one || has_sub_zero || has_div_one)
        && (mutated.len() < original.len());

    let mul_zero_simplified = has_mul_zero && mutated.trim() == "0";

    identity_removed || mul_zero_simplified
}

/// Detect boolean tautologies
fn detect_boolean_tautology(original: &str, mutated: &str) -> bool {
    // x || true → true (tautology simplification)
    let or_true = original.contains("|| true")
        && (mutated.contains("{ true }") || mutated.trim().ends_with("true"))
        && mutated.len() < original.len()
        && !mutated.contains("||");

    // x && false → false (contradiction simplification)
    let and_false = original.contains("&& false")
        && (mutated.contains("{ false }") || mutated.trim().ends_with("false"))
        && mutated.len() < original.len()
        && !mutated.contains("&&");

    // x || false → x (identity for OR)
    let or_false = original.contains("|| false")
        && !mutated.contains("||")
        && !mutated.contains("false")
        && mutated.len() < original.len();

    // x && true → x (identity for AND)
    let and_true = original.contains("&& true")
        && !mutated.contains("&&")
        && !mutated.contains("true")
        && mutated.len() < original.len();

    // !!x → x (double negation)
    let double_neg = original.contains("!!") && !mutated.contains("!!");

    or_true || and_false || or_false || and_true || double_neg
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
            && has_swapped_operands(&orig_tokens, &mut_tokens, i) {
                return true;
            }
    }

    false
}

/// Check if operands are swapped at given position
fn has_swapped_operands(orig: &[&str], mutated: &[&str], pos: usize) -> bool {
    let (op, a, b) = (orig[pos + 1], orig[pos], orig[pos + 2]);

    mutated.windows(3).any(|window| {
        window[1] == op && window[0] == b && window[2] == a
    })
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
    if original.contains("(") && mutated.contains("(")
        && original.matches('(').count() == mutated.matches('(').count() {
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
