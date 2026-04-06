
/// Toyota Way: Extract Method - Format top files with dead code (complexity ≤8)
fn format_top_dead_code_files(
    output: &mut String,
    ranked_files: &[crate::models::dead_code::FileDeadCodeMetrics],
) {
    if !ranked_files.is_empty() {
        let top_count = ranked_files.len().min(5);
        output.push_str(&format!("## Top {top_count} Files with Most Dead Code\n\n"));

        for (i, file_metrics) in ranked_files.iter().take(top_count).enumerate() {
            format_dead_code_file_entry(output, i + 1, file_metrics);
        }
    }
}

/// Toyota Way: Extract Method - Format individual file entry (complexity ≤5)
fn format_dead_code_file_entry(
    output: &mut String,
    index: usize,
    file_metrics: &crate::models::dead_code::FileDeadCodeMetrics,
) {
    let confidence_text = get_confidence_level_text(file_metrics.confidence);

    output.push_str(&format!(
        "{}. **{}** (Score: {:.1}) [{}confidence]\n",
        index, file_metrics.path, file_metrics.dead_score, confidence_text
    ));
    output.push_str(&format!(
        "   - {} dead lines ({:.1}% of file)\n",
        file_metrics.dead_lines, file_metrics.dead_percentage
    ));

    if file_metrics.dead_functions > 0 || file_metrics.dead_classes > 0 {
        output.push_str(&format!(
            "   - {} functions, {} classes\n",
            file_metrics.dead_functions, file_metrics.dead_classes
        ));
    }
    output.push('\n');
}

/// Toyota Way: Extract Method - Get confidence level text (complexity ≤3)
fn get_confidence_level_text(
    confidence: crate::models::dead_code::ConfidenceLevel,
) -> &'static str {
    match confidence {
        crate::models::dead_code::ConfidenceLevel::High => "HIGH ",
        crate::models::dead_code::ConfidenceLevel::Medium => "MEDIUM ",
        crate::models::dead_code::ConfidenceLevel::Low => "LOW ",
    }
}

/// Format dead code analysis as SARIF for MCP
fn format_dead_code_as_sarif_mcp(
    result: &crate::models::dead_code::DeadCodeRankingResult,
) -> anyhow::Result<String> {
    use serde_json::json;

    let mut results = Vec::with_capacity(256);

    for file_metrics in &result.ranked_files {
        for item in &file_metrics.items {
            results.push(json!({
                "ruleId": format!("dead-code-{}", format!("{:?}", item.item_type).to_lowercase()),
                "level": "info",
                "message": {
                    "text": format!("Dead {} '{}': {}",
                        format!("{:?}", item.item_type).to_lowercase(),
                        item.name,
                        item.reason
                    )
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": file_metrics.path
                        },
                        "region": {
                            "startLine": item.line
                        }
                    }
                }]
            }));
        }
    }

    let sarif = json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat",
                    "version": "0.1.0",
                    "informationUri": "https://github.com/paiml/mcp-agent-toolkit"
                }
            },
            "results": results
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}

/// Format dead code analysis as Markdown for MCP
/// Toyota Way: Extract Method - Format dead code analysis as markdown (complexity ≤8)
fn format_dead_code_as_markdown_mcp(
    result: &crate::models::dead_code::DeadCodeRankingResult,
) -> anyhow::Result<String> {
    let mut output = String::with_capacity(1024);

    write_dead_code_header(&mut output, &result.analysis_timestamp);
    write_dead_code_summary_section(&mut output, &result.summary);
    write_dead_code_top_files_section(&mut output, &result.ranked_files);

    Ok(output)
}

/// Toyota Way: Extract Method - Write dead code report header (complexity ≤3)
fn write_dead_code_header(output: &mut String, timestamp: &chrono::DateTime<chrono::Utc>) {
    output.push_str("# Dead Code Analysis Report\n\n");
    output.push_str(&format!(
        "**Analysis Date:** {}\n\n",
        timestamp.format("%Y-%m-%d %H:%M:%S UTC")
    ));
}

/// Toyota Way: Extract Method - Write dead code summary section (complexity ≤8)
fn write_dead_code_summary_section(
    output: &mut String,
    summary: &crate::models::dead_code::DeadCodeSummary,
) {
    output.push_str("## Summary\n\n");

    output.push_str(&format!(
        "- **Total files analyzed:** {}\n",
        summary.total_files_analyzed
    ));

    let files_with_dead_percentage = calculate_dead_files_percentage(summary);
    output.push_str(&format!(
        "- **Files with dead code:** {} ({:.1}%)\n",
        summary.files_with_dead_code, files_with_dead_percentage
    ));

    write_dead_code_metrics(output, summary);
}

/// Toyota Way: Extract Method - Calculate dead files percentage (complexity ≤3)
fn calculate_dead_files_percentage(summary: &crate::models::dead_code::DeadCodeSummary) -> f32 {
    if summary.total_files_analyzed > 0 {
        (summary.files_with_dead_code as f32 / summary.total_files_analyzed as f32) * 100.0
    } else {
        0.0
    }
}

