#![cfg_attr(coverage_nightly, coverage(off))]
// Tests for clustering quality metrics: silhouette, matrix conversion, data structures

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_quality {
    use super::super::engine::ClusteringEngine;
    use super::super::types::{
        Cluster, ClusterFilters, ClusterMember, ClusterResult, ClusteringMethod, Dendrogram,
        DendrogramMerge, Linkage, OutlierPoint,
    };
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
}
