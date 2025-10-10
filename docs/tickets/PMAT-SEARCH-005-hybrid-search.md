# PMAT-SEARCH-005: Hybrid Search Engine with RRF

**Sprint**: 30
**Status**: 🔴 RED PHASE
**Estimated**: 3 hours
**Actual**: TBD

## 🎯 Objective

Implement hybrid search combining keyword search (ripgrep) and semantic search (vector similarity) using Reciprocal Rank Fusion (RRF) algorithm for optimal precision and recall.

## 📋 Requirements

**Must Support:**
- **Keyword Search**: Fast ripgrep-based text matching
- **Vector Search**: Semantic similarity from PMAT-SEARCH-004
- **Hybrid Search**: RRF fusion of keyword + vector results
- Configurable weights for keyword vs. vector
- De-duplication of results across search modes
- Ranking explainability (show keyword vs. vector scores)

**Search Modes:**
1. **Keyword-Only**: Pure ripgrep search (fast, exact)
2. **Vector-Only**: Pure semantic search (conceptual)
3. **Hybrid**: RRF combination (balanced)

**RRF Algorithm:**
```
RRF_score = keyword_weight × RRF_keyword + vector_weight × RRF_vector

where:
  RRF_i = Σ (1 / (k + rank_i))
  k = 60 (constant from Cormack et al., 2009)
  rank_i = position in result set (1-indexed)
```

## 🔴 RED Phase: Tests First

### Test Suite

```rust
// tests/unit_hybrid_search.rs

#[tokio::test]
async fn test_keyword_only_search() {
    let engine = setup_hybrid_engine().await;
    index_test_code(&engine).await;

    let query = HybridSearchQuery {
        query: "fn add".to_string(),
        mode: HybridSearchMode::KeywordOnly,
        keyword_weight: 1.0,
        vector_weight: 0.0,
        language_filter: None,
        file_pattern: None,
        limit: 10,
    };

    let results = engine.search(&query).await?;
    assert!(results.len() > 0);
    assert!(results[0].keyword_score > 0.0);
    assert_eq!(results[0].vector_score, 0.0);
}

#[tokio::test]
async fn test_vector_only_search() {
    let engine = setup_hybrid_engine().await;
    index_test_code(&engine).await;

    let query = HybridSearchQuery {
        query: "function that calculates sum".to_string(),
        mode: HybridSearchMode::VectorOnly,
        keyword_weight: 0.0,
        vector_weight: 1.0,
        language_filter: None,
        file_pattern: None,
        limit: 10,
    };

    let results = engine.search(&query).await?;
    assert!(results.len() > 0);
    assert!(results[0].vector_score > 0.0);
    assert_eq!(results[0].keyword_score, 0.0);
}

#[tokio::test]
async fn test_hybrid_search() {
    let engine = setup_hybrid_engine().await;
    index_test_code(&engine).await;

    let query = HybridSearchQuery {
        query: "add function".to_string(),
        mode: HybridSearchMode::Hybrid,
        keyword_weight: 0.5,
        vector_weight: 0.5,
        language_filter: None,
        file_pattern: None,
        limit: 10,
    };

    let results = engine.search(&query).await?;
    assert!(results.len() > 0);
    // Both scores should contribute
    assert!(results[0].keyword_score > 0.0 || results[0].vector_score > 0.0);
    assert!(results[0].hybrid_score > 0.0);
}

#[tokio::test]
async fn test_rrf_calculation() {
    let engine = setup_hybrid_engine().await;

    // Test RRF formula: 1 / (k + rank)
    let rrf1 = engine.compute_rrf_score(1, 60); // Rank 1
    let rrf2 = engine.compute_rrf_score(2, 60); // Rank 2

    assert!((rrf1 - 1.0 / 61.0).abs() < 0.001);
    assert!((rrf2 - 1.0 / 62.0).abs() < 0.001);
    assert!(rrf1 > rrf2); // Higher rank = higher score
}

#[tokio::test]
async fn test_result_deduplication() {
    let engine = setup_hybrid_engine().await;
    index_test_code(&engine).await;

    let query = HybridSearchQuery {
        query: "add".to_string(),
        mode: HybridSearchMode::Hybrid,
        keyword_weight: 0.5,
        vector_weight: 0.5,
        language_filter: None,
        file_pattern: None,
        limit: 10,
    };

    let results = engine.search(&query).await?;

    // Check no duplicate file_path + chunk_name combinations
    let mut seen = std::collections::HashSet::new();
    for result in &results {
        let key = format!("{}:{}", result.file_path, result.chunk_name);
        assert!(!seen.contains(&key), "Duplicate result: {}", key);
        seen.insert(key);
    }
}

#[tokio::test]
async fn test_weight_adjustment() {
    let engine = setup_hybrid_engine().await;
    index_test_code(&engine).await;

    // Heavy keyword weight
    let query1 = HybridSearchQuery {
        query: "add".to_string(),
        mode: HybridSearchMode::Hybrid,
        keyword_weight: 0.9,
        vector_weight: 0.1,
        language_filter: None,
        file_pattern: None,
        limit: 5,
    };

    let results1 = engine.search(&query1).await?;

    // Heavy vector weight
    let query2 = HybridSearchQuery {
        query: "add".to_string(),
        mode: HybridSearchMode::Hybrid,
        keyword_weight: 0.1,
        vector_weight: 0.9,
        language_filter: None,
        file_pattern: None,
        limit: 5,
    };

    let results2 = engine.search(&query2).await?;

    // Results may differ based on weights
    // At least verify both queries return results
    assert!(results1.len() > 0);
    assert!(results2.len() > 0);
}

#[tokio::test]
async fn test_keyword_search_performance() {
    let engine = setup_hybrid_engine().await;
    index_test_code(&engine).await;

    let start = std::time::Instant::now();

    let query = HybridSearchQuery {
        query: "function".to_string(),
        mode: HybridSearchMode::KeywordOnly,
        keyword_weight: 1.0,
        vector_weight: 0.0,
        language_filter: None,
        file_pattern: None,
        limit: 100,
    };

    engine.search(&query).await?;

    let duration = start.elapsed();
    assert!(duration.as_millis() < 1000); // < 1 second for keyword search
}

#[tokio::test]
async fn test_hybrid_result_ranking() {
    let engine = setup_hybrid_engine().await;
    index_test_code(&engine).await;

    let query = HybridSearchQuery {
        query: "add".to_string(),
        mode: HybridSearchMode::Hybrid,
        keyword_weight: 0.5,
        vector_weight: 0.5,
        language_filter: None,
        file_pattern: None,
        limit: 10,
    };

    let results = engine.search(&query).await?;

    // Results should be sorted by hybrid_score (descending)
    for i in 1..results.len() {
        assert!(results[i - 1].hybrid_score >= results[i].hybrid_score);
    }
}

#[test]
fn test_weight_validation() {
    // Weights should sum to 1.0
    assert!(validate_weights(0.5, 0.5));
    assert!(validate_weights(0.3, 0.7));
    assert!(!validate_weights(0.6, 0.6)); // Sum > 1.0
    assert!(!validate_weights(-0.1, 1.1)); // Negative weight
}
```

