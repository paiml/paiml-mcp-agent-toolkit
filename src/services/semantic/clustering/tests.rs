#![cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::super::super::TursoVectorDB;
    use super::super::engine::ClusteringEngine;
    #[allow(unused_imports)]
    use super::super::types::{
        Cluster, ClusterFilters, ClusterMember, ClusterResult, ClusteringMethod, Dendrogram,
        DendrogramMerge, Linkage, OutlierPoint,
    };
    #[allow(unused_imports)]
    use std::collections::HashMap;
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
}
