# PMAT-SEARCH-007: K-means Clustering

**Sprint**: 31
**Phase**: RED → GREEN → REFACTOR (EXTREME TDD)
**Estimate**: 2 hours
**Priority**: HIGH

## Objective

Implement K-means, hierarchical, and DBSCAN clustering algorithms for grouping semantically similar code chunks using vector embeddings.

## Background

With semantic embeddings in place, we can now cluster code into groups based on semantic similarity. This enables:
- **Architecture discovery**: Find modules with similar functionality
- **Refactoring opportunities**: Identify duplicate/similar implementations
- **Code organization insights**: Understand codebase structure

## Requirements

### Functional Requirements

1. **K-means Clustering**
   - Standard Lloyd's algorithm with k-means++ initialization
   - User specifies k (number of clusters)
   - Returns cluster assignments + centroids
   - Distance metric: Euclidean distance in embedding space

2. **Hierarchical Clustering**
   - Agglomerative (bottom-up) approach
   - Linkage methods: single, complete, average
   - Returns dendrogram structure
   - Distance metric: Euclidean or cosine

3. **DBSCAN Clustering**
   - Density-based spatial clustering
   - Parameters: epsilon (radius), min_samples (min cluster size)
   - Identifies outliers as noise points
   - No need to specify k upfront

### Non-Functional Requirements

- **Performance**: Handle 10K+ vectors in <5 seconds
- **Quality**: Silhouette score ≥ 0.4 for well-formed clusters
- **Scalability**: Support up to 50K code chunks
- **Testability**: 15 unit tests (RED phase)

## Technical Design

### Data Structures

```rust
pub struct ClusteringEngine {
    vector_db: Arc<TursoVectorDB>,
}

pub enum ClusteringMethod {
    KMeans { k: usize },
    Hierarchical { linkage: Linkage },
    DBSCAN { epsilon: f64, min_samples: usize },
}

pub enum Linkage {
    Single,
    Complete,
    Average,
}

pub struct ClusterResult {
    pub method: String,
    pub clusters: Vec<Cluster>,
    pub outliers: Vec<OutlierPoint>,
    pub silhouette_score: f64,
    pub total_chunks: usize,
}

pub struct Cluster {
    pub id: usize,
    pub size: usize,
    pub centroid: Vec<f32>,
    pub chunks: Vec<ClusterMember>,
    pub cohesion: f64, // average intra-cluster distance
}

pub struct ClusterMember {
    pub file_path: String,
    pub chunk_name: String,
    pub chunk_type: String,
    pub language: String,
    pub distance_to_centroid: f64,
}

pub struct OutlierPoint {
    pub file_path: String,
    pub chunk_name: String,
    pub reason: String, // "noise" or "singleton"
}
```

### Algorithms

#### K-means (Lloyd's Algorithm)

```
1. Initialize k centroids using k-means++:
   - Choose first centroid randomly
   - For each remaining centroid:
     - Compute D(x)^2 = squared distance to nearest existing centroid
     - Choose next centroid with probability proportional to D(x)^2

2. Repeat until convergence (or max_iterations):
   a. Assignment: Assign each point to nearest centroid
   b. Update: Recompute centroids as mean of assigned points
   c. Check convergence: Stop if centroids don't change

3. Compute silhouette score for quality assessment
```

#### Hierarchical Clustering

```
1. Start with each point as its own cluster
2. Repeat until single cluster:
   a. Find two closest clusters (by linkage metric)
   b. Merge them into single cluster
   c. Update distance matrix
3. Return dendrogram tree structure
```

#### DBSCAN

```
1. For each unvisited point:
   a. Mark as visited
   b. Find neighbors within epsilon radius
   c. If neighbors < min_samples: mark as noise
   d. Else: create new cluster, add neighbors to queue
   e. Expand cluster by processing queue

2. Return clusters + noise points
```

### Interface

