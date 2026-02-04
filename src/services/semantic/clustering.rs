// Clustering Algorithms for Code Embeddings
// PMAT-SEARCH-007: K-means, Hierarchical, and DBSCAN clustering
//
// GREEN Phase: Implement algorithms

use super::TursoVectorDB;
use std::collections::HashMap;
use std::sync::Arc;

// Import aprender for ML algorithms (Phase 2 migration)
use aprender::prelude::*;

/// Clustering engine
pub struct ClusteringEngine {
    #[allow(dead_code)] // Reserved for future clustering Phase 2 integration
    vector_db: Arc<TursoVectorDB>,
}

/// Clustering method specification
#[derive(Debug, Clone)]
pub enum ClusteringMethod {
    KMeans { k: usize },
    Hierarchical { linkage: Linkage },
    DBSCAN { epsilon: f64, min_samples: usize },
}

/// Hierarchical clustering linkage methods
#[derive(Debug, Clone, Copy)]
pub enum Linkage {
    Single,
    Complete,
    Average,
}

/// Result of clustering operation
#[derive(Debug, Clone)]
pub struct ClusterResult {
    pub method: String,
    pub clusters: Vec<Cluster>,
    pub outliers: Vec<OutlierPoint>,
    pub silhouette_score: f64,
    pub total_chunks: usize,
}

/// A single cluster
#[derive(Debug, Clone)]
pub struct Cluster {
    pub id: usize,
    pub size: usize,
    pub centroid: Vec<f32>,
    pub chunks: Vec<ClusterMember>,
    pub cohesion: f64,
}

/// Member of a cluster
#[derive(Debug, Clone)]
pub struct ClusterMember {
    pub file_path: String,
    pub chunk_name: String,
    pub chunk_type: String,
    pub language: String,
    pub distance_to_centroid: f64,
}

/// Outlier point not belonging to any cluster
#[derive(Debug, Clone)]
pub struct OutlierPoint {
    pub file_path: String,
    pub chunk_name: String,
    pub reason: String,
}

/// Filters for clustering
#[derive(Debug, Clone, Default)]
pub struct ClusterFilters {
    pub language: Option<String>,
    pub chunk_type: Option<String>,
    pub file_pattern: Option<String>,
}

/// Dendrogram for hierarchical clustering
#[derive(Debug, Clone)]
pub struct Dendrogram {
    pub merges: Vec<DendrogramMerge>,
}

/// A single merge in the dendrogram
#[derive(Debug, Clone)]
pub struct DendrogramMerge {
    pub cluster1: usize,
    pub cluster2: usize,
    pub distance: f64,
}

impl ClusteringEngine {
    /// Create new clustering engine
    pub fn new(vector_db: Arc<TursoVectorDB>) -> Self {
        Self { vector_db }
    }

    /// Perform clustering
    pub async fn cluster(
        &self,
        method: ClusteringMethod,
        _filters: ClusterFilters,
    ) -> Result<ClusterResult, String> {
        // For now, return empty result
        let method_name = match method {
            ClusteringMethod::KMeans { .. } => "kmeans",
            ClusteringMethod::Hierarchical { .. } => "hierarchical",
            ClusteringMethod::DBSCAN { .. } => "dbscan",
        };

        Ok(ClusterResult {
            method: method_name.to_string(),
            clusters: Vec::new(),
            outliers: Vec::new(),
            silhouette_score: 0.0,
            total_chunks: 0,
        })
    }

    /// Convert Vec<Vec<f32>> to aprender Matrix
    /// Helper for Phase 2 migration to aprender
    fn vectors_to_matrix(vectors: &[Vec<f32>]) -> Result<Matrix<f32>, String> {
        if vectors.is_empty() {
            return Err("Cannot convert empty vector set".to_string());
        }

        let rows = vectors.len();
        let cols = vectors[0].len();

        // Flatten to 1D vector for Matrix::from_vec
        let data: Vec<f32> = vectors.iter().flat_map(|v| v.iter().copied()).collect();

        Matrix::from_vec(rows, cols, data).map_err(|e| format!("Matrix conversion error: {e:?}"))
    }

    /// K-means clustering implementation
    ///
    /// # Arguments
    /// * `vectors` - Array of vectors to cluster
    /// * `k` - Number of clusters
    /// * `max_iterations` - Maximum iterations before stopping
    ///
    /// # Returns
    /// Array of cluster labels (0 to k-1)
    pub fn kmeans(
        &self,
        vectors: &[Vec<f32>],
        k: usize,
        max_iterations: usize,
    ) -> Result<Vec<usize>, String> {
        // Phase 2: Use aprender for KMeans clustering (replaced custom implementation)
        if vectors.is_empty() {
            return Err("Cannot cluster empty vector set".to_string());
        }

        if k == 0 {
            return Err("k must be greater than 0".to_string());
        }

        if k > vectors.len() {
            return Err("Cannot have more clusters than points".to_string());
        }

        // Special case: single cluster
        if k == 1 {
            return Ok(vec![0; vectors.len()]);
        }

        // Convert to aprender Matrix
        let matrix = Self::vectors_to_matrix(vectors)?;

        // Use aprender KMeans
        let mut kmeans = KMeans::new(k).with_max_iter(max_iterations);

        kmeans
            .fit(&matrix)
            .map_err(|e| format!("KMeans fit error: {e:?}"))?;

        let labels = kmeans.predict(&matrix);

        Ok(labels)
    }