/// Toyota Way: Extract Method - Write dead code metrics (complexity ≤5)
fn write_dead_code_metrics(
    output: &mut String,
    summary: &crate::models::dead_code::DeadCodeSummary,
) {
    output.push_str(&format!(
        "- **Total dead lines:** {} ({:.1}% of codebase)\n",
        summary.total_dead_lines, summary.dead_percentage
    ));
    output.push_str(&format!(
        "- **Dead functions:** {}\n",
        summary.dead_functions
    ));
    output.push_str(&format!("- **Dead classes:** {}\n", summary.dead_classes));
    output.push_str(&format!("- **Dead modules:** {}\n", summary.dead_modules));
    output.push_str(&format!(
        "- **Unreachable blocks:** {}\n\n",
        summary.unreachable_blocks
    ));
}

/// Toyota Way: Extract Method - Write top dead code files section (complexity ≤8)
fn write_dead_code_top_files_section(
    output: &mut String,
    ranked_files: &[crate::models::dead_code::FileDeadCodeMetrics],
) {
    if !ranked_files.is_empty() {
        write_dead_code_table_header(output);
        write_dead_code_table_rows(output, ranked_files);
        output.push('\n');
    }
}

/// Toyota Way: Extract Method - Write dead code table header (complexity ≤3)
fn write_dead_code_table_header(output: &mut String) {
    output.push_str("## Top Files with Dead Code\n\n");
    output.push_str(
        "| Rank | File | Dead Lines | Percentage | Functions | Classes | Score | Confidence |\n",
    );
    output.push_str(
        "|------|------|------------|------------|-----------|---------|-------|------------|\n",
    );
}

/// Toyota Way: Extract Method - Write dead code table rows (complexity ≤5)
fn write_dead_code_table_rows(
    output: &mut String,
    ranked_files: &[crate::models::dead_code::FileDeadCodeMetrics],
) {
    for (i, file_metrics) in ranked_files.iter().enumerate() {
        write_single_dead_code_row(output, i + 1, file_metrics);
    }
}

/// Toyota Way: Extract Method - Write single dead code table row (complexity ≤5)
fn write_single_dead_code_row(
    output: &mut String,
    rank: usize,
    file_metrics: &crate::models::dead_code::FileDeadCodeMetrics,
) {
    let confidence_text = format_confidence_emoji(file_metrics.confidence);

    output.push_str(&format!(
        "| {:>4} | `{}` | {:>10} | {:>9.1}% | {:>9} | {:>7} | {:>5.1} | {} |\n",
        rank,
        file_metrics.path,
        file_metrics.dead_lines,
        file_metrics.dead_percentage,
        file_metrics.dead_functions,
        file_metrics.dead_classes,
        file_metrics.dead_score,
        confidence_text
    ));
}

/// Toyota Way: Extract Method - Format confidence level with emoji (complexity ≤3)
fn format_confidence_emoji(confidence: crate::models::dead_code::ConfidenceLevel) -> &'static str {
    match confidence {
        crate::models::dead_code::ConfidenceLevel::High => "🔴 High",
        crate::models::dead_code::ConfidenceLevel::Medium => "🟡 Medium",
        crate::models::dead_code::ConfidenceLevel::Low => "🟢 Low",
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct AnalyzeTdgArgs {
    project_path: Option<String>,
    format: Option<String>,
    threshold: Option<f64>,
    include_components: Option<bool>,
    max_results: Option<usize>,
}

/// Toyota Way: Extract Method - Handle TDG analysis (complexity ≤8)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) async fn handle_analyze_tdg(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    // Parse arguments
    let args = match parse_tdg_args(arguments) {
        Ok(args) => args,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid analyze_tdg arguments: {e}"),
            );
        }
    };

    // Extract project path
    let project_path = extract_tdg_project_path(&args);
    info!("Analyzing Technical Debt Gradient for {:?}", project_path);

    // Run analysis and format response
    run_and_format_tdg_analysis(request_id, project_path, args.format).await
}

/// Toyota Way Helper: Parse TDG arguments
fn parse_tdg_args(arguments: serde_json::Value) -> Result<AnalyzeTdgArgs, serde_json::Error> {
    serde_json::from_value(arguments)
}

/// Toyota Way Helper: Extract TDG project path
fn extract_tdg_project_path(args: &AnalyzeTdgArgs) -> PathBuf {
    args.project_path.as_ref().map_or_else(
        || std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        PathBuf::from,
    )
}

/// Toyota Way Helper: Run TDG analysis and format response
async fn run_and_format_tdg_analysis(
    request_id: serde_json::Value,
    project_path: PathBuf,
    format: Option<String>,
) -> McpResponse {
    use crate::services::tdg_calculator::TDGCalculator;

    // Create TDG calculator (primary service for analysis)
    let calculator = TDGCalculator::new();

    // Run TDG analysis
    let analysis = match calculator.analyze_directory(&project_path).await {
        Ok(analysis) => analysis,
        Err(e) => {
            return McpResponse::error(request_id, -32000, format!("TDG analysis failed: {e}"));
        }
    };

    // Format and respond
    format_and_respond_tdg(request_id, analysis, format)
}

