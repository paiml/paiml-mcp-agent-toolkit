// SemanticSimilarity implementation: keyword-based similarity scoring with
// stopword filtering, weighted matching, and semantic keyword boosting.

impl SemanticSimilarity {
    /// Create new similarity calculator
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        let stopwords = vec![
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "from", "as", "is", "was", "are", "were", "be", "been", "being", "have", "has",
            "had", "do", "does", "did", "will", "would", "should", "could", "may", "might", "must",
            "can", "cannot",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();

        Self { stopwords }
    }

    /// Calculate similarity between claim and fact (0.0 - 1.0)
    ///
    /// Uses enhanced keyword-based similarity with:
    /// - Stopword filtering
    /// - Weighted matching (exact > partial)
    /// - Semantic keyword boosting
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn calculate(&self, claim: &str, fact: &str) -> f32 {
        let claim_lower = claim.to_lowercase();
        let fact_lower = fact.to_lowercase();

        // Extract meaningful keywords (filter stopwords)
        let claim_words = self.extract_keywords(&claim_lower);
        let fact_words = self.extract_keywords(&fact_lower);

        if claim_words.is_empty() || fact_words.is_empty() {
            return 0.0;
        }

        // Calculate weighted similarity
        let mut score = 0.0;
        let mut total_weight = 0.0;

        for claim_word in &claim_words {
            let weight = self.get_word_weight(claim_word);
            total_weight += weight;

            // Exact match
            if fact_words.contains(claim_word) {
                score += weight;
            }
            // Partial match (substring)
            else if fact_words
                .iter()
                .any(|fw| fw.contains(claim_word.as_str()) || claim_word.contains(fw))
            {
                score += weight * 0.5;
            }
        }

        if total_weight == 0.0 {
            return 0.0;
        }

        // Normalize to 0.0-1.0 range
        let base_score = score / total_weight;

        // Boost score if key semantic keywords match
        let boost = self.semantic_keyword_boost(&claim_lower, &fact_lower);

        // Combine base score with boost (capped at 1.0)
        let result = (base_score + boost).min(1.0);
        result
    }

    /// Extract meaningful keywords (filter stopwords)
    fn extract_keywords(&self, text: &str) -> Vec<String> {
        text.split_whitespace()
            .filter(|word| !self.stopwords.contains(&word.to_string()))
            .map(|s| s.to_string())
            .collect()
    }

    /// Get weight for a word (higher weight for important words)
    fn get_word_weight(&self, word: &str) -> f32 {
        // Technical terms get higher weight
        match word {
            // Language names
            "rust" | "typescript" | "javascript" | "python" | "c" | "cpp" | "go" | "java"
            | "kotlin" | "ruby" | "php" | "swift" | "haskell" => 3.0,

            // Action verbs (capabilities)
            "analyze" | "analyzes" | "analyzing" | "analysis" => 2.5,
            "compile" | "compiles" | "compiling" | "compilation" => 2.5,
            "support" | "supports" | "supporting" | "supported" => 2.0,
            "detect" | "detects" | "detecting" | "detection" => 2.0,
            "generate" | "generates" | "generating" => 2.0,

            // Technical nouns
            "complexity" | "metrics" | "code" | "files" | "functions" => 1.5,
            "pmat" => 1.0, // Tool name is neutral

            _ => 1.0, // Default weight
        }
    }

    /// Calculate semantic keyword boost
    fn semantic_keyword_boost(&self, claim: &str, fact: &str) -> f32 {
        let mut boost = 0.0;

        // Check for explicit contradictions first (highest priority)
        // Pattern: claim says "can X" but fact says "does not X" or "cannot X"
        let action_verbs = ["compile", "compiles", "analyze", "support", "generate"];
        for verb in &action_verbs {
            // Claim is positive about verb, fact is negative
            if claim.contains(verb)
                && !claim.contains("cannot")
                && !claim.contains("does not")
                && (fact.contains(&format!("does not {}", verb))
                    || fact.contains(&format!("cannot {}", verb))
                    || fact.contains(&format!("not {}", verb))
                    || (fact.contains(verb) && (fact.contains("but not") || fact.contains("only"))))
            {
                // CONTRADICTION: claim positive, fact negative
                return -0.8; // Strong negative boost
            }
            // Both agree on capability
            if claim.contains(verb) && fact.contains(verb) {
                // Check if both are positive or both are negative
                let claim_negative = claim.contains("cannot") || claim.contains("does not");
                let fact_negative = fact.contains("cannot")
                    || fact.contains("does not")
                    || fact.contains("but not");

                if claim_negative == fact_negative {
                    boost += 0.3; // Both agree
                }
            }
        }

        // Language matching (high boost for exact match)
        let languages = ["rust", "typescript", "javascript", "python", "c", "cpp"];
        for lang in &languages {
            if claim.contains(lang) && fact.contains(lang) {
                boost += 0.4;
                break;
            }
        }

        // Complexity/metrics matching
        if (claim.contains("complexity") && fact.contains("complexity"))
            || (claim.contains("metrics") && fact.contains("metrics"))
        {
            boost += 0.2;
        }

        boost
    }
}

impl Default for SemanticSimilarity {
    fn default() -> Self {
        Self::new()
    }
}
