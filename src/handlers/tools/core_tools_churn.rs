#[derive(Debug, Deserialize, Serialize)]
struct AnalyzeCodeChurnArgs {
    project_path: Option<String>,
    period_days: Option<u32>,
    format: Option<String>,
}

/// Toyota Way: Extract Method - Handle code churn analysis (complexity <=8)
async fn handle_analyze_code_churn(
    request_id: serde_json::Value,
    arguments: serde_json::Value,
) -> McpResponse {
    // Parse arguments
    let args = match parse_code_churn_args(arguments) {
        Ok(args) => args,
        Err(e) => {
            return McpResponse::error(
                request_id,
                -32602,
                format!("Invalid analyze_code_churn arguments: {e}"),
            );
        }
    };

    // Extract analysis parameters
    let (project_path, period_days, format) = extract_churn_parameters(&args);

    info!(
        "Analyzing code churn for {:?} over {} days",
        project_path, period_days
    );

    // Run analysis and format response
    run_and_format_churn_analysis(request_id, project_path, period_days, format).await
}

/// Toyota Way Helper: Parse code churn arguments
fn parse_code_churn_args(
    arguments: serde_json::Value,
) -> Result<AnalyzeCodeChurnArgs, serde_json::Error> {
    serde_json::from_value(arguments)
}

/// Toyota Way Helper: Extract churn analysis parameters
fn extract_churn_parameters(args: &AnalyzeCodeChurnArgs) -> (PathBuf, u32, ChurnOutputFormat) {
    let project_path = args.project_path.as_ref().map_or_else(
        || std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        PathBuf::from,
    );

    let period_days = args.period_days.unwrap_or(30);

    let format = args
        .format
        .as_deref()
        .and_then(|f| f.parse::<ChurnOutputFormat>().ok())
        .unwrap_or(ChurnOutputFormat::Summary);

    (project_path, period_days, format)
}

/// Toyota Way Helper: Run analysis and format response
async fn run_and_format_churn_analysis(
    request_id: serde_json::Value,
    project_path: PathBuf,
    period_days: u32,
    format: ChurnOutputFormat,
) -> McpResponse {
    match GitAnalysisService::analyze_code_churn(&project_path, period_days) {
        Ok(analysis) => {
            let content_text = format_churn_output(&analysis, &format);
            let result = build_churn_response(content_text, analysis, &format);
            McpResponse::success(request_id, result)
        }
        Err(e) => {
            error!("Code churn analysis failed: {}", e);
            McpResponse::error(request_id, -32000, e.to_string())
        }
    }
}

/// Toyota Way Helper: Format churn output based on requested format
fn format_churn_output(
    analysis: &crate::models::churn::CodeChurnAnalysis,
    format: &ChurnOutputFormat,
) -> String {
    match format {
        ChurnOutputFormat::Json => serde_json::to_string_pretty(&analysis).unwrap_or_default(),
        ChurnOutputFormat::Markdown => format_churn_as_markdown(analysis),
        ChurnOutputFormat::Csv => format_churn_as_csv(analysis),
        ChurnOutputFormat::Summary => format_churn_summary(analysis),
    }
}

/// Toyota Way Helper: Build churn response JSON
fn build_churn_response(
    content_text: String,
    analysis: crate::models::churn::CodeChurnAnalysis,
    format: &ChurnOutputFormat,
) -> serde_json::Value {
    json!({
        "content": [{
            "type": "text",
            "text": content_text
        }],
        "analysis": analysis,
        "format": format!("{:?}", format),
    })
}

/// Formats a code churn analysis into a human-readable summary
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::handlers::tools::format_churn_summary;
/// use pmat::models::churn::{CodeChurnAnalysis, ChurnSummary};
/// use std::path::PathBuf;
/// use std::collections::HashMap;
/// use chrono::Utc;
///
/// let analysis = CodeChurnAnalysis {
///     generated_at: Utc::now(),
///     period_days: 30,
///     repository_root: PathBuf::from("/project"),
///     files: vec![],
///     summary: ChurnSummary {
///         total_commits: 150,
///         total_files_changed: 45,
///         hotspot_files: vec![PathBuf::from("src/main.rs")],
///         stable_files: vec![PathBuf::from("README.md")],
///         author_contributions: HashMap::new(),
///     },
/// };
///
/// let summary = format_churn_summary(&analysis);
/// assert!(summary.contains("Period: 30 days"));
/// assert!(summary.contains("Total commits: 150"));
/// ```
#[must_use]
pub fn format_churn_summary(analysis: &crate::models::churn::CodeChurnAnalysis) -> String {
    let mut output = String::with_capacity(1024);

    output.push_str("# Code Churn Analysis\n\n");
    output.push_str(&format!("Period: {} days\n", analysis.period_days));
    output.push_str(&format!(
        "Total files changed: {}\n",
        analysis.summary.total_files_changed
    ));
    output.push_str(&format!(
        "Total commits: {}\n\n",
        analysis.summary.total_commits
    ));

    if !analysis.summary.hotspot_files.is_empty() {
        output.push_str("## Hotspot Files (High Churn)\n");
        for (i, file) in analysis.summary.hotspot_files.iter().take(5).enumerate() {
            output.push_str(&format!("{}. {}\n", i + 1, file.display()));
        }
        output.push('\n');
    }

    if !analysis.summary.stable_files.is_empty() {
        output.push_str("## Stable Files (Low Churn)\n");
        for (i, file) in analysis.summary.stable_files.iter().take(5).enumerate() {
            output.push_str(&format!("{}. {}\n", i + 1, file.display()));
        }
    }

    output
}

