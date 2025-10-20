// RED Phase: Write failing tests first

// NOTE (Sprint 47): Use assetsearch (../../assetsearch) for MCP-based semantic search.
// All tests in this file marked #[ignore] pending migration to assetsearch.

// PMAT-SEARCH-006: MCP Tools Integration
// Test count: 20 tests

use pmat::mcp::tools::semantic_search_tools::*;
use serde_json::json;
use tempfile::TempDir;

// Helper to setup engine
async fn setup_engine() -> (std::sync::Arc<pmat::services::semantic::HybridSearchEngine>, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("mcp_test.db");

    let engine = pmat::services::semantic::HybridSearchEngine::new(
        "sk-test-key-1234567890abcdefghijklmnop",
        db_path.to_str().unwrap(),
        temp_dir.path(),
    )
    .await
    .unwrap();

    // Create test code
    std::fs::create_dir_all(temp_dir.path().join("src")).unwrap();
    std::fs::write(
        temp_dir.path().join("src/math.rs"),
        r#"
/// Add two numbers
fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#,
    )
    .unwrap();

    engine.index_directory(temp_dir.path()).await.unwrap();

    (std::sync::Arc::new(engine), temp_dir)
}

// ============================================================================
// semantic_search Tool Tests (5 tests)
// ============================================================================

#[ignore]
#[tokio::test]
async fn test_semantic_search_tool_basic() {
    let (engine, _temp_dir) = setup_engine().await;
    let tool = SemanticSearchTool::new(engine);

    let params = json!({
        "query": "add",
        "mode": "hybrid",
        "limit": 5
    });

    let result = tool.execute(params).await.unwrap();

    assert!(result["results"].is_array());
    assert!(result["total"].is_number());
    assert!(result["mode"].is_string());
    assert!(result["query_time_ms"].is_number());
}

#[ignore]
#[tokio::test]
async fn test_semantic_search_result_structure() {
    let (engine, _temp_dir) = setup_engine().await;
    let tool = SemanticSearchTool::new(engine);

    let params = json!({
        "query": "function",
        "mode": "keyword",
        "limit": 1
    });

    let result = tool.execute(params).await.unwrap();

    if let Some(results) = result["results"].as_array() {
        if !results.is_empty() {
            let first = &results[0];
            assert!(first["file_path"].is_string());
            assert!(first["chunk_name"].is_string());
            assert!(first["chunk_type"].is_string());
            assert!(first["language"].is_string());
            assert!(first["score"].is_number());
            assert!(first["snippet"].is_string());
            assert!(first["start_line"].is_number());
            assert!(first["end_line"].is_number());
        }
    }
}

#[ignore]
#[tokio::test]
async fn test_semantic_search_empty_query() {
    let (engine, _temp_dir) = setup_engine().await;
    let tool = SemanticSearchTool::new(engine);

    let params = json!({
        "query": "",
        "mode": "hybrid"
    });

    let result = tool.execute(params).await;
    assert!(result.is_err());
}

#[ignore]
#[tokio::test]
async fn test_semantic_search_invalid_mode() {
    let (engine, _temp_dir) = setup_engine().await;
    let tool = SemanticSearchTool::new(engine);

    let params = json!({
        "query": "test",
        "mode": "invalid_mode"
    });

    let result = tool.execute(params).await;
    assert!(result.is_err());
}

#[ignore]
#[tokio::test]
async fn test_semantic_search_with_filters() {
    let (engine, _temp_dir) = setup_engine().await;
    let tool = SemanticSearchTool::new(engine);

    let params = json!({
        "query": "function",
        "mode": "hybrid",
        "language": "rust",
        "limit": 10
    });

    let result = tool.execute(params).await.unwrap();
    assert!(result["results"].is_array());
}

// ============================================================================
// find_similar_code Tool Tests (4 tests)
// ============================================================================

#[ignore]
#[tokio::test]
async fn test_find_similar_code_tool() {
    let (engine, temp_dir) = setup_engine().await;
    let tool = FindSimilarCodeTool::new(engine);

    let file_path = temp_dir.path().join("src/math.rs");
    let params = json!({
        "file_path": file_path.to_str().unwrap(),
        "limit": 3
    });

    let result = tool.execute(params).await.unwrap();

    assert!(result["results"].is_array());
    assert!(result["reference_file"].is_string());
    assert!(result["total"].is_number());
}

#[ignore]
#[tokio::test]
async fn test_find_similar_result_structure() {
    let (engine, temp_dir) = setup_engine().await;
    let tool = FindSimilarCodeTool::new(engine);

    let file_path = temp_dir.path().join("src/math.rs");
    let params = json!({
        "file_path": file_path.to_str().unwrap(),
        "limit": 1
    });

    let result = tool.execute(params).await.unwrap();

    if let Some(results) = result["results"].as_array() {
        if !results.is_empty() {
            let first = &results[0];
            assert!(first["file_path"].is_string());
            assert!(first["chunk_name"].is_string());
            assert!(first["similarity"].is_number());
            assert!(first["snippet"].is_string());
        }
    }
}

#[ignore]
#[tokio::test]
async fn test_find_similar_invalid_file() {
    let (engine, _temp_dir) = setup_engine().await;
    let tool = FindSimilarCodeTool::new(engine);

    let params = json!({
        "file_path": "/nonexistent/file.rs",
        "limit": 3
    });

    let result = tool.execute(params).await;
    assert!(result.is_err());
}

