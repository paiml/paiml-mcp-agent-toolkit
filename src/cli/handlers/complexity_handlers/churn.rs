//! Churn-related complexity analysis.

use anyhow::Result;
use std::path::PathBuf;

/// Handle churn analysis command
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_churn(
    project_path: PathBuf,
    days: u32,
    format: crate::models::churn::ChurnOutputFormat,
    output: Option<PathBuf>,
    top_files: usize,
    include: Vec<String>,
    exclude: Vec<String>,
) -> Result<()> {
    use crate::services::git_analysis::GitAnalysisService;

    eprintln!("📊 Analyzing code churn for the last {days} days...");

    // Create and apply file filters
    let filter = create_and_report_file_filter(include, exclude)?;

    // Analyze code churn
    let mut analysis = GitAnalysisService::analyze_code_churn(&project_path, days)
        .map_err(|e| anyhow::anyhow!("Churn analysis failed: {e}"))?;

    // Apply filtering and limits
    apply_churn_filters(&mut analysis, &filter, top_files);

    eprintln!("✅ Analyzed {} files with changes", analysis.files.len());

    // Format and write output
    format_and_write_churn_output(analysis, format, output).await
}

/// Create file filter and report filter settings
fn create_and_report_file_filter(
    include: Vec<String>,
    exclude: Vec<String>,
) -> Result<crate::utils::file_filter::FileFilter> {
    if !include.is_empty() || !exclude.is_empty() {
        eprintln!("🔍 Applying file filters...");
        if !include.is_empty() {
            eprintln!("  Include patterns: {include:?}");
        }
        if !exclude.is_empty() {
            eprintln!("  Exclude patterns: {exclude:?}");
        }
    }

    crate::utils::file_filter::FileFilter::new(include, exclude)
}

/// Apply file filters and top files limit to churn analysis
fn apply_churn_filters(
    analysis: &mut crate::models::churn::CodeChurnAnalysis,
    filter: &crate::utils::file_filter::FileFilter,
    top_files: usize,
) {
    // Apply file filter if filters are active
    if filter.has_filters() {
        analysis
            .files
            .retain(|file| filter.should_include(&file.path));

        // Update summary
        analysis.summary.total_files_changed = analysis.files.len();
        analysis.summary.total_commits = analysis.files.iter().map(|f| f.commit_count).sum();
    }

    // Apply top_files limit if specified (0 means show all)
    if top_files > 0 && analysis.files.len() > top_files {
        analysis
            .files
            .sort_by_key(|b| std::cmp::Reverse(b.commit_count));
        analysis.files.truncate(top_files);
    }
}

/// Format churn analysis output and write to file or stdout
async fn format_and_write_churn_output(
    analysis: crate::models::churn::CodeChurnAnalysis,
    format: crate::models::churn::ChurnOutputFormat,
    output: Option<PathBuf>,
) -> Result<()> {
    use crate::models::churn::ChurnOutputFormat;

    let content = match format {
        ChurnOutputFormat::Json => serde_json::to_string_pretty(&analysis)?,
        ChurnOutputFormat::Summary => {
            crate::cli::analysis_utilities::format_churn_as_summary(&analysis)?
        }
        ChurnOutputFormat::Markdown => {
            crate::cli::analysis_utilities::format_churn_as_markdown(&analysis)?
        }
        ChurnOutputFormat::Csv => crate::cli::analysis_utilities::format_churn_as_csv(&analysis)?,
    };

    crate::cli::analysis_utilities::write_churn_output(content, output).await
}
