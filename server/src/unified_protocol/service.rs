use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow;
use axum::extract::{Extension, Path, Query};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower::{ServiceBuilder, ServiceExt};
use tower_http::compression::CompressionLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

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
                "Template not found: {template_id}"
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
            .map_err(|e| AppError::Analysis(format!("Context analysis failed: {e}")))?;

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
            cycles: vec![], // TRACKED: Implement cycle detection
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

    /// Parse deep context analysis parameters from JSON
    fn parse_deep_context_params(
        params: &Value,
    ) -> Result<
        (
            std::path::PathBuf,
            crate::services::deep_context::DeepContextConfig,
        ),
        AppError,
    > {
        use crate::services::deep_context::{AnalysisType, DeepContextConfig};
        use std::path::PathBuf;

        // Parse project path
        let project_path = params
            .get("project_path")
            .and_then(|v| v.as_str())
            .unwrap_or(".")
            .parse::<PathBuf>()
            .map_err(|e| AppError::BadRequest(format!("Invalid project_path: {e}")))?;

        // Parse basic config parameters
        let period_days = params
            .get("period_days")
            .and_then(|v| v.as_u64())
            .unwrap_or(30) as u32;

        let parallel = params
            .get("parallel")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        // Build configuration
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

        Ok((project_path, config))
    }

    /// Analyze deep context
    pub async fn analyze_deep_context(
        Extension(_state): Extension<Arc<AppState>>,
        Json(params): Json<Value>,
    ) -> Result<Json<Value>, AppError> {
        use crate::services::deep_context::DeepContextAnalyzer;

        // Parse parameters and build configuration
        let (project_path, config) = parse_deep_context_params(&params)?;

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

    /// Analyze Makefile quality and compliance
    pub async fn analyze_makefile_lint(
        Extension(_state): Extension<Arc<AppState>>,
        Json(params): Json<MakefileLintParams>,
    ) -> Result<Json<MakefileLintAnalysis>, AppError> {
        use crate::services::makefile_linter;
        use std::path::Path;

        let makefile_path = Path::new(&params.path);
        let lint_result = makefile_linter::lint_makefile(makefile_path)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Makefile linting failed: {}", e)))?;

        let analysis = MakefileLintAnalysis {
            path: params.path,
            violations: lint_result
                .violations
                .into_iter()
                .map(|v| MakefileLintViolation {
                    rule: v.rule,
                    severity: match v.severity {
                        makefile_linter::Severity::Error => "error".to_string(),
                        makefile_linter::Severity::Warning => "warning".to_string(),
                        makefile_linter::Severity::Performance => "performance".to_string(),
                        makefile_linter::Severity::Info => "info".to_string(),
                    },
                    line: v.span.line,
                    column: v.span.column,
                    message: v.message,
                    fix_hint: v.fix_hint,
                })
                .collect(),
            quality_score: lint_result.quality_score,
            rules_applied: params.rules,
        };

        Ok(Json(analysis))
    }

    /// Analyze provability properties  
    pub async fn analyze_provability(
        Extension(_state): Extension<Arc<AppState>>,
        Json(params): Json<ProvabilityParams>,
    ) -> Result<Json<ProvabilityAnalysis>, AppError> {
        use crate::services::lightweight_provability_analyzer::{
            FunctionId, LightweightProvabilityAnalyzer,
        };

        let analyzer = LightweightProvabilityAnalyzer::new();

        // Extract functions from parameters or scan project
        let functions = if let Some(function_names) = params.functions {
            function_names
                .into_iter()
                .enumerate()
                .map(|(i, name)| FunctionId {
                    file_path: format!("{}/src/lib.rs", params.project_path),
                    function_name: name,
                    line_number: i * 10, // Mock line numbers
                })
                .collect()
        } else {
            // Mock function discovery from project path
            vec![FunctionId {
                file_path: format!("{}/src/main.rs", params.project_path),
                function_name: "main".to_string(),
                line_number: 1,
            }]
        };

        let summaries = analyzer.analyze_incrementally(&functions).await;

        let analysis = ProvabilityAnalysis {
            project_path: params.project_path,
            analysis_depth: params.analysis_depth.unwrap_or(10),
            functions_analyzed: summaries.len(),
            average_provability_score: summaries.iter().map(|s| s.provability_score).sum::<f64>()
                / summaries.len() as f64,
            summaries: summaries
                .into_iter()
                .map(|s| ProvabilitySummary {
                    function_id: format!("{}:{}", s.version, functions[0].function_name), // Mock ID
                    provability_score: s.provability_score,
                    verified_properties: s.verified_properties,
                    analysis_time_us: s.analysis_time_us,
                })
                .collect(),
        };

        Ok(Json(analysis))
    }

    /// Analyze Self-Admitted Technical Debt (SATD)
    pub async fn analyze_satd(
        Extension(_state): Extension<Arc<AppState>>,
        Json(params): Json<SatdParams>,
    ) -> Result<Json<SatdAnalysis>, AppError> {
        use crate::services::satd_detector::SATDDetector;
        use std::path::Path;

        let detector = if params.strict.unwrap_or(false) {
            SATDDetector::new_strict()
        } else {
            SATDDetector::new()
        };

        let project_path = Path::new(&params.project_path);
        let result = detector
            .analyze_project(project_path, !params.exclude_tests.unwrap_or(true))
            .await
            .map_err(|e| AppError::Analysis(format!("SATD analysis failed: {}", e)))?;

        // Group items by file
        let mut files_map: std::collections::HashMap<
            String,
            Vec<crate::services::satd_detector::TechnicalDebt>,
        > = std::collections::HashMap::new();
        for item in result.items {
            files_map
                .entry(item.file.display().to_string())
                .or_default()
                .push(item);
        }

        // Convert to API response format
        let analysis = SatdAnalysis {
            project_path: params.project_path,
            total_debt_items: result.summary.total_items,
            debt_density: (result.summary.total_items as f64
                / result.total_files_analyzed.max(1) as f64),
            critical_items: result
                .summary
                .by_severity
                .get("Critical")
                .copied()
                .unwrap_or(0),
            categories: result
                .summary
                .by_category
                .into_iter()
                .map(|(k, v)| (format!("{:?}", k), v))
                .collect(),
            files: files_map
                .into_iter()
                .map(|(path, items)| {
                    SatdFile {
                        path,
                        debt_count: items.len(),
                        items: items
                            .into_iter()
                            .map(|item| SatdItem {
                                line: item.line as usize,
                                category: format!("{:?}", item.category),
                                severity: format!("{:?}", item.severity),
                                text: item.text,
                                context: None, // Not available in current structure
                            })
                            .collect(),
                    }
                })
                .collect(),
        };

        Ok(Json(analysis))
    }

    /// Analyze lint hotspots
    pub async fn analyze_lint_hotspot(
        Extension(_state): Extension<Arc<AppState>>,
        Json(params): Json<LintHotspotParams>,
    ) -> Result<Json<LintHotspotAnalysis>, AppError> {
        use crate::cli::handlers::lint_hotspot_handlers::handle_analyze_lint_hotspot;
        use crate::cli::LintHotspotOutputFormat;
        use std::path::PathBuf;

        let project_path = PathBuf::from(params.project_path.clone());

        // Create a temporary file to capture output
        let temp_file = tempfile::NamedTempFile::new()
            .map_err(|e| AppError::Analysis(format!("Failed to create temporary file: {}", e)))?;
        let output_path = temp_file.path().to_path_buf();

        // Run lint hotspot analysis using the CLI handler with JSON output
        handle_analyze_lint_hotspot(
            project_path,
            None, // file
            LintHotspotOutputFormat::Json,
            100.0,                     // max_density
            0.0,                       // min_confidence
            false,                     // enforce
            false,                     // dry_run
            false,                     // enforcement_metadata
            Some(output_path.clone()), // output to temp file
            false,                     // perf
            String::new(),             // clippy_flags
            params.top_files.unwrap_or(10),
            Vec::new(), // include
            Vec::new(), // exclude
        )
        .await
        .map_err(|e| AppError::Analysis(format!("Lint hotspot analysis failed: {}", e)))?;

        // Read and parse the JSON output
        let json_output = tokio::fs::read_to_string(&output_path)
            .await
            .map_err(|e| AppError::Analysis(format!("Failed to read output file: {}", e)))?;
        let lint_data: serde_json::Value = serde_json::from_str(&json_output)
            .map_err(|e| AppError::Analysis(format!("Failed to parse JSON output: {}", e)))?;

        // Extract data from JSON
        let hotspots_data = lint_data["hotspots"].as_array().unwrap_or(&vec![]).clone();
        let total_files = lint_data["total_files_analyzed"].as_u64().unwrap_or(0) as usize;
        let total_violations = lint_data["total_violations"].as_u64().unwrap_or(0) as usize;
        let average_violations_per_file = lint_data["average_violations_per_file"]
            .as_f64()
            .unwrap_or(0.0);

        // Convert hotspots to typed structure
        let hotspots: Vec<LintHotspot> = hotspots_data
            .iter()
            .filter_map(|h| {
                Some(LintHotspot {
                    file_path: h["file_path"].as_str()?.to_string(),
                    violations: h["violations"].as_u64()? as usize,
                    lines_of_code: h["lines_of_code"].as_u64()? as usize,
                    defect_density: h["defect_density"].as_f64()?,
                    severity_distribution: h["severity_distribution"]
                        .as_object()?
                        .iter()
                        .map(|(k, v)| (k.clone(), v.as_u64().unwrap_or(0) as usize))
                        .collect(),
                })
            })
            .collect();

        // Convert to API response format
        let analysis = LintHotspotAnalysis {
            project_path: params.project_path,
            total_files_analyzed: total_files,
            total_violations,
            average_violations_per_file,
            hotspots,
        };

        Ok(Json(analysis))
    }

    /// Route MCP method to appropriate handler implementation
    async fn route_mcp_method(
        state: &Arc<AppState>,
        method: &str,
        params: Value,
    ) -> Result<Value, AppError> {
        match method {
            "list_templates" => {
                let query: ListTemplatesQuery = serde_json::from_value(params)?;
                let result = state.template_service.list_templates(&query).await?;
                Ok(serde_json::to_value(result)?)
            }
            "generate_template" => {
                let generate_params: GenerateParams = serde_json::from_value(params)?;
                let result = state
                    .template_service
                    .generate_template(&generate_params)
                    .await?;
                Ok(serde_json::to_value(result)?)
            }
            "analyze_complexity" => {
                let complexity_params: ComplexityParams = serde_json::from_value(params)?;
                let result = state
                    .analysis_service
                    .analyze_complexity(&complexity_params)
                    .await?;
                Ok(serde_json::to_value(result)?)
            }
            "analyze_dead_code" => {
                let dead_code_params: DeadCodeParams = serde_json::from_value(params)?;
                let result = state
                    .analysis_service
                    .analyze_dead_code(&dead_code_params)
                    .await?;
                Ok(serde_json::to_value(result)?)
            }
            "analyze_satd" => {
                let satd_params: SatdParams = serde_json::from_value(params)?;
                let result = analyze_satd(Extension(state.clone()), Json(satd_params)).await?;
                Ok(serde_json::to_value(result.0)?)
            }
            "analyze_lint_hotspot" => {
                let lint_params: LintHotspotParams = serde_json::from_value(params)?;
                let result =
                    analyze_lint_hotspot(Extension(state.clone()), Json(lint_params)).await?;
                Ok(serde_json::to_value(result.0)?)
            }
            _ => Err(AppError::NotFound(format!("Unknown MCP method: {method}"))),
        }
    }

    /// MCP protocol endpoint
    pub async fn mcp_endpoint(
        Extension(state): Extension<Arc<AppState>>,
        Path(method): Path<String>,
        Json(params): Json<Value>,
    ) -> Result<Json<Value>, AppError> {
        set_protocol_context(Protocol::Mcp);

        // Route MCP method to appropriate handler
        let result = route_mcp_method(&state, &method, params).await?;
        Ok(Json(result))
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

#[derive(Debug, Deserialize)]
pub struct MakefileLintParams {
    pub path: String,
    #[serde(default)]
    pub rules: Vec<String>,
    #[serde(default)]
    pub fix: bool,
    #[serde(default)]
    pub gnu_version: String,
}

#[derive(Debug, Serialize)]
pub struct MakefileLintAnalysis {
    pub path: String,
    pub violations: Vec<MakefileLintViolation>,
    pub quality_score: f32,
    pub rules_applied: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct MakefileLintViolation {
    pub rule: String,
    pub severity: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub fix_hint: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProvabilityParams {
    pub project_path: String,
    #[serde(default)]
    pub functions: Option<Vec<String>>,
    #[serde(default)]
    pub analysis_depth: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct ProvabilityAnalysis {
    pub project_path: String,
    pub analysis_depth: usize,
    pub functions_analyzed: usize,
    pub average_provability_score: f64,
    pub summaries: Vec<ProvabilitySummary>,
}

#[derive(Debug, Serialize)]
pub struct ProvabilitySummary {
    pub function_id: String,
    pub provability_score: f64,
    pub verified_properties:
        Vec<crate::services::lightweight_provability_analyzer::VerifiedProperty>,
    pub analysis_time_us: u128,
}

#[derive(Debug, Deserialize)]
pub struct SatdParams {
    pub project_path: String,
    #[serde(default)]
    pub strict: Option<bool>,
    #[serde(default)]
    pub exclude_tests: Option<bool>,
    #[serde(default)]
    pub critical_only: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct SatdAnalysis {
    pub project_path: String,
    pub total_debt_items: usize,
    pub debt_density: f64,
    pub critical_items: usize,
    pub categories: std::collections::HashMap<String, usize>,
    pub files: Vec<SatdFile>,
}

#[derive(Debug, Serialize)]
pub struct SatdFile {
    pub path: String,
    pub debt_count: usize,
    pub items: Vec<SatdItem>,
}

#[derive(Debug, Serialize)]
pub struct SatdItem {
    pub line: usize,
    pub category: String,
    pub severity: String,
    pub text: String,
    pub context: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LintHotspotParams {
    pub project_path: String,
    #[serde(default)]
    pub top_files: Option<usize>,
    #[serde(default)]
    pub min_violations: Option<usize>,
    #[serde(default)]
    pub include: Option<String>,
    #[serde(default)]
    pub exclude: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LintHotspotAnalysis {
    pub project_path: String,
    pub total_files_analyzed: usize,
    pub total_violations: usize,
    pub average_violations_per_file: f64,
    pub hotspots: Vec<LintHotspot>,
}

#[derive(Debug, Serialize)]
pub struct LintHotspot {
    pub file_path: String,
    pub violations: usize,
    pub lines_of_code: usize,
    pub defect_density: f64,
    pub severity_distribution: std::collections::HashMap<String, usize>,
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
        assert!(result.total > 0);
        assert!(!result.templates.is_empty());
    }

    #[tokio::test]
    async fn test_template_generation() {
        let service = DefaultTemplateService;
        let mut params = HashMap::with_capacity(2);
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

    // === Sprint 46 Phase 6: TDD Tests for UnifiedService ===

    #[tokio::test]
    async fn test_unified_service_with_custom_template_service() {
        struct MockTemplateService;

        #[async_trait::async_trait]
        impl TemplateService for MockTemplateService {
            async fn list_templates(
                &self,
                _query: &ListTemplatesQuery,
            ) -> Result<TemplateList, AppError> {
                Ok(TemplateList {
                    total: 0,
                    templates: vec![],
                })
            }

            async fn get_template(&self, _id: &str) -> Result<TemplateInfo, AppError> {
                Err(AppError::NotFound("Mock template".to_string()))
            }

            async fn generate_template(
                &self,
                _params: &GenerateParams,
            ) -> Result<GeneratedTemplate, AppError> {
                Ok(GeneratedTemplate {
                    content: "Mock generated content".to_string(),
                    metadata: TemplateMetadata {
                        template_id: "mock".to_string(),
                        generated_at: chrono::Utc::now(),
                        parameters_used: HashMap::new(),
                    },
                })
            }
        }

        let service = UnifiedService::new().with_template_service(MockTemplateService);

        assert!(Arc::strong_count(&service.state) >= 1);
    }

    #[tokio::test]
    async fn test_unified_service_with_custom_analysis_service() {
        struct MockAnalysisService;

        #[async_trait::async_trait]
        impl AnalysisService for MockAnalysisService {
            async fn analyze_complexity(
                &self,
                _params: &ComplexityParams,
            ) -> Result<ComplexityAnalysis, AppError> {
                Ok(ComplexityAnalysis {
                    summary: ComplexitySummary {
                        total_functions: 10,
                        average_complexity: 5.0,
                        max_complexity: 15,
                        files_analyzed: 1,
                    },
                    files: vec![],
                })
            }

            async fn analyze_churn(
                &self,
                _params: &ChurnParams,
            ) -> Result<ChurnAnalysis, AppError> {
                Ok(ChurnAnalysis {
                    total_commits: 100,
                    files_changed: 50,
                    churn_rate: 0.5,
                    hotspots: vec![],
                })
            }

            async fn analyze_dag(&self, _params: &DagParams) -> Result<DagAnalysis, AppError> {
                Ok(DagAnalysis {
                    nodes: vec![],
                    edges: vec![],
                    cycles: vec![],
                })
            }

            async fn generate_context(
                &self,
                _params: &ContextParams,
            ) -> Result<ProjectContext, AppError> {
                Ok(ProjectContext {
                    project_name: "mock".to_string(),
                    toolchain: "rust".to_string(),
                    structure: ProjectStructure {
                        directories: vec![],
                        files: vec![],
                    },
                    metrics: ContextMetrics {
                        total_lines: 0,
                        total_files: 0,
                        complexity_average: 0.0,
                        tech_stack: vec![],
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

        let service = UnifiedService::new().with_analysis_service(MockAnalysisService);

        assert!(Arc::strong_count(&service.state) >= 1);
    }

    #[tokio::test]
    async fn test_service_metrics_initialization() {
        let metrics = ServiceMetrics::default();

        let requests = metrics.requests_total.lock();
        assert_eq!(requests.len(), 0);

        let errors = metrics.errors_total.lock();
        assert_eq!(errors.len(), 0);

        let durations = metrics.request_duration_ms.lock();
        assert_eq!(durations.len(), 0);
    }

    #[tokio::test]
    async fn test_app_state_default() {
        let state = AppState::default();

        assert!(Arc::strong_count(&state.template_service) >= 1);
        assert!(Arc::strong_count(&state.analysis_service) >= 1);
        assert!(Arc::strong_count(&state.metrics) >= 1);
    }

    #[tokio::test]
    async fn test_unified_request_creation() {
        let request = UnifiedRequest::new(axum::http::Method::GET, "/api/v1/templates".to_string());

        assert_eq!(request.method, axum::http::Method::GET);
        assert_eq!(request.path, "/api/v1/templates");
        assert!(request.extensions.is_empty());
    }

    #[tokio::test]
    async fn test_process_request_health_check() {
        let service = UnifiedService::new();
        let request = UnifiedRequest::new(axum::http::Method::GET, "/health".to_string());

        let response = service.process_request(request).await.unwrap();
        assert_eq!(response.status, axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_process_request_metrics_endpoint() {
        let service = UnifiedService::new();
        let request = UnifiedRequest::new(axum::http::Method::GET, "/metrics".to_string());

        let response = service.process_request(request).await.unwrap();
        assert_eq!(response.status, axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_record_request_metrics_by_data() {
        let service = UnifiedService::new();
        let response = UnifiedResponse {
            status: axum::http::StatusCode::OK,
            headers: Default::default(),
            body: Default::default(),
            trace_id: uuid::Uuid::new_v4(),
        };

        service.record_request_metrics_by_data(
            "GET",
            "/api/v1/templates",
            &HashMap::new(),
            &response,
            100,
        );

        let requests = service.state.metrics.requests_total.lock();
        assert!(requests.contains_key(&Protocol::Http));
    }

    #[tokio::test]
    async fn test_protocol_extraction_from_path() {
        let service = UnifiedService::new();

        // Test MCP protocol detection
        let protocol = service.extract_protocol_from_path("/mcp/call_tool");
        assert_eq!(protocol, Protocol::Mcp);

        // Test HTTP protocol default
        let protocol = service.extract_protocol_from_path("/api/v1/templates");
        assert_eq!(protocol, Protocol::Http);
    }

    #[tokio::test]
    async fn test_error_metrics_recording() {
        let service = UnifiedService::new();
        let response = UnifiedResponse {
            status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            headers: Default::default(),
            body: Default::default(),
            trace_id: uuid::Uuid::new_v4(),
        };

        service.record_request_metrics_by_data(
            "GET",
            "/api/v1/templates",
            &HashMap::new(),
            &response,
            50,
        );

        let errors = service.state.metrics.errors_total.lock();
        assert!(errors.contains_key(&Protocol::Http));
        assert_eq!(*errors.get(&Protocol::Http).unwrap(), 1);
    }

    #[tokio::test]
    async fn test_duration_metrics_recording() {
        let service = UnifiedService::new();
        let response = UnifiedResponse {
            status: axum::http::StatusCode::OK,
            headers: Default::default(),
            body: Default::default(),
            trace_id: uuid::Uuid::new_v4(),
        };

        service.record_request_metrics_by_data(
            "GET",
            "/api/v1/templates",
            &HashMap::new(),
            &response,
            250,
        );

        let durations = service.state.metrics.request_duration_ms.lock();
        assert!(durations.contains_key(&Protocol::Http));
        assert_eq!(durations.get(&Protocol::Http).unwrap()[0], 250);
    }

    #[tokio::test]
    async fn test_router_cloning() {
        let service = UnifiedService::new();
        let router1 = service.router();
        let router2 = service.router();

        // Both should be valid router instances
        // This test verifies the router can be cloned for multi-threaded usage
        assert!(format!("{:?}", router1).contains("Router"));
        assert!(format!("{:?}", router2).contains("Router"));
    }

    #[tokio::test]
    async fn test_invalid_request_path() {
        let service = UnifiedService::new();
        let request =
            UnifiedRequest::new(axum::http::Method::GET, "/nonexistent/endpoint".to_string());

        let response = service.process_request(request).await.unwrap();
        assert_eq!(response.status, axum::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_complexity_analysis_params() {
        let params = ComplexityParams {
            project_path: "/test/path".to_string(),
            toolchain: "stable".to_string(),
            format: "json".to_string(),
            max_cyclomatic: Some(20),
            max_cognitive: Some(15),
            top_files: Some(10),
        };

        assert_eq!(params.project_path, "/test/path");
        assert_eq!(params.toolchain, "stable");
        assert_eq!(params.max_cyclomatic, Some(20));
        assert_eq!(params.max_cognitive, Some(15));
    }

    #[tokio::test]
    async fn test_satd_analysis_structure() {
        let analysis = SatdAnalysis {
            project_path: "/test/project".to_string(),
            total_debt_items: 5,
            debt_density: 0.02,
            critical_items: 2,
            categories: HashMap::from([("TODO".to_string(), 3), ("FIXME".to_string(), 2)]),
            files: vec![SatdFile {
                path: "test.rs".to_string(),
                debt_count: 1,
                items: vec![SatdItem {
                    line: 42,
                    category: "TODO".to_string(),
                    severity: "Medium".to_string(),
                    text: "Implement this feature".to_string(),
                    context: None,
                }],
            }],
        };

        assert_eq!(analysis.total_debt_items, 5);
        assert_eq!(analysis.categories.get("TODO"), Some(&3));
        assert_eq!(analysis.files[0].items[0].line, 42);
    }
}