#[ignore]
#[tokio::test]
async fn test_find_similar_missing_params() {
    let (engine, _temp_dir) = setup_engine().await;
    let tool = FindSimilarCodeTool::new(engine);

    let params = json!({
        "limit": 3
        // Missing file_path
    });

    let result = tool.execute(params).await;
    assert!(result.is_err());
}

// ============================================================================
// cluster_code Tool Tests (4 tests)
// ============================================================================

#[ignore]
#[tokio::test]
async fn test_cluster_code_tool() {
    let (engine, _temp_dir) = setup_engine().await;
    let tool = ClusterCodeTool::new(engine);

    let params = json!({
        "method": "kmeans",
        "k": 3
    });

    let result = tool.execute(params).await.unwrap();

    assert!(result["clusters"].is_array());
    assert!(result["method"].is_string());
    assert!(result["total_chunks"].is_number());
    assert!(result["total_clusters"].is_number());
}

#[ignore]
#[tokio::test]
async fn test_cluster_result_structure() {
    let (engine, _temp_dir) = setup_engine().await;
    let tool = ClusterCodeTool::new(engine);

    let params = json!({
        "method": "kmeans",
        "k": 2
    });

    let result = tool.execute(params).await.unwrap();

    if let Some(clusters) = result["clusters"].as_array() {
        if !clusters.is_empty() {
            let cluster = &clusters[0];
            assert!(cluster["id"].is_number());
            assert!(cluster["size"].is_number());
            assert!(cluster["chunks"].is_array());
        }
    }
}

#[ignore]
#[tokio::test]
async fn test_cluster_invalid_method() {
    let (engine, _temp_dir) = setup_engine().await;
    let tool = ClusterCodeTool::new(engine);

    let params = json!({
        "method": "invalid_method",
        "k": 3
    });

    let result = tool.execute(params).await;
    assert!(result.is_err());
}

#[ignore]
#[tokio::test]
async fn test_cluster_missing_k() {
    let (engine, _temp_dir) = setup_engine().await;
    let tool = ClusterCodeTool::new(engine);

    let params = json!({
        "method": "kmeans"
        // Missing k parameter
    });

    let result = tool.execute(params).await;
    assert!(result.is_err());
}

// ============================================================================
// analyze_topics Tool Tests (4 tests)
// ============================================================================

#[ignore]
#[tokio::test]
async fn test_analyze_topics_tool() {
    let (engine, _temp_dir) = setup_engine().await;
    let tool = AnalyzeTopicsTool::new(engine);

    let params = json!({
        "num_topics": 5
    });

    let result = tool.execute(params).await.unwrap();

    assert!(result["topics"].is_array());
    assert!(result["num_topics"].is_number());
}

#[ignore]
#[tokio::test]
async fn test_analyze_topics_result_structure() {
    let (engine, _temp_dir) = setup_engine().await;
    let tool = AnalyzeTopicsTool::new(engine);

    let params = json!({
        "num_topics": 3
    });

    let result = tool.execute(params).await.unwrap();

    if let Some(topics) = result["topics"].as_array() {
        if !topics.is_empty() {
            let topic = &topics[0];
            assert!(topic["id"].is_number());
            assert!(topic["keywords"].is_array());
            assert!(topic["examples"].is_array());
        }
    }
}

#[ignore]
#[tokio::test]
async fn test_analyze_topics_invalid_count() {
    let (engine, _temp_dir) = setup_engine().await;
    let tool = AnalyzeTopicsTool::new(engine);

    let params = json!({
        "num_topics": 0 // Invalid: must be >= 1
    });

    let result = tool.execute(params).await;
    assert!(result.is_err());
}

#[ignore]
#[tokio::test]
async fn test_analyze_topics_too_many() {
    let (engine, _temp_dir) = setup_engine().await;
    let tool = AnalyzeTopicsTool::new(engine);

    let params = json!({
        "num_topics": 100 // Too many
    });

    let result = tool.execute(params).await;
    assert!(result.is_err());
}

// ============================================================================
// Schema Validation Tests (3 tests)
// ============================================================================

#[ignore]
#[test]
fn test_semantic_search_schema() {
    let schema = SemanticSearchTool::schema();

    assert_eq!(schema["name"], "semantic_search");
    assert!(schema["description"].is_string());
    assert!(schema["parameters"].is_object());
    assert!(schema["parameters"]["properties"]["query"].is_object());
    assert!(schema["parameters"]["properties"]["mode"].is_object());
    assert!(schema["parameters"]["required"].is_array());
}

#[ignore]
#[test]
fn test_find_similar_schema() {
    let schema = FindSimilarCodeTool::schema();

    assert_eq!(schema["name"], "find_similar_code");
    assert!(schema["description"].is_string());
    assert!(schema["parameters"]["properties"]["file_path"].is_object());
    assert!(schema["parameters"]["required"].as_array().unwrap().contains(&json!("file_path")));
}

#[ignore]
#[test]
fn test_all_tool_schemas() {
    let schemas = vec![
        SemanticSearchTool::schema(),
        FindSimilarCodeTool::schema(),
        ClusterCodeTool::schema(),
        AnalyzeTopicsTool::schema(),
    ];

    for schema in schemas {
        // All tools must have these fields
        assert!(schema["name"].is_string());
        assert!(schema["description"].is_string());
        assert!(schema["parameters"].is_object());
        assert!(schema["parameters"]["type"] == "object");
        assert!(schema["parameters"]["properties"].is_object());
    }
}
