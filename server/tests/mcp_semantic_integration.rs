// MCP Semantic Search Integration Tests
// Sprint 33 (PMAT-SEARCH-012)
//
// Tests for semantic search tool adapters in MCP integration framework

use pmat::mcp_integration::{McpError, McpTool, ToolMetadata};
use pmat::mcp_integration::tools::*;
use pmat::services::semantic::HybridSearchEngine;
use serde_json::json;
use std::sync::Arc;

// Helper function to create test engine (returns None if no API key)
async fn create_test_engine() -> Option<Arc<HybridSearchEngine>> {
    let api_key = std::env::var("OPENAI_API_KEY").ok()?;
    let temp_dir = tempfile::tempdir().ok()?;
    let db_path = temp_dir.path().join("test.db");
    let workspace = temp_dir.path();

    HybridSearchEngine::new(&api_key, &db_path.to_string_lossy(), workspace)
        .await
        .ok()
        .map(Arc::new)
}

// Test 1: SemanticSearchToolAdapter metadata
#[tokio::test]
async fn test_semantic_search_adapter_metadata() {
    let engine = match create_test_engine().await {
        Some(e) => e,
        None => {
            eprintln!("Skipping test: OPENAI_API_KEY not set");
            return;
        }
    };

    let tool = SemanticSearchToolAdapter::new(engine);
    let metadata = tool.metadata();

    assert_eq!(metadata.name, "semantic_search");
    assert!(metadata.description.contains("Search") || metadata.description.contains("search"));
    assert_eq!(metadata.input_schema["type"], "object");
    assert!(metadata.input_schema["properties"]["query"].is_object());
}

// Test 2: FindSimilarCodeToolAdapter metadata
#[tokio::test]
async fn test_find_similar_adapter_metadata() {
    let engine = match create_test_engine().await {
        Some(e) => e,
        None => {
            eprintln!("Skipping test: OPENAI_API_KEY not set");
            return;
        }
    };

    let tool = FindSimilarCodeToolAdapter::new(engine);
    let metadata = tool.metadata();

    assert_eq!(metadata.name, "find_similar_code");
    assert!(metadata.description.contains("similar") || metadata.description.contains("Similar"));
    assert_eq!(metadata.input_schema["type"], "object");
}

// Test 3: ClusterCodeToolAdapter metadata
#[tokio::test]
async fn test_cluster_adapter_metadata() {
    let engine = match create_test_engine().await {
        Some(e) => e,
        None => {
            eprintln!("Skipping test: OPENAI_API_KEY not set");
            return;
        }
    };

    let tool = ClusterCodeToolAdapter::new(engine);
    let metadata = tool.metadata();

    assert_eq!(metadata.name, "cluster_code");
    assert!(metadata.description.contains("Cluster") || metadata.description.contains("cluster"));
    assert_eq!(metadata.input_schema["type"], "object");
}

// Test 4: AnalyzeTopicsToolAdapter metadata
#[tokio::test]
async fn test_analyze_topics_adapter_metadata() {
    let engine = match create_test_engine().await {
        Some(e) => e,
        None => {
            eprintln!("Skipping test: OPENAI_API_KEY not set");
            return;
        }
    };

    let tool = AnalyzeTopicsToolAdapter::new(engine);
    let metadata = tool.metadata();

    assert_eq!(metadata.name, "analyze_topics");
    assert!(metadata.description.contains("topic") || metadata.description.contains("Topic"));
    assert_eq!(metadata.input_schema["type"], "object");
}

// Test 5: Semantic search with missing query parameter
#[tokio::test]
async fn test_semantic_search_missing_query() {
    let engine = match create_test_engine().await {
        Some(e) => e,
        None => {
            eprintln!("Skipping test: OPENAI_API_KEY not set");
            return;
        }
    };

    let tool = SemanticSearchToolAdapter::new(engine);
    let params = json!({});

    let result = tool.execute(params).await;
    assert!(result.is_err());

    if let Err(err) = result {
        assert!(err.message.contains("query") || err.message.contains("required"));
    }
}

// Test 6: Semantic search with empty query
#[tokio::test]
async fn test_semantic_search_empty_query() {
    let engine = match create_test_engine().await {
        Some(e) => e,
        None => {
            eprintln!("Skipping test: OPENAI_API_KEY not set");
            return;
        }
    };

    let tool = SemanticSearchToolAdapter::new(engine);
    let params = json!({
        "query": ""
    });

    let result = tool.execute(params).await;
    assert!(result.is_err());

    if let Err(err) = result {
        assert!(err.message.contains("empty") || err.message.contains("Query"));
    }
}

// Test 7: Semantic search with invalid mode
#[tokio::test]
async fn test_semantic_search_invalid_mode() {
    let engine = match create_test_engine().await {
        Some(e) => e,
        None => {
            eprintln!("Skipping test: OPENAI_API_KEY not set");
            return;
        }
    };

    let tool = SemanticSearchToolAdapter::new(engine);
    let params = json!({
        "query": "test query",
        "mode": "invalid_mode"
    });

    let result = tool.execute(params).await;
    assert!(result.is_err());

    if let Err(err) = result {
        assert!(err.message.contains("Invalid") || err.message.contains("mode"));
    }
}

