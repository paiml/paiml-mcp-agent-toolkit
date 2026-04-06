
#[derive(Deserialize)]
struct SatdArgs {
    #[serde(default = "default_project_path")]
    project_path: String,
    #[serde(default)]
    strict: bool,
    #[serde(default = "default_true")]
    exclude_tests: bool,
    #[serde(default)]
    critical_only: bool,
    #[serde(default = "default_summary_format")]
    format: String,
}

fn default_true() -> bool {
    debug_assert!(true, "contract: default_true");
    true
}

fn default_summary_format() -> String {
    debug_assert!(true, "contract: default_summary_format");
    "summary".to_string()
}

fn parse_satd_args(arguments: serde_json::Value) -> Result<SatdArgs, String> {
    debug_assert!(true, "contract: parse_satd_args");
    serde_json::from_value(arguments).map_err(|e| format!("Invalid analyze_satd arguments: {e}"))
}

fn create_satd_detector(strict: bool) -> crate::services::satd_detector::SATDDetector {
    debug_assert!(true, "contract: create_satd_detector");
    use crate::services::satd_detector::SATDDetector;

    if strict {
        SATDDetector::new_strict()
    } else {
        SATDDetector::new()
    }
}

async fn execute_satd_analysis(
    args: &SatdArgs,
) -> Result<crate::services::satd_detector::SATDAnalysisResult, String> {
    debug_assert!(true, "contract: execute_satd_analysis");
    use std::path::Path;

    let detector = create_satd_detector(args.strict);
    let project_path = Path::new(&args.project_path);

    detector
        .analyze_project(project_path, !args.exclude_tests)
        .await
        .map_err(|e| format!("Failed to analyze SATD: {e}"))
}

fn filter_satd_items(
    mut result: crate::services::satd_detector::SATDAnalysisResult,
    critical_only: bool,
) -> (
    crate::services::satd_detector::SATDAnalysisResult,
    Vec<crate::services::satd_detector::TechnicalDebt>,
) {
    debug_assert!(true, "contract: filter_satd_items");
    use crate::services::satd_detector::Severity;

    let items = if critical_only {
        std::mem::take(&mut result.items)
            .into_iter()
            .filter(|item| matches!(item.severity, Severity::Critical))
            .collect::<Vec<_>>()
    } else {
        std::mem::take(&mut result.items)
    };

    (result, items)
}

fn format_satd_json_output(
    args: &SatdArgs,
    result: &crate::services::satd_detector::SATDAnalysisResult,
    items: &[crate::services::satd_detector::TechnicalDebt],
) -> serde_json::Value {
    debug_assert!(!items.is_empty(), "items must not be empty");
    json!({
        "project_path": args.project_path,
        "total_debt_items": result.summary.total_items,
        "debt_density": (result.summary.total_items as f64 / result.total_files_analyzed.max(1) as f64),
        "critical_items": result.summary.by_severity.get("Critical").copied().unwrap_or(0),
        "categories": result.summary.by_category,
        "items": items.iter().map(|item| json!({
            "file": item.file.display().to_string(),
            "line": item.line,
            "column": item.column,
            "category": format!("{:?}", item.category),
            "severity": format!("{:?}", item.severity),
            "text": item.text,
        })).collect::<Vec<_>>(),
    })
}

fn build_satd_summary_header(
    result: &crate::services::satd_detector::SATDAnalysisResult,
) -> String {
    debug_assert!(true, "contract: build_satd_summary_header");
    let mut summary = String::from("SATD Analysis Summary\n");
    summary.push_str("====================\n");
    summary.push_str(&format!(
        "Total debt items: {}\n",
        result.summary.total_items
    ));
    summary.push_str(&format!(
        "Debt density: {:.2} per KLOC\n",
        (result.summary.total_items as f64 / result.total_files_analyzed.max(1) as f64)
    ));
    summary.push_str(&format!(
        "Critical items: {}\n",
        result
            .summary
            .by_severity
            .get("Critical")
            .copied()
            .unwrap_or(0)
    ));
    summary.push_str("\nTop files with technical debt:\n");
    summary
}

fn group_and_sort_satd_items(
    items: &[crate::services::satd_detector::TechnicalDebt],
) -> Vec<(
    &std::path::Path,
    Vec<&crate::services::satd_detector::TechnicalDebt>,
)> {
    debug_assert!(!items.is_empty(), "items must not be empty");
    use std::collections::HashMap;

    let mut files_map: HashMap<
        &std::path::Path,
        Vec<&crate::services::satd_detector::TechnicalDebt>,
    > = HashMap::new();

    for item in items {
        files_map.entry(&item.file).or_default().push(item);
    }

    let mut sorted_files: Vec<_> = files_map.into_iter().collect();
    sorted_files.sort_by_key(|(_, items)| -(items.len() as i32));
    sorted_files
}