/// Toyota Way Helper: Format TDG analysis and build response
fn format_and_respond_tdg(
    request_id: serde_json::Value,
    analysis: crate::models::tdg::TDGSummary,
    format: Option<String>,
) -> McpResponse {
    // Format output
    let content_text = match format.as_deref() {
        Some("json") => serde_json::to_string_pretty(&analysis).unwrap_or_default(),
        _ => format_tdg_summary(&analysis),
    };

    let result = json!({
        "content": [{
            "type": "text",
            "text": content_text
        }],
        "analysis": analysis,
        "format": format.unwrap_or_else(|| "summary".to_string()),
    });

    McpResponse::success(request_id, result)
}

/// Toyota Way: Extract Method - Format TDG summary (complexity ≤8)
fn format_tdg_summary(summary: &crate::models::tdg::TDGSummary) -> String {
    let mut output = String::with_capacity(1024);

    output.push_str("# Technical Debt Gradient Analysis\n\n");

    // Build each section
    append_tdg_summary_section(&mut output, summary);
    append_tdg_hotspots_section(&mut output, summary);
    append_tdg_severity_section(&mut output, summary);

    output
}

/// Toyota Way Helper: Append TDG summary statistics
fn append_tdg_summary_section(output: &mut String, summary: &crate::models::tdg::TDGSummary) {
    output.push_str("## Summary\n\n");
    output.push_str(&format!("**Total files:** {}\n", summary.total_files));

    // Calculate and append percentages
    let critical_pct = calculate_percentage(summary.critical_files, summary.total_files);
    let warning_pct = calculate_percentage(summary.warning_files, summary.total_files);

    output.push_str(&format!(
        "**Critical files:** {} ({:.1}%)\n",
        summary.critical_files, critical_pct
    ));
    output.push_str(&format!(
        "**Warning files:** {} ({:.1}%)\n",
        summary.warning_files, warning_pct
    ));

    // Append metrics
    append_tdg_metrics(output, summary);
}

/// Toyota Way Helper: Append TDG metrics
fn append_tdg_metrics(output: &mut String, summary: &crate::models::tdg::TDGSummary) {
    output.push_str(&format!("**Average TDG:** {:.2}\n", summary.average_tdg));
    output.push_str(&format!(
        "**95th percentile TDG:** {:.2}\n",
        summary.p95_tdg
    ));
    output.push_str(&format!(
        "**99th percentile TDG:** {:.2}\n",
        summary.p99_tdg
    ));
    output.push_str(&format!(
        "**Estimated technical debt:** {:.0} hours\n\n",
        summary.estimated_debt_hours
    ));
}

/// Toyota Way Helper: Append TDG hotspots table
fn append_tdg_hotspots_section(output: &mut String, summary: &crate::models::tdg::TDGSummary) {
    if summary.hotspots.is_empty() {
        return;
    }

    output.push_str("## Top Hotspots\n\n");
    output.push_str("| File | TDG Score | Primary Factor | Estimated Hours |\n");
    output.push_str("|------|-----------|----------------|----------------|\n");

    for hotspot in &summary.hotspots {
        output.push_str(&format!(
            "| {} | {:.2} | {} | {:.0} |\n",
            hotspot.path, hotspot.tdg_score, hotspot.primary_factor, hotspot.estimated_hours
        ));
    }
    output.push('\n');
}

/// Toyota Way Helper: Append TDG severity distribution
fn append_tdg_severity_section(output: &mut String, summary: &crate::models::tdg::TDGSummary) {
    output.push_str("## Severity Distribution\n\n");
    output.push_str(&format!(
        "- 🔴 Critical (>2.5): {} files\n",
        summary.critical_files
    ));
    output.push_str(&format!(
        "- 🟡 Warning (1.5-2.5): {} files\n",
        summary.warning_files
    ));

    let normal_files = summary
        .total_files
        .saturating_sub(summary.critical_files + summary.warning_files);
    output.push_str(&format!("- 🟢 Normal (<1.5): {normal_files} files\n"));
}

/// Toyota Way Helper: Calculate percentage safely
fn calculate_percentage(part: usize, total: usize) -> f64 {
    if total > 0 {
        (part as f64 / total as f64) * 100.0
    } else {
        0.0
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct AnalyzeDeepContextArgs {
    project_path: Option<String>,
    format: Option<String>,
    include_analyses: Option<Vec<String>>,
    exclude_analyses: Option<Vec<String>>,
    period_days: Option<u32>,
    dag_type: Option<String>,
    max_depth: Option<usize>,
    include_pattern: Option<Vec<String>>,
    exclude_pattern: Option<Vec<String>>,
    cache_strategy: Option<String>,
    parallel: Option<usize>,
}
