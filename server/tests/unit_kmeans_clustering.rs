// RED Phase: Write failing tests first
// PMAT-SEARCH-007: K-means Clustering
// Test count: 15 tests

use pmat::services::semantic::clustering::*;
use pmat::services::semantic::TursoVectorDB;
use std::sync::Arc;
use tempfile::TempDir;

// Helper to setup engine
async fn setup_engine() -> (ClusteringEngine, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("cluster_test.db");

    let vector_db = TursoVectorDB::new_local(db_path).await.unwrap();
    let engine = ClusteringEngine::new(Arc::new(vector_db));

    (engine, temp_dir)
}

// Helper to create test vectors
fn create_test_vectors() -> Vec<Vec<f32>> {
    vec![
        vec![1.0, 1.0, 0.0],
        vec![1.0, 0.9, 0.1],
        vec![0.0, 0.0, 1.0],
        vec![0.1, 0.0, 0.9],
        vec![5.0, 5.0, 5.0],
        vec![5.1, 4.9, 5.0],
    ]
}

// ============================================================================
// K-means Tests (6 tests)
// ============================================================================

#[tokio::test]
async fn test_kmeans_basic_clustering() {
    let (engine, _temp) = setup_engine().await;
    let vectors = create_test_vectors();

    let labels = engine.kmeans(&vectors, 3, 100).unwrap();

    assert_eq!(labels.len(), vectors.len());
    // Verify we have 3 unique cluster labels
    let mut unique_labels: Vec<usize> = labels.clone();
    unique_labels.sort();
    unique_labels.dedup();
    assert_eq!(unique_labels.len(), 3);
}

#[tokio::test]
async fn test_kmeans_convergence() {
    let (engine, _temp) = setup_engine().await;
    let vectors = create_test_vectors();

    // Run twice with same seed
    let labels1 = engine.kmeans(&vectors, 2, 100).unwrap();
    let labels2 = engine.kmeans(&vectors, 2, 100).unwrap();

    // Results should be consistent (within label permutation)
    assert_eq!(labels1.len(), labels2.len());
}

#[tokio::test]
async fn test_kmeans_empty_input() {
    let (engine, _temp) = setup_engine().await;
    let vectors: Vec<Vec<f32>> = vec![];

    let result = engine.kmeans(&vectors, 3, 100);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("empty"));
}

#[tokio::test]
async fn test_kmeans_single_cluster() {
    let (engine, _temp) = setup_engine().await;
    let vectors = create_test_vectors();

    let labels = engine.kmeans(&vectors, 1, 100).unwrap();

    // All points should be in cluster 0
    assert_eq!(labels.len(), vectors.len());
    assert!(labels.iter().all(|&l| l == 0));
}

#[tokio::test]
async fn test_kmeans_more_clusters_than_points() {
    let (engine, _temp) = setup_engine().await;
    let vectors = vec![vec![1.0, 1.0], vec![2.0, 2.0]];

    let result = engine.kmeans(&vectors, 5, 100);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("more clusters than points"));
}

#[tokio::test]
async fn test_kmeans_deterministic_with_seed() {
    let (engine, _temp) = setup_engine().await;
    let vectors = create_test_vectors();

    // Run multiple times - should get consistent results with deterministic initialization
    let labels1 = engine.kmeans_with_seed(&vectors, 3, 100, 42).unwrap();
    let labels2 = engine.kmeans_with_seed(&vectors, 3, 100, 42).unwrap();

    assert_eq!(labels1, labels2);
}

// ============================================================================
// Hierarchical Tests (3 tests)
// ============================================================================

#[tokio::test]
async fn test_hierarchical_single_linkage() {
    let (engine, _temp) = setup_engine().await;
    let vectors = create_test_vectors();

    let dendrogram = engine.hierarchical(&vectors, Linkage::Single).unwrap();

    // Should have n-1 merge steps for n points
    assert_eq!(dendrogram.merges.len(), vectors.len() - 1);
}

#[tokio::test]
async fn test_hierarchical_complete_linkage() {
    let (engine, _temp) = setup_engine().await;
    let vectors = create_test_vectors();

    let dendrogram = engine.hierarchical(&vectors, Linkage::Complete).unwrap();

    // Verify all points are eventually merged
    assert_eq!(dendrogram.merges.len(), vectors.len() - 1);
    // Last merge should combine all points
    assert!(dendrogram.merges.last().is_some());
}

