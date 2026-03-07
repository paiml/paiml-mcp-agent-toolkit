#![cfg_attr(coverage_nightly, coverage(off))]
//! Core tests for HTTP server: router, AppError, AppState, health, openapi

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::super::handlers::{health_check, openapi_spec};
    use super::super::openapi::generate_openapi_spec;
    use super::super::router::create_router;
    use super::super::types::{AppError, AppState};
    use crate::contracts::service::ContractService;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        response::IntoResponse,
    };
    use serde_json::{json, Value};
    use std::sync::Arc;
    use tower::ServiceExt;

    // ==========================================================================
    // Test: create_router function
    // ==========================================================================

    #[test]
    fn test_create_router_success() {
        let result = create_router();
        assert!(result.is_ok(), "create_router should return Ok");
    }

    // ==========================================================================
    // Test: health_check endpoint
    // ==========================================================================

    #[tokio::test]
    async fn test_health_check_returns_expected_json() {
        let router = create_router().expect("Failed to create router");

        let request = Request::builder()
            .uri("/health")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["status"], "healthy");
        assert_eq!(json["service"], "pmat");
        assert_eq!(json["contracts"], "uniform");
        assert!(json["version"].is_string());
    }

    // ==========================================================================
    // Test: openapi_spec endpoint
    // ==========================================================================

    #[tokio::test]
    async fn test_openapi_spec_returns_valid_spec() {
        let router = create_router().expect("Failed to create router");

        let request = Request::builder()
            .uri("/api/openapi")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 65536)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["openapi"], "3.0.0");
        assert!(json["info"]["title"].is_string());
        assert!(json["paths"].is_object());
        assert!(json["components"]["schemas"].is_object());
    }

    // ==========================================================================
    // Test: generate_openapi_spec function
    // ==========================================================================

    #[test]
    fn test_generate_openapi_spec_structure() {
        let spec = generate_openapi_spec();

        assert_eq!(spec["openapi"], "3.0.0");
        assert_eq!(spec["info"]["title"], "PMAT API");
        assert!(spec["info"]["version"].is_string());
        assert!(spec["info"]["description"]
            .as_str()
            .unwrap()
            .contains("uniform contracts"));

        assert!(spec["servers"].is_array());
        assert_eq!(spec["servers"][0]["url"], "http://localhost:8080");

        let paths = &spec["paths"];
        assert!(paths["/api/analyze/complexity"]["post"].is_object());
        assert!(paths["/api/analyze/satd"]["post"].is_object());
        assert!(paths["/api/analyze/dead-code"]["post"].is_object());
        assert!(paths["/api/analyze/tdg"]["post"].is_object());
        assert!(paths["/api/analyze/lint-hotspot"]["post"].is_object());
        assert!(paths["/api/quality-gate"]["post"].is_object());
        assert!(paths["/api/refactor/auto"]["post"].is_object());
    }

    #[test]
    fn test_generate_openapi_spec_components() {
        let spec = generate_openapi_spec();
        let schemas = &spec["components"]["schemas"];

        assert!(schemas["BaseAnalysisContract"].is_object());
        assert!(schemas["AnalyzeComplexityContract"].is_object());
        assert!(schemas["AnalyzeSatdContract"].is_object());
        assert!(schemas["AnalyzeDeadCodeContract"].is_object());
        assert!(schemas["AnalyzeTdgContract"].is_object());
        assert!(schemas["AnalyzeLintHotspotContract"].is_object());
        assert!(schemas["QualityGateContract"].is_object());
        assert!(schemas["RefactorAutoContract"].is_object());
    }

    #[test]
    fn test_generate_openapi_spec_base_contract_properties() {
        let spec = generate_openapi_spec();
        let base = &spec["components"]["schemas"]["BaseAnalysisContract"];

        assert_eq!(base["type"], "object");
        assert!(base["required"]
            .as_array()
            .unwrap()
            .contains(&json!("path")));

        let props = &base["properties"];
        assert!(props["path"].is_object());
        assert!(props["format"].is_object());
        assert!(props["output"].is_object());
        assert!(props["top_files"].is_object());
        assert!(props["include_tests"].is_object());
        assert!(props["timeout"].is_object());
    }

    // ==========================================================================
    // Test: AppError enum
    // ==========================================================================

    #[test]
    fn test_app_error_bad_request_debug() {
        let err = AppError::BadRequest("test message".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("BadRequest"));
        assert!(debug_str.contains("test message"));
    }

    #[test]
    fn test_app_error_internal_debug() {
        let err = AppError::Internal(anyhow::anyhow!("internal error"));
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Internal"));
    }

    #[test]
    fn test_app_error_from_anyhow() {
        let anyhow_err = anyhow::anyhow!("some error");
        let app_err: AppError = AppError::from(anyhow_err);
        match app_err {
            AppError::Internal(e) => {
                assert!(e.to_string().contains("some error"));
            }
            _ => panic!("Expected Internal error"),
        }
    }

    #[test]
    fn test_app_error_bad_request_into_response() {
        let err = AppError::BadRequest("invalid params".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_app_error_internal_into_response() {
        let err = AppError::Internal(anyhow::anyhow!("internal failure"));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_app_error_response_body_format() {
        let err = AppError::BadRequest("test error message".to_string());
        let response = err.into_response();

        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"], "test error message");
    }

    // ==========================================================================
    // Test: AppState clone
    // ==========================================================================

    #[test]
    fn test_app_state_clone() {
        let service = Arc::new(ContractService::new().expect("Failed to create service"));
        let state = AppState {
            service: service.clone(),
        };
        let cloned = state.clone();
        assert!(Arc::ptr_eq(&state.service, &cloned.service));
    }

    // ==========================================================================
    // Test: OpenAPI spec format/enum validation
    // ==========================================================================

    #[test]
    fn test_openapi_spec_format_enum() {
        let spec = generate_openapi_spec();
        let base_props = &spec["components"]["schemas"]["BaseAnalysisContract"]["properties"];
        let format_enum = &base_props["format"]["enum"];

        assert!(format_enum.is_array());
        let formats: Vec<&str> = format_enum
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();

        assert!(formats.contains(&"table"));
        assert!(formats.contains(&"json"));
        assert!(formats.contains(&"yaml"));
        assert!(formats.contains(&"markdown"));
        assert!(formats.contains(&"csv"));
        assert!(formats.contains(&"summary"));
    }

    #[test]
    fn test_openapi_spec_quality_profile_enum() {
        let spec = generate_openapi_spec();
        let quality_gate = &spec["components"]["schemas"]["QualityGateContract"];

        let all_of = quality_gate.get("allOf");
        if let Some(all_of_arr) = all_of.and_then(|v| v.as_array()) {
            let profile_schema = all_of_arr
                .iter()
                .find(|v| v.get("properties").and_then(|p| p.get("profile")).is_some());

            if let Some(schema) = profile_schema {
                let profile_enum = &schema["properties"]["profile"]["enum"];
                assert!(profile_enum.is_array());
            }
        }
    }

    #[test]
    fn test_openapi_spec_satd_severity_enum() {
        let spec = generate_openapi_spec();
        let satd_contract = &spec["components"]["schemas"]["AnalyzeSatdContract"];

        let all_of = satd_contract.get("allOf");
        if let Some(all_of_arr) = all_of.and_then(|v| v.as_array()) {
            let severity_schema = all_of_arr.iter().find(|v| {
                v.get("properties")
                    .and_then(|p| p.get("severity"))
                    .is_some()
            });

            if let Some(schema) = severity_schema {
                let severity_enum = &schema["properties"]["severity"]["enum"];
                assert!(severity_enum.is_array());
                let severities: Vec<&str> = severity_enum
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap())
                    .collect();

                assert!(severities.contains(&"low"));
                assert!(severities.contains(&"medium"));
                assert!(severities.contains(&"high"));
                assert!(severities.contains(&"critical"));
            }
        }
    }

    // ==========================================================================
    // Test: Router routes existence
    // ==========================================================================

    #[tokio::test]
    async fn test_router_has_all_routes() {
        let router = create_router().expect("Failed to create router");

        let routes = vec![("/health", "GET"), ("/api/openapi", "GET")];

        for (path, method) in routes {
            let request = Request::builder()
                .uri(path)
                .method(method)
                .body(Body::empty())
                .unwrap();

            let response = router.clone().oneshot(request).await.unwrap();
            assert_ne!(
                response.status(),
                StatusCode::NOT_FOUND,
                "Route {} {} should exist",
                method,
                path
            );
        }
    }

    #[tokio::test]
    async fn test_unknown_route_returns_404() {
        let router = create_router().expect("Failed to create router");

        let request = Request::builder()
            .uri("/api/unknown/route")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // ==========================================================================
    // Test: CORS layer is present (router accepts OPTIONS)
    // ==========================================================================

    #[tokio::test]
    async fn test_cors_preflight_request() {
        let router = create_router().expect("Failed to create router");

        let request = Request::builder()
            .uri("/api/analyze/complexity")
            .method("OPTIONS")
            .header("Origin", "http://localhost:3000")
            .header("Access-Control-Request-Method", "POST")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // ==========================================================================
    // Test: direct function calls
    // ==========================================================================

    #[tokio::test]
    async fn test_health_check_direct_call() {
        let result = health_check().await;
        let json = result.0;

        assert_eq!(json["status"], "healthy");
        assert_eq!(json["service"], "pmat");
        assert_eq!(json["contracts"], "uniform");
    }

    #[tokio::test]
    async fn test_openapi_spec_direct_call() {
        let result = openapi_spec().await;
        let json = result.0;

        assert_eq!(json["openapi"], "3.0.0");
    }
}
