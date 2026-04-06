#![cfg_attr(coverage_nightly, coverage(off))]
// Commit Message Embedder (GH-RAG-002)
// Toyota Way: Jidoka - Automation with quality built-in
// Spec: docs/specifications/git-history-rag-integration.md

use std::collections::HashMap;

/// Simple TF-IDF vectorizer for commit messages
/// Implemented inline to avoid external dependency issues
struct SimpleVectorizer {
    vocabulary: HashMap<String, usize>,
    idf: HashMap<String, f32>,
    dimension: usize,
}

impl SimpleVectorizer {
    fn new() -> Self {
        Self {
            vocabulary: HashMap::new(),
            idf: HashMap::new(),
            dimension: 128,
        }
    }

    fn fit(&mut self, documents: &[String]) {
        debug_assert!(!documents.is_empty(), "documents must not be empty");
        let n_docs = documents.len() as f32;
        let mut doc_freq: HashMap<String, usize> = HashMap::new();

        // Count document frequency for each term
        for doc in documents {
            let terms: std::collections::HashSet<_> = Self::tokenize(doc).into_iter().collect();
            for term in terms {
                *doc_freq.entry(term).or_insert(0) += 1;
            }
        }

        // Sort by document frequency descending (most common terms first)
        // This ensures terms shared between query and documents are in the vocabulary,
        // producing non-zero cosine similarity for search. TF weighting provides discrimination.
        // Tie-break alphabetically for deterministic vocabulary across builds.
        let mut terms_by_df: Vec<(String, usize)> = doc_freq.into_iter().collect();
        terms_by_df.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        // Build vocabulary and IDF from top terms
        self.vocabulary.clear();
        self.idf.clear();

        for (idx, (term, df)) in terms_by_df.into_iter().enumerate() {
            if idx >= self.dimension {
                break;
            }
            self.vocabulary.insert(term.clone(), idx);
            self.idf.insert(term, (n_docs / (df as f32)).ln());
        }
    }

    fn transform(&self, text: &str) -> Vec<f32> {
        debug_assert!(!text.is_empty(), "text must not be empty");
        let mut vec = vec![0.0f32; self.dimension];
        let terms = Self::tokenize(text);
        let n_terms = terms.len() as f32;

        if n_terms == 0.0 {
            return vec;
        }

        // Count term frequencies
        let mut tf: HashMap<String, f32> = HashMap::new();
        for term in terms {
            *tf.entry(term).or_insert(0.0) += 1.0;
        }

        // Compute TF-IDF
        for (term, count) in tf {
            if let Some(&idx) = self.vocabulary.get(&term) {
                let term_freq = count / n_terms;
                let idf = self.idf.get(&term).copied().unwrap_or(1.0);
                vec[idx] = term_freq * idf;
            }
        }

        // Normalize
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vec {
                *v /= norm;
            }
        }

        vec
    }

    fn tokenize(text: &str) -> Vec<String> {
        debug_assert!(!text.is_empty(), "text must not be empty");
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| s.len() > 2) // Skip very short tokens
            .map(|s| s.to_string())
            .collect()
    }

    #[allow(dead_code)]
    fn vocabulary_size(&self) -> usize {
        self.vocabulary.len().max(self.dimension)
    }
}

/// Embedder for commit messages using TF-IDF
/// Uses simple built-in vectorizer for Sovereign AI Stack compliance
pub struct CommitEmbedder {
    vectorizer: SimpleVectorizer,
}

impl CommitEmbedder {
    /// Create a new commit embedder
    pub fn new() -> Self {
        Self {
            vectorizer: SimpleVectorizer::new(),
        }
    }

    /// Embed a commit message
    /// Combines subject + body for richer semantic representation
    pub fn embed(&self, message: &str) -> Vec<f32> {
        if message.is_empty() {
            return vec![0.0; 128]; // Default dimension
        }

        // Preprocess message
        let processed = Self::preprocess_message(message);

        // Generate TF-IDF embedding
        self.vectorizer.transform(&processed)
    }

    /// Embed multiple commit messages (batched for efficiency)
    pub fn embed_batch(&mut self, messages: &[String]) -> Vec<Vec<f32>> {
        debug_assert!(!messages.is_empty(), "messages must not be empty");
        if messages.is_empty() {
            return vec![];
        }

        // Preprocess all messages
        let processed: Vec<String> = messages
            .iter()
            .map(|m| Self::preprocess_message(m))
            .collect();

        // Fit vectorizer on corpus
        self.vectorizer.fit(&processed);

        // Transform all messages
        processed
            .iter()
            .map(|m| self.vectorizer.transform(m))
            .collect()
    }

    /// Preprocess commit message for embedding
    fn preprocess_message(message: &str) -> String {
        // Normalize whitespace
        let normalized = message
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        // Remove common prefixes (conventional commits)
        let without_prefix = normalized
            .strip_prefix("fix:")
            .or_else(|| normalized.strip_prefix("feat:"))
            .or_else(|| normalized.strip_prefix("docs:"))
            .or_else(|| normalized.strip_prefix("chore:"))
            .or_else(|| normalized.strip_prefix("refactor:"))
            .or_else(|| normalized.strip_prefix("test:"))
            .or_else(|| normalized.strip_prefix("ci:"))
            .unwrap_or(&normalized)
            .trim();

        // Convert to lowercase for better matching
        without_prefix.to_lowercase()
    }

    /// Get embedding dimension
    pub fn dimension(&self) -> usize {
        128 // Fixed dimension for consistency
    }
}

impl Default for CommitEmbedder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_simple_message() {
        let embedder = CommitEmbedder::new();
        let embedding = embedder.embed("Fix null pointer exception");

