#![cfg_attr(coverage_nightly, coverage(off))]
//! Endpoint-specific tests for HTTP server

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod endpoint_tests {
    use super::super::openapi::generate_openapi_spec;
    use super::super::router::create_router;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use serde_json::{json, Value};
    use tower::ServiceExt;

    // ==========================================================================
    // Test: analyze_complexity endpoint - invalid params
    // ==========================================================================

    #[tokio::test]
    async fn test_analyze_complexity_invalid_json() {
        let router = create_router().expect("Failed to create router");

        // Path to non-existent directory should fail validation
        let request = Request::builder()
            .uri("/api/analyze/complexity")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"path": "/nonexistent/path/that/does/not/exist/1234567890"}"#,
            ))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_analyze_complexity_malformed_json() {
        let router = create_router().expect("Failed to create router");

        let request = Request::builder()
            .uri("/api/analyze/complexity")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"not json at all"#))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        // Should fail due to JSON parsing
        assert!(response.status().is_client_error() || response.status().is_server_error());
    }

    // ==========================================================================
    // Test: analyze_satd endpoint - invalid params
    // ==========================================================================

    #[tokio::test]
    async fn test_analyze_satd_invalid_params() {
        let router = create_router().expect("Failed to create router");

        // Path to non-existent directory should fail validation
        let request = Request::builder()
            .uri("/api/analyze/satd")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"path": "/nonexistent/path/that/does/not/exist/satd"}"#,
            ))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    // ==========================================================================
    // Test: analyze_dead_code endpoint - invalid params
    // ==========================================================================

    #[tokio::test]
    async fn test_analyze_dead_code_invalid_params() {
        let router = create_router().expect("Failed to create router");

        // Path to non-existent directory should fail validation
        let request = Request::builder()
            .uri("/api/analyze/dead-code")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"path": "/nonexistent/path/that/does/not/exist/deadcode"}"#,
            ))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    // ==========================================================================
    // Test: analyze_tdg endpoint - invalid params
    // ==========================================================================

    #[tokio::test]
    async fn test_analyze_tdg_invalid_params() {
        let router = create_router().expect("Failed to create router");

        let request = Request::builder()
            .uri("/api/analyze/tdg")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"threshold": "not a number"}"#))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    // ==========================================================================
    // Test: analyze_lint_hotspot endpoint - invalid params
    // ==========================================================================

    #[tokio::test]
    async fn test_analyze_lint_hotspot_invalid_params() {
        let router = create_router().expect("Failed to create router");

        let request = Request::builder()
            .uri("/api/analyze/lint-hotspot")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"max_density": "invalid"}"#))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    // ==========================================================================
    // Test: quality_gate endpoint - invalid params
    // ==========================================================================

    #[tokio::test]
    async fn test_quality_gate_invalid_params() {
        let router = create_router().expect("Failed to create router");

        let request = Request::builder()
            .uri("/api/quality-gate")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"profile": 123}"#))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    // ==========================================================================
    // Test: refactor_auto endpoint - invalid params
    // ==========================================================================

    #[tokio::test]
    async fn test_refactor_auto_invalid_params() {
        let router = create_router().expect("Failed to create router");

        let request = Request::builder()
            .uri("/api/refactor/auto")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"file": 123}"#))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    // ==========================================================================
    // Test: backward compatibility mapping through endpoints
    // ==========================================================================

    #[tokio::test]
    async fn test_backward_compat_project_path_mapping() {
        let router = create_router().expect("Failed to create router");

        // Use project_path (old parameter) instead of path
        let request = Request::builder()
            .uri("/api/analyze/complexity")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"project_path": "/nonexistent/path", "format": "json"}"#,
            ))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        // Path doesn't exist, but we should get to the validation step
        // (meaning backward compat mapping worked)
        let status = response.status();
        assert!(
            status == StatusCode::BAD_REQUEST || status == StatusCode::INTERNAL_SERVER_ERROR,
            "Expected 400 or 500, got {}",
            status
        );
    }

    // ==========================================================================
    // Test: endpoint with valid path but non-existent file
    // ==========================================================================

    #[tokio::test]
    async fn test_analyze_complexity_with_nonexistent_path() {
        let router = create_router().expect("Failed to create router");

        let request = Request::builder()
            .uri("/api/analyze/complexity")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"path": "/this/path/does/not/exist/xyz123", "format": "json"}"#,
            ))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        let status = response.status();
        // Should fail due to non-existent path
        assert!(
            status == StatusCode::BAD_REQUEST || status == StatusCode::INTERNAL_SERVER_ERROR,
            "Expected error status, got {}",
            status
        );
    }

    // ==========================================================================
    // Test: multiple error scenarios
    // ==========================================================================

    #[tokio::test]
    async fn test_empty_body_request() {
        let router = create_router().expect("Failed to create router");

        let request = Request::builder()
            .uri("/api/analyze/complexity")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        // Should fail due to empty body
        assert!(response.status().is_client_error() || response.status().is_server_error());
    }

    #[tokio::test]
    async fn test_null_body_request() {
        let router = create_router().expect("Failed to create router");

        let request = Request::builder()
            .uri("/api/analyze/satd")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from("null"))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        // Should fail due to null body
        assert!(response.status().is_client_error() || response.status().is_server_error());
    }

    // ==========================================================================
    // Test: contract with valid structure but invalid values
    // ==========================================================================

    #[tokio::test]
    async fn test_analyze_dead_code_invalid_max_percentage() {
        let router = create_router().expect("Failed to create router");

        let request = Request::builder()
            .uri("/api/analyze/dead-code")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"path": "/tmp", "max_percentage": 150.0, "include_unreachable": false, "min_dead_lines": 0, "fail_on_violation": false}"#,
            ))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        // Invalid percentage should trigger validation error
        let status = response.status();
        assert!(
            status == StatusCode::BAD_REQUEST || status == StatusCode::INTERNAL_SERVER_ERROR,
            "Expected error for invalid max_percentage, got {}",
            status
        );
    }

    #[tokio::test]
    async fn test_analyze_tdg_negative_threshold() {
        let router = create_router().expect("Failed to create router");

        let request = Request::builder()
            .uri("/api/analyze/tdg")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"path": "/tmp", "threshold": -1.0, "include_components": false, "critical_only": false}"#,
            ))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        let status = response.status();
        assert!(
            status == StatusCode::BAD_REQUEST || status == StatusCode::INTERNAL_SERVER_ERROR,
            "Expected error for negative threshold, got {}",
            status
        );
    }

    // ==========================================================================
    // Test: Response body contains error key for errors
    // ==========================================================================

    #[tokio::test]
    async fn test_error_response_has_error_key() {
        let router = create_router().expect("Failed to create router");

        // Path to non-existent directory should fail validation and return error
        let request = Request::builder()
            .uri("/api/analyze/complexity")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"path": "/nonexistent/path/that/does/not/exist/error_test"}"#,
            ))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert!(
            json.get("error").is_some(),
            "Error response should have 'error' key"
        );
    }

    // ==========================================================================
    // Property tests for OpenAPI spec consistency
    // ==========================================================================

    #[test]
    fn test_openapi_all_paths_have_post_summary() {
        let spec = generate_openapi_spec();
        let paths = spec["paths"].as_object().unwrap();

        for (path, methods) in paths {
            if let Some(post) = methods.get("post") {
                assert!(
                    post.get("summary").is_some(),
                    "Path {} POST should have summary",
                    path
                );
            }
        }
    }

    #[test]
    fn test_openapi_all_post_endpoints_require_body() {
        let spec = generate_openapi_spec();
        let paths = spec["paths"].as_object().unwrap();

        for (path, methods) in paths {
            if let Some(post) = methods.get("post") {
                let request_body = post.get("requestBody");
                assert!(
                    request_body.is_some(),
                    "Path {} POST should have requestBody",
                    path
                );
                if let Some(rb) = request_body {
                    assert_eq!(
                        rb.get("required"),
                        Some(&json!(true)),
                        "Path {} requestBody should be required",
                        path
                    );
                }
            }
        }
    }
}
