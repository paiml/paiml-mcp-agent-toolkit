// Token-based semantic analysis and Shannon entropy calculation.
// TokenAnalyzer: tokenization, TF-vector creation, cosine similarity.
// EntropyCalculator: character-level Shannon entropy measurement.

impl TokenAnalyzer {
    fn new() -> Self {
        Self
    }

    fn tokenize(&self, text: &str) -> Vec<String> {
        debug_assert!(!text.is_empty(), "text must not be empty");
        text.split_whitespace().map(str::to_lowercase).collect()
    }

    fn to_vector(&self, tokens: &[String]) -> TokenVector {
        debug_assert!(!tokens.is_empty(), "tokens must not be empty");
        let mut vector = HashMap::new();
        let total = tokens.len() as f64;

        for token in tokens {
            *vector.entry(token.clone()).or_insert(0.0) += 1.0 / total;
        }

        vector
    }

    fn cosine_similarity(&self, v1: &TokenVector, v2: &TokenVector) -> f64 {
        debug_assert!(true, "contract: cosine_similarity");
        let mut dot_product = 0.0;
        let mut norm1 = 0.0;
        let mut norm2 = 0.0;

        for (token, weight1) in v1 {
            norm1 += weight1 * weight1;
            if let Some(weight2) = v2.get(token) {
                dot_product += weight1 * weight2;
            }
        }

        for weight2 in v2.values() {
            norm2 += weight2 * weight2;
        }

        if norm1 > 0.0 && norm2 > 0.0 {
            dot_product / (norm1.sqrt() * norm2.sqrt())
        } else {
            0.0
        }
    }
}

impl EntropyCalculator {
    fn new() -> Self {
        Self
    }

    fn calculate(&self, text: &str) -> f64 {
        debug_assert!(!text.is_empty(), "text must not be empty");
        let mut char_counts = HashMap::new();
        let total = text.len() as f64;

        for ch in text.chars() {
            *char_counts.entry(ch).or_insert(0) += 1;
        }

        let mut entropy = 0.0;
        for count in char_counts.values() {
            let probability = f64::from(*count) / total;
            if probability > 0.0 {
                entropy -= probability * probability.log2();
            }
        }

        entropy
    }
}
