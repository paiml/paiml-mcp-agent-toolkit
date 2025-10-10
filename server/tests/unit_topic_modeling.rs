// RED Phase: Write failing tests first
// PMAT-SEARCH-008: Topic Modeling with LDA
// Test count: 10 tests

use pmat::services::semantic::topic_modeling::*;
use pmat::services::semantic::TursoVectorDB;
use std::sync::Arc;
use tempfile::TempDir;

// Helper to setup engine
async fn setup_engine() -> (TopicEngine, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("topic_test.db");

    let vector_db = TursoVectorDB::new_local(db_path).await.unwrap();
    let engine = TopicEngine::new(Arc::new(vector_db));

    (engine, temp_dir)
}

// ============================================================================
// Core LDA Tests (4 tests)
// ============================================================================

#[tokio::test]
async fn test_extract_topics_basic() {
    let (engine, _temp) = setup_engine().await;

    let result = engine.extract_topics(3, TopicFilters::default()).await.unwrap();

    assert_eq!(result.num_topics, 3);
    assert!(result.topics.len() <= 3);
    assert!(result.coherence_score >= 0.0 && result.coherence_score <= 1.0);
}

#[tokio::test]
async fn test_topic_result_structure() {
    let (engine, _temp) = setup_engine().await;

    let result = engine.extract_topics(2, TopicFilters::default()).await.unwrap();

    // Verify structure
    assert_eq!(result.num_topics, 2);
    assert!(result.total_chunks >= 0);

    for topic in &result.topics {
        assert!(topic.id < result.num_topics);
        assert!(!topic.keywords.is_empty());
        assert!(topic.strength >= 0.0 && topic.strength <= 1.0);

        for chunk in &topic.top_chunks {
            assert!(!chunk.file_path.is_empty());
            assert!(!chunk.chunk_name.is_empty());
            assert!(chunk.topic_probability >= 0.0 && chunk.topic_probability <= 1.0);
        }
    }
}

#[tokio::test]
async fn test_extract_topics_invalid_count() {
    let (engine, _temp) = setup_engine().await;

    // num_topics < 1
    let result = engine.extract_topics(0, TopicFilters::default()).await;
    assert!(result.is_err());

    // num_topics > 20
    let result = engine.extract_topics(25, TopicFilters::default()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_extract_topics_empty_data() {
    let (engine, _temp) = setup_engine().await;

    let result = engine.extract_topics(3, TopicFilters::default()).await;

    // Should succeed but return empty topics
    assert!(result.is_ok());
    let topics = result.unwrap();
    assert_eq!(topics.total_chunks, 0);
}

// ============================================================================
// Topic Quality Tests (3 tests)
// ============================================================================

#[tokio::test]
async fn test_topic_keywords_extraction() {
    let (engine, _temp) = setup_engine().await;

    let chunk_names = vec![
        "handle_error".to_string(),
        "error_handler".to_string(),
        "process_data".to_string(),
        "data_processor".to_string(),
    ];

    let keywords = engine.extract_keywords(&chunk_names, 3);

    // Should extract meaningful keywords
    assert!(!keywords.is_empty());
    assert!(keywords.len() <= 3);
    // Keywords should be distinct
    let mut unique_keywords = keywords.clone();
    unique_keywords.sort();
    unique_keywords.dedup();
    assert_eq!(keywords.len(), unique_keywords.len());
}

#[tokio::test]
async fn test_topic_strength_computation() {
    let (engine, _temp) = setup_engine().await;

    let result = engine.extract_topics(2, TopicFilters::default()).await.unwrap();

    for topic in &result.topics {
        // Strength should be normalized
        assert!(topic.strength >= 0.0);
        assert!(topic.strength <= 1.0);
    }
}

#[tokio::test]
async fn test_coherence_score_computation() {
    let (engine, _temp) = setup_engine().await;

    let result = engine.extract_topics(3, TopicFilters::default()).await.unwrap();

    // Coherence score should be in valid range
    assert!(result.coherence_score >= 0.0);
    assert!(result.coherence_score <= 1.0);

    // If we have distinct topics, coherence should be > 0
    if result.topics.len() > 1 {
        assert!(result.coherence_score > 0.0);
    }
}

// ============================================================================
// Integration Tests (3 tests)
// ============================================================================

#[tokio::test]
async fn test_extract_topics_with_language_filter() {
    let (engine, _temp) = setup_engine().await;

    let filters = TopicFilters {
        language: Some("rust".to_string()),
        chunk_type: None,
        file_pattern: None,
    };

    let result = engine.extract_topics(2, filters).await.unwrap();

    // Should only contain Rust chunks
    for topic in &result.topics {
        for chunk in &topic.top_chunks {
            assert_eq!(chunk.language, "rust");
        }
    }
}

#[tokio::test]
async fn test_chunk_topic_assignment() {
    let (engine, _temp) = setup_engine().await;

    let result = engine.extract_topics(3, TopicFilters::default()).await.unwrap();

    // Each chunk should be assigned to exactly one dominant topic
    for topic in &result.topics {
        for chunk in &topic.top_chunks {
            // Topic probability should be highest for this topic
            assert!(chunk.topic_probability > 0.0);
        }
    }
}

#[tokio::test]
async fn test_topic_probability_distribution() {
    let (engine, _temp) = setup_engine().await;

    let result = engine.extract_topics(3, TopicFilters::default()).await.unwrap();

    // For a given chunk, topic probabilities across all topics should sum to ~1.0
    // This test verifies that we have a valid probability distribution
    // Since we don't have a per-chunk API, we verify individual chunk probabilities are valid
    for topic in &result.topics {
        for chunk in &topic.top_chunks {
            assert!(chunk.topic_probability >= 0.0);
            assert!(chunk.topic_probability <= 1.0);
        }
    }
}
