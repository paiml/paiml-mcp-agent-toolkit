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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn from_mutant_pair(mutant: &Mutant, original: &str) -> Self {
        debug_assert!(!original.is_empty(), "original must not be empty");
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
