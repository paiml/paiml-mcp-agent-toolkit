// HelpNlpProcessor implementation - tokenization, stemming, BM25 scoring

impl HelpNlpProcessor {
    /// Create a new NLP processor
    pub fn new() -> Self {
        let mut stop_words = HashSet::new();
        // Common English stop words + domain-specific
        for word in &[
            "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has",
            "had", "do", "does", "did", "will", "would", "could", "should", "may", "might", "must",
            "shall", "can", "to", "of", "in", "for", "on", "with", "at", "by", "from", "as",
            "into", "through", "during", "before", "after", "above", "below", "between", "under",
            "again", "further", "then", "once", "here", "there", "when", "where", "why", "how",
            "all", "each", "few", "more", "most", "other", "some", "such", "no", "nor", "not",
            "only", "own", "same", "so", "than", "too", "very", "just", "and", "but", "if", "or",
            "because", "until", "while", "this", "that", "these", "those", "it", "its",
            // Domain-specific
            "pmat", "command", "run", "execute", "use", "using",
        ] {
            stop_words.insert(word.to_string());
        }

        Self { stop_words }
    }

    /// Simple tokenization - split on whitespace and punctuation
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
            .filter(|s| !s.is_empty() && s.len() > 1)
            .map(|s| s.to_string())
            .collect()
    }

    /// Simple Porter-like stemming (suffix removal)
    fn stem(&self, word: &str) -> String {
        let word = word.to_lowercase();
        // Simple suffix removal rules
        if word.ends_with("ing") && word.len() > 5 {
            return word.get(..word.len() - 3).unwrap_or_default().to_string();
        }
        if word.ends_with("ed") && word.len() > 4 {
            return word.get(..word.len() - 2).unwrap_or_default().to_string();
        }
        if word.ends_with("ies") && word.len() > 4 {
            return format!("{}y", word.get(..word.len() - 3).unwrap_or_default());
        }
        if word.ends_with("es") && word.len() > 4 {
            return word.get(..word.len() - 2).unwrap_or_default().to_string();
        }
        if word.ends_with("s") && word.len() > 3 && !word.ends_with("ss") {
            return word.get(..word.len() - 1).unwrap_or_default().to_string();
        }
        if word.ends_with("ly") && word.len() > 4 {
            return word.get(..word.len() - 2).unwrap_or_default().to_string();
        }
        word
    }

    /// Preprocess text for search (tokenize, filter, stem)
    pub fn preprocess(&self, text: &str) -> Vec<String> {
        self.tokenize(text)
            .into_iter()
            .filter(|t| !self.stop_words.contains(t))
            .map(|t| self.stem(&t))
            .collect()
    }

    /// Calculate term frequency for a document
    pub fn term_frequency(&self, text: &str) -> HashMap<String, f64> {
        let tokens = self.preprocess(text);
        let total = tokens.len() as f64;

        let mut tf = HashMap::new();
        for token in tokens {
            *tf.entry(token).or_insert(0.0) += 1.0;
        }

        // Normalize by document length
        for freq in tf.values_mut() {
            *freq /= total.max(1.0);
        }

        tf
    }

    /// Calculate BM25 score between query and document
    pub fn bm25_score(&self, query: &str, document: &str, k1: f64, b: f64) -> f64 {
        let query_tokens = self.preprocess(query);
        let doc_tf = self.term_frequency(document);
        let avg_dl = 100.0; // Approximate average document length

        let doc_len = self.preprocess(document).len() as f64;
        let norm = 1.0 - b + b * (doc_len / avg_dl);

        query_tokens
            .iter()
            .map(|term| {
                let tf = doc_tf.get(term).copied().unwrap_or(0.0);
                if tf > 0.0 {
                    tf * (k1 + 1.0) / (tf + k1 * norm)
                } else {
                    0.0
                }
            })
            .sum()
    }
}

impl Default for HelpNlpProcessor {
    fn default() -> Self {
        Self::new()
    }
}
