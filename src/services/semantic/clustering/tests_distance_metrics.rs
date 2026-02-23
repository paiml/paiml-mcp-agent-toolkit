#![cfg_attr(coverage_nightly, coverage(off))]
// Tests for cluster distance and intra/inter cluster distance metrics

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_distance_metrics {
    use super::super::engine::ClusteringEngine;
    use super::super::types::Linkage;
    use crate::services::semantic::TursoVectorDB;
    use std::collections::HashMap;
    use std::sync::Arc;

    /// Helper to create a clustering engine for tests
    async fn create_test_engine() -> ClusteringEngine {
        let db = TursoVectorDB::new_local(":memory:")
            .await
            .expect("Failed to create test database");
        ClusteringEngine::new(Arc::new(db))
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
