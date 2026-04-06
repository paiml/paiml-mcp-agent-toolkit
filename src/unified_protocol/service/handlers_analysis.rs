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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