// Test 8: Find similar code with missing file_path
#[tokio::test]
async fn test_find_similar_missing_file_path() {
    let engine = match create_test_engine().await {
        Some(e) => e,
        None => {
            eprintln!("Skipping test: OPENAI_API_KEY not set");
            return;
        }
    };

    let tool = FindSimilarCodeToolAdapter::new(engine);
    let params = json!({});

    let result = tool.execute(params).await;
    assert!(result.is_err());

    if let Err(err) = result {
        assert!(err.message.contains("file") || err.message.contains("required") || err.message.contains("path"));
    }
}

// Test 9: Cluster code with missing method
#[tokio::test]
async fn test_cluster_missing_method() {
    let engine = match create_test_engine().await {
        Some(e) => e,
        None => {
            eprintln!("Skipping test: OPENAI_API_KEY not set");
            return;
        }
    };

    let tool = ClusterCodeToolAdapter::new(engine);
    let params = json!({});

    let result = tool.execute(params).await;
    assert!(result.is_err());

    if let Err(err) = result {
        assert!(err.message.contains("method") || err.message.contains("required"));
    }
}

// Test 10: Cluster code with invalid method
#[tokio::test]
async fn test_cluster_invalid_method() {
    let engine = match create_test_engine().await {
        Some(e) => e,
        None => {
            eprintln!("Skipping test: OPENAI_API_KEY not set");
            return;
        }
    };

    let tool = ClusterCodeToolAdapter::new(engine);
    let params = json!({
        "method": "invalid_method"
    });

    let result = tool.execute(params).await;
    assert!(result.is_err());

    if let Err(err) = result {
        assert!(err.message.contains("Invalid") || err.message.contains("method"));
    }
}

// Test 11: Analyze topics with missing num_topics
#[tokio::test]
async fn test_analyze_topics_missing_num_topics() {
    let engine = match create_test_engine().await {
        Some(e) => e,
        None => {
            eprintln!("Skipping test: OPENAI_API_KEY not set");
            return;
        }
    };

    let tool = AnalyzeTopicsToolAdapter::new(engine);
    let params = json!({});

    let result = tool.execute(params).await;
    assert!(result.is_err());

    if let Err(err) = result {
        assert!(err.message.contains("topics") || err.message.contains("required") || err.message.contains("num"));
    }
}

// Test 12: Error conversion from String to McpError
#[tokio::test]
async fn test_error_conversion() {
    let engine = match create_test_engine().await {
        Some(e) => e,
        None => {
            eprintln!("Skipping test: OPENAI_API_KEY not set");
            return;
        }
    };

    let tool = SemanticSearchToolAdapter::new(engine);
    let params = json!({
        "query": "test",
        "limit": 999  // Exceeds max limit of 100
    });

    let result = tool.execute(params).await;
    assert!(result.is_err());

    if let Err(err) = result {
        // Should be McpError with INTERNAL_ERROR code
        assert_eq!(err.code, pmat::mcp_integration::error_codes::INTERNAL_ERROR);
        assert!(!err.message.is_empty());
    }
}

// Test 13: Metadata input schema structure for semantic_search
#[tokio::test]
async fn test_semantic_search_schema_structure() {
    let engine = match create_test_engine().await {
        Some(e) => e,
        None => {
            eprintln!("Skipping test: OPENAI_API_KEY not set");
            return;
        }
    };

    let tool = SemanticSearchToolAdapter::new(engine);
    let metadata = tool.metadata();

    // Verify schema has required properties
    let properties = &metadata.input_schema["properties"];
    assert!(properties["query"].is_object());
    assert!(properties["mode"].is_object());
    assert!(properties["limit"].is_object());

    // Verify required fields
    let required = metadata.input_schema["required"].as_array();
    assert!(required.is_some());
    let required = required.unwrap();
    assert!(required.iter().any(|v| v.as_str() == Some("query")));
}

// Test 14: All adapters can be created and implement McpTool
#[tokio::test]
async fn test_all_adapters_implement_mcp_tool() {
    let engine = match create_test_engine().await {
        Some(e) => e,
        None => {
            eprintln!("Skipping test: OPENAI_API_KEY not set");
            return;
        }
    };

    // Create all adapters
    let tools: Vec<Box<dyn McpTool>> = vec![
        Box::new(SemanticSearchToolAdapter::new(engine.clone())),
        Box::new(FindSimilarCodeToolAdapter::new(engine.clone())),
        Box::new(ClusterCodeToolAdapter::new(engine.clone())),
        Box::new(AnalyzeTopicsToolAdapter::new(engine)),
    ];

    // Verify all tools have valid metadata
    for tool in tools.iter() {
        let metadata = tool.metadata();
        assert!(!metadata.name.is_empty());
        assert!(!metadata.description.is_empty());
        assert!(metadata.input_schema["type"] == "object");
    }

    // Verify unique tool names
    let names: Vec<String> = tools.iter().map(|t| t.metadata().name).collect();
    assert_eq!(names.len(), 4);
    assert!(names.contains(&"semantic_search".to_string()));
    assert!(names.contains(&"find_similar_code".to_string()));
    assert!(names.contains(&"cluster_code".to_string()));
    assert!(names.contains(&"analyze_topics".to_string()));
}