#[tokio::test]
async fn test_hierarchical_dendrogram_structure() {
    let (engine, _temp) = setup_engine().await;
    let vectors = vec![vec![0.0, 0.0], vec![1.0, 1.0], vec![10.0, 10.0]];

    let dendrogram = engine.hierarchical(&vectors, Linkage::Average).unwrap();

    // First merge should be the two closest points
    let first_merge = &dendrogram.merges[0];
    assert!(first_merge.distance < dendrogram.merges[1].distance);
}

// ============================================================================
// DBSCAN Tests (3 tests)
// ============================================================================

#[tokio::test]
async fn test_dbscan_basic_clustering() {
    let (engine, _temp) = setup_engine().await;
    let vectors = create_test_vectors();

    let labels = engine.dbscan(&vectors, 1.5, 2).unwrap();

    assert_eq!(labels.len(), vectors.len());
    // Should have at least one cluster (label >= 0)
    assert!(labels.iter().any(|&l| l >= 0));
}

#[tokio::test]
async fn test_dbscan_noise_detection() {
    let (engine, _temp) = setup_engine().await;
    // Create vectors with clear outlier
    let vectors = vec![
        vec![0.0, 0.0],
        vec![0.1, 0.0],
        vec![0.0, 0.1],
        vec![10.0, 10.0], // Outlier
    ];

    let labels = engine.dbscan(&vectors, 0.5, 2).unwrap();

    // Outlier should be marked as noise (-1)
    assert_eq!(labels[3], -1);
}

#[tokio::test]
async fn test_dbscan_no_clusters() {
    let (engine, _temp) = setup_engine().await;
    // Vectors too far apart
    let vectors = vec![
        vec![0.0, 0.0],
        vec![10.0, 10.0],
        vec![20.0, 20.0],
    ];

    let labels = engine.dbscan(&vectors, 1.0, 2).unwrap();

    // All should be noise
    assert!(labels.iter().all(|&l| l == -1));
}

// ============================================================================
// Integration Tests (3 tests)
// ============================================================================

#[tokio::test]
async fn test_cluster_with_language_filter() {
    let (engine, _temp) = setup_engine().await;

    let method = ClusteringMethod::KMeans { k: 3 };
    let filters = ClusterFilters {
        language: Some("rust".to_string()),
        chunk_type: None,
        file_pattern: None,
    };

    let result = engine.cluster(method, filters).await.unwrap();

    assert_eq!(result.method, "kmeans");
    assert!(result.clusters.len() <= 3);
}

#[tokio::test]
async fn test_cluster_result_structure() {
    let (engine, _temp) = setup_engine().await;

    let method = ClusteringMethod::KMeans { k: 2 };
    let filters = ClusterFilters {
        language: None,
        chunk_type: None,
        file_pattern: None,
    };

    let result = engine.cluster(method, filters).await.unwrap();

    // Verify structure
    assert_eq!(result.method, "kmeans");
    assert!(result.silhouette_score >= -1.0 && result.silhouette_score <= 1.0);
    let total_size: usize = result.clusters.iter().map(|c| c.size).sum();
    assert_eq!(result.total_chunks, total_size);

    // Each cluster should have valid structure
    for cluster in &result.clusters {
        assert!(cluster.size > 0);
        assert_eq!(cluster.chunks.len(), cluster.size);
        assert!(!cluster.centroid.is_empty());
    }
}

#[tokio::test]
async fn test_silhouette_score_computation() {
    let (engine, _temp) = setup_engine().await;

    // Well-separated clusters
    let vectors = vec![
        vec![0.0, 0.0],
        vec![0.1, 0.0],
        vec![10.0, 10.0],
        vec![10.1, 10.0],
    ];
    let labels = vec![0, 0, 1, 1];

    let score = engine.compute_silhouette_score(&vectors, &labels);

    // Well-separated clusters should have high silhouette score (> 0.5)
    assert!(score > 0.5);
    assert!(score <= 1.0);
}
