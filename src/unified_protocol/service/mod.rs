#![cfg_attr(coverage_nightly, coverage(off))]
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow;
use axum::extract::Extension;
use axum::routing::{get, post};
use axum::Router;
use serde_json::Value;
use tower::ServiceBuilder;
use tower::ServiceExt;
use tower_http::compression::CompressionLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use super::error::{self, AppError};
use super::{AdapterRegistry, Protocol, UnifiedRequest, UnifiedResponse};

mod defaults;
pub mod handlers;
mod traits;
mod types;

pub use defaults::*;
pub use traits::*;
pub use types::*;

/// Main unified service that handles all protocols through a single router
#[derive(Clone)]
pub struct UnifiedService {
    router: Router,
    #[allow(dead_code)]
    adapters: Arc<AdapterRegistry>,
    state: Arc<AppState>,
}

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub template_service: Arc<dyn TemplateService>,
    pub analysis_service: Arc<dyn AnalysisService>,
    pub metrics: Arc<ServiceMetrics>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            template_service: Arc::new(DefaultTemplateService),
            analysis_service: Arc::new(DefaultAnalysisService),
            metrics: Arc::new(ServiceMetrics::default()),
        }
    }
}

/// Metrics collection for the unified service
#[derive(Default)]
pub struct ServiceMetrics {
    pub requests_total: Arc<parking_lot::Mutex<HashMap<Protocol, u64>>>,
    pub errors_total: Arc<parking_lot::Mutex<HashMap<Protocol, u64>>>,
    pub request_duration_ms: Arc<parking_lot::Mutex<HashMap<Protocol, Vec<u64>>>>,
}

