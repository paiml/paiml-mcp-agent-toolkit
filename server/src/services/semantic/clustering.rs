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
                .min_by(|a, b| a.partial_cmp(b).expect("internal error"))
                .expect("internal error"),
            Linkage::Complete => *dists
                .iter()
                .max_by(|a, b| a.partial_cmp(b).expect("internal error"))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_euclidean_distance() {
        let db = TursoVectorDB::new_local(":memory:").await.expect("internal error");
        let engine = ClusteringEngine::new(Arc::new(db));

        let v1 = vec![0.0, 0.0, 0.0];
        let v2 = vec![1.0, 0.0, 0.0];

        let dist = engine.euclidean_distance(&v1, &v2);
        assert!((dist - 1.0).abs() < 1e-6);
    }
}
