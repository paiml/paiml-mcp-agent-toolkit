// Advanced Analysis Tool Handlers
//
// Extracted from tools.rs for file health compliance (CB-040).
// Contains handlers for: defect probability, dead code, TDG, deep context,
// makefile lint, provability, SATD, lint hotspot, and QDD analysis.

use crate::models::mcp::McpResponse;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use tracing::{error, info};

/// R22-1 / D101 — Reject missing, null, or empty/whitespace-only
/// `project_path` arguments across MCP advanced-analysis handlers.
///
/// This is the parity fix for R21-5 / D99 (PR #371), which added the same
/// guard to the `src/agent/mcp_server_protocol.rs` handlers. The live
/// dispatcher in `src/handlers/tools_advanced*` was still silently
/// defaulting to `std::env::current_dir()`, which lets a remote MCP client
/// exfiltrate information about the server's launch directory by sending
/// `{}` / `{"project_path": null}` / `{"project_path": ""}`.
///
/// Mirrors the R21-1 / D100 fail-loud pattern already used by
/// `handle_analyze_dead_code`. Returns an error string tagged for the
/// caller to wrap in JSON-RPC `-32602` (Invalid params).
fn require_project_path_advanced(project_path_arg: Option<String>) -> Result<PathBuf, String> {
    let Some(raw) = project_path_arg else {
        return Err(
            "'project_path' is required and must be a non-empty string — \
null/missing is rejected to avoid silently analyzing the server's current \
directory (R22-1 / D101)"
                .to_string(),
        );
    };
    if raw.trim().is_empty() {
        return Err(
            "'project_path' must be a non-empty string — empty/whitespace values \
are rejected to avoid silently analyzing the server's current directory \
(R22-1 / D101)"
                .to_string(),
        );
    }
    Ok(PathBuf::from(raw))
}

/// R22-1 / D101 — String-taking variant of `require_project_path_advanced`
/// for handlers whose arg struct uses `project_path: String` (typically via
/// `#[serde(default = "default_project_path")]`). Rejects empty/whitespace
/// values; missing/null case is handled by serde before we get here.
fn require_non_empty_path(raw: &str, field_name: &str) -> Result<PathBuf, String> {
    if raw.trim().is_empty() {
        return Err(format!(
            "'{field_name}' must be a non-empty string — empty/whitespace values \
are rejected to avoid silently analyzing the server's current directory \
(R22-1 / D101)"
        ));
    }
    Ok(PathBuf::from(raw))
}

#[derive(Debug, Deserialize, Serialize)]
struct AnalyzeDefectProbabilityArgs {
    project_path: Option<String>,
    format: Option<String>,
}

fn get_relative_path(path: &Path, project_path: &Path) -> String {
    path.strip_prefix(project_path)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

fn calculate_cyclomatic_complexity(content: &str) -> u32 {
    let control_flow_keywords = ["if", "else", "for", "while", "match", "loop", "?"];
    control_flow_keywords
        .iter()
        .map(|kw| content.matches(kw).count() as u32)
        .sum::<u32>()
        + 1
}

fn calculate_cognitive_complexity(cyclomatic_complexity: u32) -> u32 {
    (cyclomatic_complexity as f32 * 1.5) as u32
}

fn calculate_duplicate_ratio(lines: &[&str]) -> f32 {
    let mut line_counts = std::collections::HashMap::new();
    let mut duplicate_lines = 0;

    // Count non-empty, non-comment lines
    for line in lines {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            *line_counts.entry(trimmed).or_insert(0) += 1;
        }
    }

    // Count duplicates
    for count in line_counts.values() {
        if *count > 1 {
            duplicate_lines += count - 1;
        }
    }

    if lines.is_empty() {
        0.0
    } else {
        duplicate_lines as f32 / lines.len() as f32
    }
}

fn calculate_efferent_coupling(content: &str) -> f32 {
    content
        .lines()
        .filter(|line| line.trim().starts_with("use "))
        .count() as f32
}