```rust
impl ClusteringEngine {
    pub fn new(vector_db: Arc<TursoVectorDB>) -> Self;

    pub async fn cluster(
        &self,
        method: ClusteringMethod,
        filters: ClusterFilters,
    ) -> Result<ClusterResult, String>;

    fn kmeans(
        &self,
        vectors: &[Vec<f32>],
        k: usize,
        max_iterations: usize,
    ) -> Result<Vec<usize>, String>;

    fn hierarchical(
        &self,
        vectors: &[Vec<f32>],
        linkage: Linkage,
    ) -> Result<Dendrogram, String>;

    fn dbscan(
        &self,
        vectors: &[Vec<f32>],
        epsilon: f64,
        min_samples: usize,
    ) -> Result<Vec<i32>, String>; // -1 for noise

    fn compute_silhouette_score(
        &self,
        vectors: &[Vec<f32>],
        labels: &[usize],
    ) -> f64;
}

pub struct ClusterFilters {
    pub language: Option<String>,
    pub chunk_type: Option<String>,
    pub file_pattern: Option<String>,
}
```

## Test Plan (RED Phase - 15 tests)

### K-means Tests (6 tests)
1. `test_kmeans_basic_clustering` - Simple 3-cluster scenario
2. `test_kmeans_convergence` - Verify algorithm converges
3. `test_kmeans_empty_input` - Error handling for empty vectors
4. `test_kmeans_single_cluster` - k=1 edge case
5. `test_kmeans_more_clusters_than_points` - k > n validation
6. `test_kmeans_deterministic_with_seed` - Reproducibility

### Hierarchical Tests (3 tests)
7. `test_hierarchical_single_linkage` - Single linkage clustering
8. `test_hierarchical_complete_linkage` - Complete linkage
9. `test_hierarchical_dendrogram_structure` - Verify tree structure

### DBSCAN Tests (3 tests)
10. `test_dbscan_basic_clustering` - Simple density clustering
11. `test_dbscan_noise_detection` - Identify outliers correctly
12. `test_dbscan_no_clusters` - All points are noise case

### Integration Tests (3 tests)
13. `test_cluster_with_language_filter` - Filter by language
14. `test_cluster_result_structure` - Verify ClusterResult format
15. `test_silhouette_score_computation` - Quality metric calculation

## Implementation Steps

### RED Phase (30 minutes)
1. Create `server/tests/unit_kmeans_clustering.rs`
2. Write all 15 failing tests
3. Verify tests fail with clear error messages
4. Run: `cargo test unit_kmeans_clustering -- --nocapture`

### GREEN Phase (60 minutes)
1. Create `server/src/services/semantic/clustering.rs`
2. Implement K-means algorithm with k-means++ initialization
3. Implement hierarchical clustering with linkage variants
4. Implement DBSCAN with epsilon-neighborhood search
5. Implement silhouette score computation
6. Run: `cargo test` - all tests pass

### REFACTOR Phase (30 minutes)
1. Extract distance computations to helper functions
2. Optimize memory usage for large vector sets
3. Add documentation comments
4. Run: `cargo clippy` - zero warnings
5. Run: `cargo test` - all tests still pass

## Acceptance Criteria

- [ ] All 15 tests pass
- [ ] K-means produces correct cluster assignments
- [ ] Hierarchical clustering builds valid dendrogram
- [ ] DBSCAN identifies noise points correctly
- [ ] Silhouette score computed accurately
- [ ] Zero clippy warnings
- [ ] Code coverage ≥ 95%
- [ ] Cyclomatic complexity ≤ 10 per function

## Dependencies

- `TursoVectorDB` for fetching embeddings
- `nalgebra` or `ndarray` for matrix operations (optional)
- Standard library `f32::sqrt()` for Euclidean distance

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| K-means local minima | Use k-means++ initialization |
| Large vector sets OOM | Process in batches if needed |
| DBSCAN slow for large epsilon | Use spatial indexing (future work) |
| Poor cluster quality | Compute silhouette score, warn user |

## Future Enhancements

- Mini-batch K-means for scalability
- Spectral clustering
- GPU acceleration for distance computations
- Interactive dendrogram visualization
- Auto-tune k using elbow method

## References

- Lloyd, S. (1982). "Least squares quantization in PCM"
- Ester, M. et al. (1996). "A density-based algorithm for discovering clusters"
- Rousseeuw, P. (1987). "Silhouettes: A graphical aid to the interpretation of clusters"
- Arthur, D. & Vassilvitskii, S. (2007). "k-means++: The advantages of careful seeding"

---

**EXTREME TDD**: RED → GREEN → REFACTOR