/// Formats a code churn analysis as a Markdown report
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::handlers::tools::format_churn_as_markdown;
/// use pmat::models::churn::{CodeChurnAnalysis, ChurnSummary};
/// use std::path::PathBuf;
/// use std::collections::HashMap;
/// use chrono::Utc;
///
/// let analysis = CodeChurnAnalysis {
///     generated_at: Utc::now(),
///     period_days: 7,
///     repository_root: PathBuf::from("/repo"),
///     files: vec![],
///     summary: ChurnSummary {
///         total_commits: 25,
///         total_files_changed: 12,
///         hotspot_files: vec![],
///         stable_files: vec![],
///         author_contributions: HashMap::new(),
///     },
/// };
///
/// let markdown = format_churn_as_markdown(&analysis);
/// assert!(markdown.contains("# Code Churn Analysis Report"));
/// assert!(markdown.contains("**Period:** 7 days"));
/// ```
#[must_use]
pub fn format_churn_as_markdown(analysis: &crate::models::churn::CodeChurnAnalysis) -> String {
    let mut output = String::with_capacity(1024);

    output.push_str("# Code Churn Analysis Report\n\n");
    output.push_str(&format!(
        "**Generated:** {}\n",
        analysis.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
    ));
    output.push_str(&format!(
        "**Repository:** {}\n",
        analysis.repository_root.display()
    ));
    output.push_str(&format!("**Period:** {} days\n\n", analysis.period_days));

    output.push_str("## Summary\n\n");
    output.push_str(&format!(
        "- Total files changed: {}\n",
        analysis.summary.total_files_changed
    ));
    output.push_str(&format!(
        "- Total commits: {}\n",
        analysis.summary.total_commits
    ));
    output.push_str(&format!(
        "- Unique contributors: {}\n\n",
        analysis.summary.author_contributions.len()
    ));

    output.push_str("## Top 10 Files by Churn Score\n\n");
    output.push_str("| File | Commits | Changes | Churn Score | Authors |\n");
    output.push_str("|------|---------|---------|-------------|----------|\n");

    for file in analysis.files.iter().take(10) {
        output.push_str(&format!(
            "| {} | {} | +{} -{}  | {:.2} | {} |\n",
            file.relative_path,
            file.commit_count,
            file.additions,
            file.deletions,
            file.churn_score,
            file.unique_authors.len()
        ));
    }

    output
}

/// Formats a code churn analysis as CSV data
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::handlers::tools::format_churn_as_csv;
/// use pmat::models::churn::{CodeChurnAnalysis, ChurnSummary, FileChurnMetrics};
/// use std::path::PathBuf;
/// use std::collections::HashMap;
/// use chrono::Utc;
///
/// let analysis = CodeChurnAnalysis {
///     generated_at: Utc::now(),
///     period_days: 30,
///     repository_root: PathBuf::from("/repo"),
///     files: vec![FileChurnMetrics {
///         path: PathBuf::from("/repo/src/main.rs"),
///         relative_path: "src/main.rs".to_string(),
///         commit_count: 5,
///         unique_authors: vec![],
///         additions: 100,
///         deletions: 50,
///         churn_score: 0.75,
///         last_modified: Utc::now(),
///         first_seen: Utc::now(),
///     }],
///     summary: ChurnSummary {
///         total_commits: 5,
///         total_files_changed: 1,
///         hotspot_files: vec![],
///         stable_files: vec![],
///         author_contributions: HashMap::new(),
///     },
/// };
///
/// let csv = format_churn_as_csv(&analysis);
/// assert!(csv.starts_with("file_path,commits,additions,deletions,churn_score,unique_authors,last_modified"));
/// assert!(csv.contains("src/main.rs,5,100,50,0.750,0"));
/// ```
#[must_use]
pub fn format_churn_as_csv(analysis: &crate::models::churn::CodeChurnAnalysis) -> String {
    let mut output = String::with_capacity(1024);

    output.push_str(
        "file_path,commits,additions,deletions,churn_score,unique_authors,last_modified\n",
    );

    for file in &analysis.files {
        output.push_str(&format!(
            "{},{},{},{},{:.3},{},{}\n",
            file.relative_path,
            file.commit_count,
            file.additions,
            file.deletions,
            file.churn_score,
            file.unique_authors.len(),
            file.last_modified.format("%Y-%m-%d")
        ));
    }

    output
}