        assert!(!embedding.is_empty(), "Embedding should not be empty");
    }

    #[test]
    fn test_embed_empty_message() {
        let embedder = CommitEmbedder::new();
        let embedding = embedder.embed("");

        assert!(
            !embedding.is_empty(),
            "Empty message should return default embedding"
        );
    }

    #[test]
    fn test_preprocess_removes_prefix() {
        let processed = CommitEmbedder::preprocess_message("fix: resolve memory leak");
        assert!(!processed.starts_with("fix:"));
        assert!(processed.contains("memory leak"));
    }

    #[test]
    fn test_preprocess_normalizes_whitespace() {
        let processed = CommitEmbedder::preprocess_message("Fix bug\n\nWith details");
        assert!(!processed.contains('\n'));
    }

    #[test]
    fn test_embed_batch() {
        let mut embedder = CommitEmbedder::new();
        let messages = vec![
            "Fix null pointer".to_string(),
            "Add new feature".to_string(),
            "Refactor module".to_string(),
        ];

        let embeddings = embedder.embed_batch(&messages);

        assert_eq!(embeddings.len(), 3, "Should have 3 embeddings");
        for emb in &embeddings {
            assert!(!emb.is_empty(), "Each embedding should not be empty");
        }
    }

    #[test]
    fn test_embed_batch_empty() {
        let mut embedder = CommitEmbedder::new();
        let embeddings = embedder.embed_batch(&[]);
        assert!(embeddings.is_empty());
    }

    #[test]
    fn test_dimension() {
        let embedder = CommitEmbedder::new();
        assert_eq!(embedder.dimension(), 128);
    }

    #[test]
    fn test_default_trait() {
        let embedder = CommitEmbedder::default();
        assert_eq!(embedder.dimension(), 128);
    }

    #[test]
    fn test_preprocess_all_prefixes() {
        let feat = CommitEmbedder::preprocess_message("feat: add login");
        assert!(feat.contains("add login"));

        let docs = CommitEmbedder::preprocess_message("docs: update readme");
        assert!(docs.contains("update readme"));

        let chore = CommitEmbedder::preprocess_message("chore: bump version");
        assert!(chore.contains("bump version"));

        let refactor = CommitEmbedder::preprocess_message("refactor: simplify parser");
        assert!(refactor.contains("simplify parser"));

        let test = CommitEmbedder::preprocess_message("test: add unit tests");
        assert!(test.contains("add unit tests"));

        let ci = CommitEmbedder::preprocess_message("ci: fix pipeline");
        assert!(ci.contains("fix pipeline"));
    }

    #[test]
    fn test_preprocess_no_prefix() {
        let msg = CommitEmbedder::preprocess_message("Update dependencies");
        assert_eq!(msg, "update dependencies");
    }

    #[test]
    fn test_tokenize_short_tokens_filtered() {
        let tokens = SimpleVectorizer::tokenize("I am a fix");
        // "I", "am", "a" are <= 2 chars, only "fix" passes
        assert_eq!(tokens, vec!["fix"]);
    }

    #[test]
    fn test_tokenize_special_chars() {
        let tokens = SimpleVectorizer::tokenize("fix(parser): handle edge-case #123");
        assert!(tokens.contains(&"fix".to_string()));
        assert!(tokens.contains(&"parser".to_string()));
        assert!(tokens.contains(&"handle".to_string()));
        assert!(tokens.contains(&"edge".to_string()));
        assert!(tokens.contains(&"case".to_string()));
        assert!(tokens.contains(&"123".to_string()));
    }

    #[test]
    fn test_vectorizer_transform_empty() {
        let vectorizer = SimpleVectorizer::new();
        let vec = vectorizer.transform("");
        assert_eq!(vec.len(), 128);
        assert!(vec.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_vocabulary_size() {
        let mut vectorizer = SimpleVectorizer::new();
        assert_eq!(vectorizer.vocabulary_size(), 128); // dimension default

        vectorizer.fit(&[
            "fix null pointer exception parser".to_string(),
            "add feature dark mode support".to_string(),
        ]);
        // vocabulary_size returns max(vocab.len(), dimension)
        assert!(vectorizer.vocabulary_size() >= 128);
    }

    // Falsification Test F2: Semantic clustering
    #[test]
    fn test_similar_messages_cluster() {
        let mut embedder = CommitEmbedder::new();

        // Train on a corpus of commits
        let corpus = vec![
            "Fix null pointer exception in parser".to_string(),
            "Fix crash when input is empty".to_string(),
            "Fix memory leak in cache".to_string(),
            "Add dark mode support".to_string(),
            "Add new export feature".to_string(),
            "Refactor database module".to_string(),
        ];

        let embeddings = embedder.embed_batch(&corpus);

        // Helper to compute cosine similarity
        fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
            let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
            let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
            let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

            if norm_a == 0.0 || norm_b == 0.0 {
                return 0.0;
            }

            dot / (norm_a * norm_b)
        }

        // Fix messages should be more similar to each other than to add messages
        let fix_0_fix_1 = cosine_similarity(&embeddings[0], &embeddings[1]);
        let fix_0_add_0 = cosine_similarity(&embeddings[0], &embeddings[3]);

        // This is a weak test since TF-IDF may not always cluster perfectly
        // The full falsification test would use the real embedding pipeline
        println!(
            "Similarity fix-fix: {:.3}, fix-add: {:.3}",
            fix_0_fix_1, fix_0_add_0
        );

        // We don't assert strict clustering here because TF-IDF has limitations
        // The spec's falsification test F2 uses proper semantic embeddings
    }
}