fn is_public_declaration(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("pub fn")
        || trimmed.starts_with("pub struct")
        || trimmed.starts_with("pub enum")
        || trimmed.starts_with("pub trait")
        || trimmed.starts_with("pub mod")
}

fn calculate_afferent_coupling(content: &str) -> f32 {
    content
        .lines()
        .filter(|line| is_public_declaration(line))
        .count() as f32
}

fn get_churn_score(relative_path: &str, churn_map: &std::collections::HashMap<String, f32>) -> f32 {
    churn_map.get(relative_path).copied().unwrap_or(0.1)
}

// Helper function to calculate file metrics
async fn calculate_file_metrics(
    path: PathBuf,
    project_path: PathBuf,
    churn_map: std::collections::HashMap<String, f32>,
) -> crate::services::defect_probability::FileMetrics {
    use crate::services::defect_probability::FileMetrics;

    let relative_path = get_relative_path(&path, &project_path);
    let content = tokio::fs::read_to_string(&path).await.unwrap_or_default();
    let lines: Vec<&str> = content.lines().collect();
    let lines_of_code = lines.len();

    let cyclomatic_complexity = calculate_cyclomatic_complexity(&content);
    let cognitive_complexity = calculate_cognitive_complexity(cyclomatic_complexity);
    let churn_score = get_churn_score(&relative_path, &churn_map);
    let duplicate_ratio = calculate_duplicate_ratio(&lines);
    let efferent_coupling = calculate_efferent_coupling(&content);
    let afferent_coupling = calculate_afferent_coupling(&content);

    FileMetrics {
        file_path: relative_path,
        churn_score,
        complexity: cyclomatic_complexity as f32,
        duplicate_ratio,
        afferent_coupling,
        efferent_coupling,
        lines_of_code,
        cyclomatic_complexity,
        cognitive_complexity,
    }
}

/// Toyota Way: Extract Method - Handle defect probability analysis (complexity ≤8)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) async fn handle_analyze_defect_probability(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    // Parse arguments
    let (args, project_path) = match parse_defect_probability_args(arguments) {
        Ok(result) => result,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid analyze_defect_probability arguments: {e}"),
            );
        }
    };

    info!("Analyzing defect probability for {:?}", project_path);

    // Build churn map from git analysis
    let churn_map = build_churn_map(&project_path);

    // Discover and analyze files
    let file_metrics =
        match discover_and_analyze_files(&project_path, churn_map, request_id.clone()).await {
            Ok(metrics) => metrics,
            Err(response) => return response,
        };

    // Calculate defect probabilities and create response
    create_defect_probability_response(request_id, args, file_metrics)
}

/// Toyota Way: Extract Method - Parse defect probability arguments (complexity ≤5)
///
/// R22-1 / D101: project_path is required. Missing, null, and
/// empty/whitespace values are rejected with an error that the caller maps
/// to JSON-RPC `-32602` — we never silently default to the server's cwd.
fn parse_defect_probability_args(
    arguments: serde_json::Value,
) -> Result<(AnalyzeDefectProbabilityArgs, PathBuf), Box<dyn std::error::Error>> {
    let args: AnalyzeDefectProbabilityArgs = serde_json::from_value(arguments)?;

    let project_path = require_project_path_advanced(args.project_path.clone())
        .map_err(Box::<dyn std::error::Error>::from)?;

    Ok((args, project_path))
}

/// Toyota Way: Extract Method - Build churn map from git analysis (complexity ≤5)
fn build_churn_map(project_path: &Path) -> std::collections::HashMap<String, f32> {
    use crate::services::git_analysis::GitAnalysisService;

    let churn_analysis = GitAnalysisService::analyze_code_churn(project_path, 30).ok();
    churn_analysis
        .map(|analysis| {
            analysis
                .files
                .into_iter()
                .map(|f| (f.relative_path, f.churn_score))
                .collect()
        })
        .unwrap_or_default()
}

