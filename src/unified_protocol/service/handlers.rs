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
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(30) as u32;

    let parallel = params
        .get("parallel")
        .and_then(serde_json::Value::as_u64)
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
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Makefile linting failed: {e}")))?;

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
        .map_err(|e| AppError::Analysis(format!("SATD analysis failed: {e}")))?;

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
            .map(|(k, v)| (format!("{k:?}"), v))
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
        .map_err(|e| AppError::Analysis(format!("Failed to create temporary file: {e}")))?;
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
    .map_err(|e| AppError::Analysis(format!("Lint hotspot analysis failed: {e}")))?;

    // Read and parse the JSON output
    let json_output = tokio::fs::read_to_string(&output_path)
        .await
        .map_err(|e| AppError::Analysis(format!("Failed to read output file: {e}")))?;
    let lint_data: serde_json::Value = serde_json::from_str(&json_output)
        .map_err(|e| AppError::Analysis(format!("Failed to parse JSON output: {e}")))?;

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
