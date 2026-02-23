#![cfg_attr(coverage_nightly, coverage(off))]
// Clustering type definitions for PMAT-SEARCH-007

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