/// Toyota Way: Extract Method - Discover and analyze files (complexity ≤8)
async fn discover_and_analyze_files(
    project_path: &Path,
    churn_map: std::collections::HashMap<String, f32>,
    request_id: serde_json::Value,
) -> Result<Vec<crate::services::defect_probability::FileMetrics>, McpResponse> {
    use crate::services::file_discovery::ProjectFileDiscovery;
    use futures::stream::{self, StreamExt};

    // Discover files
    let discovery = ProjectFileDiscovery::new(project_path.to_path_buf());
    let discovered_files = match discovery.discover_files() {
        Ok(files) => files,
        Err(e) => {
            error!("Failed to discover files: {}", e);
            return Err(McpResponse::error(
                request_id,
                -32603,
                format!("Failed to discover files: {e}"),
            ));
        }
    };

    // Process files in parallel
    let metrics_futures: Vec<_> = discovered_files
        .into_iter()
        .filter(|path| path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs"))
        .map(|path| {
            let project_path = project_path.to_path_buf();
            let churn_map = churn_map.clone();
            calculate_file_metrics(path, project_path, churn_map)
        })
        .collect();

    // Execute futures concurrently
    let file_metrics = stream::iter(metrics_futures)
        .buffer_unordered(8)
        .collect()
        .await;

    Ok(file_metrics)
}

/// Toyota Way: Extract Method - Create defect probability response (complexity ≤6)
fn create_defect_probability_response(
    request_id: serde_json::Value,
    args: AnalyzeDefectProbabilityArgs,
    file_metrics: Vec<crate::services::defect_probability::FileMetrics>,
) -> McpResponse {
    use crate::services::defect_probability::{DefectProbabilityCalculator, ProjectDefectAnalysis};

    let calculator = DefectProbabilityCalculator::new();
    let scores = calculator.calculate_batch(&file_metrics);
    let analysis = ProjectDefectAnalysis::from_scores(scores);

    let content_text = format_defect_probability_output(&args, &analysis);

    let result = json!({
        "content": [{
            "type": "text",
            "text": content_text
        }],
        "analysis": analysis,
        "format": args.format.unwrap_or_else(|| "summary".to_string()),
    });

    McpResponse::success(request_id, result)
}

/// Toyota Way: Extract Method - Format defect probability output (complexity ≤5)
fn format_defect_probability_output(
    args: &AnalyzeDefectProbabilityArgs,
    analysis: &crate::services::defect_probability::ProjectDefectAnalysis,
) -> String {
    match args.format.as_deref() {
        Some("json") => serde_json::to_string_pretty(analysis).unwrap_or_default(),
        _ => format!(
            "# Defect Probability Analysis\n\nTotal files: {}\nHigh-risk files: {}\nMedium-risk files: {}\nAverage probability: {:.2}",
            analysis.total_files,
            analysis.high_risk_files.len(),
            analysis.medium_risk_files.len(),
            analysis.average_probability
        ),
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct AnalyzeDeadCodeArgs {
    project_path: Option<String>,
    format: Option<String>,
    top_files: Option<usize>,
    include_unreachable: Option<bool>,
    min_dead_lines: Option<usize>,
    include_tests: Option<bool>,
}

/// Toyota Way: Extract Method - Handle dead code analysis (complexity ≤8)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) async fn handle_analyze_dead_code(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    // Parse arguments
    let (args, project_path) = match parse_dead_code_args(arguments) {
        Ok(result) => result,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid analyze_dead_code arguments: {e}"),
            );
        }
    };

    info!("Analyzing dead code for {:?}", project_path);

    // Run dead code analysis
    let mut result = match run_dead_code_analysis(&project_path, &args).await {
        Ok(r) => r,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32000,
                format!("Dead code analysis failed: {e}"),
            );
        }
    };

    // Apply top_files limit if specified
    if let Some(limit) = args.top_files {
        result.ranked_files.truncate(limit);
    }

    // Format and respond
    format_and_respond_dead_code(request_id, args, result)
}

