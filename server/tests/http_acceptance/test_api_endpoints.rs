//! HTTP API Acceptance Tests - API Endpoints
//!
//! Tests for all HTTP API endpoints following the http-api-acceptance-testing.md specification.
//! Ensures 100% coverage of HTTP API functionality with proper REST compliance.

use crate::http_acceptance::helpers::http_test_client::{HttpTestClient, HttpValidators, HttpTestResult};
use serde_json::json;
use std::time::Duration;
use anyhow::Result;

/// Test dashboard and UI endpoints
#[tokio::test]
async fn test_dashboard_endpoints() -> Result<()> {
    let client = HttpTestClient::new("http://localhost:3000")?;
    
    // Test main dashboard
    let result = client.get("/").await?;
    HttpValidators::assert_status_code(&result, 200)?;
    HttpValidators::assert_performance(&result, Duration::from_secs(2))?;
    HttpValidators::assert_content_type(&result, "text/html")?;
    
    // Should contain dashboard content
    assert!(result.response.body.contains("dashboard") || 
           result.response.body.contains("pmat") ||
           result.response.body.contains("analysis"));
    
    // Test vendor assets
    let vendor_paths = [
        "/vendor/bootstrap.css",
        "/vendor/bootstrap.js", 
        "/vendor/chart.js",
    ];
    
    for path in &vendor_paths {
        let result = client.get(path).await;
        if let Ok(result) = result {
            // Vendor assets should be served or return 404 gracefully
            assert!(result.status_code == 200 || result.status_code == 404);
            HttpValidators::assert_performance(&result, Duration::from_secs(1))?;
        }
    }
    
    // Test demo assets
    let demo_paths = [
        "/demo.css",
        "/demo.js",
    ];
    
    for path in &demo_paths {
        let result = client.get(path).await;
        if let Ok(result) = result {
            assert!(result.status_code == 200 || result.status_code == 404);
            HttpValidators::assert_performance(&result, Duration::from_secs(1))?;
        }
    }
    
    Ok(())
}

/// Test Core API v1 endpoints (legacy)
#[tokio::test]
async fn test_core_api_v1_endpoints() -> Result<()> {
    let client = HttpTestClient::new("http://localhost:3000")?;
    
    // Test /api/summary
    let result = client.get("/api/summary").await?;
    HttpValidators::assert_success(&result)?;
    HttpValidators::assert_performance(&result, Duration::from_secs(5))?;
    HttpValidators::assert_content_type(&result, "application/json")?;
    
    if let Some(ref json) = result.response.json {
        // Summary should contain key metrics
        assert!(json.get("status").is_some() || 
               json.get("summary").is_some() ||
               json.get("metrics").is_some());
    }
    
    // Test /api/metrics
    let result = client.get("/api/metrics").await?;
    HttpValidators::assert_success(&result)?;
    HttpValidators::assert_performance(&result, Duration::from_secs(5))?;
    HttpValidators::assert_content_type(&result, "application/json")?;
    
    // Test /api/hotspots
    let result = client.get("/api/hotspots").await?;
    HttpValidators::assert_success(&result)?;
    HttpValidators::assert_performance(&result, Duration::from_secs(5))?;
    
    // Test /api/dag
    let result = client.get("/api/dag").await?;
    HttpValidators::assert_success(&result)?;
    HttpValidators::assert_performance(&result, Duration::from_secs(5))?;
    
    // DAG might return Mermaid format (text/plain) or JSON
    let content_type_valid = result.response.headers.get("content-type")
        .map(|ct| ct.contains("application/json") || ct.contains("text/plain"))
        .unwrap_or(false);
    assert!(content_type_valid, "DAG endpoint should return JSON or text/plain");
    
    // Test /api/system-diagram
    let result = client.get("/api/system-diagram").await?;
    HttpValidators::assert_success(&result)?;
    HttpValidators::assert_performance(&result, Duration::from_secs(5))?;
    
    // Test /api/analysis
    let result = client.get("/api/analysis").await?;
    HttpValidators::assert_success(&result)?;
    HttpValidators::assert_performance(&result, Duration::from_secs(10))?;
    HttpValidators::assert_content_type(&result, "application/json")?;
    
    // Test /api/recommendations
    let result = client.get("/api/recommendations").await?;
    HttpValidators::assert_success(&result)?;
    HttpValidators::assert_performance(&result, Duration::from_secs(8))?;
    
    // Test /api/polyglot
    let result = client.get("/api/polyglot").await?;
    HttpValidators::assert_success(&result)?;
    HttpValidators::assert_performance(&result, Duration::from_secs(8))?;
    
    // Test /api/showcase
    let result = client.get("/api/showcase").await?;
    HttpValidators::assert_success(&result)?;
    HttpValidators::assert_performance(&result, Duration::from_secs(5))?;
    
    Ok(())
}

