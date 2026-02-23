#![cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::super::super::TursoVectorDB;
    use super::super::engine::ClusteringEngine;
    use super::super::types::{
        Cluster, ClusterFilters, ClusterMember, ClusterResult, ClusteringMethod, Dendrogram,
        DendrogramMerge, Linkage, OutlierPoint,
    };
    use std::collections::HashMap;
    use std::sync::Arc;

    /// Helper to create a clustering engine for tests
    async fn create_test_engine() -> ClusteringEngine {
        let db = TursoVectorDB::new_local(":memory:")
            .await
            .expect("Failed to create test database");
        ClusteringEngine::new(Arc::new(db))
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
}
