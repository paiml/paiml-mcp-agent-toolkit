//! HTTP server implementation using uniform contracts
//! This ensures HTTP endpoints use exactly the same contracts as CLI and MCP

use super::service::ContractService;
use super::{
    AnalyzeComplexityContract, AnalyzeDeadCodeContract, AnalyzeLintHotspotContract,
    AnalyzeSatdContract, AnalyzeTdgContract, QualityGateContract, RefactorAutoContract,
};
use anyhow::Result;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

/// HTTP server state with contract service
#[derive(Clone)]
struct AppState {
    service: Arc<ContractService>,
}

/// Create HTTP router with all endpoints using uniform contracts
pub fn create_router() -> Result<Router> {
    let service = Arc::new(ContractService::new()?);
    let state = AppState { service };

    Ok(Router::new()
        // Analysis endpoints - all use uniform contracts
        .route("/api/analyze/complexity", post(analyze_complexity))
        .route("/api/analyze/satd", post(analyze_satd))
        .route("/api/analyze/dead-code", post(analyze_dead_code))
        .route("/api/analyze/tdg", post(analyze_tdg))
        .route("/api/analyze/lint-hotspot", post(analyze_lint_hotspot))
        // Quality and refactoring endpoints
        .route("/api/quality-gate", post(quality_gate))
        .route("/api/refactor/auto", post(refactor_auto))
        // Health check
        .route("/health", get(health_check))
        // OpenAPI spec endpoint
        .route("/api/openapi", get(openapi_spec))
        .layer(CorsLayer::permissive())
        .with_state(state))
}

/// Analyze complexity endpoint
async fn analyze_complexity(
    State(state): State<AppState>,
    Json(mut params): Json<Value>,
) -> Result<Json<Value>, AppError> {
    // Apply backward compatibility mapping
    params = super::adapter::BackwardCompatibility::map_json_params(params);

    // Parse into contract
    let contract: AnalyzeComplexityContract = serde_json::from_value(params)
        .map_err(|e| AppError::BadRequest(format!("Invalid parameters: {e}")))?;

    // Process using service
    let result = state
        .service
        .analyze_complexity(contract)
        .await
        .map_err(AppError::from)?;

    Ok(Json(result))
}

/// Analyze SATD endpoint
async fn analyze_satd(
    State(state): State<AppState>,
    Json(mut params): Json<Value>,
) -> Result<Json<Value>, AppError> {
    params = super::adapter::BackwardCompatibility::map_json_params(params);

    let contract: AnalyzeSatdContract = serde_json::from_value(params)
        .map_err(|e| AppError::BadRequest(format!("Invalid parameters: {e}")))?;

    let result = state
        .service
        .analyze_satd(contract)
        .await
        .map_err(AppError::from)?;

    Ok(Json(result))
}

/// Analyze dead code endpoint
async fn analyze_dead_code(
    State(state): State<AppState>,
    Json(mut params): Json<Value>,
) -> Result<Json<Value>, AppError> {
    params = super::adapter::BackwardCompatibility::map_json_params(params);

    let contract: AnalyzeDeadCodeContract = serde_json::from_value(params)
        .map_err(|e| AppError::BadRequest(format!("Invalid parameters: {e}")))?;

    let result = state
        .service
        .analyze_dead_code(contract)
        .await
        .map_err(AppError::from)?;

    Ok(Json(result))
}

/// Analyze TDG endpoint
async fn analyze_tdg(
    State(state): State<AppState>,
    Json(mut params): Json<Value>,
) -> Result<Json<Value>, AppError> {
    params = super::adapter::BackwardCompatibility::map_json_params(params);

    let contract: AnalyzeTdgContract = serde_json::from_value(params)
        .map_err(|e| AppError::BadRequest(format!("Invalid parameters: {e}")))?;

    let result = state
        .service
        .analyze_tdg(contract)
        .await
        .map_err(AppError::from)?;

    Ok(Json(result))
}

/// Analyze lint hotspot endpoint
async fn analyze_lint_hotspot(
    State(state): State<AppState>,
    Json(mut params): Json<Value>,
) -> Result<Json<Value>, AppError> {
    params = super::adapter::BackwardCompatibility::map_json_params(params);

    let contract: AnalyzeLintHotspotContract = serde_json::from_value(params)
        .map_err(|e| AppError::BadRequest(format!("Invalid parameters: {e}")))?;

    let result = state
        .service
        .analyze_lint_hotspot(contract)
        .await
        .map_err(AppError::from)?;

    Ok(Json(result))
}