impl UnifiedService {
    pub fn new() -> Self {
        let state = Arc::new(AppState::default());

        let router = Router::new()
            // Template API endpoints
            .route("/api/v1/templates", get(handlers::list_templates))
            .route(
                "/api/v1/templates/{template_id}",
                get(handlers::get_template),
            )
            .route("/api/v1/generate", post(handlers::generate_template))
            // Analysis API endpoints
            .route(
                "/api/v1/analyze/complexity",
                post(handlers::analyze_complexity).get(handlers::analyze_complexity_get),
            )
            .route("/api/v1/analyze/churn", post(handlers::analyze_churn))
            .route("/api/v1/analyze/dag", post(handlers::analyze_dag))
            .route("/api/v1/analyze/context", post(handlers::generate_context))
            .route(
                "/api/v1/analyze/dead-code",
                post(handlers::analyze_dead_code),
            )
            .route(
                "/api/v1/analyze/deep-context",
                post(handlers::analyze_deep_context),
            )
            .route(
                "/api/v1/analyze/makefile-lint",
                post(handlers::analyze_makefile_lint),
            )
            .route(
                "/api/v1/analyze/provability",
                post(handlers::analyze_provability),
            )
            .route("/api/v1/analyze/satd", post(handlers::analyze_satd))
            .route(
                "/api/v1/analyze/lint-hotspot",
                post(handlers::analyze_lint_hotspot),
            )
            // MCP protocol endpoint
            .route("/mcp/{method}", post(handlers::mcp_endpoint))
            // Health and status endpoints
            .route("/health", get(handlers::health_check))
            .route("/metrics", get(handlers::metrics))
            // Apply middleware stack
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CompressionLayer::new())
                    .layer(TimeoutLayer::with_status_code(
                        axum::http::StatusCode::REQUEST_TIMEOUT,
                        Duration::from_secs(30),
                    ))
                    .layer(Extension(state.clone())),
            );

        Self {
            router,
            adapters: Arc::new(AdapterRegistry::new()),
            state,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_template_service<T: TemplateService + 'static>(mut self, service: T) -> Self {
        let state = Arc::make_mut(&mut self.state);
        state.template_service = Arc::new(service);
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_analysis_service<A: AnalysisService + 'static>(mut self, service: A) -> Self {
        let state = Arc::make_mut(&mut self.state);
        state.analysis_service = Arc::new(service);
        self
    }

    /// Get the router for HTTP server usage
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn router(&self) -> Router {
        self.router.clone()
    }

    /// Process a unified request through the router
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn process_request(
        &self,
        request: UnifiedRequest,
    ) -> Result<UnifiedResponse, AppError> {
        let start = std::time::Instant::now();
        let trace_id = request.trace_id;

        // Extract data needed for metrics before moving request
        let request_method = request.method.clone();
        let request_path = request.path.clone();
        let request_extensions = request.extensions.clone();

        // Convert to Axum request
        let axum_request = axum::http::Request::builder()
            .method(&request.method)
            .uri(&request.path)
            .body(request.body)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to build request: {e}")))?;

        // Process through router
        let response = self
            .router
            .clone()
            .oneshot(axum_request)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Router error: {e}")))?;

        // Convert back to unified response
        let (parts, body) = response.into_parts();
        let unified_response = UnifiedResponse {
            status: parts.status,
            headers: parts.headers,
            body,
            trace_id,
        };

        // Record metrics
        let duration = start.elapsed().as_millis() as u64;
        self.record_request_metrics_by_data(
            request_method.as_str(),
            &request_path,
            &request_extensions,
            &unified_response,
            duration,
        );

        Ok(unified_response)
    }

    #[allow(dead_code)]
    fn record_request_metrics(
        &self,
        request: &UnifiedRequest,
        response: &UnifiedResponse,
        duration_ms: u64,
    ) {
        // Extract protocol from request extensions or path
        let protocol = request
            .get_extension::<Protocol>("protocol")
            .unwrap_or(Protocol::Http);

        // Update request counters
        {
            let mut requests = self.state.metrics.requests_total.lock();
            *requests.entry(protocol).or_insert(0) += 1;
        }

        // Update error counters if error response
        if response.status.is_client_error() || response.status.is_server_error() {
            let mut errors = self.state.metrics.errors_total.lock();
            *errors.entry(protocol).or_insert(0) += 1;
        }

        // Record duration
        {
            let mut durations = self.state.metrics.request_duration_ms.lock();
            durations.entry(protocol).or_default().push(duration_ms);
        }

        info!(
            protocol = %protocol,
            method = %request.method,
            path = %request.path,
            status = %response.status,
            duration_ms = duration_ms,
            "Request processed"
        );
    }

    /// Extract protocol from request path
    #[allow(dead_code)]
    fn extract_protocol_from_path(&self, path: &str) -> Protocol {
        if path.starts_with("/mcp/") {
            Protocol::Mcp
        } else {
            Protocol::Http
        }
    }

    fn record_request_metrics_by_data(
        &self,
        method: &str,
        path: &str,
        extensions: &HashMap<String, Value>,
        response: &UnifiedResponse,
        duration_ms: u64,
    ) {
        // Extract protocol from extensions or default to HTTP
        let protocol = extensions
            .get("protocol")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or(Protocol::Http);

        // Update request counters
        {
            let mut requests = self.state.metrics.requests_total.lock();
            *requests.entry(protocol).or_insert(0) += 1;
        }

        // Update error counters if error response
        if response.status.is_client_error() || response.status.is_server_error() {
            let mut errors = self.state.metrics.errors_total.lock();
            *errors.entry(protocol).or_insert(0) += 1;
        }

        // Record duration
        {
            let mut durations = self.state.metrics.request_duration_ms.lock();
            durations.entry(protocol).or_default().push(duration_ms);
        }

        info!(
            protocol = ?protocol,
            method = method,
            path = path,
            status = %response.status,
            duration_ms = duration_ms,
            "Request processed"
        );
    }
}

impl Default for UnifiedService {
    fn default() -> Self {
        Self::new()
    }
}

// Tests extracted to service_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "../service_tests.rs"]
mod tests;