    /// K-means with seed for deterministic results
    pub fn kmeans_with_seed(
        &self,
        vectors: &[Vec<f32>],
        k: usize,
        max_iterations: usize,
        seed: u64,
    ) -> Result<Vec<usize>, String> {
        // Phase 2: Use aprender with seed for deterministic results
        if vectors.is_empty() {
            return Err("Cannot cluster empty vector set".to_string());
        }

        if k == 0 {
            return Err("k must be greater than 0".to_string());
        }

        if k > vectors.len() {
            return Err("Cannot have more clusters than points".to_string());
        }

        // Special case: single cluster
        if k == 1 {
            return Ok(vec![0; vectors.len()]);
        }

        // Convert to aprender Matrix
        let matrix = Self::vectors_to_matrix(vectors)?;

        // Use aprender KMeans with seed
        let mut kmeans = KMeans::new(k)
            .with_max_iter(max_iterations)
            .with_random_state(seed);

        kmeans
            .fit(&matrix)
            .map_err(|e| format!("KMeans fit error: {e:?}"))?;

        let labels = kmeans.predict(&matrix);

        Ok(labels)
    }

    /// Hierarchical clustering
    ///
    /// Note: This uses a custom implementation (not aprender) because it returns
    /// a Dendrogram structure showing the merge history, which is useful for
    /// visualization. aprender's HierarchicalClustering returns cluster labels only.
    pub fn hierarchical(
        &self,
        vectors: &[Vec<f32>],
        linkage: Linkage,
    ) -> Result<Dendrogram, String> {
        if vectors.is_empty() {
            return Err("Cannot cluster empty vector set".to_string());
        }

        let n = vectors.len();
        let mut merges = Vec::new();
        let mut clusters: Vec<Vec<usize>> = (0..n).map(|i| vec![i]).collect();

        // Compute initial distance matrix
        let mut distances = HashMap::new();
        for i in 0..n {
            for j in (i + 1)..n {
                let dist = self.euclidean_distance(&vectors[i], &vectors[j]);
                distances.insert((i, j), dist);
            }
        }

        // Agglomerative clustering
        while clusters.len() > 1 {
            // Find closest pair
            let mut min_dist = f64::MAX;
            let mut min_i = 0;
            let mut min_j = 1;

            for i in 0..clusters.len() {
                for j in (i + 1)..clusters.len() {
                    let dist = self.cluster_distance(
                        &clusters[i],
                        &clusters[j],
                        &distances,
                        vectors,
                        linkage,
                    );
                    if dist < min_dist {
                        min_dist = dist;
                        min_i = i;
                        min_j = j;
                    }
                }
            }

            // Merge clusters
            merges.push(DendrogramMerge {
                cluster1: min_i,
                cluster2: min_j,
                distance: min_dist,
            });

            let mut merged = clusters[min_i].clone();
            merged.extend(&clusters[min_j]);

            // Remove merged clusters and add new one
            clusters.remove(min_j);
            clusters.remove(min_i);
            clusters.push(merged);
        }

        Ok(Dendrogram { merges })
    }

    /// Compute distance between two clusters
    fn cluster_distance(
        &self,
        cluster1: &[usize],
        cluster2: &[usize],
        distances: &HashMap<(usize, usize), f64>,
        _vectors: &[Vec<f32>],
        linkage: Linkage,
    ) -> f64 {
        let mut dists = Vec::new();

        for &i in cluster1 {
            for &j in cluster2 {
                let key = if i < j { (i, j) } else { (j, i) };
                if let Some(&dist) = distances.get(&key) {
                    dists.push(dist);
                }
            }
        }

        if dists.is_empty() {
            return f64::MAX;
        }

        match linkage {
            Linkage::Single => *dists
                .iter()
                .min_by(|a, b| a.total_cmp(b))
                .expect("internal error"),
            Linkage::Complete => *dists
                .iter()
                .max_by(|a, b| a.total_cmp(b))
                .expect("internal error"),
            Linkage::Average => dists.iter().sum::<f64>() / dists.len() as f64,
        }
    }

    /// DBSCAN clustering
    ///
    /// # Arguments
    /// * `vectors` - Array of vectors to cluster
    /// * `epsilon` - Maximum distance for neighborhood
    /// * `min_samples` - Minimum points for core point
    ///
    /// # Returns
    /// Array of cluster labels (-1 for noise, 0+ for clusters)
    pub fn dbscan(
        &self,
        vectors: &[Vec<f32>],
        epsilon: f64,
        min_samples: usize,
    ) -> Result<Vec<i32>, String> {
        // Phase 2: Use aprender for DBSCAN clustering (replaced custom implementation)
        if vectors.is_empty() {
            return Err("Cannot cluster empty vector set".to_string());
        }

        // Convert to aprender Matrix
        let matrix = Self::vectors_to_matrix(vectors)?;

        // Use aprender DBSCAN (cast epsilon to f32 for aprender API)
        let mut dbscan = DBSCAN::new(epsilon as f32, min_samples);

        dbscan
            .fit(&matrix)
            .map_err(|e| format!("DBSCAN fit error: {e:?}"))?;

        let labels = dbscan.predict(&matrix);

        Ok(labels)
    }

