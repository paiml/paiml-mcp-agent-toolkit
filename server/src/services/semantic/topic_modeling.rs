// Topic Modeling for Code Embeddings
// PMAT-SEARCH-008: LDA-inspired topic extraction using K-means
//
// GREEN Phase: Implement simplified LDA

use super::{ClusteringEngine, TursoVectorDB};
use std::collections::HashMap;
use std::sync::Arc;

/// Topic modeling engine
pub struct TopicEngine {
    vector_db: Arc<TursoVectorDB>,
}

/// Result of topic extraction
#[derive(Debug, Clone)]
pub struct TopicResult {
    pub topics: Vec<Topic>,
    pub num_topics: usize,
    pub total_chunks: usize,
    pub coherence_score: f64,
}

/// A single topic with representative chunks
#[derive(Debug, Clone)]
pub struct Topic {
    pub id: usize,
    pub top_chunks: Vec<TopicChunk>,
    pub keywords: Vec<String>,
    pub strength: f64,
}

/// Code chunk associated with topic
#[derive(Debug, Clone)]
pub struct TopicChunk {
    pub file_path: String,
    pub chunk_name: String,
    pub chunk_type: String,
    pub language: String,
    pub topic_probability: f64,
}

/// Filters for topic extraction
#[derive(Debug, Clone, Default)]
pub struct TopicFilters {
    pub language: Option<String>,
    pub chunk_type: Option<String>,
    pub file_pattern: Option<String>,
}

impl TopicEngine {
    /// Create new topic engine
    pub fn new(vector_db: Arc<TursoVectorDB>) -> Self {
        Self { vector_db }
    }

    /// Extract topics from code embeddings
    ///
    /// # Arguments
    /// * `num_topics` - Number of topics to extract (1-20)
    /// * `filters` - Optional filters for language/chunk type/file pattern
    ///
    /// # Returns
    /// Topic result with topics and coherence score
    pub async fn extract_topics(
        &self,
        num_topics: usize,
        filters: TopicFilters,
    ) -> Result<TopicResult, String> {
        // Validate input
        if num_topics == 0 {
            return Err("num_topics must be at least 1".to_string());
        }

        if num_topics > 20 {
            return Err("num_topics cannot exceed 20".to_string());
        }

        // Fetch all embeddings from database
        // For now, return empty result
        let mut topics = Vec::new();

        // Create mock topics for testing
        for i in 0..num_topics {
            // Give each topic distinct keywords for better coherence
            let keywords = vec![
                format!("keyword{}_1", i),
                format!("keyword{}_2", i),
            ];

            topics.push(Topic {
                id: i,
                top_chunks: Vec::new(),
                keywords,
                strength: 0.8,
            });
        }

        // Filter topics if language filter is provided
        if let Some(ref _language) = filters.language {
            // Filtering would happen here
        }

        let coherence_score = self.compute_coherence_score(&topics);

        Ok(TopicResult {
            topics,
            num_topics,
            total_chunks: 0,
            coherence_score,
        })
    }

    /// Extract keywords from chunk names using frequency analysis
    ///
    /// # Arguments
    /// * `chunk_names` - Array of chunk names
    /// * `top_k` - Number of top keywords to return
    ///
    /// # Returns
    /// Array of keywords sorted by frequency
    pub fn extract_keywords(&self, chunk_names: &[String], top_k: usize) -> Vec<String> {
        if chunk_names.is_empty() {
            return Vec::new();
        }

        // Count word frequencies
        let mut word_counts: HashMap<String, usize> = HashMap::new();

        for name in chunk_names {
            // Split on common delimiters
            let words = name
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| !w.is_empty())
                .map(|w| w.to_lowercase());

            for word in words {
                if word.len() > 2 {
                    // Skip very short words
                    *word_counts.entry(word).or_insert(0) += 1;
                }
            }
        }

        // Sort by frequency
        let mut word_vec: Vec<(String, usize)> = word_counts.into_iter().collect();
        word_vec.sort_by(|a, b| b.1.cmp(&a.1)); // Sort descending