/// Toyota Way: Extract Method - Parse dead code arguments (complexity ≤5)
///
/// # R21-1 / D100 fail-loud fix
///
/// Previously this silently fell back to `std::env::current_dir()` when
/// `project_path` was null, missing, or an empty string. That produced what
/// looked like "hardcoded fake data" to MCP clients because the server had
/// no idea what project the client meant and analyzed wherever the server
/// process happened to be running. Now we reject those three cases up front
/// with a clear error — callers must pass a real path.
fn parse_dead_code_args(
    arguments: serde_json::Value,
) -> Result<(AnalyzeDeadCodeArgs, PathBuf), Box<dyn std::error::Error>> {
    let args: AnalyzeDeadCodeArgs = serde_json::from_value(arguments)?;

    let raw_path = args.project_path.as_ref().ok_or_else(|| {
        Box::<dyn std::error::Error>::from(
            "'project_path' is required and must be a non-empty string — \
null/missing is rejected to avoid silently analyzing the server's current \
directory (R21-1 / D100)",
        )
    })?;
    if raw_path.trim().is_empty() {
        return Err(Box::<dyn std::error::Error>::from(
            "'project_path' must be a non-empty string (R21-1 / D100)",
        ));
    }
    let project_path = PathBuf::from(raw_path);

    Ok((args, project_path))
}

/// Toyota Way: Extract Method - Run dead code analysis (complexity ≤6)
async fn run_dead_code_analysis(
    project_path: &Path,
    args: &AnalyzeDeadCodeArgs,
) -> Result<crate::models::dead_code::DeadCodeRankingResult, Box<dyn std::error::Error>> {
    use crate::models::dead_code::DeadCodeAnalysisConfig;
    use crate::services::dead_code_analyzer::DeadCodeAnalyzer;

    let mut analyzer = DeadCodeAnalyzer::new(10000);

    let config = DeadCodeAnalysisConfig {
        include_unreachable: args.include_unreachable.unwrap_or(false),
        include_tests: args.include_tests.unwrap_or(false),
        min_dead_lines: args.min_dead_lines.unwrap_or(10),
    };

    Ok(analyzer.analyze_with_ranking(project_path, config).await?)
}

/// Toyota Way: Extract Method - Format and respond with dead code results (complexity ≤8)
fn format_and_respond_dead_code(
    request_id: serde_json::Value,
    args: AnalyzeDeadCodeArgs,
    result: crate::models::dead_code::DeadCodeRankingResult,
) -> McpResponse {
    let format = args.format.as_deref().unwrap_or("summary");

    let content_text = match format_dead_code_output(&result, format) {
        Ok(content) => content,
        Err(e) => {
            return McpResponse::error(request_id, -32000, format!("Failed to format output: {e}"));
        }
    };

    let response = build_dead_code_response(format, content_text, &result);
    McpResponse::success(request_id, response)
}

/// Toyota Way: Extract Method - Build dead code response JSON (complexity ≤5)
fn build_dead_code_response(
    format: &str,
    content_text: String,
    result: &crate::models::dead_code::DeadCodeRankingResult,
) -> serde_json::Value {
    json!({
        "content": [{
            "type": "text",
            "text": content_text
        }],
        "result": result,
        "format": format,
        "files_analyzed": result.summary.total_files_analyzed,
        "files_with_dead_code": result.summary.files_with_dead_code,
        "total_dead_lines": result.summary.total_dead_lines,
        "dead_percentage": result.summary.dead_percentage,
    })
}