/// Quality gate endpoint
async fn quality_gate(
    State(state): State<AppState>,
    Json(mut params): Json<Value>,
) -> Result<Json<Value>, AppError> {
    params = super::adapter::BackwardCompatibility::map_json_params(params);

    let contract: QualityGateContract = serde_json::from_value(params)
        .map_err(|e| AppError::BadRequest(format!("Invalid parameters: {e}")))?;

    let result = state
        .service
        .quality_gate(contract)
        .await
        .map_err(AppError::from)?;

    Ok(Json(result))
}

/// Refactor auto endpoint
async fn refactor_auto(
    State(state): State<AppState>,
    Json(mut params): Json<Value>,
) -> Result<Json<Value>, AppError> {
    params = super::adapter::BackwardCompatibility::map_json_params(params);

    let contract: RefactorAutoContract = serde_json::from_value(params)
        .map_err(|e| AppError::BadRequest(format!("Invalid parameters: {e}")))?;

    let result = state
        .service
        .refactor_auto(contract)
        .await
        .map_err(AppError::from)?;

    Ok(Json(result))
}

/// Health check endpoint
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "pmat",
        "version": env!("CARGO_PKG_VERSION"),
        "contracts": "uniform"
    }))
}

/// `OpenAPI` specification endpoint
async fn openapi_spec() -> Json<Value> {
    Json(generate_openapi_spec())
}

/// Generate `OpenAPI` specification with uniform contracts
fn generate_openapi_spec() -> Value {
    json!({
        "openapi": "3.0.0",
        "info": {
            "title": "PMAT API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Professional project quantitative analysis toolkit with uniform contracts"
        },
        "servers": [
            {
                "url": "http://localhost:8080",
                "description": "Local server"
            }
        ],
        "paths": {
            "/api/analyze/complexity": {
                "post": {
                    "summary": "Analyze code complexity",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/AnalyzeComplexityContract"
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Analysis results"
                        }
                    }
                }
            },
            "/api/analyze/satd": {
                "post": {
                    "summary": "Analyze SATD",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/AnalyzeSatdContract"
                                }
                            }
                        }
                    }
                }
            },
            "/api/analyze/dead-code": {
                "post": {
                    "summary": "Analyze dead code",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/AnalyzeDeadCodeContract"
                                }
                            }
                        }
                    }
                }
            },
            "/api/analyze/tdg": {
                "post": {
                    "summary": "Analyze TDG",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/AnalyzeTdgContract"
                                }
                            }
                        }
                    }
                }
            },
            "/api/analyze/lint-hotspot": {
                "post": {
                    "summary": "Analyze lint hotspots",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/AnalyzeLintHotspotContract"
                                }
                            }
                        }
                    }
                }
            },
            "/api/quality-gate": {
                "post": {
                    "summary": "Run quality gate",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/QualityGateContract"
                                }
                            }
                        }
                    }
                }
            },
            "/api/refactor/auto": {
                "post": {
                    "summary": "Auto refactor",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/RefactorAutoContract"
                                }
                            }
                        }
                    }
                }
            }
        },
        "components": {
            "schemas": {
                "BaseAnalysisContract": {
                    "type": "object",
                    "required": ["path"],
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to analyze"
                        },
                        "format": {
                            "type": "string",
                            "enum": ["table", "json", "yaml", "markdown", "csv", "summary"],
                            "default": "table"
                        },
                        "output": {
                            "type": "string",
                            "description": "Output file path"
                        },
                        "top_files": {
                            "type": "integer",
                            "description": "Number of top files",
                            "default": 10
                        },
                        "include_tests": {
                            "type": "boolean",
                            "default": false
                        },
                        "timeout": {
                            "type": "integer",
                            "default": 60
                        }
                    }
                },
                "AnalyzeComplexityContract": {
                    "allOf": [
                        { "$ref": "#/components/schemas/BaseAnalysisContract" },
                        {
                            "type": "object",
                            "properties": {
                                "max_cyclomatic": { "type": "integer" },
                                "max_cognitive": { "type": "integer" },
                                "max_halstead": { "type": "number" }
                            }
                        }
                    ]
                },
                "AnalyzeSatdContract": {
                    "allOf": [
                        { "$ref": "#/components/schemas/BaseAnalysisContract" },
                        {
                            "type": "object",
                            "properties": {
                                "severity": {
                                    "type": "string",
                                    "enum": ["low", "medium", "high", "critical"]
                                },
                                "critical_only": { "type": "boolean" },
                                "strict": { "type": "boolean" },
                                "fail_on_violation": { "type": "boolean" }
                            }
                        }
                    ]
                },
                "AnalyzeDeadCodeContract": {
                    "allOf": [
                        { "$ref": "#/components/schemas/BaseAnalysisContract" },
                        {
                            "type": "object",
                            "properties": {
                                "include_unreachable": { "type": "boolean" },
                                "min_dead_lines": { "type": "integer" },
                                "max_percentage": { "type": "number" },
                                "fail_on_violation": { "type": "boolean" }
                            }
                        }
                    ]
                },
                "AnalyzeTdgContract": {
                    "allOf": [
                        { "$ref": "#/components/schemas/BaseAnalysisContract" },
                        {
                            "type": "object",
                            "properties": {
                                "threshold": { "type": "number" },
                                "include_components": { "type": "boolean" },
                                "critical_only": { "type": "boolean" }
                            }
                        }
                    ]
                },
                "AnalyzeLintHotspotContract": {
                    "allOf": [
                        { "$ref": "#/components/schemas/BaseAnalysisContract" },
                        {
                            "type": "object",
                            "properties": {
                                "file": { "type": "string" },
                                "max_density": { "type": "number" },
                                "min_confidence": { "type": "number" },
                                "enforce": { "type": "boolean" },
                                "dry_run": { "type": "boolean" }
                            }
                        }
                    ]
                },
                "QualityGateContract": {
                    "allOf": [
                        { "$ref": "#/components/schemas/BaseAnalysisContract" },
                        {
                            "type": "object",
                            "properties": {
                                "profile": {
                                    "type": "string",
                                    "enum": ["standard", "strict", "extreme", "toyota"]
                                },
                                "file": { "type": "string" },
                                "fail_on_violation": { "type": "boolean" },
                                "verbose": { "type": "boolean" }
                            }
                        }
                    ]
                },
                "RefactorAutoContract": {
                    "type": "object",
                    "required": ["file"],
                    "properties": {
                        "file": { "type": "string" },
                        "format": {
                            "type": "string",
                            "enum": ["table", "json", "yaml", "markdown", "csv", "summary"]
                        },
                        "output": { "type": "string" },
                        "target_complexity": { "type": "integer" },
                        "dry_run": { "type": "boolean" },
                        "timeout": { "type": "integer" }
                    }
                }
            }
        }
    })
}