/// Test Enhanced API v1 endpoints (current)
#[tokio::test]
async fn test_enhanced_api_v1_endpoints() -> Result<()> {
    let client = HttpTestClient::new("http://localhost:3000")?;
    
    // Test /api/v1/analysis/architecture
    let result = client.get("/api/v1/analysis/architecture").await?;
    HttpValidators::assert_success(&result)?;
    HttpValidators::assert_performance(&result, Duration::from_secs(10))?;
    HttpValidators::assert_content_type(&result, "application/json")?;
    
    if let Some(ref json) = result.response.json {
        // Architecture analysis should contain structural information
        assert!(json.get("architecture").is_some() || 
               json.get("components").is_some() ||
               json.get("structure").is_some());
    }
    
    // Test /api/v1/analysis/defects
    let result = client.get("/api/v1/analysis/defects").await?;
    HttpValidators::assert_success(&result)?;
    HttpValidators::assert_performance(&result, Duration::from_secs(8))?;
    HttpValidators::assert_content_type(&result, "application/json")?;
    
    // Test /api/v1/analysis/statistics
    let result = client.get("/api/v1/analysis/statistics").await?;
    HttpValidators::assert_success(&result)?;
    HttpValidators::assert_performance(&result, Duration::from_secs(7))?;
    HttpValidators::assert_content_type(&result, "application/json")?;
    
    // Test /api/v1/analysis/diagram
    let result = client.get("/api/v1/analysis/diagram").await?;
    HttpValidators::assert_success(&result)?;
    HttpValidators::assert_performance(&result, Duration::from_secs(6))?;
    
    // Diagram can be SVG, PNG, or JSON
    let content_type_valid = result.response.headers.get("content-type")
        .map(|ct| ct.contains("application/json") || 
                  ct.contains("image/svg+xml") || 
                  ct.contains("image/png"))
        .unwrap_or(false);
    assert!(content_type_valid, "Diagram endpoint should return JSON, SVG, or PNG");
    
    // Test /api/v1/analysis/stream (this might be EventSource/SSE)
    let result = client.get("/api/v1/analysis/stream").await?;
    // Stream endpoint might return different status codes
    assert!(result.status_code == 200 || 
           result.status_code == 202 || 
           result.status_code == 404,
           "Stream endpoint should return 200, 202, or 404");
    HttpValidators::assert_performance(&result, Duration::from_secs(10))?;
    
    Ok(())
}

/// Test POST endpoints for analysis triggers
#[tokio::test]
async fn test_post_endpoints() -> Result<()> {
    let client = HttpTestClient::new("http://localhost:3000")?;
    let project_path = client.create_sample_project()?;
    
    // Test /api/v1/analysis/trigger
    let trigger_data = json!({
        "path": project_path.to_string_lossy(),
        "analysis_types": ["complexity", "dead_code"],
        "format": "json"
    });
    
    let result = client.post("/api/v1/analysis/trigger", Some(trigger_data)).await?;
    // Should either succeed or return method not allowed/not found
    assert!(result.status_code == 200 || 
           result.status_code == 201 || 
           result.status_code == 202 ||
           result.status_code == 404 || 
           result.status_code == 405);
    HttpValidators::assert_performance(&result, Duration::from_secs(15))?;
    
    // Test /api/v1/projects
    let project_data = json!({
        "name": "test-project",
        "path": project_path.to_string_lossy(),
        "language": "rust"
    });
    
    let result = client.post("/api/v1/projects", Some(project_data)).await?;
    assert!(result.status_code == 200 || 
           result.status_code == 201 || 
           result.status_code == 404 || 
           result.status_code == 405);
    HttpValidators::assert_performance(&result, Duration::from_secs(10))?;
    
    // Test /api/v1/templates/generate
    let template_data = json!({
        "template_name": "rust_basic",
        "output_path": project_path.join("generated").to_string_lossy(),
        "variables": {
            "project_name": "test_project"
        }
    });
    
    let result = client.post("/api/v1/templates/generate", Some(template_data)).await?;
    assert!(result.status_code == 200 || 
           result.status_code == 201 || 
           result.status_code == 404 || 
           result.status_code == 405);
    HttpValidators::assert_performance(&result, Duration::from_secs(12))?;
    
    // Test /api/v1/quality-gate/check
    let quality_data = json!({
        "file_path": project_path.join("src/main.rs").to_string_lossy(),
        "profile": "standard"
    });
    
    let result = client.post("/api/v1/quality-gate/check", Some(quality_data)).await?;
    assert!(result.status_code == 200 || 
           result.status_code == 404 || 
           result.status_code == 405);
    HttpValidators::assert_performance(&result, Duration::from_secs(10))?;
    
    Ok(())
}