/// Format dead code analysis output for MCP response
fn format_dead_code_output(
    result: &crate::models::dead_code::DeadCodeRankingResult,
    format: &str,
) -> anyhow::Result<String> {
    use crate::cli::DeadCodeOutputFormat;

    let output_format = match format {
        "summary" => DeadCodeOutputFormat::Summary,
        "json" => DeadCodeOutputFormat::Json,
        "sarif" => DeadCodeOutputFormat::Sarif,
        "markdown" => DeadCodeOutputFormat::Markdown,
        _ => DeadCodeOutputFormat::Summary,
    };

    // Use the existing formatting functions from CLI module
    match output_format {
        DeadCodeOutputFormat::Summary => {
            // Import the function from the CLI module
            format_dead_code_summary_mcp(result)
        }
        DeadCodeOutputFormat::Json => Ok(serde_json::to_string_pretty(result)?),
        DeadCodeOutputFormat::Sarif => format_dead_code_as_sarif_mcp(result),
        DeadCodeOutputFormat::Markdown => format_dead_code_as_markdown_mcp(result),
    }
}

/// Toyota Way: Extract Method - Format dead code analysis as summary text for MCP (complexity ≤8)
fn format_dead_code_summary_mcp(
    result: &crate::models::dead_code::DeadCodeRankingResult,
) -> anyhow::Result<String> {
    let mut output = String::with_capacity(1024);

    output.push_str("# Dead Code Analysis Summary\n\n");
    format_dead_code_summary_stats(&mut output, &result.summary);
    format_top_dead_code_files(&mut output, &result.ranked_files);

    Ok(output)
}

/// Toyota Way: Extract Method - Format summary statistics section (complexity ≤5)
fn format_dead_code_summary_stats(
    output: &mut String,
    summary: &crate::models::dead_code::DeadCodeSummary,
) {
    output.push_str(&format!(
        "**Total files analyzed:** {}\n",
        summary.total_files_analyzed
    ));

    let files_with_dead_percentage = if summary.total_files_analyzed > 0 {
        (summary.files_with_dead_code as f32 / summary.total_files_analyzed as f32) * 100.0
    } else {
        0.0
    };

    output.push_str(&format!(
        "**Files with dead code:** {} ({:.1}%)\n",
        summary.files_with_dead_code, files_with_dead_percentage
    ));
    output.push_str(&format!(
        "**Total dead lines:** {} ({:.1}% of codebase)\n",
        summary.total_dead_lines, summary.dead_percentage
    ));
    output.push_str(&format!("**Dead functions:** {}\n", summary.dead_functions));
    output.push_str(&format!("**Dead classes:** {}\n", summary.dead_classes));
    output.push_str(&format!("**Dead modules:** {}\n", summary.dead_modules));
    output.push_str(&format!(
        "**Unreachable blocks:** {}\n\n",
        summary.unreachable_blocks
    ));
}

// ---------------------------------------------------------------------------
// R21-1 / D100: fail-loud tests for analyze_dead_code argument validation.
// ---------------------------------------------------------------------------

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod dead_code_fail_loud_tests {
    use super::*;
    use serde_json::json;

    fn err_msg(args: serde_json::Value) -> String {
        match parse_dead_code_args(args) {
            Err(e) => e.to_string(),
            Ok(_) => panic!("expected error, got Ok"),
        }
    }

    #[test]
    fn rejects_null_project_path() {
        let msg = err_msg(json!({ "project_path": null }));
        assert!(
            msg.contains("project_path"),
            "error must name the field, got: {msg}"
        );
        assert!(
            msg.contains("required") || msg.contains("non-empty"),
            "error must explain why, got: {msg}"
        );
    }

    #[test]
    fn rejects_missing_project_path() {
        let msg = err_msg(json!({}));
        assert!(msg.contains("project_path"), "got: {msg}");
    }

    #[test]
    fn rejects_empty_string_project_path() {
        let msg = err_msg(json!({ "project_path": "" }));
        assert!(msg.contains("non-empty"), "got: {msg}");
    }

    #[test]
    fn rejects_whitespace_only_project_path() {
        let msg = err_msg(json!({ "project_path": "   " }));
        assert!(msg.contains("non-empty"), "got: {msg}");
    }

    #[test]
    fn accepts_valid_project_path() {
        let (_args, path) =
            parse_dead_code_args(json!({ "project_path": "/tmp/my-project" })).unwrap();
        assert_eq!(path, PathBuf::from("/tmp/my-project"));
    }
}
