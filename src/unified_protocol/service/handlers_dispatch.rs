/// Analyze lint hotspots
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    debug_assert!(!method.is_empty(), "method must not be empty");
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
