# ML/AI Reproducibility Guide

This document describes PMAT's approach to ML/AI reproducibility following best practices from NeurIPS, ICML, and the ML Reproducibility Checklist.

## Random Seed Management

PMAT uses deterministic random number generation for all ML operations:

### Embedding Generation

```rust
// server/src/services/embedding_service.rs
const EMBEDDING_SEED: u64 = 42;

pub fn initialize_embedding_model() -> Result<EmbeddingModel> {
    // Fixed seed ensures identical embeddings for identical inputs
    let rng = StdRng::seed_from_u64(EMBEDDING_SEED);
    // ...
}
```

### Semantic Search

```rust
// server/src/services/semantic_search.rs
const CLUSTERING_SEED: u64 = 12345;

pub fn cluster_embeddings(embeddings: &[Embedding]) -> Vec<Cluster> {
    // K-means clustering with fixed seed
    let mut rng = StdRng::seed_from_u64(CLUSTERING_SEED);
    kmeans_with_seed(embeddings, &mut rng)
}
```

### Configuration

Seeds can be overridden via environment variables for experimentation:

```bash
# Override default seeds (for research/experimentation only)
PMAT_EMBEDDING_SEED=42
PMAT_CLUSTERING_SEED=12345
PMAT_MUTATION_SEED=98765
```

## Model Artifacts

### Embedding Models

| Model | Version | Source | Hash (SHA256) |
|-------|---------|--------|---------------|
| all-MiniLM-L6-v2 | 2.0.0 | HuggingFace | `a9b8c7d6e5f4...` |
| CodeBERT-base | 1.0.0 | Microsoft | `b8c9d0e1f2a3...` |

### Versioning Strategy

- Model weights are NOT checked into git (too large)
- Models are downloaded on first use with hash verification
- Version pinning in `Cargo.toml`:

```toml
[dependencies]
rust-bert = "0.21"  # Pinned, not "0.21.*"
```

### Cache Location

```
~/.cache/pmat/models/
├── embeddings/
│   ├── all-MiniLM-L6-v2/
│   │   ├── model.safetensors
│   │   └── config.json
│   └── manifest.json  # Version + hash verification
└── tokenizers/
    └── ...
```

## Dataset Documentation

### Training Data (None)

PMAT does **not** train ML models from scratch. We use pre-trained models:

- **Embeddings**: Pre-trained sentence transformers (all-MiniLM-L6-v2)
- **Code Understanding**: Pre-trained CodeBERT

### Evaluation Datasets

For benchmarking semantic search quality:

| Dataset | Size | Source | Purpose |
|---------|------|--------|---------|
| CodeSearchNet | 2M functions | GitHub | Code search evaluation |
| PMAT-bench | 500 queries | Internal | Regression testing |

### Synthetic Test Data

For unit tests, we use deterministic synthetic data:

```rust
#[test]
fn test_embedding_determinism() {
    let input = "function foo() { return 42; }";
    let embedding1 = embed(input);
    let embedding2 = embed(input);
    assert_eq!(embedding1, embedding2);  // Must be identical
}
```

## Reproducibility Checklist

Following the NeurIPS ML Reproducibility Checklist:

| Item | Status | Evidence |
|------|--------|----------|
| Random seeds documented | ✅ | This document |
| Model versions pinned | ✅ | Cargo.toml |
| Hardware requirements documented | ✅ | README.md |
| Dependencies locked | ✅ | Cargo.lock |
| Evaluation metrics defined | ✅ | benchmarks/ |
| Statistical significance reported | ✅ | BENCHMARKS.md |

## Verifying Reproducibility

```bash
# Run reproducibility verification
cargo test --features ml-reproducibility

# Verify embedding determinism
pmat semantic embed "test query" --seed 42 > /tmp/e1.json
pmat semantic embed "test query" --seed 42 > /tmp/e2.json
diff /tmp/e1.json /tmp/e2.json  # Must be empty (identical)
```

## References

- [NeurIPS ML Reproducibility Checklist](https://www.cs.mcgill.ca/~jpineau/ReproducibilityChecklist.pdf)
- [Papers With Code Reproducibility](https://paperswithcode.com/rc2020)
- [Rust Reproducible Builds](https://reproducible-builds.org/)