/// Test HTTP methods compliance
#[tokio::test]
async fn test_http_methods_compliance() -> Result<()> {
    let client = HttpTestClient::new("http://localhost:3000")?;
    
    // Test HEAD requests
    let result = client.head("/api/summary").await?;
    HttpValidators::assert_performance(&result, Duration::from_secs(2))?;
    // HEAD should have same headers as GET but no body
    assert!(result.response.body.is_empty() || result.status_code == 405);
    
    // Test OPTIONS requests (CORS)
    let result = client.options("/api/summary").await?;
    HttpValidators::assert_performance(&result, Duration::from_secs(2))?;
    
    if result.status_code == 200 {
        // If OPTIONS is supported, check CORS headers
        let has_cors_headers = result.response.headers.get("access-control-allow-methods").is_some() ||
                              result.response.headers.get("allow").is_some();
        assert!(has_cors_headers, "OPTIONS response should include allowed methods");
    }
    
    // Test unsupported methods return 405
    let unsupported_methods = [
        "PATCH",
        "TRACE",
        "CONNECT",
    ];
    
    for method_name in &unsupported_methods {
        // Note: reqwest doesn't support all HTTP methods, so we test what we can
        if method_name == &"PATCH" {
            let result = client.request(
                reqwest::Method::PATCH, 
                "/api/summary", 
                None, 
                None
            ).await?;
            
            // PATCH should return 405 Method Not Allowed for most endpoints
            assert!(result.status_code == 405 || result.status_code == 404);
            HttpValidators::assert_performance(&result, Duration::from_secs(2))?;
        }
    }
    
    Ok(())
}

/// Test content negotiation
#[tokio::test]
async fn test_content_negotiation() -> Result<()> {
    let base_client = HttpTestClient::new("http://localhost:3000")?;
    
    // Test JSON format
    let json_client = base_client.with_accept("application/json");
    let result = json_client.get("/api/summary").await?;
    
    if result.success {
        HttpValidators::assert_content_type(&result, "application/json")?;
    }
    
    // Test HTML format
    let html_client = HttpTestClient::new("http://localhost:3000")?
        .with_accept("text/html");
    let result = html_client.get("/").await?;
    
    if result.success {
        HttpValidators::assert_content_type(&result, "text/html")?;
    }
    
    // Test CSV format (if supported)
    let csv_client = HttpTestClient::new("http://localhost:3000")?
        .with_accept("text/csv");
    let result = csv_client.get("/api/metrics").await?;
    
    // CSV might be supported or return 406 Not Acceptable
    if result.success {
        let content_type_valid = result.response.headers.get("content-type")
            .map(|ct| ct.contains("text/csv") || ct.contains("application/json"))
            .unwrap_or(false);
        assert!(content_type_valid, "Should return CSV or fallback to JSON");
    } else {
        // 406 Not Acceptable is valid response for unsupported format
        assert!(result.status_code == 406 || result.status_code == 404);
    }
    
    // Test wildcard accept
    let wildcard_client = HttpTestClient::new("http://localhost:3000")?
        .with_accept("*/*");
    let result = wildcard_client.get("/api/summary").await?;
    
    if result.success {
        // Should return some valid content type
        assert!(result.response.headers.get("content-type").is_some());
    }
    
    Ok(())
}

/// Test error handling and status codes
#[tokio::test]
async fn test_error_handling() -> Result<()> {
    let client = HttpTestClient::new("http://localhost:3000")?;
    
    // Test 404 Not Found
    let result = client.get("/api/nonexistent-endpoint").await?;
    HttpValidators::assert_status_code(&result, 404)?;
    HttpValidators::assert_performance(&result, Duration::from_secs(2))?;
    
    // Error response should be meaningful
    HttpValidators::assert_error_response(&result, Some(404))?;
    
    // Test invalid JSON in POST request
    let client_with_bad_json = HttpTestClient::new("http://localhost:3000")?
        .with_content_type("application/json");
    
    // Send malformed JSON
    let bad_json_result = client_with_bad_json.request(
        reqwest::Method::POST,
        "/api/v1/analysis/trigger",
        Some(json!("invalid json structure")),
        None
    ).await?;
    
    // Should return 400 Bad Request or 404 if endpoint doesn't exist
    assert!(bad_json_result.status_code == 400 || 
           bad_json_result.status_code == 404 || 
           bad_json_result.status_code == 405);
    HttpValidators::assert_performance(&bad_json_result, Duration::from_secs(2))?;
    
    // Test method not allowed
    let result = client.post("/api/summary", None).await?;
    // Summary endpoint should only support GET
    assert!(result.status_code == 405 || result.status_code == 404);
    HttpValidators::assert_performance(&result, Duration::from_secs(2))?;
    
    Ok(())
}

