#![cfg_attr(coverage_nightly, coverage(off))]
// Clustering Algorithms for Code Embeddings
// PMAT-SEARCH-007: K-means, Hierarchical, and DBSCAN clustering
//
// GREEN Phase: Implement algorithms

mod engine;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_part2;
#[cfg(test)]
mod tests_part3;
pub mod types;

pub use engine::ClusteringEngine;
pub use types::{
    Cluster, ClusterFilters, ClusterMember, ClusterResult, ClusteringMethod, Dendrogram,
    DendrogramMerge, Linkage, OutlierPoint,
};
