#![cfg_attr(coverage_nightly, coverage(off))]
//! HTTP handler functions for uniform contract endpoints

use super::error::AppError;
use super::openapi::generate_openapi_spec;
use super::router::AppState;
use crate::contracts::{
    AnalyzeComplexityContract, AnalyzeDeadCodeContract, AnalyzeLintHotspotContract,
    AnalyzeSatdContract, AnalyzeTdgContract, QualityGateContract, RefactorAutoContract,
};
use axum::{
    extract::{Json, State},
    response::Json as JsonResponse,
};
use serde_json::{json, Value};

/// Analyze complexity endpoint
pub(super) async fn analyze_complexity(
    State(state): State<AppState>,
    Json(mut params): Json<Value>,
) -> Result<JsonResponse<Value>, AppError> {
    // Apply backward compatibility mapping
    params = super::super::adapter::BackwardCompatibility::map_json_params(params);

    // Parse into contract
    let contract: AnalyzeComplexityContract = serde_json::from_value(params)
        .map_err(|e| AppError::BadRequest(format!("Invalid parameters: {e}")))?;

    // Process using service
    let result = state
        .service
        .analyze_complexity(contract)
        .await
        .map_err(AppError::from)?;

    Ok(JsonResponse(result))
}

/// Analyze SATD endpoint
pub(super) async fn analyze_satd(
    State(state): State<AppState>,
    Json(mut params): Json<Value>,
) -> Result<JsonResponse<Value>, AppError> {
    params = super::super::adapter::BackwardCompatibility::map_json_params(params);

    let contract: AnalyzeSatdContract = serde_json::from_value(params)
        .map_err(|e| AppError::BadRequest(format!("Invalid parameters: {e}")))?;

    let result = state
        .service
        .analyze_satd(contract)
        .await
        .map_err(AppError::from)?;

    Ok(JsonResponse(result))
}

/// Analyze dead code endpoint
pub(super) async fn analyze_dead_code(
    State(state): State<AppState>,
    Json(mut params): Json<Value>,
) -> Result<JsonResponse<Value>, AppError> {
    params = super::super::adapter::BackwardCompatibility::map_json_params(params);

    let contract: AnalyzeDeadCodeContract = serde_json::from_value(params)
        .map_err(|e| AppError::BadRequest(format!("Invalid parameters: {e}")))?;

    let result = state
        .service
        .analyze_dead_code(contract)
        .await
        .map_err(AppError::from)?;

    Ok(JsonResponse(result))
}

/// Analyze TDG endpoint
pub(super) async fn analyze_tdg(
    State(state): State<AppState>,
    Json(mut params): Json<Value>,
) -> Result<JsonResponse<Value>, AppError> {
    params = super::super::adapter::BackwardCompatibility::map_json_params(params);

    let contract: AnalyzeTdgContract = serde_json::from_value(params)
        .map_err(|e| AppError::BadRequest(format!("Invalid parameters: {e}")))?;

    let result = state
        .service
        .analyze_tdg(contract)
        .await
        .map_err(AppError::from)?;

    Ok(JsonResponse(result))
}

/// Analyze lint hotspot endpoint
pub(super) async fn analyze_lint_hotspot(
    State(state): State<AppState>,
    Json(mut params): Json<Value>,
) -> Result<JsonResponse<Value>, AppError> {
    params = super::super::adapter::BackwardCompatibility::map_json_params(params);

    let contract: AnalyzeLintHotspotContract = serde_json::from_value(params)
        .map_err(|e| AppError::BadRequest(format!("Invalid parameters: {e}")))?;

    let result = state
        .service
        .analyze_lint_hotspot(contract)
        .await
        .map_err(AppError::from)?;

    Ok(JsonResponse(result))
}

/// Quality gate endpoint
pub(super) async fn quality_gate(
    State(state): State<AppState>,
    Json(mut params): Json<Value>,
) -> Result<JsonResponse<Value>, AppError> {
    params = super::super::adapter::BackwardCompatibility::map_json_params(params);

    let contract: QualityGateContract = serde_json::from_value(params)
        .map_err(|e| AppError::BadRequest(format!("Invalid parameters: {e}")))?;

    let result = state
        .service
        .quality_gate(contract)
        .await
        .map_err(AppError::from)?;

    Ok(JsonResponse(result))
}

/// Refactor auto endpoint
pub(super) async fn refactor_auto(
    State(state): State<AppState>,
    Json(mut params): Json<Value>,
) -> Result<JsonResponse<Value>, AppError> {
    params = super::super::adapter::BackwardCompatibility::map_json_params(params);

    let contract: RefactorAutoContract = serde_json::from_value(params)
        .map_err(|e| AppError::BadRequest(format!("Invalid parameters: {e}")))?;

    let result = state
        .service
        .refactor_auto(contract)
        .await
        .map_err(AppError::from)?;

    Ok(JsonResponse(result))
}

/// Health check endpoint
pub(super) async fn health_check() -> JsonResponse<Value> {
    JsonResponse(json!({
        "status": "healthy",
        "service": "pmat",
        "version": env!("CARGO_PKG_VERSION"),
        "contracts": "uniform"
    }))
}

/// `OpenAPI` specification endpoint
pub(super) async fn openapi_spec() -> JsonResponse<Value> {
    JsonResponse(generate_openapi_spec())
}