    /// Compute silhouette score for clustering quality
    pub fn compute_silhouette_score(&self, vectors: &[Vec<f32>], labels: &[usize]) -> f64 {
        if vectors.is_empty() || labels.is_empty() {
            return 0.0;
        }

        let n = vectors.len();
        let mut silhouette_sum = 0.0;

        for i in 0..n {
            let a = self.intra_cluster_distance(vectors, labels, i);
            let b = self.nearest_cluster_distance(vectors, labels, i);

            let silhouette = if a < b {
                1.0 - (a / b)
            } else if a > b {
                (b / a) - 1.0
            } else {
                0.0
            };

            silhouette_sum += silhouette;
        }

        silhouette_sum / n as f64
    }

    /// Average distance to points in same cluster
    fn intra_cluster_distance(
        &self,
        vectors: &[Vec<f32>],
        labels: &[usize],
        point_idx: usize,
    ) -> f64 {
        let cluster_label = labels[point_idx];
        let mut sum = 0.0;
        let mut count = 0;

        for (i, &label) in labels.iter().enumerate() {
            if label == cluster_label && i != point_idx {
                sum += self.euclidean_distance(&vectors[point_idx], &vectors[i]);
                count += 1;
            }
        }

        if count == 0 {
            0.0
        } else {
            sum / count as f64
        }
    }

    /// Average distance to nearest cluster
    fn nearest_cluster_distance(
        &self,
        vectors: &[Vec<f32>],
        labels: &[usize],
        point_idx: usize,
    ) -> f64 {
        let current_cluster = labels[point_idx];
        let mut min_avg_dist = f64::MAX;

        // Find all unique clusters
        let mut clusters: Vec<usize> = labels.to_vec();
        clusters.sort();
        clusters.dedup();

        for &cluster_label in &clusters {
            if cluster_label == current_cluster {
                continue;
            }

            let mut sum = 0.0;
            let mut count = 0;

            for (i, &label) in labels.iter().enumerate() {
                if label == cluster_label {
                    sum += self.euclidean_distance(&vectors[point_idx], &vectors[i]);
                    count += 1;
                }
            }

            if count > 0 {
                let avg_dist = sum / count as f64;
                if avg_dist < min_avg_dist {
                    min_avg_dist = avg_dist;
                }
            }
        }

        min_avg_dist
    }