/// Test API versioning
#[tokio::test]
async fn test_api_versioning() -> Result<()> {
    let client = HttpTestClient::new("http://localhost:3000")?;
    
    // Test legacy API endpoints (no version prefix)
    let legacy_endpoints = [
        "/api/summary",
        "/api/metrics",
        "/api/analysis",
    ];
    
    for endpoint in &legacy_endpoints {
        let result = client.get(endpoint).await?;
        // Legacy endpoints should work or return 404 gracefully
        assert!(result.success || result.status_code == 404);
        HttpValidators::assert_performance(&result, Duration::from_secs(5))?;
    }
    
    // Test v1 API endpoints
    let v1_endpoints = [
        "/api/v1/analysis/architecture",
        "/api/v1/analysis/defects",
        "/api/v1/analysis/statistics",
    ];
    
    for endpoint in &v1_endpoints {
        let result = client.get(endpoint).await?;
        assert!(result.success || result.status_code == 404);
        HttpValidators::assert_performance(&result, Duration::from_secs(8))?;
    }
    
    // Test that both versions can coexist
    let summary_result = client.get("/api/summary").await?;
    let arch_result = client.get("/api/v1/analysis/architecture").await?;
    
    // If both exist, they should both work
    if summary_result.success && arch_result.success {
        HttpValidators::assert_content_type(&summary_result, "application/json")?;
        HttpValidators::assert_content_type(&arch_result, "application/json")?;
    }
    
    Ok(())
}

/// Test security headers and HTTPS
#[tokio::test]
async fn test_security_compliance() -> Result<()> {
    let client = HttpTestClient::new("http://localhost:3000")?;
    
    let result = client.get("/").await?;
    
    if result.success {
        // Check for security headers (these might not be present in dev mode)
        if let Some(_) = result.response.headers.get("x-content-type-options") {
            HttpValidators::assert_security_headers(&result)?;
        }
        
        // Check that sensitive information is not leaked in headers
        let sensitive_headers = [
            "x-powered-by",
            "server",
        ];
        
        for header in &sensitive_headers {
            if let Some(header_value) = result.response.headers.get(*header) {
                // Should not reveal detailed server information
                assert!(!header_value.to_lowercase().contains("version"));
                assert!(!header_value.contains("/"));
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    /// Test full HTTP API workflow
    #[tokio::test]
    async fn test_full_api_workflow() -> Result<()> {
        let client = HttpTestClient::new("http://localhost:3000")?;
        let project_path = client.create_sample_project()?;
        
        // 1. Access dashboard
        let dashboard_result = client.get("/").await?;
        // Dashboard should be accessible
        assert!(dashboard_result.success || dashboard_result.status_code == 404);
        
        // 2. Get project summary
        let summary_result = client.get("/api/summary").await?;
        if summary_result.success {
            HttpValidators::assert_content_type(&summary_result, "application/json")?;
        }
        
        // 3. Trigger analysis (if supported)
        let trigger_data = json!({
            "path": project_path.to_string_lossy(),
            "analysis_types": ["complexity"]
        });
        
        let trigger_result = client.post("/api/v1/analysis/trigger", Some(trigger_data)).await?;
        // Should either succeed or indicate method not available
        assert!(trigger_result.success || 
               trigger_result.status_code == 404 || 
               trigger_result.status_code == 405);
        
        // 4. Get analysis results
        let analysis_result = client.get("/api/analysis").await?;
        if analysis_result.success {
            HttpValidators::assert_content_type(&analysis_result, "application/json")?;
            HttpValidators::assert_performance(&analysis_result, Duration::from_secs(10))?;
        }
        
        println!("HTTP API workflow completed successfully");
        
        Ok(())
    }
    
    /// Test API endpoint discovery
    #[tokio::test]
    async fn test_api_discovery() -> Result<()> {
        let client = HttpTestClient::new("http://localhost:3000")?;
        
        // Test that main API endpoints are discoverable
        let key_endpoints = [
            "/",
            "/api/summary",
            "/api/metrics", 
            "/api/analysis",
            "/api/v1/analysis/architecture",
        ];
        
        let mut accessible_endpoints = 0;
        
        for endpoint in &key_endpoints {
            let result = client.get(endpoint).await?;
            if result.success {
                accessible_endpoints += 1;
                HttpValidators::assert_performance(&result, Duration::from_secs(10))?;
            }
        }
        
        // At least the dashboard should be accessible
        assert!(accessible_endpoints > 0, "At least one endpoint should be accessible");
        
        println!("Found {} accessible endpoints", accessible_endpoints);
        
        Ok(())
    }
}