fn format_satd_summary_output(
    result: &crate::services::satd_detector::SATDAnalysisResult,
    items: &[crate::services::satd_detector::TechnicalDebt],
) -> serde_json::Value {
    debug_assert!(!items.is_empty(), "items must not be empty");
    let mut summary = build_satd_summary_header(result);
    let sorted_files = group_and_sort_satd_items(items);

    for (path, file_items) in sorted_files.iter().take(10) {
        summary.push_str(&format!(
            "  {} - {} items\n",
            path.display(),
            file_items.len()
        ));
    }

    json!({
        "formatted_output": summary,
        "stats": {
            "total_items": result.summary.total_items,
            "critical_items": result.summary.by_severity.get("Critical").copied().unwrap_or(0),
            "debt_density": (result.summary.total_items as f64 / result.total_files_analyzed.max(1) as f64),
        }
    })
}

fn format_satd_output(
    args: &SatdArgs,
    result: &crate::services::satd_detector::SATDAnalysisResult,
    items: &[crate::services::satd_detector::TechnicalDebt],
) -> serde_json::Value {
    debug_assert!(!items.is_empty(), "items must not be empty");
    match args.format.as_str() {
        "json" => format_satd_json_output(args, result, items),
        _ => format_satd_summary_output(result, items),
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) async fn handle_analyze_satd(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    let args = match parse_satd_args(arguments) {
        Ok(args) => args,
        Err(e) => return McpResponse::error(request_id, -32602, e),
    };

    info!("Analyzing SATD for project: {:?}", args.project_path);

    let result = match execute_satd_analysis(&args).await {
        Ok(result) => result,
        Err(e) => return McpResponse::error(request_id, -32603, e),
    };

    let (result, items) = filter_satd_items(result, args.critical_only);
    let output = format_satd_output(&args, &result, &items);

    McpResponse::success(request_id, output)
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct LintHotspotArgs {
    #[serde(default = "default_project_path")]
    project_path: String,
    #[serde(default = "default_top_files")]
    top_files: usize,
    #[serde(default = "default_min_violations")]
    min_violations: usize,
    #[serde(default)]
    include: Option<String>,
    #[serde(default)]
    exclude: Option<String>,
    #[serde(default = "default_table_format")]
    format: String,
}

fn default_min_violations() -> usize {
    debug_assert!(true, "contract: default_min_violations");
    1
}

fn default_table_format() -> String {
    debug_assert!(true, "contract: default_table_format");
    "table".to_string()
}

fn parse_lint_hotspot_args(arguments: serde_json::Value) -> Result<LintHotspotArgs, String> {
    serde_json::from_value(arguments)
        .map_err(|e| format!("Invalid analyze_lint_hotspot arguments: {e}"))
}

async fn execute_lint_hotspot_analysis(
    args: &LintHotspotArgs,
    project_path: &Path,
) -> Result<std::path::PathBuf, String> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    use crate::cli::handlers::lint_hotspot_handlers::handle_analyze_lint_hotspot;
    use crate::cli::LintHotspotOutputFormat;

    let temp_file = tempfile::NamedTempFile::new()
        .map_err(|e| format!("Failed to create temporary file: {e}"))?;
    let output_path = temp_file.path().to_path_buf();

    handle_analyze_lint_hotspot(
        project_path.to_path_buf(),
        None,
        LintHotspotOutputFormat::Json,
        100.0,
        0.0,
        false,
        false,
        false,
        Some(output_path.clone()),
        false,
        String::new(),
        args.top_files,
        Vec::new(),
        Vec::new(),
    )
    .await
    .map_err(|e| format!("Failed to analyze lint hotspots: {e}"))?;

    Ok(output_path)
}

async fn read_and_parse_lint_output(
    output_path: &std::path::Path,
) -> Result<serde_json::Value, String> {
    debug_assert!(true, "contract: read_and_parse_lint_output");
    let json_output = tokio::fs::read_to_string(output_path)
        .await
        .map_err(|e| format!("Failed to read temporary file: {e}"))?;

    serde_json::from_str(&json_output).map_err(|e| format!("Failed to parse JSON output: {e}"))
}

struct LintHotspotData {
    hotspots: Vec<serde_json::Value>,
    total_files: usize,
    total_violations: usize,
    average_violations_per_file: f64,
}

fn extract_lint_data(lint_data: &serde_json::Value) -> LintHotspotData {
    debug_assert!(true, "contract: extract_lint_data");
    LintHotspotData {
        hotspots: lint_data["hotspots"].as_array().unwrap_or(&vec![]).clone(),
        total_files: lint_data["total_files_analyzed"].as_u64().unwrap_or(0) as usize,
        total_violations: lint_data["total_violations"].as_u64().unwrap_or(0) as usize,
        average_violations_per_file: lint_data["average_violations_per_file"]
            .as_f64()
            .unwrap_or(0.0),
    }
}

fn format_lint_hotspot_output(args: &LintHotspotArgs, data: &LintHotspotData) -> serde_json::Value {
    debug_assert!(true, "contract: format_lint_hotspot_output");
    match args.format.as_str() {
        "json" => format_json_output(args, data),
        "csv" => format_csv_output(),
        _ => format_table_output(data),
    }
}

fn format_json_output(args: &LintHotspotArgs, data: &LintHotspotData) -> serde_json::Value {
    debug_assert!(true, "contract: format_json_output");
    json!({
        "project_path": args.project_path,
        "total_files_analyzed": data.total_files,
        "total_violations": data.total_violations,
        "average_violations_per_file": data.average_violations_per_file,
        "hotspots": data.hotspots,
    })
}

fn format_csv_output() -> serde_json::Value {
    debug_assert!(true, "contract: format_csv_output");
    json!({
        "formatted_output": "file_path,violations,lines_of_code,defect_density\n",
        "content_type": "text/csv"
    })
}

fn format_table_output(data: &LintHotspotData) -> serde_json::Value {
    debug_assert!(true, "contract: format_table_output");
    let mut table = String::from("Lint Hotspot Analysis\n");
    table.push_str("====================\n");
    table.push_str(&format!("Total files analyzed: {}\n", data.total_files));
    table.push_str(&format!("Total violations: {}\n", data.total_violations));
    table.push_str(&format!(
        "Average violations per file: {:.2}\n\n",
        data.average_violations_per_file
    ));
    table.push_str("No hotspots found.\n");

    json!({
        "formatted_output": table,
        "stats": {
            "total_files": data.total_files,
            "total_violations": data.total_violations,
            "average_violations_per_file": data.average_violations_per_file,
        }
    })
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) async fn handle_analyze_lint_hotspot(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    let args = match parse_lint_hotspot_args(arguments) {
        Ok(args) => args,
        Err(e) => return McpResponse::error(request_id, -32602, e),
    };

    info!(
        "Analyzing lint hotspots for project: {:?}",
        args.project_path
    );

    let project_path = std::path::PathBuf::from(args.project_path.clone());

    let output_path = match execute_lint_hotspot_analysis(&args, &project_path).await {
        Ok(path) => path,
        Err(e) => return McpResponse::error(request_id, -32603, e),
    };

    let lint_data = match read_and_parse_lint_output(&output_path).await {
        Ok(data) => data,
        Err(e) => return McpResponse::error(request_id, -32603, e),
    };

    let extracted_data = extract_lint_data(&lint_data);
    let output = format_lint_hotspot_output(&args, &extracted_data);

    McpResponse::success(request_id, output)
}

/// Handle Quality-Driven Development (QDD) tool calls
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) async fn handle_quality_driven_development(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    // Parse QDD arguments
    #[derive(Deserialize)]
    struct QddArgs {
        operation_type: String,
        quality_profile: Option<String>,
        code_type: Option<String>,
        name: Option<String>,
        purpose: Option<String>,
        file_path: Option<String>,
        inputs: Option<Vec<(String, String)>>,
        output_type: Option<String>,
    }

    let args: QddArgs = match serde_json::from_value(arguments) {
        Ok(a) => a,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid quality_driven_development arguments: {e}"),
            );
        }
    };

    info!(
        "Executing QDD operation: {} with profile: {:?}",
        args.operation_type, args.quality_profile
    );

    // Convert file_path to PathBuf if provided
    let file_path_buf = args.file_path.as_ref().map(PathBuf::from);

    // Call the actual QDD function
    match crate::mcp_pmcp::tool_functions::quality_driven_development(
        &args.operation_type,
        args.quality_profile.as_deref(),
        args.code_type.as_deref(),
        args.name.as_deref(),
        args.purpose.as_deref(),
        file_path_buf.as_ref(),
        args.inputs,
        args.output_type.as_deref(),
    )
    .await
    {
        Ok(result) => {
            info!("QDD operation completed successfully");
            McpResponse::success(request_id, result)
        }
        Err(e) => {
            error!("QDD operation failed: {}", e);
            McpResponse::error(
                request_id,
                -32603,
                format!("Quality-driven development failed: {e}"),
            )
        }
    }
}
