#![cfg_attr(coverage_nightly, coverage(off))]
use std::sync::Arc;

use anyhow;
use axum::extract::{Extension, Path, Query};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::Value;

use super::error::{set_protocol_context, AppError};
use super::types::*;
use super::AppState;
use super::Protocol;

/// List available templates
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn list_templates(
    Extension(state): Extension<Arc<AppState>>,
    Query(query): Query<ListTemplatesQuery>,
) -> Result<Json<TemplateList>, AppError> {
    let templates = state.template_service.list_templates(&query).await?;
    Ok(Json(templates))
}

/// Get a specific template
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn get_template(
    Extension(state): Extension<Arc<AppState>>,
    Path(template_id): Path<String>,
) -> Result<Json<TemplateInfo>, AppError> {
    let template = state.template_service.get_template(&template_id).await?;
    Ok(Json(template))
}

/// Generate a template
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn generate_template(
    Extension(state): Extension<Arc<AppState>>,
    Json(params): Json<GenerateParams>,
) -> Result<Json<GeneratedTemplate>, AppError> {
    let result = state.template_service.generate_template(&params).await?;
    Ok(Json(result))
}

/// Analyze code complexity (POST)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn analyze_complexity(
    Extension(state): Extension<Arc<AppState>>,
    Json(params): Json<ComplexityParams>,
) -> Result<Json<ComplexityAnalysis>, AppError> {
    let analysis = state.analysis_service.analyze_complexity(&params).await?;
    Ok(Json(analysis))
}

/// Analyze code complexity (GET with query parameters)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn analyze_complexity_get(
    Extension(state): Extension<Arc<AppState>>,
    Query(query): Query<ComplexityQueryParams>,
) -> Result<Json<ComplexityAnalysis>, AppError> {
    // Convert query parameters to ComplexityParams
    let params = ComplexityParams {
        project_path: query.project_path.unwrap_or_else(|| ".".to_string()),
        toolchain: query.toolchain.unwrap_or_else(|| "rust".to_string()),
        format: query.format.unwrap_or_else(|| "json".to_string()),
        max_cyclomatic: query.max_cyclomatic,
        max_cognitive: query.max_cognitive,
        top_files: query.top_files,
    };

    let analysis = state.analysis_service.analyze_complexity(&params).await?;
    Ok(Json(analysis))
}

/// Analyze code churn
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn analyze_churn(
    Extension(state): Extension<Arc<AppState>>,
    Json(params): Json<ChurnParams>,
) -> Result<Json<ChurnAnalysis>, AppError> {
    let analysis = state.analysis_service.analyze_churn(&params).await?;
    Ok(Json(analysis))
}

/// Analyze dependency graph
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn analyze_dag(
    Extension(state): Extension<Arc<AppState>>,
    Json(params): Json<DagParams>,
) -> Result<Json<DagAnalysis>, AppError> {
    let analysis = state.analysis_service.analyze_dag(&params).await?;
    Ok(Json(analysis))
}

/// Generate project context
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn generate_context(
    Extension(state): Extension<Arc<AppState>>,
    Json(params): Json<ContextParams>,
) -> Result<Json<ProjectContext>, AppError> {
    let context = state.analysis_service.generate_context(&params).await?;
    Ok(Json(context))
}

/// Analyze dead code
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn analyze_dead_code(
    Extension(state): Extension<Arc<AppState>>,
    Json(params): Json<DeadCodeParams>,
) -> Result<Json<DeadCodeAnalysis>, AppError> {
    let analysis = state.analysis_service.analyze_dead_code(&params).await?;
    Ok(Json(analysis))
}

/// Health check endpoint
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Metrics endpoint
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn metrics(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    let requests = state.metrics.requests_total.lock().clone();
    let errors = state.metrics.errors_total.lock().clone();

    Json(serde_json::json!({
        "requests_total": requests,
        "errors_total": errors,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// Analysis handlers: deep_context, makefile_lint, provability, satd
include!("handlers_analysis.rs");

// Dispatch handlers: lint_hotspot, MCP routing, mcp_endpoint
include!("handlers_dispatch.rs");
