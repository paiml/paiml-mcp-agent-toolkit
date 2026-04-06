// LocalEmbedder implementation
// Extracted from search_engine.rs for modularity

impl LocalEmbedder {
    /// Create a new local embedder with fixed dimension
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            dimension: 256,
            document_frequencies: RwLock::new(HashMap::new()),
            doc_count: RwLock::new(0),
        }
    }

    /// Create embedder with custom dimension
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_dimension(dimension: usize) -> Self {
        Self {
            dimension,
            document_frequencies: RwLock::new(HashMap::new()),
            doc_count: RwLock::new(0),
        }
    }

    /// Fit the embedder on a corpus of documents (builds IDF statistics)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn fit(&self, documents: &[String]) -> Result<(), String> {
        let mut df = self
            .document_frequencies
            .write()
            .map_err(|e| format!("Lock error: {e}"))?;
        let mut count = self
            .doc_count
            .write()
            .map_err(|e| format!("Lock error: {e}"))?;

        df.clear();
        *count = documents.len();

        for doc in documents {
            let tokens: std::collections::HashSet<String> =
                self.tokenize(doc).into_iter().collect();
            for token in tokens {
                *df.entry(token).or_insert(0) += 1;
            }
        }

        Ok(())
    }

    /// Generate embedding for a single text using feature hashing + TF-IDF weighting
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        let tokens = self.tokenize(text);
        let mut embedding = vec![0.0f32; self.dimension];

        // Count term frequencies
        let mut tf: HashMap<String, usize> = HashMap::new();
        for token in &tokens {
            *tf.entry(token.clone()).or_insert(0) += 1;
        }

        let doc_count = *self
            .doc_count
            .read()
            .map_err(|e| format!("Lock error: {e}"))?;
        let df = self
            .document_frequencies
            .read()
            .map_err(|e| format!("Lock error: {e}"))?;

        // Feature hashing with TF-IDF weighting
        for (token, count) in &tf {
            // Hash token to get dimension index
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            token.hash(&mut hasher);
            let hash = hasher.finish();
            let idx = (hash as usize) % self.dimension;

            // TF: log(1 + count)
            let term_freq = (1.0 + *count as f32).ln();

            // IDF: log(N / df) where N = doc_count, df = document frequency
            let doc_freq = df.get(token).copied().unwrap_or(1) as f32;
            let n = (doc_count.max(1)) as f32;
            let inv_doc_freq = (n / doc_freq).ln();

            // TF-IDF weight
            let weight = term_freq * inv_doc_freq;

            // Use sign from secondary hash for better distribution
            let sign = if (hash >> 32) & 1 == 0 { 1.0 } else { -1.0 };
            embedding[idx] += sign * weight;
        }

        // L2 normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut embedding {
                *x /= norm;
            }
        }

        Ok(embedding)
    }

    /// Tokenize text into words
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| s.len() > 1)
            .map(|s| s.to_string())
            .collect()
    }

    /// Get embedding dimension
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn dimension(&self) -> usize {
        self.dimension
    }
}

impl Default for LocalEmbedder {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: LocalEmbedder contains only owned data (dimension: usize, no interior mutability)
// and implements no shared mutable state, making it safe to send between and share across threads.
unsafe impl Send for LocalEmbedder {}
unsafe impl Sync for LocalEmbedder {}
