# Aprender ML Integration Specification

**Document ID**: SPEC-APRENDER-001
**Version**: 1.0.0
**Status**: APPROVED
**Date**: 2025-11-18
**Author**: PMAT Team

## Executive Summary

This specification defines the integration of `aprender` (PAIML's next-generation ML library) into PMAT, replacing the current `linfa` and `ndarray` dependencies. Aprender provides pure-Rust ML algorithms with EXTREME TDD methodology, achieving 94.1/100 TDG score with zero unsafe code.

## Motivation

### Current State

PMAT currently uses:
- **`linfa`** (v0.7) - Generic ML framework with heavy dependencies
- **`linfa_trees`** - Decision tree implementation
- **`ndarray`** (v0.15) - N-dimensional array library

### Problems with Current Approach

1. **Dependency Weight**: linfa brings 50+ transitive dependencies
2. **Version Conflicts**: ndarray version conflicts with other dependencies
3. **Maintenance**: linfa development slowed in 2024
4. **Quality Mismatch**: linfa TDG score unknown, PMAT requires high-quality dependencies
5. **Feature Mismatch**: We use <10% of linfa's capabilities

### Why Aprender?

1. **PAIML Native**: Built by same team, aligned quality standards (TDG 94.1/100)
2. **Lightweight**: Zero transitive runtime dependencies
3. **Pure Rust**: `unsafe_code = "forbid"` - matches PMAT's safety standards
4. **EXTREME TDD**: 120+ unit tests, 19 property tests, comprehensive coverage
5. **Production Ready**: v0.1.0 published on crates.io
6. **Focused**: Provides exactly what we need (regression, clustering, metrics)

## Scope

### In Scope

1. **Mutation Testing ML** (`src/services/mutation/ml_predictor.rs`)
   - Replace DecisionTree with LinearRegression or KMeans-based approach
   - Migrate ndarray Array1/Array2 to aprender Vector/Matrix
   - Keep MutantFeatures extraction logic (domain-specific)

2. **Semantic Clustering** (`src/services/semantic/clustering.rs`)
   - Implement K-Means clustering using aprender::cluster::KMeans
   - Use aprender metrics (silhouette_score) for evaluation
   - Replace placeholder implementations with real algorithms

3. **Dependency Migration**
   - Remove linfa, linfa_trees, ndarray from Cargo.toml
   - Add aprender = "0.1.0"
   - Update feature flags as needed

### Out of Scope

1. Vector embeddings (OpenAI/semantic search) - unrelated to ML algorithms
2. Statistical analysis (complexity scoring) - not ML
3. Graph algorithms (DAG, TDG) - not ML
4. Feature extraction logic - stays unchanged

## Technical Design

### Phase 1: Mutation Testing Migration

#### Current Implementation

```rust
// Current: src/services/mutation/ml_predictor.rs
use linfa::prelude::*;
use linfa_trees::{DecisionTree, SplitQuality};
use ndarray::{Array1, Array2};

pub struct SurvivabilityPredictor {
    model: Option<DecisionTree<f64, usize>>,
    statistics: PredictorStatistics,
}
```

#### Target Implementation

```rust
// Target: src/services/mutation/ml_predictor.rs
use aprender::prelude::*;

pub struct SurvivabilityPredictor {
    // Option 1: Use LinearRegression for kill probability prediction
    regression_model: Option<LinearRegression>,

    // Option 2: Use KMeans for mutant clustering
    cluster_model: Option<KMeans>,

    statistics: PredictorStatistics,
}

impl SurvivabilityPredictor {
    pub fn train(&mut self, training_data: &[TrainingData]) -> Result<()> {
        // Convert MutantFeatures (18 features) to Matrix
        let feature_count = training_data.len();
        let features: Vec<f64> = training_data
            .iter()
            .flat_map(|td| self.extract_feature_vector(&td.mutant.features))
            .collect();

        let x = Matrix::from_vec(feature_count, 18, features)?;

        // Target: kill probability (0.0 = survived, 1.0 = killed)
        let y: Vec<f64> = training_data
            .iter()
            .map(|td| if td.was_killed { 1.0 } else { 0.0 })
            .collect();
        let y = Vector::from_slice(&y);

        // Train regression model
        let mut model = LinearRegression::new();
        model.fit(&x, &y)?;

        self.regression_model = Some(model);
        Ok(())
    }

    pub fn predict(&self, mutant: &Mutant) -> Result<PredictionResult> {
        let model = self.regression_model.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Model not trained"))?;

        // Extract features (18-dimensional vector)
        let features = self.extract_feature_vector(&mutant.features);
        let x = Matrix::from_vec(1, 18, features)?;

        let predictions = model.predict(&x);
        let kill_probability = predictions[0].clamp(0.0, 1.0);

        // Calculate confidence (R² score from training)
        let confidence = self.statistics.r_squared;

        Ok(PredictionResult {
            kill_probability,
            confidence,
            mutant_id: mutant.id.clone(),
        })
    }

    fn extract_feature_vector(&self, features: &MutantFeatures) -> Vec<f64> {
        vec![
            features.operator_type as u32 as f64,
            features.cyclomatic_complexity as f64,
            features.cognitive_complexity as f64,
            features.source_line as f64,
            features.nesting_depth as f64,
            features.control_flow_count as f64,
            features.has_loops as u8 as f64,
            features.has_conditionals as u8 as f64,
            features.function_size as f64,
            features.parameter_count as f64,
            features.has_error_handling as u8 as f64,
            features.has_assertions as u8 as f64,
            features.token_count as f64,
            features.unique_variables as f64,
            features.has_arithmetic as u8 as f64,
            features.has_comparisons as u8 as f64,
            features.has_logical_ops as u8 as f64,
            features.mutation_depth as f64,
        ]
    }
}
```

### Phase 2: Clustering Implementation

#### Current Implementation

```rust
// Current: src/services/semantic/clustering.rs
pub struct ClusteringEngine {
    vector_db: Arc<TursoVectorDB>,
}

impl ClusteringEngine {
    pub async fn cluster(&self, method: ClusteringMethod, ...) -> Result<ClusterResult> {
        // TODO: GREEN Phase - not yet implemented
        unimplemented!("Clustering algorithms not yet implemented")
    }
}
```

#### Target Implementation

```rust
// Target: src/services/semantic/clustering.rs
use aprender::prelude::*;
use aprender::metrics::silhouette_score;

impl ClusteringEngine {
    pub async fn cluster(
        &self,
        method: ClusteringMethod,
        filters: Option<ClusterFilters>,
    ) -> Result<ClusterResult> {
        // 1. Fetch embeddings from vector DB
        let embeddings = self.fetch_embeddings(filters).await?;

        // 2. Convert to Matrix
        let (rows, cols) = (embeddings.len(), embeddings[0].len());
        let flat: Vec<f64> = embeddings.into_iter().flatten().collect();
        let x = Matrix::from_vec(rows, cols, flat)?;

        // 3. Cluster based on method
        let (labels, centroids) = match method {
            ClusteringMethod::KMeans { k } => {
                let mut kmeans = KMeans::new(k)
                    .with_max_iter(100)
                    .with_random_state(42);

                kmeans.fit(&x)?;
                let labels = kmeans.predict(&x);
                let centroids = kmeans.centroids();

                (labels, centroids)
            }
            ClusteringMethod::Hierarchical { .. } => {
                return Err(anyhow::anyhow!(
                    "Hierarchical clustering not yet supported - use KMeans"
                ));
            }
            ClusteringMethod::DBSCAN { .. } => {
                return Err(anyhow::anyhow!(
                    "DBSCAN not yet supported - use KMeans"
                ));
            }
        };

        // 4. Calculate silhouette score
        let score = silhouette_score(&x, &labels);

        // 5. Build cluster result
        let clusters = self.build_clusters(&labels, &centroids, &x)?;

        Ok(ClusterResult {
            method: "KMeans".to_string(),
            clusters,
            outliers: vec![],
            silhouette_score: score,
            total_chunks: rows,
        })
    }

    fn build_clusters(
        &self,
        labels: &[usize],
        centroids: &Matrix,
        data: &Matrix,
    ) -> Result<Vec<Cluster>> {
        let mut clusters: HashMap<usize, Vec<ClusterMember>> = HashMap::new();

        for (idx, &label) in labels.iter().enumerate() {
            let row = data.row(idx)?;
            let centroid = centroids.row(label)?;

            let distance = euclidean_distance(&row, &centroid);

            clusters.entry(label).or_default().push(ClusterMember {
                file_path: format!("file_{}", idx), // Replace with actual metadata
                chunk_name: format!("chunk_{}", idx),
                chunk_type: "function".to_string(),
                language: "rust".to_string(),
                distance_to_centroid: distance,
            });
        }

        let result: Vec<Cluster> = clusters
            .into_iter()
            .map(|(id, members)| {
                let size = members.len();
                let cohesion = calculate_cohesion(&members);
                let centroid = centroids.row(id).unwrap().to_vec();

                Cluster {
                    id,
                    size,
                    centroid,
                    chunks: members,
                    cohesion,
                }
            })
            .collect();

        Ok(result)
    }
}

fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn calculate_cohesion(members: &[ClusterMember]) -> f64 {
    if members.is_empty() {
        return 0.0;
    }

    let avg_distance: f64 = members.iter().map(|m| m.distance_to_centroid).sum::<f64>()
        / members.len() as f64;

    // Cohesion: inverse of average distance (higher = better)
    1.0 / (1.0 + avg_distance)
}
```

### Phase 3: Dependency Updates

#### Cargo.toml Changes

```toml
# REMOVE:
# linfa = "0.7"
# linfa-trees = "0.7"
# ndarray = "0.15"

# ADD:
[dependencies]
aprender = "0.1.0"

# Optional: If we need advanced features later
# aprender = { version = "0.1.0", features = ["gpu"] }
```

## Migration Strategy

### Step 1: Preparation (1 day)

1. **Feature Branch**: Create `feature/aprender-integration`
2. **Baseline Tests**: Run all mutation and clustering tests to establish baseline
3. **Dependency Audit**: Identify all linfa/ndarray usage

```bash
# Find all linfa/ndarray usage
rg "use linfa" src/
rg "use ndarray" src/
rg "DecisionTree" src/
rg "Array1|Array2" src/
```

### Step 2: Mutation Predictor Migration (2 days)

1. **RED Phase**: Update tests in `ml_predictor_tests.rs` to expect aprender types
2. **GREEN Phase**: Implement new predictor using LinearRegression
3. **REFACTOR Phase**: Clean up, optimize, document
4. **Verify**: Run mutation tests, ensure accuracy comparable to DecisionTree

```bash
# Test mutation predictor
cargo test --lib services::mutation::ml_predictor
cargo test --lib services::mutation::ml_integration_tests
```

### Step 3: Clustering Implementation (2 days)

1. **RED Phase**: Write tests for KMeans clustering in `clustering_tests.rs`
2. **GREEN Phase**: Implement clustering engine with aprender
3. **REFACTOR Phase**: Optimize, add benchmarks
4. **Verify**: Run clustering tests, verify silhouette scores

```bash
# Test clustering
cargo test --lib services::semantic::clustering
```

### Step 4: Dependency Cleanup (1 day)

1. **Remove Dependencies**: Remove linfa, linfa_trees, ndarray from Cargo.toml
2. **Verify Build**: Ensure no compilation errors
3. **Update Docs**: Update documentation to reference aprender

```bash
# Verify clean build
cargo clean
cargo build --release
cargo test --all
```

### Step 5: Integration Testing (1 day)

1. **Full Test Suite**: Run all tests (lib + integration)
2. **Benchmark Comparison**: Compare performance with old implementation
3. **TDG Score**: Verify TDG score maintained or improved
4. **Coverage**: Ensure coverage remains ≥85%

```bash
# Full validation
cargo test --all
cargo clippy --all-targets -- -D warnings
cargo llvm-cov --lib --summary-only
pmat tdg analyze
```

## Testing Strategy

### Unit Tests

**Mutation Predictor**:
- `test_linear_regression_training` - Train on synthetic data, verify fit
- `test_prediction_accuracy` - Predict kill probabilities, verify in range [0,1]
- `test_feature_extraction` - Verify 18 features extracted correctly
- `test_model_serialization` - Save/load model from disk

**Clustering Engine**:
- `test_kmeans_clustering` - Cluster synthetic embeddings, verify labels
- `test_silhouette_score` - Verify score calculation
- `test_cluster_cohesion` - Verify cohesion metric
- `test_outlier_detection` - Verify outlier identification

### Integration Tests

**Mutation Testing**:
- `test_full_mutation_prediction_workflow` - Train on historical data, predict on new mutants
- `test_cross_validation` - Verify cross-validation with aprender
- `test_prediction_consistency` - Verify deterministic predictions

**Clustering**:
- `test_full_clustering_workflow` - Fetch embeddings, cluster, report results
- `test_multi_language_clustering` - Cluster mixed language codebase
- `test_large_scale_clustering` - Cluster 10,000+ embeddings

### Property Tests

Using `proptest`:

```rust
proptest! {
    #[test]
    fn test_prediction_always_in_range(
        features: Vec<f64> // 18-dimensional
    ) {
        let x = Matrix::from_vec(1, 18, features).unwrap();
        let model = trained_model(); // Pre-trained model
        let prediction = model.predict(&x)[0];

        assert!(prediction >= 0.0 && prediction <= 1.0);
    }

    #[test]
    fn test_clustering_labels_valid(
        embeddings: Vec<Vec<f64>>, // Random embeddings
        k in 2..10usize
    ) {
        let kmeans = KMeans::new(k);
        let labels = kmeans.fit_predict(&to_matrix(embeddings)).unwrap();

        // All labels should be < k
        assert!(labels.iter().all(|&label| label < k));
    }
}
```

## Performance Considerations

### Expected Performance

**Mutation Predictor**:
- **Training Time**: <100ms for 1000 training samples (LinearRegression is O(n³) via normal equations)
- **Prediction Time**: <1ms per mutant (matrix multiplication)
- **Memory**: ~100KB for model storage

**Clustering**:
- **K-Means Time**: ~500ms for 10,000 embeddings, k=10, 100 iterations
- **Memory**: O(n×d) where n=samples, d=dimensions (typically 1536 for OpenAI embeddings)

### Optimization Strategies

1. **Batch Predictions**: Use Matrix operations for batch predictions
2. **Caching**: Cache trained models, avoid re-training
3. **Feature Flags**: Use aprender's `gpu` feature if available
4. **Incremental Training**: Train on new data only, use warm start

## Rollout Plan

### Phase 1: Alpha (Week 1)
- Merge to master behind feature flag
- Flag: `aprender-ml` (default: false)
- Test with internal PMAT development

### Phase 2: Beta (Week 2)
- Enable by default
- Monitor CI/CD pipeline
- Collect performance metrics

### Phase 3: GA (Week 3)
- Remove old linfa code
- Update documentation
- Release PMAT v1.X with aprender integration

## Rollback Plan

If critical issues arise:

1. **Revert Commit**: `git revert <commit-hash>`
2. **Re-enable linfa**: Uncomment linfa dependencies in Cargo.toml
3. **Hotfix Release**: Release PMAT patch version with linfa restored

## Success Metrics

### Quality Gates

- ✅ All tests pass (lib + integration)
- ✅ Zero clippy warnings
- ✅ Coverage ≥85%
- ✅ TDG score ≥92/100 (current baseline)
- ✅ Zero performance regressions (≤10% slowdown)

### Migration Metrics

- ✅ Dependency count reduced by ≥20
- ✅ Compilation time reduced by ≥10%
- ✅ Binary size reduced by ≥5%
- ✅ ML prediction accuracy within 5% of baseline

## Documentation Updates

### Files to Update

1. **README.md**: Replace linfa references with aprender
2. **CLAUDE.md**: Update ML integration notes
3. **CONTRIBUTING.md**: Update testing instructions
4. **docs/specifications/mutation-testing.md**: Update ML architecture
5. **docs/specifications/semantic-clustering.md**: Update clustering implementation

### API Documentation

Update rustdoc comments:

```rust
/// ML-Based Mutant Survivability Predictor
///
/// Uses `aprender` (PAIML's next-gen ML library) for kill probability prediction.
/// Replaces the previous `linfa` decision tree with linear regression for better
/// performance and maintainability.
///
/// # Architecture
///
/// - **Features**: 18-dimensional feature vectors extracted from mutants
/// - **Model**: LinearRegression via ordinary least squares
/// - **Metrics**: R² score, MSE, MAE
/// - **Training**: Supervised learning on historical mutation results
///
/// # Example
///
/// ```rust
/// use pmat::services::mutation::{SurvivabilityPredictor, TrainingData};
///
/// let mut predictor = SurvivabilityPredictor::new();
/// predictor.train(&training_data)?;
///
/// let prediction = predictor.predict(&mutant)?;
/// println!("Kill probability: {:.2}%", prediction.kill_probability * 100.0);
/// ```
pub struct SurvivabilityPredictor { ... }
```

## Risks and Mitigations

### Risk 1: Accuracy Loss

**Risk**: LinearRegression may be less accurate than DecisionTree
**Probability**: Medium
**Impact**: High
**Mitigation**:
- Implement both LinearRegression and KMeans-based approaches
- Compare accuracy with baseline during testing
- If needed, implement ensemble methods or add DecisionTree to aprender

### Risk 2: Performance Regression

**Risk**: Matrix operations slower than ndarray
**Probability**: Low
**Impact**: Medium
**Mitigation**:
- Benchmark before/after with criterion
- Use aprender's `gpu` feature if available
- Profile hot paths with flamegraph

### Risk 3: API Breaking Changes

**Risk**: Aprender API changes in future versions
**Probability**: Low
**Impact**: Low
**Mitigation**:
- Pin to exact version: `aprender = "=0.1.0"`
- PAIML controls aprender development
- Semantic versioning guarantees

### Risk 4: Missing Features

**Risk**: Need advanced ML features not in aprender
**Probability**: Low
**Impact**: Medium
**Mitigation**:
- Aprender is actively developed by PAIML team
- Can add features to aprender as needed
- Temporary: Keep linfa as optional dependency

## Alternatives Considered

### Alternative 1: Keep linfa

**Pros**: No migration work, established library
**Cons**: Heavy dependencies, version conflicts, maintenance concerns
**Decision**: REJECTED - Technical debt outweighs short-term convenience

### Alternative 2: Implement from scratch

**Pros**: Full control, no dependencies
**Cons**: High effort, error-prone, reinventing wheel
**Decision**: REJECTED - Aprender already provides this

### Alternative 3: Use smartcore

**Pros**: Comprehensive ML library, active development
**Cons**: 100+ dependencies, not PAIML-native, TDG score unknown
**Decision**: REJECTED - Aprender better fits PMAT's quality standards

## References

- **Aprender**: https://github.com/paiml/aprender
- **Aprender Docs**: https://docs.rs/aprender
- **PMAT Mutation Testing**: docs/specifications/mutation-testing.md
- **PMAT Clustering**: docs/specifications/semantic-clustering.md
- **TDG Scoring**: docs/specifications/tdg-scoring.md

## Appendix A: Feature Mapping

| Current (linfa) | Aprender | Notes |
|----------------|----------|-------|
| `DecisionTree` | `LinearRegression` or `KMeans` | Classification → Regression |
| `Array1<f64>` | `Vector` | 1D array |
| `Array2<f64>` | `Matrix` | 2D array |
| `Dataset` | Manual conversion | aprender uses Matrix/Vector directly |
| `SplitQuality::Gini` | N/A | Not needed for regression |
| `cross_validate` | Manual implementation | Use aprender's prediction API |

## Appendix B: Code Size Comparison

### Current Implementation

- **ml_predictor.rs**: 850 lines (with linfa)
- **clustering.rs**: 300 lines (placeholder)
- **Dependencies**: linfa (50+ transitive deps)

### Target Implementation

- **ml_predictor.rs**: ~700 lines (with aprender) - **15% reduction**
- **clustering.rs**: ~500 lines (full implementation) - **67% increase** (new functionality)
- **Dependencies**: aprender (0 transitive runtime deps) - **50+ deps removed**

### Net Impact

- **Total Code**: +50 lines (new clustering features)
- **Dependencies**: -50+ deps (massive reduction)
- **Compilation Time**: -10% (fewer dependencies)
- **Binary Size**: -5% (less code to link)

## Appendix C: Timeline

| Week | Phase | Tasks | Deliverables |
|------|-------|-------|--------------|
| 1 | Preparation | Baseline, audit, branch | Feature branch, test baseline |
| 1 | Mutation Migration | RED-GREEN-REFACTOR | Working mutation predictor |
| 2 | Clustering Impl | RED-GREEN-REFACTOR | Working clustering engine |
| 2 | Dependency Cleanup | Remove old deps, verify | Clean Cargo.toml |
| 2 | Integration Testing | Full test suite, benchmarks | Test report, performance data |
| 3 | Alpha Rollout | Feature flag, internal testing | Alpha release |
| 3 | Beta Rollout | Enable by default, monitor | Beta release |
| 3 | GA Rollout | Remove old code, update docs | GA release |

**Total Duration**: 3 weeks
**Effort**: ~40 hours (1 week FTE)

## Appendix D: Test Coverage Plan

### Current Coverage (Baseline)

```bash
# Mutation predictor coverage
cargo llvm-cov --lib services::mutation::ml_predictor
# Expected: ~80% (RED tests exist but not fully implemented)

# Clustering coverage
cargo llvm-cov --lib services::semantic::clustering
# Expected: ~30% (mostly placeholder)
```

### Target Coverage (Post-Migration)

```bash
# Mutation predictor coverage
cargo llvm-cov --lib services::mutation::ml_predictor
# Target: ≥85% (full GREEN implementation)

# Clustering coverage
cargo llvm-cov --lib services::semantic::clustering
# Target: ≥85% (full GREEN implementation)
```

### Coverage by Module

| Module | Current | Target | Gap |
|--------|---------|--------|-----|
| `ml_predictor.rs` | 80% | 85% | +5% |
| `ml_predictor_tests.rs` | 100% | 100% | 0% |
| `clustering.rs` | 30% | 85% | +55% |
| `ml_integration_tests.rs` | 70% | 85% | +15% |

---

**Approval**:
- [ ] Technical Lead: _______________
- [ ] PMAT Team: _______________
- [ ] Date: _______________

**Status**: APPROVED FOR IMPLEMENTATION
