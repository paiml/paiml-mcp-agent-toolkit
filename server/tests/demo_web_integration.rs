#![cfg(feature = "demo")]

use pmat::demo::{DemoContent, LocalDemoServer};
use pmat::demo::server::{EnhancedHotspot, LanguageStats};
use pmat::services::recommendation_engine::RepositoryRecommendation;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;

fn create_test_demo_content() -> DemoContent {
    DemoContent {
        mermaid_diagram: "graph TD\n  A --> B".to_string(),
        system_diagram: None,
        files_analyzed: 10,
        functions_analyzed: 5,
        avg_complexity: 5.5,
        p90_complexity: 15,
        hotspot_functions: 2,
        quality_score: 0.85,
        tech_debt_hours: 20,
        hotspots: vec![],
        language_stats: HashMap::new(),
        ast_time_ms: 100,
        complexity_time_ms: 150,
        churn_time_ms: 200,
        dag_time_ms: 250,
        recommendations: vec![],
        polyglot_analysis: None,
    }
}

#[tokio::test]
async fn test_demo_server_startup_and_shutdown() {
    // Create test content
    let content = create_test_demo_content();

    // Start server
    let (server, port) = LocalDemoServer::spawn(content.clone()).await.unwrap();
    assert!(port > 0);

    // Verify server is running
    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://127.0.0.1:{port}/"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);

    // Shutdown server
    server.shutdown();

    // Verify server has stopped (with timeout)
    let result = timeout(Duration::from_secs(2), async {
        loop {
            if client
                .get(format!("http://127.0.0.1:{port}/"))
                .send()
                .await
                .is_err()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await;

    assert!(result.is_ok(), "Server did not shut down in time");
}

#[tokio::test]
async fn test_demo_server_api_endpoints() {
    // Create test content with hotspots
    let mut content = create_test_demo_content();
    content.mermaid_diagram = "graph TD\n  A[Main] --> B[Utils]\n  A --> C[Helpers]".to_string();
    content.files_analyzed = 25;
    content.avg_complexity = 8.3;
    content.tech_debt_hours = 45;
    content.hotspots = vec![
        EnhancedHotspot {
            function: "complex_algorithm".to_string(),
            file: "src/complex.rs".to_string(),
            path: "src/complex.rs".to_string(),
            complexity: 15,
            loc: 80,
            language: "rust".to_string(),
            churn_score: 80,
            refactor_suggestion: "Consider breaking down this function".to_string(),
        },
        EnhancedHotspot {
            function: "utility_helper".to_string(),
            file: "src/utils.rs".to_string(),
            path: "src/utils.rs".to_string(),
            complexity: 12,
            loc: 60,
            language: "rust".to_string(),
            churn_score: 60,
            refactor_suggestion: "Extract common patterns".to_string(),
        },
    ];
    content.ast_time_ms = 120;
    content.complexity_time_ms = 180;
    content.churn_time_ms = 240;
    content.dag_time_ms = 300;

    let (server, port) = LocalDemoServer::spawn(content).await.unwrap();
    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{port}");

    // Test root endpoint (HTML dashboard)
    let response = client.get(&base_url).send().await.unwrap();
    assert_eq!(response.status(), 200);
    let html = response.text().await.unwrap();
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("PAIML MCP Agent Toolkit Demo"));
    assert!(html.contains("25")); // files_analyzed
    assert!(html.contains("8.3")); // avg_complexity

    // Test summary API endpoint
    let response = client
        .get(format!("{base_url}/api/summary"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let summary: serde_json::Value = response.json().await.unwrap();
    assert_eq!(summary["files_analyzed"], 25);
    assert_eq!(summary["avg_complexity"], 8.3);
    assert_eq!(summary["tech_debt_hours"], 45);
    assert_eq!(summary["time_context"], 100);

    // Test metrics API endpoint
    let response = client
        .get(format!("{base_url}/api/metrics"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let metrics: serde_json::Value = response.json().await.unwrap();
    assert_eq!(metrics["files_analyzed"], 25);

    // Test hotspots API endpoint
    let response = client
        .get(format!("{base_url}/api/hotspots"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let hotspots: Vec<serde_json::Value> = response.json().await.unwrap();
    // Returns fallback hotspots since complexity analysis is empty
    assert_eq!(hotspots.len(), 5);

    // Test DAG API endpoint
    let response = client
        .get(format!("{base_url}/api/dag"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let dag = response.text().await.unwrap();
    assert!(dag.contains("graph TD"));

    // Test 404 handling
    let response = client
        .get(format!("{base_url}/nonexistent"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 404);

    server.shutdown();
}

#[tokio::test]
async fn test_demo_server_static_assets() {
    let mut content = create_test_demo_content();
    content.mermaid_diagram = "graph TD".to_string();
    content.files_analyzed = 1;
    content.avg_complexity = 1.0;
    content.tech_debt_hours = 0;

    let (server, port) = LocalDemoServer::spawn(content).await.unwrap();
    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{port}");

    // Test vendor assets (if they exist)
    let vendor_paths = vec!["/vendor/gridjs.min.js", "/vendor/gridjs-mermaid.min.css"];

    for path in vendor_paths {
        let response = client
            .get(format!("{base_url}{path}"))
            .send()
            .await
            .unwrap();
        // Assets might not exist during tests, so we accept either 200 or 404
        assert!(
            response.status() == 200 || response.status() == 404,
            "Unexpected status for {}: {}",
            path,
            response.status()
        );
    }

    server.shutdown();
}

#[tokio::test]
async fn test_demo_server_concurrent_requests() {
    let mut content = create_test_demo_content();
    content.mermaid_diagram = "graph TD\n  A --> B".to_string();
    content.files_analyzed = 100;
    content.avg_complexity = std::f64::consts::PI;
    content.tech_debt_hours = 8;

    let (server, port) = LocalDemoServer::spawn(content).await.unwrap();
    let base_url = format!("http://127.0.0.1:{port}");

    // Spawn multiple concurrent requests
    let mut handles = vec![];
    for i in 0..10 {
        let url = base_url.clone();
        let handle = tokio::spawn(async move {
            let client = reqwest::Client::new();
            let endpoint = match i % 4 {
                0 => "/",
                1 => "/api/summary",
                2 => "/api/metrics",
                _ => "/api/dag",
            };
            let response = client.get(format!("{url}{endpoint}")).send().await.unwrap();
            assert_eq!(response.status(), 200);
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        handle.await.unwrap();
    }

    server.shutdown();
}

#[tokio::test]
async fn test_demo_server_response_headers() {
    let mut content = create_test_demo_content();
    content.mermaid_diagram = "graph TD".to_string();
    content.files_analyzed = 1;
    content.avg_complexity = 1.0;
    content.tech_debt_hours = 0;

    let (server, port) = LocalDemoServer::spawn(content).await.unwrap();
    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{port}");

    // Test HTML response headers
    let response = client.get(&base_url).send().await.unwrap();
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "text/html; charset=utf-8"
    );
    assert_eq!(response.headers().get("cache-control").unwrap(), "no-cache");

    // Test JSON response headers
    let response = client
        .get(format!("{base_url}/api/summary"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/json"
    );

    // Test DAG response headers
    let response = client
        .get(format!("{base_url}/api/dag"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "text/plain"
    );

    server.shutdown();
}

#[tokio::test]
async fn test_demo_content_rendering() {
    // Test with specific values to ensure proper rendering
    let mut content = create_test_demo_content();
    content.mermaid_diagram = "graph TD\n  API[API Server] --> DB[Database]\n  API --> Cache[Redis Cache]".to_string();
    content.files_analyzed = 42;
    content.avg_complexity = 6.78;
    content.tech_debt_hours = 123;
    content.hotspots = vec![
        EnhancedHotspot {
            function: "main".to_string(),
            file: "src/main.rs".to_string(),
            path: "src/main.rs".to_string(),
            complexity: 25,
            loc: 150,
            language: "rust".to_string(),
            churn_score: 95,
            refactor_suggestion: "High complexity function needs refactoring".to_string(),
        },
        EnhancedHotspot {
            function: "handle_api".to_string(),
            file: "src/handlers/api.rs".to_string(),
            path: "src/handlers/api.rs".to_string(),
            complexity: 18,
            loc: 120,
            language: "rust".to_string(),
            churn_score: 70,
            refactor_suggestion: "Consider breaking into smaller functions".to_string(),
        },
        EnhancedHotspot {
            function: "database_query".to_string(),
            file: "src/services/database.rs".to_string(),
            path: "src/services/database.rs".to_string(),
            complexity: 15,
            loc: 100,
            language: "rust".to_string(),
            churn_score: 50,
            refactor_suggestion: "Extract query builder patterns".to_string(),
        },
    ];

    let (server, port) = LocalDemoServer::spawn(content).await.unwrap();
    let client = reqwest::Client::new();

    // Check that the HTML contains our specific values
    let response = client
        .get(format!("http://127.0.0.1:{port}/"))
        .send()
        .await
        .unwrap();
    let html = response.text().await.unwrap();

    // Verify specific content is rendered
    assert!(html.contains("42")); // files_analyzed
    assert!(html.contains("6.78")); // avg_complexity formatted
    assert!(html.contains("123")); // tech_debt_hours

    // Verify timing values
    assert!(html.contains("100")); // time_context
    assert!(html.contains("150")); // time_complexity
    assert!(html.contains("200")); // time_dag
    assert!(html.contains("250")); // time_churn

    server.shutdown();
}

// Legacy test compatibility
#[cfg(test)]
mod demo_web_tests {
    use super::*;
    use pmat::models::dag::{DependencyGraph, NodeInfo, NodeType};

    #[tokio::test]
    async fn test_demo_server_starts() {
        let mut content = create_test_demo_content();
        content.mermaid_diagram = String::from("graph TD\n    A[Test] --> B[Demo]");
        content.files_analyzed = 10;
        content.avg_complexity = 5.5;
        content.tech_debt_hours = 12;
        content.hotspots = vec![EnhancedHotspot {
            function: "test_function".to_string(),
            file: String::from("test.rs"),
            path: String::from("test.rs"),
            complexity: 15,
            loc: 80,
            language: "rust".to_string(),
            churn_score: 80,
            refactor_suggestion: "Test function needs optimization".to_string(),
        }];

        // Start the server
        let (server, port) = LocalDemoServer::spawn(content)
            .await
            .expect("Failed to start demo server");

        // Verify we got a port
        assert!(port > 0);
        assert_eq!(server.port(), port);

        // Server will shut down when dropped
    }

    #[test]
    fn test_demo_content_from_analysis() {
        let mut graph = DependencyGraph::new();
        graph.add_node(NodeInfo {
            id: "test".to_string(),
            label: "Test Node".to_string(),
            node_type: NodeType::Function,
            file_path: String::new(),
            line_number: 0,
            complexity: 5,
            metadata: rustc_hash::FxHashMap::default(),
        });

        let content =
            DemoContent::from_analysis_results(&graph, 10, 5.5, 12, vec![], 100, 150, 200, 250);

        assert_eq!(content.files_analyzed, 10);
        assert_eq!(content.avg_complexity, 5.5);
        assert_eq!(content.tech_debt_hours, 12);
        assert!(content.mermaid_diagram.contains("graph TD"));
        assert!(content.mermaid_diagram.contains("test[Test Node]"));
    }
}
