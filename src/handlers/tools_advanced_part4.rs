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
    true
}

fn default_summary_format() -> String {
    "summary".to_string()
}

fn parse_satd_args(arguments: serde_json::Value) -> Result<SatdArgs, String> {
    serde_json::from_value(arguments).map_err(|e| format!("Invalid analyze_satd arguments: {e}"))
}

fn create_satd_detector(strict: bool) -> crate::services::satd_detector::SATDDetector {
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
    let mut args = match parse_satd_args(arguments) {
        Ok(args) => args,
        Err(e) => return McpResponse::error(request_id, -32602, e),
    };

    // R22-1 / D101: reject missing/empty/whitespace project_path to avoid
    // silently analyzing the server's cwd. `SatdArgs.project_path` uses
    // `#[serde(default = "default_project_path")]` which now yields ""; the
    // empty-string check catches both "missing" and "empty".
    if let Err(e) = require_non_empty_path(&args.project_path, "project_path") {
        return McpResponse::error(request_id, -32602, e);
    }

    // R22-2 / D102: glob-aware `project_path` with fail-loud -32602 on empty
    // expansion. Shared helper: `services::path_glob`.
    args.project_path = match resolve_project_path_with_globs(&args.project_path) {
        ResolvedProjectPath::Concrete(p) => p.to_string_lossy().into_owned(),
        e @ ResolvedProjectPath::EmptyGlob(_) => {
            return McpResponse::error(request_id, -32602, e.into_error_message());
        }
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
    1
}

fn default_table_format() -> String {
    "table".to_string()
}

fn parse_lint_hotspot_args(arguments: serde_json::Value) -> Result<LintHotspotArgs, String> {
    serde_json::from_value(arguments)
        .map_err(|e| format!("Invalid analyze_lint_hotspot arguments: {e}"))
}

/// Run clippy analysis in-memory — avoids tempfile lifetime bugs (D91).
///
/// Previously this wrote JSON to a `NamedTempFile` and re-read it from disk;
/// the tempfile dropped at scope exit and deleted the file before the read,
/// failing every MCP invocation with
/// `Failed to read temporary file: No such file or directory (os error 2)`.
async fn execute_lint_hotspot_analysis(
    project_path: &Path,
) -> Result<Option<crate::cli::handlers::lint_hotspot_handlers::LintHotspotResult>, String> {
    use crate::cli::handlers::lint_hotspot_handlers::clippy::run_clippy_analysis;

    match run_clippy_analysis(project_path, "").await {
        Ok(result) => Ok(Some(result)),
        // Clean project — not an error condition over MCP; callers get an empty result.
        Err(e) if e.to_string().contains("No lint violations found") => Ok(None),
        Err(e) => Err(format!("Failed to analyze lint hotspots: {e}")),
    }
}

struct LintHotspotData {
    hotspots: Vec<serde_json::Value>,
    total_files: usize,
    total_violations: usize,
    average_violations_per_file: f64,
}

fn extract_lint_data(
    result: Option<&crate::cli::handlers::lint_hotspot_handlers::LintHotspotResult>,
    top_files: usize,
) -> LintHotspotData {
    let Some(result) = result else {
        return LintHotspotData {
            hotspots: Vec::new(),
            total_files: 0,
            total_violations: 0,
            average_violations_per_file: 0.0,
        };
    };

    let total_files = result.summary_by_file.len();
    let total_violations = result.total_project_violations;
    let average_violations_per_file = if total_files > 0 {
        total_violations as f64 / total_files as f64
    } else {
        0.0
    };

    // Sort files by defect density (highest first) and take top N.
    let mut sorted: Vec<_> = result.summary_by_file.iter().collect();
    sorted.sort_by(|a, b| {
        b.1.defect_density
            .partial_cmp(&a.1.defect_density)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let hotspots = sorted
        .into_iter()
        .take(top_files.max(1))
        .map(|(path, summary)| {
            json!({
                "file": path.display().to_string(),
                "total_violations": summary.total_violations,
                "errors": summary.errors,
                "warnings": summary.warnings,
                "sloc": summary.sloc,
                "defect_density": summary.defect_density,
            })
        })
        .collect();

    LintHotspotData {
        hotspots,
        total_files,
        total_violations,
        average_violations_per_file,
    }
}

fn format_lint_hotspot_output(args: &LintHotspotArgs, data: &LintHotspotData) -> serde_json::Value {
    match args.format.as_str() {
        "json" => format_json_output(args, data),
        "csv" => format_csv_output(),
        _ => format_table_output(data),
    }
}

fn format_json_output(args: &LintHotspotArgs, data: &LintHotspotData) -> serde_json::Value {
    json!({
        "project_path": args.project_path,
        "total_files_analyzed": data.total_files,
        "total_violations": data.total_violations,
        "average_violations_per_file": data.average_violations_per_file,
        "hotspots": data.hotspots,
    })
}

fn format_csv_output() -> serde_json::Value {
    json!({
        "formatted_output": "file_path,violations,lines_of_code,defect_density\n",
        "content_type": "text/csv"
    })
}

fn format_table_output(data: &LintHotspotData) -> serde_json::Value {
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
    let mut args = match parse_lint_hotspot_args(arguments) {
        Ok(args) => args,
        Err(e) => return McpResponse::error(request_id, -32602, e),
    };

    // R22-1 / D101: reject missing/empty/whitespace project_path to avoid
    // silently running clippy on the server's cwd.
    if let Err(e) = require_non_empty_path(&args.project_path, "project_path") {
        return McpResponse::error(request_id, -32602, e);
    }

    // R22-2 / D102: glob-aware `project_path`, `-32602` on empty expansion.
    args.project_path = match resolve_project_path_with_globs(&args.project_path) {
        ResolvedProjectPath::Concrete(p) => p.to_string_lossy().into_owned(),
        e @ ResolvedProjectPath::EmptyGlob(_) => {
            return McpResponse::error(request_id, -32602, e.into_error_message());
        }
    };
    let project_path = PathBuf::from(&args.project_path);

    info!(
        "Analyzing lint hotspots for project: {:?}",
        args.project_path
    );

    let result = match execute_lint_hotspot_analysis(&project_path).await {
        Ok(r) => r,
        Err(e) => return McpResponse::error(request_id, -32603, e),
    };

    let extracted_data = extract_lint_data(result.as_ref(), args.top_files);
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

// ============================================================================
// D91 / R21-3 regression tests
// ============================================================================
#[cfg(test)]
mod d91_r21_3_tests {
    use super::*;

    /// `extract_lint_data(None, _)` must not panic and must yield zero totals.
    /// Before the fix, the analogous code path relied on a tempfile that was
    /// deleted before reading, so this in-memory path didn't exist at all.
    #[test]
    fn extract_lint_data_none_yields_zero() {
        let data = extract_lint_data(None, 10);
        assert_eq!(data.total_files, 0);
        assert_eq!(data.total_violations, 0);
        assert!(data.hotspots.is_empty());
        assert_eq!(data.average_violations_per_file, 0.0);
    }

    /// `extract_lint_data(Some(result), top)` must expose hotspots sorted by
    /// defect density, respect `top_files`, and surface totals from the
    /// in-memory `LintHotspotResult` (no tempfile).
    #[test]
    fn extract_lint_data_sorts_and_truncates_hotspots() {
        use crate::cli::handlers::lint_hotspot_handlers::{
            FileSummary, LintHotspot, LintHotspotResult, QualityGateStatus, SeverityDistribution,
        };
        use std::collections::HashMap;

        let mut summary_by_file: HashMap<PathBuf, FileSummary> = HashMap::new();
        summary_by_file.insert(
            PathBuf::from("src/cool.rs"),
            FileSummary {
                total_violations: 1,
                errors: 0,
                warnings: 1,
                sloc: 100,
                defect_density: 0.01,
            },
        );
        summary_by_file.insert(
            PathBuf::from("src/hot.rs"),
            FileSummary {
                total_violations: 10,
                errors: 2,
                warnings: 8,
                sloc: 100,
                defect_density: 0.10,
            },
        );

        let result = LintHotspotResult {
            hotspot: LintHotspot {
                file: PathBuf::from("src/hot.rs"),
                defect_density: 0.10,
                total_violations: 10,
                sloc: 100,
                severity_distribution: SeverityDistribution::default(),
                top_lints: vec![],
                detailed_violations: vec![],
            },
            all_violations: vec![],
            summary_by_file,
            total_project_violations: 11,
            enforcement: None,
            refactor_chain: None,
            quality_gate: QualityGateStatus {
                passed: true,
                violations: vec![],
                blocking: false,
            },
        };

        // top_files=1 forces truncation to the hottest file only.
        let data = extract_lint_data(Some(&result), 1);
        assert_eq!(data.total_files, 2);
        assert_eq!(data.total_violations, 11);
        assert_eq!(data.hotspots.len(), 1);
        assert_eq!(
            data.hotspots[0]["file"].as_str(),
            Some("src/hot.rs"),
            "hottest file must be first"
        );

        // top_files=10 returns both, hot first.
        let data = extract_lint_data(Some(&result), 10);
        assert_eq!(data.hotspots.len(), 2);
        assert_eq!(data.hotspots[0]["file"].as_str(), Some("src/hot.rs"));
        assert_eq!(data.hotspots[1]["file"].as_str(), Some("src/cool.rs"));
    }

    /// The MCP handler must NEVER return the D91 error string
    /// `"Failed to read temporary file"`. The new implementation has no
    /// tempfile at all; this test asserts that even for an invalid project
    /// path, the error surface doesn't mention tempfiles.
    #[tokio::test]
    async fn analyze_lint_hotspot_never_emits_tempfile_error() {
        // Non-existent path — handler will error, but NOT with a tempfile error.
        let arguments = json!({
            "project_path": "/nonexistent/r21_3/path",
            "top_files": 5,
        });

        let response = handle_analyze_lint_hotspot(json!(1), arguments).await;
        let serialized = serde_json::to_string(&response).expect("serialize response");

        assert!(
            !serialized.contains("Failed to read temporary file"),
            "D91 regression: handler still emits tempfile error: {serialized}"
        );
        assert!(
            !serialized.contains("Failed to create temporary file"),
            "D91 regression: handler still creates a tempfile: {serialized}"
        );
    }
}

// ============================================================================
// R22-1 / D101 regression tests — cwd-exfiltration parity fix
// ============================================================================
//
// These tests mirror the R21-5 / D99 test layout in
// `src/agent/mcp_server_tests.rs` and prove that every advanced-analysis
// MCP handler in the live `src/handlers/tools_advanced*` tree now rejects
// missing/null/empty `project_path` with JSON-RPC `-32602` instead of
// silently defaulting to `std::env::current_dir()`.

#[cfg(test)]
mod r22_1_d101_cwd_guard_tests {
    use super::*;
    use serde_json::json;

    // --- helpers ------------------------------------------------------------

    fn assert_invalid_params(response: &McpResponse, context: &str) {
        assert!(
            response.error.is_some(),
            "{context}: expected an error response, got success: {response:?}"
        );
        let err = response.error.as_ref().unwrap();
        assert_eq!(
            err.code, -32602,
            "{context}: expected JSON-RPC -32602 (Invalid params), got {}: message={}",
            err.code, err.message
        );
        assert!(
            err.message.contains("project_path") || err.message.contains("path"),
            "{context}: error message should name the offending field: {}",
            err.message
        );
    }

    // --- handle_analyze_tdg ------------------------------------------------

    #[tokio::test]
    async fn analyze_tdg_rejects_missing_project_path() {
        let response = handle_analyze_tdg(json!(1), json!({})).await;
        assert_invalid_params(&response, "analyze_tdg / missing project_path");
    }

    #[tokio::test]
    async fn analyze_tdg_rejects_null_project_path() {
        let response = handle_analyze_tdg(json!(1), json!({ "project_path": null })).await;
        assert_invalid_params(&response, "analyze_tdg / null project_path");
    }

    #[tokio::test]
    async fn analyze_tdg_rejects_empty_project_path() {
        let response = handle_analyze_tdg(json!(1), json!({ "project_path": "" })).await;
        assert_invalid_params(&response, "analyze_tdg / empty project_path");
    }

    #[tokio::test]
    async fn analyze_tdg_rejects_whitespace_project_path() {
        let response = handle_analyze_tdg(json!(1), json!({ "project_path": "   " })).await;
        assert_invalid_params(&response, "analyze_tdg / whitespace project_path");
    }

    // --- handle_analyze_defect_probability --------------------------------

    #[tokio::test]
    async fn analyze_defect_probability_rejects_missing_project_path() {
        let response = handle_analyze_defect_probability(json!(1), json!({})).await;
        assert_invalid_params(&response, "analyze_defect_probability / missing");
    }

    #[tokio::test]
    async fn analyze_defect_probability_rejects_empty_project_path() {
        let response =
            handle_analyze_defect_probability(json!(1), json!({ "project_path": "" })).await;
        assert_invalid_params(&response, "analyze_defect_probability / empty");
    }

    // --- handle_analyze_deep_context --------------------------------------

    #[tokio::test]
    async fn analyze_deep_context_rejects_missing_project_path() {
        let response = handle_analyze_deep_context(json!(1), json!({})).await;
        assert_invalid_params(&response, "analyze_deep_context / missing");
    }

    #[tokio::test]
    async fn analyze_deep_context_rejects_empty_project_path() {
        let response = handle_analyze_deep_context(json!(1), json!({ "project_path": "" })).await;
        assert_invalid_params(&response, "analyze_deep_context / empty");
    }

    // --- handle_analyze_provability ---------------------------------------

    #[tokio::test]
    async fn analyze_provability_rejects_empty_project_path() {
        let response =
            handle_analyze_provability(json!(1), Some(json!({ "project_path": "" }))).await;
        assert_invalid_params(&response, "analyze_provability / empty");
    }

    #[tokio::test]
    async fn analyze_provability_rejects_whitespace_project_path() {
        let response =
            handle_analyze_provability(json!(1), Some(json!({ "project_path": " " }))).await;
        assert_invalid_params(&response, "analyze_provability / whitespace");
    }

    // --- handle_analyze_makefile_lint -------------------------------------

    #[tokio::test]
    async fn analyze_makefile_lint_rejects_empty_path() {
        let response = handle_analyze_makefile_lint(json!(1), Some(json!({ "path": "" }))).await;
        assert_invalid_params(&response, "analyze_makefile_lint / empty path");
    }

    // --- handle_analyze_satd ----------------------------------------------

    #[tokio::test]
    async fn analyze_satd_rejects_missing_project_path() {
        let response = handle_analyze_satd(json!(1), json!({})).await;
        assert_invalid_params(&response, "analyze_satd / missing (via serde default=\"\")");
    }

    #[tokio::test]
    async fn analyze_satd_rejects_empty_project_path() {
        let response = handle_analyze_satd(json!(1), json!({ "project_path": "" })).await;
        assert_invalid_params(&response, "analyze_satd / empty");
    }

    // --- handle_analyze_lint_hotspot --------------------------------------

    #[tokio::test]
    async fn analyze_lint_hotspot_rejects_missing_project_path() {
        let response = handle_analyze_lint_hotspot(json!(1), json!({})).await;
        assert_invalid_params(&response, "analyze_lint_hotspot / missing");
    }

    #[tokio::test]
    async fn analyze_lint_hotspot_rejects_empty_project_path() {
        let response = handle_analyze_lint_hotspot(json!(1), json!({ "project_path": "" })).await;
        assert_invalid_params(&response, "analyze_lint_hotspot / empty");
    }

    // --- require_project_path_advanced helper ------------------------------

    #[test]
    fn require_project_path_advanced_accepts_nonempty() {
        let out = require_project_path_advanced(Some("/tmp/x".to_string())).unwrap();
        assert_eq!(out, PathBuf::from("/tmp/x"));
    }

    #[test]
    fn require_project_path_advanced_rejects_none() {
        let err = require_project_path_advanced(None).unwrap_err();
        assert!(err.contains("project_path"), "{err}");
        assert!(err.contains("D101"), "{err}");
    }

    #[test]
    fn require_project_path_advanced_rejects_empty() {
        let err = require_project_path_advanced(Some(String::new())).unwrap_err();
        assert!(err.contains("non-empty"), "{err}");
    }

    #[test]
    fn require_project_path_advanced_rejects_whitespace() {
        let err = require_project_path_advanced(Some("  \t ".to_string())).unwrap_err();
        assert!(err.contains("non-empty"), "{err}");
    }

    #[test]
    fn require_non_empty_path_rejects_empty() {
        let err = require_non_empty_path("", "project_path").unwrap_err();
        assert!(err.contains("project_path"), "{err}");
        assert!(err.contains("D101"), "{err}");
    }

    #[test]
    fn require_non_empty_path_accepts_value() {
        let out = require_non_empty_path("/some/path", "project_path").unwrap();
        assert_eq!(out, PathBuf::from("/some/path"));
    }
}
