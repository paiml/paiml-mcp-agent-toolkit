# ML & Analytics

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 9

## Sovereign ML Stack

### Aprender (Primary ML Library)

Replaces linfa, nalgebra, and smartcore:
- Gradient boosting for mutation survival prediction
- TF-IDF for commit message embeddings
- Text similarity for semantic search
- Graph algorithms via trueno-graph

### Trueno (SIMD Compute)

SIMD-accelerated tensor operations:
- Matrix multiplication for embeddings
- Vector similarity (cosine, Euclidean)
- Compressed storage via trueno-zram-core

## TF-IDF Implementation

### Commit Embedder

128-dimensional vocabulary for git commit search:
```rust
struct CommitEmbedder {
    vocabulary: Vec<String>,  // Top-128 terms by document frequency
    idf_scores: Vec<f32>,    // Inverse document frequency per term
}
```

**Critical**: Vocabulary selection must be deterministic — sort by document
frequency descending before selection (HashMap iteration order fix).

### Function Embedder

Used for semantic search in `pmat query`:
- Tokenizes function names, signatures, and source
- Porter stemming via FTS5 tokenizer
- BM25 scoring for ranking

## Model Serialization

### Realizar Integration

Model persistence pipeline:
1. Train model with Aprender
2. Serialize via Realizar format
3. Store in `.pmat/models/` directory
4. Load on-demand for prediction

### Format

```
.pmat/models/
├── mutation-predictor.realizar  # Mutation survival model
├── commit-embedder.realizar     # TF-IDF vocabulary + IDF scores
└── quality-predictor.realizar   # CI failure predictor
```

## Analytics Workloads

### Churn Analysis

Git volatility over configurable window (default 90 days):
- Per-file commit count
- Churn score: normalized change frequency
- Hotspot detection: files with >50% churn

### Entropy Analysis

Pattern diversity via information entropy:
- Low diversity (<30%): repetitive boilerplate
- High diversity (>80%): unique, non-templated code
- Used to identify abstraction opportunities

### Duplicate Detection

MinHash + LSH for code clone detection:
- Configurable similarity threshold
- Cross-language support (Rust, Python, TS, JS, C, C++, Kotlin)
- Minimum 10 lines for analysis

## Key Files

| File | Purpose |
|------|---------|
| `src/services/commit_embedder.rs` | TF-IDF commit embeddings |
| `src/services/duplicate_detector.rs` | MinHash/LSH clone detection |
| `src/services/big_o_analyzer.rs` | Algorithmic complexity analysis |

## References

- Consolidated from: aprender-ml-integration, integrate-ml-trueno-latest-spec,
  integrate-ml-trueno-a3-summary, model-serialization-request-spec-aprender,
  model-serialization-manifest, model-serialization-realizar-integration,
  ml-model-serialization-spec
