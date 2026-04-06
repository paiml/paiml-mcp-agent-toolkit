#![cfg_attr(coverage_nightly, coverage(off))]
//! HTTP router creation with all endpoints

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::contracts::service::ContractService;

use super::handlers::{
    analyze_complexity, analyze_dead_code, analyze_lint_hotspot, analyze_satd, analyze_tdg,
    health_check, openapi_spec, quality_gate, refactor_auto,
};
use super::types::AppState;

/// Create HTTP router with all endpoints using uniform contracts
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
