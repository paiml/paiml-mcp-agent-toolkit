# PMAT-SEARCH-008: Topic Modeling with LDA

**Sprint**: 31
**Phase**: RED → GREEN → REFACTOR (EXTREME TDD)
**Estimate**: 1.5 hours
**Priority**: MEDIUM

## Objective

Implement Latent Dirichlet Allocation (LDA) topic modeling to extract semantic topics from code embeddings, enabling automatic discovery of thematic patterns in large codebases.

## Background

After clustering code by similarity, we want to extract semantic topics that describe what the code does. Topic modeling reveals:
- **Common patterns**: Error handling, state management, data transformation
- **Architecture themes**: Frontend/backend, API/database, business logic
- **Cross-cutting concerns**: Logging, security, testing
- **Technical debt areas**: Deprecated patterns, code smells

## Requirements

### Functional Requirements

1. **LDA Topic Extraction**
   - Input: Code embeddings (1536-d vectors)
   - Output: K topics, each with top N keywords/chunks
   - Use dimensionality reduction (PCA/t-SNE) if needed
   - Support 1-20 topics

2. **Topic Representation**
   - Each topic has:
     - Topic ID (0 to k-1)
     - Top 10 representative code chunks
     - Topic strength score (0.0 to 1.0)
     - Dominant keywords (extracted from chunk names)

3. **Chunk-Topic Mapping**
   - Assign each code chunk to dominant topic
   - Provide topic distribution (probabilities)
   - Support filtering by language/file pattern

### Non-Functional Requirements

- **Performance**: Handle 10K+ vectors in <10 seconds
- **Quality**: Topics should be interpretable and distinct
- **Scalability**: Support up to 50K code chunks
- **Testability**: 10 unit tests (RED phase)

## Technical Design

### Data Structures

```rust
pub struct TopicEngine {
    vector_db: Arc<TursoVectorDB>,
}

pub struct TopicResult {
    pub topics: Vec<Topic>,
    pub num_topics: usize,
    pub total_chunks: usize,
    pub coherence_score: f64,
}

pub struct Topic {
    pub id: usize,
    pub top_chunks: Vec<TopicChunk>,
    pub keywords: Vec<String>,
    pub strength: f64, // Average topic probability
}

pub struct TopicChunk {
    pub file_path: String,
    pub chunk_name: String,
    pub chunk_type: String,
    pub language: String,
    pub topic_probability: f64,
}

pub struct TopicFilters {
    pub language: Option<String>,
    pub chunk_type: Option<String>,
    pub file_pattern: Option<String>,
}
```

### Algorithm

#### Simplified LDA (K-means based)

Since full LDA requires probabilistic inference (EM algorithm), we'll use a simplified approach:

```
1. Cluster embeddings into K clusters using K-means
2. For each cluster:
   - Identify most representative chunks (closest to centroid)
   - Extract keywords from chunk names
   - Compute topic strength (average distance to centroid)
3. Assign each chunk to dominant topic
4. Compute coherence score (inter-topic distance)
```

#### Alternative: NMF (Non-negative Matrix Factorization)

If we implement full matrix operations:

```
1. Build term-document matrix from embeddings
2. Factorize: V ≈ WH
   - W: document-topic matrix
   - H: topic-term matrix
3. Extract top terms per topic from H
4. Assign documents to topics from W
```

### Interface

```rust
impl TopicEngine {
    pub fn new(vector_db: Arc<TursoVectorDB>) -> Self;

    pub async fn extract_topics(
        &self,
        num_topics: usize,
        filters: TopicFilters,
    ) -> Result<TopicResult, String>;

    fn simplified_lda(
        &self,
        vectors: &[Vec<f32>],
        chunks: &[ChunkMetadata],
        num_topics: usize,
    ) -> Result<Vec<Topic>, String>;

    fn extract_keywords(
        &self,
        chunk_names: &[String],
        top_k: usize,
    ) -> Vec<String>;

    fn compute_coherence_score(&self, topics: &[Topic]) -> f64;
}

struct ChunkMetadata {
    file_path: String,
    chunk_name: String,
    chunk_type: String,
    language: String,
}
```

## Test Plan (RED Phase - 10 tests)

### Core LDA Tests (4 tests)
1. `test_extract_topics_basic` - Extract 3 topics from test data
2. `test_topic_result_structure` - Verify TopicResult format
3. `test_extract_topics_invalid_count` - Error for num_topics < 1 or > 20
4. `test_extract_topics_empty_data` - Handle empty database

### Topic Quality Tests (3 tests)
5. `test_topic_keywords_extraction` - Extract keywords from chunk names
6. `test_topic_strength_computation` - Verify strength scores are valid
7. `test_coherence_score_computation` - Compute inter-topic coherence

### Integration Tests (3 tests)
8. `test_extract_topics_with_language_filter` - Filter by language
9. `test_chunk_topic_assignment` - Each chunk assigned to topic
10. `test_topic_probability_distribution` - Topic probabilities sum to ~1.0

## Implementation Steps

### RED Phase (20 minutes)
1. Create `server/tests/unit_topic_modeling.rs`
2. Write all 10 failing tests
3. Verify tests fail with clear error messages
4. Run: `cargo test unit_topic_modeling -- --nocapture`

### GREEN Phase (40 minutes)
1. Create `server/src/services/semantic/topic_modeling.rs`
2. Implement simplified LDA using K-means clustering
3. Implement keyword extraction (TF-IDF or frequency-based)
4. Implement coherence score computation
5. Run: `cargo test` - all tests pass

### REFACTOR Phase (30 minutes)
1. Extract helper functions for readability
2. Add documentation comments
3. Optimize for large datasets
4. Run: `cargo clippy` - zero warnings
5. Run: `cargo test` - all tests still pass

## Acceptance Criteria

- [ ] All 10 tests pass
- [ ] Topics are distinct and interpretable
- [ ] Keywords accurately represent topic themes
- [ ] Coherence score is meaningful
- [ ] Zero clippy warnings
- [ ] Code coverage ≥ 95%
- [ ] Cyclomatic complexity ≤ 10 per function

## Dependencies

- `ClusteringEngine` for K-means clustering
- `TursoVectorDB` for fetching embeddings
- Standard library for string processing

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Topics not interpretable | Use chunk names for keywords |
| Poor topic quality | Increase num_topics or filter data |
| Slow for large datasets | Use sampling or mini-batch approach |
| Coherence score not meaningful | Use silhouette score as proxy |

## Future Enhancements

- Full LDA with Gibbs sampling
- Hierarchical topic modeling
- Dynamic topic evolution over time
- Interactive topic visualization
- Topic labeling using LLM

## References

- Blei, D. et al. (2003). "Latent Dirichlet Allocation"
- Griffiths, T. & Steyvers, M. (2004). "Finding scientific topics"
- Röder, M. et al. (2015). "Exploring the space of topic coherence measures"

---

**EXTREME TDD**: RED → GREEN → REFACTOR