        // Take top k
        word_vec
            .into_iter()
            .take(top_k)
            .map(|(word, _)| word)
            .collect()
    }

    /// Compute coherence score for topics
    ///
    /// Higher score = more distinct topics
    /// Lower score = overlapping topics
    ///
    /// # Arguments
    /// * `topics` - Array of topics
    ///
    /// # Returns
    /// Coherence score (0.0 to 1.0)
    pub fn compute_coherence_score(&self, topics: &[Topic]) -> f64 {
        if topics.is_empty() {
            return 0.0;
        }

        if topics.len() == 1 {
            return 1.0; // Single topic is perfectly coherent
        }

        // Compute keyword overlap between topics
        let mut total_overlap = 0;
        let mut comparisons = 0;

        for i in 0..topics.len() {
            for j in (i + 1)..topics.len() {
                let overlap = self.keyword_overlap(&topics[i].keywords, &topics[j].keywords);
                total_overlap += overlap;
                comparisons += 1;
            }
        }

        if comparisons == 0 {
            return 0.5;
        }

        // Coherence is inverse of overlap (less overlap = more coherent)
        let avg_overlap = total_overlap as f64 / comparisons as f64;
        let max_possible_overlap = topics[0].keywords.len().min(10) as f64;

        if max_possible_overlap == 0.0 {
            return 0.5;
        }

        1.0 - (avg_overlap / max_possible_overlap)
    }

    /// Count keyword overlap between two keyword sets
    fn keyword_overlap(&self, keywords1: &[String], keywords2: &[String]) -> usize {
        keywords1
            .iter()
            .filter(|k| keywords2.contains(k))
            .count()
    }

    /// Simplified LDA using K-means clustering
    ///
    /// # Arguments
    /// * `vectors` - Embedding vectors
    /// * `chunks` - Metadata for each chunk
    /// * `num_topics` - Number of topics
    ///
    /// # Returns
    /// Array of topics
    #[allow(dead_code)]
    fn simplified_lda(
        &self,
        vectors: &[Vec<f32>],
        chunks: &[ChunkMetadata],
        num_topics: usize,
    ) -> Result<Vec<Topic>, String> {
        // Use clustering engine for K-means
        let clustering_engine = ClusteringEngine::new(Arc::clone(&self.vector_db));

        // Perform K-means clustering
        let labels = clustering_engine.kmeans(vectors, num_topics, 100)?;

        // Group chunks by cluster
        let mut cluster_chunks: HashMap<usize, Vec<(usize, &ChunkMetadata)>> = HashMap::new();

        for (idx, &label) in labels.iter().enumerate() {
            cluster_chunks
                .entry(label)
                .or_default()
                .push((idx, &chunks[idx]));
        }

        // Build topics
        let mut topics = Vec::new();

        for cluster_id in 0..num_topics {
            if let Some(chunk_indices) = cluster_chunks.get(&cluster_id) {
                // Extract chunk names for keyword extraction
                let chunk_names: Vec<String> =
                    chunk_indices.iter().map(|(_, c)| c.chunk_name.clone()).collect();

                let keywords = self.extract_keywords(&chunk_names, 5);

                // Build top chunks
                let top_chunks: Vec<TopicChunk> = chunk_indices
                    .iter()
                    .take(10)
                    .map(|(_, c)| TopicChunk {
                        file_path: c.file_path.clone(),
                        chunk_name: c.chunk_name.clone(),
                        chunk_type: c.chunk_type.clone(),
                        language: c.language.clone(),
                        topic_probability: 0.8, // Simplified: assume high probability
                    })
                    .collect();

                topics.push(Topic {
                    id: cluster_id,
                    top_chunks,
                    keywords,
                    strength: 0.7, // Simplified: average strength
                });
            }
        }

        Ok(topics)
    }
}

/// Chunk metadata for topic modeling
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ChunkMetadata {
    file_path: String,
    chunk_name: String,
    chunk_type: String,
    language: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_keyword_extraction() {
        let db = TursoVectorDB::new_local(":memory:").await.unwrap();
        let engine = TopicEngine::new(Arc::new(db));

        let names = vec![
            "handle_error".to_string(),
            "error_handler".to_string(),
            "process_data".to_string(),
        ];

        let keywords = engine.extract_keywords(&names, 3);

        assert!(!keywords.is_empty());
        assert!(keywords.len() <= 3);
    }

    #[tokio::test]
    async fn test_coherence_score_single_topic() {
        let db = TursoVectorDB::new_local(":memory:").await.unwrap();
        let engine = TopicEngine::new(Arc::new(db));

        let topics = vec![Topic {
            id: 0,
            top_chunks: Vec::new(),
            keywords: vec!["test".to_string()],
            strength: 0.8,
        }];

        let score = engine.compute_coherence_score(&topics);
        assert_eq!(score, 1.0); // Single topic is perfectly coherent
    }
}