**Total Tests**: 25
- Keyword-only search (3 tests)
- Vector-only search (3 tests)
- Hybrid search (5 tests)
- RRF calculation (3 tests)
- Deduplication (2 tests)
- Weight adjustment (3 tests)
- Performance (3 tests)
- Edge cases (3 tests)

## 🟢 GREEN Phase: Implementation

**File**: `server/src/services/semantic/hybrid_search.rs`

**Key Structures:**

```rust
pub struct HybridSearchEngine {
    semantic_engine: Arc<SemanticSearchEngine>,
    ripgrep_searcher: RipgrepSearcher,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HybridSearchMode {
    KeywordOnly,
    VectorOnly,
    Hybrid,
}

pub struct HybridSearchQuery {
    pub query: String,
    pub mode: HybridSearchMode,
    pub keyword_weight: f64,
    pub vector_weight: f64,
    pub language_filter: Option<String>,
    pub file_pattern: Option<String>,
    pub limit: usize,
}

pub struct HybridSearchResult {
    pub file_path: String,
    pub chunk_name: String,
    pub chunk_type: String,
    pub language: String,
    pub start_line: usize,
    pub end_line: usize,
    pub keyword_score: f64,
    pub vector_score: f64,
    pub hybrid_score: f64,
    pub snippet: String,
}
```

**Key Functions:**
- `new(semantic_engine: Arc<SemanticSearchEngine>) -> Self`
- `search(&self, query: &HybridSearchQuery) -> Result<Vec<HybridSearchResult>>`
- `keyword_search(&self, query: &str, limit: usize) -> Result<Vec<KeywordMatch>>`
- `compute_rrf_score(rank: usize, k: usize) -> f64`
- `merge_results(keyword: Vec<KeywordMatch>, vector: Vec<SearchResult>, weights: (f64, f64)) -> Vec<HybridSearchResult>`

## 🔵 REFACTOR Phase: Quality

**Complexity Target**: ≤10 cyclomatic per function
**Coverage Target**: ≥95%
**SATD**: 0 violations

**Refactoring checklist:**
- Extract RRF computation to separate module
- Extract result merging to builder pattern
- Add caching for repeated queries
- Optimize ripgrep invocation
- Document RRF algorithm with citations

## ✅ Exit Criteria

- [ ] 25 tests passing
- [ ] Supports 3 search modes (keyword, vector, hybrid)
- [ ] RRF algorithm correctly implemented
- [ ] Result deduplication working
- [ ] Configurable weights (0.0-1.0)
- [ ] Keyword search completes in <1s
- [ ] Hybrid search combines both result sets
- [ ] Cyclomatic ≤10 for all functions
- [ ] Zero clippy warnings

## 📊 Performance Targets

**Search Latency**:
- Keyword-only: <100ms for 10K files
- Vector-only: <500ms for 1K chunks
- Hybrid: <600ms (parallel execution)

**Accuracy**:
- Keyword precision: >95% (exact match)
- Vector recall: >80% (semantic concepts)
- Hybrid F1: >85% (balanced)

## 🔗 Integration

Will be used by:
- PMAT-SEARCH-006: MCP Tools (expose hybrid search)
- PMAT-SEARCH-009: CLI Commands (user interface)

## 📚 References

**Reciprocal Rank Fusion (RRF)**:
- Cormack, G. V., Clarke, C. L., & Buettcher, S. (2009)
- "Reciprocal rank fusion outperforms the best known automatic evaluation"
- SIGIR '09 Proceedings

**Formula**:
```
RRF(d) = Σ_{r ∈ R} 1 / (k + r(d))

where:
  d = document
  R = set of rankers
  r(d) = rank of document d in ranker r
  k = 60 (empirically determined constant)
```
