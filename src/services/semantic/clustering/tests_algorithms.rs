#![cfg_attr(coverage_nightly, coverage(off))]
// Tests for clustering algorithms: distance, k-means, hierarchical, DBSCAN

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_algorithms {
    use super::super::engine::ClusteringEngine;
    use super::super::types::{
        ClusterFilters, ClusteringMethod, Linkage,
    };
    use crate::services::semantic::TursoVectorDB;
    use std::sync::Arc;

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

}