    /// Compute Euclidean distance between two vectors
    fn euclidean_distance(&self, v1: &[f32], v2: &[f32]) -> f64 {
        if v1.len() != v2.len() {
            return f64::MAX;
        }

        let sum: f32 = v1
            .iter()
            .zip(v2.iter())
            .map(|(a, b)| (a - b) * (a - b))
            .sum();

        (sum as f64).sqrt()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a clustering engine for tests
    async fn create_test_engine() -> ClusteringEngine {
        let db = TursoVectorDB::new_local(":memory:")
            .await
            .expect("Failed to create test database");
        ClusteringEngine::new(Arc::new(db))
    }

    // ==================== Euclidean Distance Tests ====================

    #[tokio::test]
    async fn test_euclidean_distance_unit_vector() {
        let engine = create_test_engine().await;

        let v1 = vec![0.0, 0.0, 0.0];
        let v2 = vec![1.0, 0.0, 0.0];

        let dist = engine.euclidean_distance(&v1, &v2);
        assert!((dist - 1.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_euclidean_distance_same_point() {
        let engine = create_test_engine().await;

        let v = vec![1.0, 2.0, 3.0];
        let dist = engine.euclidean_distance(&v, &v);
        assert!((dist - 0.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_euclidean_distance_3d() {
        let engine = create_test_engine().await;

        let v1 = vec![0.0, 0.0, 0.0];
        let v2 = vec![1.0, 1.0, 1.0];

        let dist = engine.euclidean_distance(&v1, &v2);
        // sqrt(1 + 1 + 1) = sqrt(3) ≈ 1.732
        assert!((dist - 3.0_f64.sqrt()).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_euclidean_distance_high_dimensional() {
        let engine = create_test_engine().await;

        let v1 = vec![0.0; 128];
        let mut v2 = vec![0.0; 128];
        v2[0] = 3.0;
        v2[1] = 4.0;

        let dist = engine.euclidean_distance(&v1, &v2);
        // sqrt(9 + 16) = 5
        assert!((dist - 5.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_euclidean_distance_different_lengths() {
        let engine = create_test_engine().await;

        let v1 = vec![1.0, 2.0, 3.0];
        let v2 = vec![1.0, 2.0];

        let dist = engine.euclidean_distance(&v1, &v2);
        assert_eq!(dist, f64::MAX);
    }

    #[tokio::test]
    async fn test_euclidean_distance_empty_vectors() {
        let engine = create_test_engine().await;

        let v1: Vec<f32> = vec![];
        let v2: Vec<f32> = vec![];

        let dist = engine.euclidean_distance(&v1, &v2);
        assert!((dist - 0.0).abs() < 1e-6);
    }

    // ==================== K-Means Tests ====================

    #[tokio::test]
    async fn test_kmeans_basic_clustering() {
        let engine = create_test_engine().await;

        // Two well-separated clusters
        let vectors = vec![
            vec![0.0, 0.0],
            vec![0.1, 0.1],
            vec![0.2, 0.0],
            vec![10.0, 10.0],
            vec![10.1, 10.1],
            vec![10.2, 10.0],
        ];

        let labels = engine.kmeans(&vectors, 2, 100).unwrap();

        assert_eq!(labels.len(), 6);

        // Points 0, 1, 2 should be in same cluster
        assert_eq!(labels[0], labels[1]);
        assert_eq!(labels[1], labels[2]);

        // Points 3, 4, 5 should be in same cluster
        assert_eq!(labels[3], labels[4]);
        assert_eq!(labels[4], labels[5]);

        // The two groups should be in different clusters
        assert_ne!(labels[0], labels[3]);
    }

    #[tokio::test]
    async fn test_kmeans_single_cluster() {
        let engine = create_test_engine().await;

        let vectors = vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]];

        let labels = engine.kmeans(&vectors, 1, 100).unwrap();

        assert_eq!(labels.len(), 3);
        assert!(labels.iter().all(|&l| l == 0));
    }

    #[tokio::test]
    async fn test_kmeans_empty_vectors() {
        let engine = create_test_engine().await;

        let vectors: Vec<Vec<f32>> = vec![];
        let result = engine.kmeans(&vectors, 2, 100);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot cluster empty vector set");
    }

    #[tokio::test]
    async fn test_kmeans_k_zero() {
        let engine = create_test_engine().await;

        let vectors = vec![vec![1.0, 2.0]];
        let result = engine.kmeans(&vectors, 0, 100);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "k must be greater than 0");
    }

    #[tokio::test]
    async fn test_kmeans_k_greater_than_points() {
        let engine = create_test_engine().await;

        let vectors = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let result = engine.kmeans(&vectors, 5, 100);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot have more clusters than points");
    }

    #[tokio::test]
    async fn test_kmeans_single_point() {
        let engine = create_test_engine().await;

        let vectors = vec![vec![1.0, 2.0, 3.0]];
        let labels = engine.kmeans(&vectors, 1, 100).unwrap();

        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0], 0);
    }

    #[tokio::test]
    async fn test_kmeans_k_equals_points() {
        let engine = create_test_engine().await;

        let vectors = vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]];

        let labels = engine.kmeans(&vectors, 3, 100).unwrap();

        assert_eq!(labels.len(), 3);
        // Each point should be in its own cluster (or at least all labels used)
        let unique_labels: std::collections::HashSet<_> = labels.iter().collect();
        assert_eq!(unique_labels.len(), 3);
    }

    // ==================== K-Means with Seed Tests ====================

    #[tokio::test]
    async fn test_kmeans_with_seed_deterministic() {
        let engine = create_test_engine().await;

        let vectors = vec![
            vec![0.0, 0.0],
            vec![0.1, 0.1],
            vec![10.0, 10.0],
            vec![10.1, 10.1],
        ];

        let labels1 = engine.kmeans_with_seed(&vectors, 2, 100, 42).unwrap();
        let labels2 = engine.kmeans_with_seed(&vectors, 2, 100, 42).unwrap();

        assert_eq!(labels1, labels2);
    }

    #[tokio::test]
    async fn test_kmeans_with_seed_different_seeds() {
        let engine = create_test_engine().await;

        // Using well-separated clusters to ensure clustering is deterministic
        let vectors = vec![
            vec![0.0, 0.0],
            vec![0.01, 0.01],
            vec![100.0, 100.0],
            vec![100.01, 100.01],
        ];

        let labels1 = engine.kmeans_with_seed(&vectors, 2, 100, 42).unwrap();
        let labels2 = engine.kmeans_with_seed(&vectors, 2, 100, 123).unwrap();

        // Both should correctly cluster (same groupings even if labels differ)
        assert_eq!(labels1[0], labels1[1]);
        assert_eq!(labels1[2], labels1[3]);
        assert_ne!(labels1[0], labels1[2]);

        assert_eq!(labels2[0], labels2[1]);
        assert_eq!(labels2[2], labels2[3]);
        assert_ne!(labels2[0], labels2[2]);
    }

    #[tokio::test]
    async fn test_kmeans_with_seed_empty_vectors() {
        let engine = create_test_engine().await;

        let vectors: Vec<Vec<f32>> = vec![];
        let result = engine.kmeans_with_seed(&vectors, 2, 100, 42);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot cluster empty vector set");
    }

    #[tokio::test]
    async fn test_kmeans_with_seed_k_zero() {
        let engine = create_test_engine().await;

        let vectors = vec![vec![1.0, 2.0]];
        let result = engine.kmeans_with_seed(&vectors, 0, 100, 42);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "k must be greater than 0");
    }

    #[tokio::test]
    async fn test_kmeans_with_seed_k_greater_than_points() {
        let engine = create_test_engine().await;

        let vectors = vec![vec![1.0, 2.0]];
        let result = engine.kmeans_with_seed(&vectors, 3, 100, 42);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot have more clusters than points");
    }

    #[tokio::test]
    async fn test_kmeans_with_seed_single_cluster() {
        let engine = create_test_engine().await;

        let vectors = vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]];

        let labels = engine.kmeans_with_seed(&vectors, 1, 100, 42).unwrap();

        assert_eq!(labels.len(), 3);
        assert!(labels.iter().all(|&l| l == 0));
    }

    // ==================== Hierarchical Clustering Tests ====================

    #[tokio::test]
    async fn test_hierarchical_basic() {
        let engine = create_test_engine().await;

        let vectors = vec![
            vec![0.0, 0.0],
            vec![1.0, 0.0],
            vec![10.0, 0.0],
            vec![11.0, 0.0],
        ];

        let dendrogram = engine.hierarchical(&vectors, Linkage::Single).unwrap();

        // Should have n-1 merges
        assert_eq!(dendrogram.merges.len(), 3);
    }

    #[tokio::test]
    async fn test_hierarchical_single_linkage() {
        let engine = create_test_engine().await;

        let vectors = vec![vec![0.0, 0.0], vec![1.0, 0.0], vec![2.0, 0.0]];

        let dendrogram = engine.hierarchical(&vectors, Linkage::Single).unwrap();

        assert_eq!(dendrogram.merges.len(), 2);
        // First merge should have distance ~1.0
        assert!((dendrogram.merges[0].distance - 1.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_hierarchical_complete_linkage() {
        let engine = create_test_engine().await;

        let vectors = vec![vec![0.0, 0.0], vec![1.0, 0.0], vec![2.0, 0.0]];

        let dendrogram = engine.hierarchical(&vectors, Linkage::Complete).unwrap();

        assert_eq!(dendrogram.merges.len(), 2);
    }

    #[tokio::test]
    async fn test_hierarchical_average_linkage() {
        let engine = create_test_engine().await;

        let vectors = vec![vec![0.0, 0.0], vec![1.0, 0.0], vec![2.0, 0.0]];

        let dendrogram = engine.hierarchical(&vectors, Linkage::Average).unwrap();

        assert_eq!(dendrogram.merges.len(), 2);
    }

    #[tokio::test]
    async fn test_hierarchical_empty_vectors() {
        let engine = create_test_engine().await;

        let vectors: Vec<Vec<f32>> = vec![];
        let result = engine.hierarchical(&vectors, Linkage::Single);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot cluster empty vector set");
    }

    #[tokio::test]
    async fn test_hierarchical_single_point() {
        let engine = create_test_engine().await;

        let vectors = vec![vec![1.0, 2.0, 3.0]];
        let dendrogram = engine.hierarchical(&vectors, Linkage::Single).unwrap();

        // No merges for single point
        assert_eq!(dendrogram.merges.len(), 0);
    }

    #[tokio::test]
    async fn test_hierarchical_two_points() {
        let engine = create_test_engine().await;

        let vectors = vec![vec![0.0, 0.0], vec![3.0, 4.0]];
        let dendrogram = engine.hierarchical(&vectors, Linkage::Single).unwrap();

        assert_eq!(dendrogram.merges.len(), 1);
        // Distance should be 5 (3-4-5 triangle)
        assert!((dendrogram.merges[0].distance - 5.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_hierarchical_merge_ordering() {
        let engine = create_test_engine().await;

        // Clusters: (0, 0), (0.1, 0) are very close
        // (10, 0) is far
        let vectors = vec![vec![0.0, 0.0], vec![0.1, 0.0], vec![10.0, 0.0]];

        let dendrogram = engine.hierarchical(&vectors, Linkage::Single).unwrap();

        assert_eq!(dendrogram.merges.len(), 2);
        // First merge should have small distance
        assert!(dendrogram.merges[0].distance < 1.0);
        // Second merge should have larger distance
        assert!(dendrogram.merges[1].distance > 1.0);
    }

    // ==================== DBSCAN Tests ====================

    #[tokio::test]
    async fn test_dbscan_basic() {
        let engine = create_test_engine().await;

        // Two clusters of 3 points each
        let vectors = vec![
            vec![0.0, 0.0],
            vec![0.1, 0.0],
            vec![0.0, 0.1],
            vec![10.0, 10.0],
            vec![10.1, 10.0],
            vec![10.0, 10.1],
        ];

        let labels = engine.dbscan(&vectors, 0.5, 2).unwrap();

        assert_eq!(labels.len(), 6);

        // First three points should be in same cluster
        assert_eq!(labels[0], labels[1]);
        assert_eq!(labels[1], labels[2]);
        assert!(labels[0] >= 0);

        // Last three points should be in same cluster
        assert_eq!(labels[3], labels[4]);
        assert_eq!(labels[4], labels[5]);
        assert!(labels[3] >= 0);

        // Different clusters
        assert_ne!(labels[0], labels[3]);
    }

    #[tokio::test]
    async fn test_dbscan_noise_points() {
        let engine = create_test_engine().await;

        // One cluster and one isolated noise point
        let vectors = vec![
            vec![0.0, 0.0],
            vec![0.1, 0.0],
            vec![0.0, 0.1],
            vec![100.0, 100.0], // Isolated point
        ];

        let labels = engine.dbscan(&vectors, 0.5, 2).unwrap();

        // Last point should be noise (-1)
        assert_eq!(labels[3], -1);

        // First three should be in a cluster
        assert_eq!(labels[0], labels[1]);
        assert_eq!(labels[1], labels[2]);
        assert!(labels[0] >= 0);
    }

    #[tokio::test]
    async fn test_dbscan_all_noise() {
        let engine = create_test_engine().await;

        // All points too far apart
        let vectors = vec![
            vec![0.0, 0.0],
            vec![10.0, 0.0],
            vec![20.0, 0.0],
            vec![30.0, 0.0],
        ];

        let labels = engine.dbscan(&vectors, 0.5, 2).unwrap();

        // All points should be noise
        assert!(labels.iter().all(|&l| l == -1));
    }

    #[tokio::test]
    async fn test_dbscan_single_cluster() {
        let engine = create_test_engine().await;

        // All points close together
        let vectors = vec![
            vec![0.0, 0.0],
            vec![0.1, 0.0],
            vec![0.2, 0.0],
            vec![0.3, 0.0],
        ];

        let labels = engine.dbscan(&vectors, 0.5, 2).unwrap();

        // All points should be in same cluster
        assert!(labels.iter().all(|&l| l == labels[0]));
        assert!(labels[0] >= 0);
    }

    #[tokio::test]
    async fn test_dbscan_empty_vectors() {
        let engine = create_test_engine().await;

        let vectors: Vec<Vec<f32>> = vec![];
        let result = engine.dbscan(&vectors, 0.5, 2);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot cluster empty vector set");
    }

    #[tokio::test]
    async fn test_dbscan_large_epsilon() {
        let engine = create_test_engine().await;

        // With large epsilon, all points should be in one cluster
        let vectors = vec![vec![0.0, 0.0], vec![5.0, 0.0], vec![10.0, 0.0]];

        let labels = engine.dbscan(&vectors, 100.0, 2).unwrap();

        // All in same cluster
        assert_eq!(labels[0], labels[1]);
        assert_eq!(labels[1], labels[2]);
        assert!(labels[0] >= 0);
    }

    #[tokio::test]
    async fn test_dbscan_high_min_samples() {
        let engine = create_test_engine().await;

        // With high min_samples, all become noise
        let vectors = vec![vec![0.0, 0.0], vec![0.1, 0.0], vec![0.2, 0.0]];

        let labels = engine.dbscan(&vectors, 0.5, 10).unwrap();

        // All noise since min_samples > number of points
        assert!(labels.iter().all(|&l| l == -1));
    }

    // ==================== Silhouette Score Tests ====================

    #[tokio::test]
    async fn test_silhouette_score_perfect_clusters() {
        let engine = create_test_engine().await;

        // Two perfectly separated clusters
        let vectors = vec![
            vec![0.0, 0.0],
            vec![0.01, 0.01],
            vec![100.0, 100.0],
            vec![100.01, 100.01],
        ];
        let labels = vec![0, 0, 1, 1];

        let score = engine.compute_silhouette_score(&vectors, &labels);

        // Should be close to 1.0 for well-separated clusters
        assert!(score > 0.9);
    }

    #[tokio::test]
    async fn test_silhouette_score_overlapping_clusters() {
        let engine = create_test_engine().await;

        // Overlapping clusters - poor separation
        let vectors = vec![
            vec![0.0, 0.0],
            vec![1.0, 1.0],
            vec![0.5, 0.5],
            vec![1.5, 1.5],
        ];
        let labels = vec![0, 0, 1, 1];

        let score = engine.compute_silhouette_score(&vectors, &labels);

        // Score should be lower due to overlap
        assert!(score < 0.9);
    }

    #[tokio::test]
    async fn test_silhouette_score_empty_vectors() {
        let engine = create_test_engine().await;

        let vectors: Vec<Vec<f32>> = vec![];
        let labels: Vec<usize> = vec![];

        let score = engine.compute_silhouette_score(&vectors, &labels);

        assert_eq!(score, 0.0);
    }

    #[tokio::test]
    async fn test_silhouette_score_single_point() {
        let engine = create_test_engine().await;

        let vectors = vec![vec![1.0, 2.0, 3.0]];
        let labels = vec![0];

        let score = engine.compute_silhouette_score(&vectors, &labels);

        // Single point: a=0, b=MAX, silhouette = 1.0 - (0/MAX) = 1.0
        assert!(score > 0.9);
    }

    #[tokio::test]
    async fn test_silhouette_score_single_cluster() {
        let engine = create_test_engine().await;

        let vectors = vec![vec![0.0, 0.0], vec![1.0, 0.0], vec![2.0, 0.0]];
        let labels = vec![0, 0, 0];

        let score = engine.compute_silhouette_score(&vectors, &labels);

        // Single cluster: nearest_cluster_distance returns MAX
        // silhouette = 1.0 - (a / MAX) ≈ 1.0
        assert!(score > 0.9);
    }

    // ==================== Matrix Conversion Tests ====================

    #[tokio::test]
    async fn test_vectors_to_matrix_basic() {
        let vectors = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];

        let matrix = ClusteringEngine::vectors_to_matrix(&vectors).unwrap();

        assert_eq!(matrix.n_rows(), 2);
        assert_eq!(matrix.n_cols(), 3);
    }

    #[tokio::test]
    async fn test_vectors_to_matrix_empty() {
        let vectors: Vec<Vec<f32>> = vec![];

        let result = ClusteringEngine::vectors_to_matrix(&vectors);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot convert empty vector set");
    }

    #[tokio::test]
    async fn test_vectors_to_matrix_single_vector() {
        let vectors = vec![vec![1.0, 2.0, 3.0, 4.0, 5.0]];

        let matrix = ClusteringEngine::vectors_to_matrix(&vectors).unwrap();

        assert_eq!(matrix.n_rows(), 1);
        assert_eq!(matrix.n_cols(), 5);
    }

    #[tokio::test]
    async fn test_vectors_to_matrix_high_dimensional() {
        let vectors = vec![vec![0.0; 128], vec![1.0; 128], vec![2.0; 128]];

        let matrix = ClusteringEngine::vectors_to_matrix(&vectors).unwrap();

        assert_eq!(matrix.n_rows(), 3);
        assert_eq!(matrix.n_cols(), 128);
    }

    // ==================== Cluster Method Tests ====================

    #[tokio::test]
    async fn test_cluster_async_kmeans() {
        let engine = create_test_engine().await;

        let result = engine
            .cluster(ClusteringMethod::KMeans { k: 3 }, ClusterFilters::default())
            .await
            .unwrap();

        assert_eq!(result.method, "kmeans");
        assert_eq!(result.total_chunks, 0);
    }

    #[tokio::test]
    async fn test_cluster_async_hierarchical() {
        let engine = create_test_engine().await;

        let result = engine
            .cluster(
                ClusteringMethod::Hierarchical {
                    linkage: Linkage::Complete,
                },
                ClusterFilters::default(),
            )
            .await
            .unwrap();

        assert_eq!(result.method, "hierarchical");
    }

    #[tokio::test]
    async fn test_cluster_async_dbscan() {
        let engine = create_test_engine().await;

        let result = engine
            .cluster(
                ClusteringMethod::DBSCAN {
                    epsilon: 0.5,
                    min_samples: 5,
                },
                ClusterFilters::default(),
            )
            .await
            .unwrap();

        assert_eq!(result.method, "dbscan");
    }

    // ==================== Data Structure Tests ====================

    #[test]
    fn test_cluster_result_creation() {
        let cluster = Cluster {
            id: 0,
            size: 3,
            centroid: vec![1.0, 2.0, 3.0],
            chunks: vec![],
            cohesion: 0.95,
        };

        let result = ClusterResult {
            method: "kmeans".to_string(),
            clusters: vec![cluster],
            outliers: vec![],
            silhouette_score: 0.85,
            total_chunks: 10,
        };

        assert_eq!(result.method, "kmeans");
        assert_eq!(result.clusters.len(), 1);
        assert_eq!(result.clusters[0].id, 0);
        assert_eq!(result.clusters[0].size, 3);
        assert!((result.clusters[0].cohesion - 0.95).abs() < 1e-6);
        assert!((result.silhouette_score - 0.85).abs() < 1e-6);
    }

    #[test]
    fn test_cluster_member_creation() {
        let member = ClusterMember {
            file_path: "src/main.rs".to_string(),
            chunk_name: "process_data".to_string(),
            chunk_type: "function".to_string(),
            language: "rust".to_string(),
            distance_to_centroid: 0.123,
        };

        assert_eq!(member.file_path, "src/main.rs");
        assert_eq!(member.chunk_name, "process_data");
        assert_eq!(member.chunk_type, "function");
        assert_eq!(member.language, "rust");
        assert!((member.distance_to_centroid - 0.123).abs() < 1e-6);
    }

    #[test]
    fn test_outlier_point_creation() {
        let outlier = OutlierPoint {
            file_path: "src/utils.rs".to_string(),
            chunk_name: "helper_fn".to_string(),
            reason: "Too far from any cluster centroid".to_string(),
        };

        assert_eq!(outlier.file_path, "src/utils.rs");
        assert_eq!(outlier.chunk_name, "helper_fn");
        assert!(outlier.reason.contains("centroid"));
    }

    #[test]
    fn test_cluster_filters_default() {
        let filters = ClusterFilters::default();

        assert!(filters.language.is_none());
        assert!(filters.chunk_type.is_none());
        assert!(filters.file_pattern.is_none());
    }

    #[test]
    fn test_cluster_filters_with_values() {
        let filters = ClusterFilters {
            language: Some("rust".to_string()),
            chunk_type: Some("function".to_string()),
            file_pattern: Some("src/**/*.rs".to_string()),
        };

        assert_eq!(filters.language.as_deref(), Some("rust"));
        assert_eq!(filters.chunk_type.as_deref(), Some("function"));
        assert_eq!(filters.file_pattern.as_deref(), Some("src/**/*.rs"));
    }

    #[test]
    fn test_dendrogram_creation() {
        let merges = vec![
            DendrogramMerge {
                cluster1: 0,
                cluster2: 1,
                distance: 0.5,
            },
            DendrogramMerge {
                cluster1: 2,
                cluster2: 3,
                distance: 1.0,
            },
        ];

        let dendrogram = Dendrogram { merges };

        assert_eq!(dendrogram.merges.len(), 2);
        assert_eq!(dendrogram.merges[0].cluster1, 0);
        assert_eq!(dendrogram.merges[0].cluster2, 1);
        assert!((dendrogram.merges[0].distance - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_linkage_enum() {
        let single = Linkage::Single;
        let complete = Linkage::Complete;
        let average = Linkage::Average;

        // Test Debug trait
        assert_eq!(format!("{:?}", single), "Single");
        assert_eq!(format!("{:?}", complete), "Complete");
        assert_eq!(format!("{:?}", average), "Average");

        // Test Copy trait
        let copied = single;
        assert!(matches!(copied, Linkage::Single));
    }

    #[test]
    fn test_clustering_method_enum() {
        let kmeans = ClusteringMethod::KMeans { k: 5 };
        let hierarchical = ClusteringMethod::Hierarchical {
            linkage: Linkage::Average,
        };
        let dbscan = ClusteringMethod::DBSCAN {
            epsilon: 0.5,
            min_samples: 3,
        };

        // Test Debug trait
        assert!(format!("{:?}", kmeans).contains("KMeans"));
        assert!(format!("{:?}", hierarchical).contains("Hierarchical"));
        assert!(format!("{:?}", dbscan).contains("DBSCAN"));

        // Test Clone trait
        let cloned_kmeans = kmeans.clone();
        if let ClusteringMethod::KMeans { k } = cloned_kmeans {
            assert_eq!(k, 5);
        } else {
            panic!("Expected KMeans variant");
        }
    }

    // ==================== Cluster Distance Tests ====================

    #[tokio::test]
    async fn test_cluster_distance_single_linkage() {
        let engine = create_test_engine().await;

        let vectors = vec![
            vec![0.0, 0.0],
            vec![1.0, 0.0],
            vec![10.0, 0.0],
            vec![11.0, 0.0],
        ];

        // Build distance map
        let mut distances = HashMap::new();
        for i in 0..4 {
            for j in (i + 1)..4 {
                let dist = engine.euclidean_distance(&vectors[i], &vectors[j]);
                distances.insert((i, j), dist);
            }
        }

        let cluster1 = vec![0, 1]; // Points at 0, 1
        let cluster2 = vec![2, 3]; // Points at 10, 11

        let dist =
            engine.cluster_distance(&cluster1, &cluster2, &distances, &vectors, Linkage::Single);

        // Single linkage: min distance = distance from 1 to 10 = 9
        assert!((dist - 9.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_cluster_distance_complete_linkage() {
        let engine = create_test_engine().await;

        let vectors = vec![
            vec![0.0, 0.0],
            vec![1.0, 0.0],
            vec![10.0, 0.0],
            vec![11.0, 0.0],
        ];

        let mut distances = HashMap::new();
        for i in 0..4 {
            for j in (i + 1)..4 {
                let dist = engine.euclidean_distance(&vectors[i], &vectors[j]);
                distances.insert((i, j), dist);
            }
        }

        let cluster1 = vec![0, 1];
        let cluster2 = vec![2, 3];

        let dist = engine.cluster_distance(
            &cluster1,
            &cluster2,
            &distances,
            &vectors,
            Linkage::Complete,
        );

        // Complete linkage: max distance = distance from 0 to 11 = 11
        assert!((dist - 11.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_cluster_distance_average_linkage() {
        let engine = create_test_engine().await;

        let vectors = vec![
            vec![0.0, 0.0],
            vec![1.0, 0.0],
            vec![10.0, 0.0],
            vec![11.0, 0.0],
        ];

        let mut distances = HashMap::new();
        for i in 0..4 {
            for j in (i + 1)..4 {
                let dist = engine.euclidean_distance(&vectors[i], &vectors[j]);
                distances.insert((i, j), dist);
            }
        }

        let cluster1 = vec![0, 1];
        let cluster2 = vec![2, 3];

        let dist =
            engine.cluster_distance(&cluster1, &cluster2, &distances, &vectors, Linkage::Average);

        // Average: (10 + 11 + 9 + 10) / 4 = 40 / 4 = 10
        assert!((dist - 10.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_cluster_distance_empty_result() {
        let engine = create_test_engine().await;

        let vectors = vec![vec![0.0, 0.0]];
        let distances = HashMap::new(); // Empty - no distances

        let cluster1 = vec![0];
        let cluster2 = vec![1]; // Point 1 doesn't exist in distances

        let dist =
            engine.cluster_distance(&cluster1, &cluster2, &distances, &vectors, Linkage::Single);

        assert_eq!(dist, f64::MAX);
    }

    // ==================== Intra/Inter Cluster Distance Tests ====================

    #[tokio::test]
    async fn test_intra_cluster_distance_basic() {
        let engine = create_test_engine().await;

        let vectors = vec![vec![0.0, 0.0], vec![2.0, 0.0], vec![4.0, 0.0]];
        let labels = vec![0, 0, 0];

        let dist = engine.intra_cluster_distance(&vectors, &labels, 1);

        // Distance from point 1 (at 2,0) to points 0 (at 0,0) and 2 (at 4,0)
        // = (2 + 2) / 2 = 2
        assert!((dist - 2.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_intra_cluster_distance_single_point() {
        let engine = create_test_engine().await;

        let vectors = vec![vec![0.0, 0.0], vec![10.0, 0.0]];
        let labels = vec![0, 1]; // Each point in its own cluster

        let dist = engine.intra_cluster_distance(&vectors, &labels, 0);

        // No other points in cluster 0
        assert_eq!(dist, 0.0);
    }

    #[tokio::test]
    async fn test_nearest_cluster_distance_basic() {
        let engine = create_test_engine().await;

        let vectors = vec![
            vec![0.0, 0.0],
            vec![1.0, 0.0],
            vec![10.0, 0.0],
            vec![11.0, 0.0],
        ];
        let labels = vec![0, 0, 1, 1];

        let dist = engine.nearest_cluster_distance(&vectors, &labels, 0);

        // Nearest cluster is 1, average distance to cluster 1 from point 0
        // = (10 + 11) / 2 = 10.5
        assert!((dist - 10.5).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_nearest_cluster_distance_single_cluster() {
        let engine = create_test_engine().await;

        let vectors = vec![vec![0.0, 0.0], vec![1.0, 0.0], vec![2.0, 0.0]];
        let labels = vec![0, 0, 0];

        let dist = engine.nearest_cluster_distance(&vectors, &labels, 0);

        // No other clusters, returns MAX
        assert_eq!(dist, f64::MAX);
    }
}
