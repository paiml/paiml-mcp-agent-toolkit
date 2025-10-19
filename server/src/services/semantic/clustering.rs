// Clustering Algorithms for Code Embeddings
// PMAT-SEARCH-007: K-means, Hierarchical, and DBSCAN clustering
//
// GREEN Phase: Implement algorithms

use super::TursoVectorDB;
use std::collections::HashMap;
use std::sync::Arc;

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

        // k-means++ initialization
        let centroids = self.kmeans_plusplus_init(vectors, k);

        // Lloyd's algorithm
        let mut labels = vec![0; vectors.len()];
        let mut current_centroids = centroids;

        for _ in 0..max_iterations {
            // Assignment step
            let new_labels = self.assign_to_nearest_centroid(vectors, &current_centroids);

            // Check for convergence
            if new_labels == labels {
                return Ok(labels);
            }

            labels = new_labels;

            // Update step
            current_centroids = self.recompute_centroids(vectors, &labels, k)?;
        }

        Ok(labels)
    }

    /// K-means with seed for deterministic results
    pub fn kmeans_with_seed(
        &self,
        vectors: &[Vec<f32>],
        k: usize,
        max_iterations: usize,
        _seed: u64,
    ) -> Result<Vec<usize>, String> {
        // For now, just call regular kmeans
        // TODO: Use seed for deterministic initialization
        self.kmeans(vectors, k, max_iterations)
    }

    /// k-means++ initialization
    fn kmeans_plusplus_init(&self, vectors: &[Vec<f32>], k: usize) -> Vec<Vec<f32>> {
        let mut centroids = Vec::new();

        // Choose first centroid randomly (for now, just take the first vector)
        centroids.push(vectors[0].clone());

        // Choose remaining centroids
        for _ in 1..k {
            let mut max_dist = 0.0;
            let mut farthest_idx = 0;

            // Find vector farthest from all existing centroids
            for (i, vec) in vectors.iter().enumerate() {
                let mut min_dist = f64::MAX;
                for centroid in &centroids {
                    let dist = self.euclidean_distance(vec, centroid);
                    if dist < min_dist {
                        min_dist = dist;
                    }
                }

                if min_dist > max_dist {
                    max_dist = min_dist;
                    farthest_idx = i;
                }
            }

            centroids.push(vectors[farthest_idx].clone());
        }

        centroids
    }

    /// Assign each vector to nearest centroid
    fn assign_to_nearest_centroid(
        &self,
        vectors: &[Vec<f32>],
        centroids: &[Vec<f32>],
    ) -> Vec<usize> {
        vectors
            .iter()
            .map(|vec| {
                let mut min_dist = f64::MAX;
                let mut nearest = 0;

                for (i, centroid) in centroids.iter().enumerate() {
                    let dist = self.euclidean_distance(vec, centroid);
                    if dist < min_dist {
                        min_dist = dist;
                        nearest = i;
                    }
                }

                nearest
            })
            .collect()
    }

    /// Recompute centroids as mean of assigned points
    fn recompute_centroids(
        &self,
        vectors: &[Vec<f32>],
        labels: &[usize],
        k: usize,
    ) -> Result<Vec<Vec<f32>>, String> {
        let dim = vectors[0].len();
        let mut centroids = vec![vec![0.0; dim]; k];
        let mut counts = vec![0; k];

        // Sum vectors in each cluster
        for (vec, &label) in vectors.iter().zip(labels.iter()) {
            for (i, &val) in vec.iter().enumerate() {
                centroids[label][i] += val;
            }
            counts[label] += 1;
        }

        // Divide by count to get mean
        for (centroid, &count) in centroids.iter_mut().zip(counts.iter()) {
            if count == 0 {
                return Err("Empty cluster detected".to_string());
            }
            for val in centroid.iter_mut() {
                *val /= count as f32;
            }
        }

        Ok(centroids)
    }

    /// Hierarchical clustering
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
                    let dist = self.cluster_distance(&clusters[i], &clusters[j], &distances, vectors, linkage);
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
            Linkage::Single => *dists.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            Linkage::Complete => *dists.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            Linkage::Average => dists.iter().sum::<f64>() / dists.len() as f64,
        }
    }

    /// DBSCAN clustering
    pub fn dbscan(
        &self,
        vectors: &[Vec<f32>],
        epsilon: f64,
        min_samples: usize,
    ) -> Result<Vec<i32>, String> {
        if vectors.is_empty() {
            return Err("Cannot cluster empty vector set".to_string());
        }

        let n = vectors.len();
        let mut labels = vec![-1; n]; // -1 = noise
        let mut cluster_id = 0;

        for i in 0..n {
            if labels[i] != -1 {
                continue; // Already processed
            }

            // Find neighbors
            let neighbors = self.find_neighbors(vectors, i, epsilon);

            if neighbors.len() < min_samples {
                labels[i] = -1; // Mark as noise
                continue;
            }

            // Start new cluster
            self.expand_cluster(
                vectors,
                &mut labels,
                i,
                cluster_id,
                epsilon,
                min_samples,
            );
            cluster_id += 1;
        }

        Ok(labels)
    }

    /// Find neighbors within epsilon distance
    fn find_neighbors(&self, vectors: &[Vec<f32>], point_idx: usize, epsilon: f64) -> Vec<usize> {
        let mut neighbors = Vec::new();

        for (i, vec) in vectors.iter().enumerate() {
            let dist = self.euclidean_distance(&vectors[point_idx], vec);
            if dist <= epsilon {
                neighbors.push(i);
            }
        }

        neighbors
    }

    /// Expand cluster from seed point
    fn expand_cluster(
        &self,
        vectors: &[Vec<f32>],
        labels: &mut [i32],
        seed_idx: usize,
        cluster_id: i32,
        epsilon: f64,
        min_samples: usize,
    ) {
        let mut seeds = vec![seed_idx];
        labels[seed_idx] = cluster_id;

        let mut i = 0;
        while i < seeds.len() {
            let current = seeds[i];
            let neighbors = self.find_neighbors(vectors, current, epsilon);

            if neighbors.len() >= min_samples {
                for &neighbor in &neighbors {
                    if labels[neighbor] == -1 {
                        labels[neighbor] = cluster_id;
                        seeds.push(neighbor);
                    }
                }
            }

            i += 1;
        }
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
    fn intra_cluster_distance(&self, vectors: &[Vec<f32>], labels: &[usize], point_idx: usize) -> f64 {
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
    fn nearest_cluster_distance(&self, vectors: &[Vec<f32>], labels: &[usize], point_idx: usize) -> f64 {
        let current_cluster = labels[point_idx];
        let mut min_avg_dist = f64::MAX;

        // Find all unique clusters
        let mut clusters: Vec<usize> = labels.iter().copied().collect();
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

        let sum: f32 = v1.iter()
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
        let db = TursoVectorDB::new_local(":memory:").await.unwrap();
        let engine = ClusteringEngine::new(Arc::new(db));

        let v1 = vec![0.0, 0.0, 0.0];
        let v2 = vec![1.0, 0.0, 0.0];

        let dist = engine.euclidean_distance(&v1, &v2);
        assert!((dist - 1.0).abs() < 1e-6);
    }
}
