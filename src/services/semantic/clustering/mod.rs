#![cfg_attr(coverage_nightly, coverage(off))]
// Clustering Algorithms for Code Embeddings
// PMAT-SEARCH-007: K-means, Hierarchical, and DBSCAN clustering
//
// GREEN Phase: Implement algorithms

pub mod engine;
pub mod types;

mod tests_algorithms;
mod tests_distance_metrics;
mod tests_quality;

pub use engine::ClusteringEngine;
pub use types::{
    Cluster, ClusterFilters, ClusterMember, ClusterResult, ClusteringMethod, Dendrogram,
    DendrogramMerge, Linkage, OutlierPoint,
};
