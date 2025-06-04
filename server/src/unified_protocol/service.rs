use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow;
use axum::extract::{Extension, Path, Query};
use axum::http::Method;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower::{ServiceBuilder, ServiceExt};
use tower_http::compression::CompressionLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, instrument};

use super::error::{set_protocol_context, AppError};
use super::{AdapterRegistry, Protocol, UnifiedRequest, UnifiedResponse};

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
                    .layer(TimeoutLayer::new(Duration::from_secs(30)))
                    .layer(Extension(state.clone())),
            );

        Self {
            router,
            adapters: Arc::new(AdapterRegistry::new()),
            state,
        }
    }

    pub fn with_template_service<T: TemplateService + 'static>(mut self, service: T) -> Self {
        let state = Arc::make_mut(&mut self.state);
        state.template_service = Arc::new(service);
        self
    }

    pub fn with_analysis_service<A: AnalysisService + 'static>(mut self, service: A) -> Self {
        let state = Arc::make_mut(&mut self.state);
        state.analysis_service = Arc::new(service);
        self
    }

    /// Get the router for HTTP server usage
    pub fn router(&self) -> Router {
        self.router.clone()
    }

    /// Process a unified request through the router
    #[instrument(skip_all, fields(
        method = %request.method,
        path = %request.path,
        trace_id = %request.trace_id
    ))]
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
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to build request: {}", e)))?;

        // Process through router
        let response = self
            .router
            .clone()
            .oneshot(axum_request)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Router error: {}", e)))?;

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
            &request_method,
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

    fn record_request_metrics_by_data(
        &self,
        method: &Method,
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
            protocol = %protocol,
            method = %method,
            path = %path,
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

/// Template service trait for dependency injection
#[async_trait::async_trait]
pub trait TemplateService: Send + Sync {
    async fn list_templates(&self, query: &ListTemplatesQuery) -> Result<TemplateList, AppError>;
    async fn get_template(&self, template_id: &str) -> Result<TemplateInfo, AppError>;
    async fn generate_template(
        &self,
        params: &GenerateParams,
    ) -> Result<GeneratedTemplate, AppError>;
}

/// Analysis service trait for dependency injection
#[async_trait::async_trait]
pub trait AnalysisService: Send + Sync {
    async fn analyze_complexity(
        &self,
        params: &ComplexityParams,
    ) -> Result<ComplexityAnalysis, AppError>;
    async fn analyze_churn(&self, params: &ChurnParams) -> Result<ChurnAnalysis, AppError>;
    async fn analyze_dag(&self, params: &DagParams) -> Result<DagAnalysis, AppError>;
    async fn generate_context(&self, params: &ContextParams) -> Result<ProjectContext, AppError>;
    async fn analyze_dead_code(
        &self,
        params: &DeadCodeParams,
    ) -> Result<DeadCodeAnalysis, AppError>;
}

/// Default implementations for testing
#[derive(Default)]
pub struct DefaultTemplateService;

#[async_trait::async_trait]
impl TemplateService for DefaultTemplateService {
    async fn list_templates(&self, _query: &ListTemplatesQuery) -> Result<TemplateList, AppError> {
        Ok(TemplateList {
            templates: vec![TemplateInfo {
                id: "makefile/rust/cli".to_string(),
                name: "Rust CLI Makefile".to_string(),
                description: "Makefile template for Rust CLI projects".to_string(),
                version: "1.0.0".to_string(),
                parameters: vec![],
            }],
            total: 1,
        })
    }

    async fn get_template(&self, template_id: &str) -> Result<TemplateInfo, AppError> {
        if template_id == "makefile/rust/cli" {
            Ok(TemplateInfo {
                id: template_id.to_string(),
                name: "Rust CLI Makefile".to_string(),
                description: "Makefile template for Rust CLI projects".to_string(),
                version: "1.0.0".to_string(),
                parameters: vec![],
            })
        } else {
            Err(AppError::NotFound(format!(
                "Template not found: {}",
                template_id
            )))
        }
    }

    async fn generate_template(
        &self,
        params: &GenerateParams,
    ) -> Result<GeneratedTemplate, AppError> {
        Ok(GeneratedTemplate {
            template_id: params.template_uri.clone(),
            content: format!(
                "# Generated Makefile for {}\n\nall:\n\techo 'Building project'\n",
                params
                    .parameters
                    .get("project_name")
                    .unwrap_or(&Value::String("unknown".to_string()))
            ),
            metadata: TemplateMetadata {
                name: "Generated Makefile".to_string(),
                version: "1.0.0".to_string(),
                generated_at: chrono::Utc::now().to_rfc3339(),
            },
        })
    }
}

#[derive(Default)]
pub struct DefaultAnalysisService;

#[async_trait::async_trait]
impl AnalysisService for DefaultAnalysisService {
    async fn analyze_complexity(
        &self,
        _params: &ComplexityParams,
    ) -> Result<ComplexityAnalysis, AppError> {
        Ok(ComplexityAnalysis {
            summary: ComplexitySummary {
                total_functions: 0,
                average_complexity: 0.0,
                max_complexity: 0,
                files_analyzed: 0,
            },
            files: vec![],
        })
    }

    async fn analyze_churn(&self, _params: &ChurnParams) -> Result<ChurnAnalysis, AppError> {
        Ok(ChurnAnalysis {
            summary: ChurnSummary {
                total_commits: 0,
                files_changed: 0,
                period_days: 0,
            },
            hotspots: vec![],
        })
    }

    async fn analyze_dag(&self, params: &DagParams) -> Result<DagAnalysis, AppError> {
        use crate::cli::DagType;
        use crate::services::dag_builder::DagBuilder;
        use crate::services::mermaid_generator::{MermaidGenerator, MermaidOptions};

        // Parse DAG type from string
        let dag_type = match params.dag_type.as_str() {
            "call-graph" => DagType::CallGraph,
            "import-graph" => DagType::ImportGraph,
            "inheritance" => DagType::Inheritance,
            "full-dependency" => DagType::FullDependency,
            _ => DagType::CallGraph, // Default fallback
        };

        let project_path = std::path::Path::new(&params.project_path);

        // Use context analysis to get project data, then build DAG
        let context = crate::services::context::analyze_project(project_path, "rust")
            .await
            .map_err(|e| AppError::Analysis(format!("Context analysis failed: {}", e)))?;

        // Build dependency graph with edge truncation
        let dependency_graph = DagBuilder::build_from_project(&context);

        // Filter by DAG type
        let filtered_graph = match dag_type {
            DagType::CallGraph => crate::services::dag_builder::filter_call_edges(dependency_graph),
            DagType::ImportGraph => {
                crate::services::dag_builder::filter_import_edges(dependency_graph)
            }
            DagType::Inheritance => {
                crate::services::dag_builder::filter_inheritance_edges(dependency_graph)
            }
            DagType::FullDependency => dependency_graph,
        };

        // Generate Mermaid graph
        let options = MermaidOptions {
            show_complexity: params.show_complexity,
            ..Default::default()
        };
        let mermaid_generator = MermaidGenerator::new(options);
        let graph_string = mermaid_generator.generate(&filtered_graph);

        Ok(DagAnalysis {
            graph: graph_string,
            nodes: filtered_graph.nodes.len(),
            edges: filtered_graph.edges.len(),
            cycles: vec![], // TODO: Implement cycle detection
        })
    }

    async fn generate_context(&self, _params: &ContextParams) -> Result<ProjectContext, AppError> {
        Ok(ProjectContext {
            project_name: "unknown".to_string(),
            toolchain: "unknown".to_string(),
            structure: ProjectStructure {
                directories: vec![],
                files: vec![],
            },
            metrics: ContextMetrics {
                total_files: 0,
                total_lines: 0,
                complexity_score: 0.0,
            },
        })
    }

    async fn analyze_dead_code(
        &self,
        _params: &DeadCodeParams,
    ) -> Result<DeadCodeAnalysis, AppError> {
        Ok(DeadCodeAnalysis {
            summary: DeadCodeSummary {
                total_files_analyzed: 0,
                files_with_dead_code: 0,
                total_dead_lines: 0,
                dead_percentage: 0.0,
            },
            files: vec![],
        })
    }
}

/// Handler modules containing the actual endpoint implementations
pub mod handlers {
    use super::*;

    /// List available templates
    pub async fn list_templates(
        Extension(state): Extension<Arc<AppState>>,
        Query(query): Query<ListTemplatesQuery>,
    ) -> Result<Json<TemplateList>, AppError> {
        let templates = state.template_service.list_templates(&query).await?;
        Ok(Json(templates))
    }

    /// Get a specific template
    pub async fn get_template(
        Extension(state): Extension<Arc<AppState>>,
        Path(template_id): Path<String>,
    ) -> Result<Json<TemplateInfo>, AppError> {
        let template = state.template_service.get_template(&template_id).await?;
        Ok(Json(template))
    }

    /// Generate a template
    pub async fn generate_template(
        Extension(state): Extension<Arc<AppState>>,
        Json(params): Json<GenerateParams>,
    ) -> Result<Json<GeneratedTemplate>, AppError> {
        let result = state.template_service.generate_template(&params).await?;
        Ok(Json(result))
    }

    /// Analyze code complexity (POST)
    pub async fn analyze_complexity(
        Extension(state): Extension<Arc<AppState>>,
        Json(params): Json<ComplexityParams>,
    ) -> Result<Json<ComplexityAnalysis>, AppError> {
        let analysis = state.analysis_service.analyze_complexity(&params).await?;
        Ok(Json(analysis))
    }

    /// Analyze code complexity (GET with query parameters)
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
    pub async fn analyze_churn(
        Extension(state): Extension<Arc<AppState>>,
        Json(params): Json<ChurnParams>,
    ) -> Result<Json<ChurnAnalysis>, AppError> {
        let analysis = state.analysis_service.analyze_churn(&params).await?;
        Ok(Json(analysis))
    }

    /// Analyze dependency graph
    pub async fn analyze_dag(
        Extension(state): Extension<Arc<AppState>>,
        Json(params): Json<DagParams>,
    ) -> Result<Json<DagAnalysis>, AppError> {
        let analysis = state.analysis_service.analyze_dag(&params).await?;
        Ok(Json(analysis))
    }

    /// Generate project context
    pub async fn generate_context(
        Extension(state): Extension<Arc<AppState>>,
        Json(params): Json<ContextParams>,
    ) -> Result<Json<ProjectContext>, AppError> {
        let context = state.analysis_service.generate_context(&params).await?;
        Ok(Json(context))
    }

    /// Analyze dead code
    pub async fn analyze_dead_code(
        Extension(state): Extension<Arc<AppState>>,
        Json(params): Json<DeadCodeParams>,
    ) -> Result<Json<DeadCodeAnalysis>, AppError> {
        let analysis = state.analysis_service.analyze_dead_code(&params).await?;
        Ok(Json(analysis))
    }

    /// Analyze deep context
    pub async fn analyze_deep_context(
        Extension(_state): Extension<Arc<AppState>>,
        Json(params): Json<Value>,
    ) -> Result<Json<Value>, AppError> {
        use crate::services::deep_context::{AnalysisType, DeepContextAnalyzer, DeepContextConfig};
        use std::path::PathBuf;

        // Parse parameters from JSON
        let project_path = params
            .get("project_path")
            .and_then(|v| v.as_str())
            .unwrap_or(".")
            .parse::<PathBuf>()
            .map_err(|e| AppError::BadRequest(format!("Invalid project_path: {}", e)))?;

        let period_days = params
            .get("period_days")
            .and_then(|v| v.as_u64())
            .unwrap_or(30) as u32;

        let parallel = params
            .get("parallel")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        // Build configuration from HTTP params
        let mut config = DeepContextConfig {
            period_days,
            ..DeepContextConfig::default()
        };

        if let Some(p) = parallel {
            config.parallel = p;
        }

        // Parse include/exclude filters
        if let Some(include) = params.get("include").and_then(|v| v.as_array()) {
            config.include_analyses = include
                .iter()
                .filter_map(|v| v.as_str())
                .filter_map(|s| match s {
                    "ast" => Some(AnalysisType::Ast),
                    "complexity" => Some(AnalysisType::Complexity),
                    "churn" => Some(AnalysisType::Churn),
                    "dag" => Some(AnalysisType::Dag),
                    "dead-code" => Some(AnalysisType::DeadCode),
                    "satd" => Some(AnalysisType::Satd),
                    "tdg" => Some(AnalysisType::TechnicalDebtGradient),
                    _ => None,
                })
                .collect();
        }

        // Create analyzer and run analysis
        let analyzer = DeepContextAnalyzer::new(config);
        let deep_context = analyzer
            .analyze_project(&project_path)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

        // Return JSON response
        Ok(Json(
            serde_json::to_value(&deep_context)
                .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?,
        ))
    }

    /// MCP protocol endpoint
    pub async fn mcp_endpoint(
        Extension(state): Extension<Arc<AppState>>,
        Path(method): Path<String>,
        Json(params): Json<Value>,
    ) -> Result<Json<Value>, AppError> {
        set_protocol_context(Protocol::Mcp);

        // Route MCP method to appropriate handler
        match method.as_str() {
            "list_templates" => {
                let query: ListTemplatesQuery = serde_json::from_value(params)?;
                let result = state.template_service.list_templates(&query).await?;
                Ok(Json(serde_json::to_value(result)?))
            }
            "generate_template" => {
                let generate_params: GenerateParams = serde_json::from_value(params)?;
                let result = state
                    .template_service
                    .generate_template(&generate_params)
                    .await?;
                Ok(Json(serde_json::to_value(result)?))
            }
            "analyze_complexity" => {
                let complexity_params: ComplexityParams = serde_json::from_value(params)?;
                let result = state
                    .analysis_service
                    .analyze_complexity(&complexity_params)
                    .await?;
                Ok(Json(serde_json::to_value(result)?))
            }
            "analyze_dead_code" => {
                let dead_code_params: DeadCodeParams = serde_json::from_value(params)?;
                let result = state
                    .analysis_service
                    .analyze_dead_code(&dead_code_params)
                    .await?;
                Ok(Json(serde_json::to_value(result)?))
            }
            _ => Err(AppError::NotFound(format!(
                "Unknown MCP method: {}",
                method
            ))),
        }
    }

    /// Health check endpoint
    pub async fn health_check() -> impl IntoResponse {
        Json(serde_json::json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "version": env!("CARGO_PKG_VERSION")
        }))
    }

    /// Metrics endpoint
    pub async fn metrics(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
        let requests = state.metrics.requests_total.lock().clone();
        let errors = state.metrics.errors_total.lock().clone();

        Json(serde_json::json!({
            "requests_total": requests,
            "errors_total": errors,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}

// Data structures for API requests and responses
#[derive(Debug, Deserialize)]
pub struct ListTemplatesQuery {
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TemplateList {
    pub templates: Vec<TemplateInfo>,
    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub parameters: Vec<TemplateParameter>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateParams {
    pub template_uri: String,
    pub parameters: HashMap<String, Value>,
}

#[derive(Debug, Serialize)]
pub struct GeneratedTemplate {
    pub template_id: String,
    pub content: String,
    pub metadata: TemplateMetadata,
}

#[derive(Debug, Serialize)]
pub struct TemplateMetadata {
    pub name: String,
    pub version: String,
    pub generated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ComplexityParams {
    pub project_path: String,
    pub toolchain: String,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub max_cyclomatic: Option<u32>,
    #[serde(default)]
    pub max_cognitive: Option<u32>,
    #[serde(default)]
    pub top_files: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ComplexityQueryParams {
    #[serde(default)]
    pub project_path: Option<String>,
    #[serde(default)]
    pub toolchain: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub max_cyclomatic: Option<u32>,
    #[serde(default)]
    pub max_cognitive: Option<u32>,
    #[serde(default)]
    pub top_files: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct ComplexityAnalysis {
    pub summary: ComplexitySummary,
    pub files: Vec<FileComplexity>,
}

#[derive(Debug, Serialize)]
pub struct ComplexitySummary {
    pub total_functions: usize,
    pub average_complexity: f64,
    pub max_complexity: u32,
    pub files_analyzed: usize,
}

#[derive(Debug, Serialize)]
pub struct FileComplexity {
    pub path: String,
    pub functions: Vec<FunctionComplexity>,
}

#[derive(Debug, Serialize)]
pub struct FunctionComplexity {
    pub name: String,
    pub cyclomatic: u32,
    pub cognitive: u32,
    pub line_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct ChurnParams {
    pub project_path: String,
    #[serde(default)]
    pub period_days: u32,
    #[serde(default)]
    pub format: String,
}

#[derive(Debug, Serialize)]
pub struct ChurnAnalysis {
    pub summary: ChurnSummary,
    pub hotspots: Vec<ChurnHotspot>,
}

#[derive(Debug, Serialize)]
pub struct ChurnSummary {
    pub total_commits: usize,
    pub files_changed: usize,
    pub period_days: u32,
}

#[derive(Debug, Serialize)]
pub struct ChurnHotspot {
    pub file: String,
    pub changes: u32,
    pub authors: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DagParams {
    pub project_path: String,
    #[serde(default)]
    pub dag_type: String,
    #[serde(default)]
    pub show_complexity: bool,
    #[serde(default)]
    pub format: String,
}

#[derive(Debug, Serialize)]
pub struct DagAnalysis {
    pub graph: String,
    pub nodes: usize,
    pub edges: usize,
    pub cycles: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ContextParams {
    pub toolchain: String,
    pub project_path: String,
    #[serde(default)]
    pub format: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectContext {
    pub project_name: String,
    pub toolchain: String,
    pub structure: ProjectStructure,
    pub metrics: ContextMetrics,
}

#[derive(Debug, Serialize)]
pub struct ProjectStructure {
    pub directories: Vec<String>,
    pub files: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ContextMetrics {
    pub total_files: usize,
    pub total_lines: usize,
    pub complexity_score: f64,
}

#[derive(Debug, Deserialize)]
pub struct DeadCodeParams {
    pub project_path: String,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub top_files: Option<usize>,
    #[serde(default)]
    pub include_unreachable: bool,
    #[serde(default)]
    pub min_dead_lines: usize,
    #[serde(default)]
    pub include_tests: bool,
}

#[derive(Debug, Serialize)]
pub struct DeadCodeAnalysis {
    pub summary: DeadCodeSummary,
    pub files: Vec<FileDeadCode>,
}

#[derive(Debug, Serialize)]
pub struct DeadCodeSummary {
    pub total_files_analyzed: usize,
    pub files_with_dead_code: usize,
    pub total_dead_lines: usize,
    pub dead_percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct FileDeadCode {
    pub path: String,
    pub dead_lines: usize,
    pub dead_percentage: f64,
    pub dead_functions: usize,
    pub dead_classes: usize,
    pub confidence: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unified_service_creation() {
        let service = UnifiedService::new();
        assert!(Arc::strong_count(&service.state) >= 1);
    }

    #[tokio::test]
    async fn test_default_template_service() {
        let service = DefaultTemplateService;
        let query = ListTemplatesQuery {
            format: None,
            category: None,
        };

        let result = service.list_templates(&query).await.unwrap();
        assert_eq!(result.total, 1);
        assert_eq!(result.templates[0].id, "makefile/rust/cli");
    }

    #[tokio::test]
    async fn test_template_generation() {
        let service = DefaultTemplateService;
        let mut params = HashMap::new();
        params.insert(
            "project_name".to_string(),
            Value::String("test-project".to_string()),
        );

        let generate_params = GenerateParams {
            template_uri: "makefile/rust/cli".to_string(),
            parameters: params,
        };

        let result = service.generate_template(&generate_params).await.unwrap();
        assert!(result.content.contains("test-project"));
    }
}