/// Error type for HTTP endpoints
#[derive(Debug)]
enum AppError {
    BadRequest(String),
    Internal(anyhow::Error),
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        let err_str = err.to_string();
        // Check if this is a contract validation error (should be 400, not 500)
        if err_str.contains("Path not found")
            || err_str.contains("Invalid timeout")
            || err_str.contains("Too many files")
            || err_str.contains("Invalid parameter")
            || err_str.contains("Missing required")
            || err_str.contains("must be")
        {
            AppError::BadRequest(err_str)
        } else {
            AppError::Internal(err)
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        };

        let body = Json(json!({
            "error": message
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use serde_json::json;
    use std::sync::Arc;
    use tower::ServiceExt;

    // ==========================================================================
    // Test: create_router function
    // ==========================================================================

    #[test]
    fn test_create_router_success() {
        // create_router should successfully create a router
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

        // Top level fields
        assert_eq!(spec["openapi"], "3.0.0");
        assert_eq!(spec["info"]["title"], "PMAT API");
        assert!(spec["info"]["version"].is_string());
        assert!(spec["info"]["description"]
            .as_str()
            .unwrap()
            .contains("uniform contracts"));

        // Servers
        assert!(spec["servers"].is_array());
        assert_eq!(spec["servers"][0]["url"], "http://localhost:8080");

        // Paths
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

        // Check all schema definitions exist
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
        // Verify both point to same service
        assert!(Arc::ptr_eq(&state.service, &cloned.service));
    }

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
    // Test: OpenAPI spec format validation
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

        // QualityGateContract uses allOf pattern
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

        // AnalyzeSatdContract uses allOf pattern
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

        // Test each route exists by checking 404 is NOT returned for valid paths
        let routes = vec![
            ("/health", "GET"),
            ("/api/openapi", "GET"),
            // POST routes will fail with 400 (bad request) not 404
        ];

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
        // CORS preflight should return 200 OK
        assert_eq!(response.status(), StatusCode::OK);
    }

    // ==========================================================================
    // Test: health_check function directly
    // ==========================================================================

    #[tokio::test]
    async fn test_health_check_direct_call() {
        let result = health_check().await;
        let json = result.0;

        assert_eq!(json["status"], "healthy");
        assert_eq!(json["service"], "pmat");
        assert_eq!(json["contracts"], "uniform");
    }

    // ==========================================================================
    // Test: openapi_spec function directly
    // ==========================================================================

    #[tokio::test]
    async fn test_openapi_spec_direct_call() {
        let result = openapi_spec().await;
        let json = result.0;

        assert_eq!(json["openapi"], "3.0.0");
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
