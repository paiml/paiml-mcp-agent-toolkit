// Churn handlers - extracted for file health (CB-040)

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_churn(
    project_path: PathBuf,
    days: u32,
    format: crate::models::churn::ChurnOutputFormat,
    output: Option<PathBuf>,
    top_files: usize,
) -> Result<()> {
    use crate::services::git_analysis::GitAnalysisService;

    eprintln!("📊 Analyzing code churn for the last {days} days...");

    // Analyze code churn
    let mut analysis = GitAnalysisService::analyze_code_churn(&project_path, days)
        .map_err(|e| anyhow::anyhow!("Churn analysis failed: {e}"))?;

    eprintln!("✅ Analyzed {} files with changes", analysis.files.len());

    // Apply filtering and sorting to analysis results
    apply_churn_file_filtering(&mut analysis, top_files);

    // Format and write output
    let content = format_churn_content(&analysis, format)?;
    write_churn_output(content, output).await?;
    Ok(())
}

// Helper function to format churn analysis as JSON
fn format_churn_as_json(analysis: &crate::models::churn::CodeChurnAnalysis) -> Result<String> {
    Ok(serde_json::to_string_pretty(analysis)?)
}

/// Format churn analysis as summary with top files display
///
/// # Examples
///
/// ```no_run
/// use pmat::models::churn::*;
/// use chrono::Utc;
/// use std::path::{Path, PathBuf};
///
/// let analysis = CodeChurnAnalysis {
///     generated_at: Utc::now(),
///     period_days: 30,
///     repository_root: PathBuf::from("."),
///     files: vec![
///         FileChurnMetrics {
///             path: PathBuf::from("src/main.rs"),
///             relative_path: "src/main.rs".to_string(),
///             commit_count: 15,
///             unique_authors: vec!["dev1".to_string(), "dev2".to_string()],
///             additions: 100,
///             deletions: 50,
///             churn_score: 0.75,
///             last_modified: Utc::now(),
///             first_seen: Utc::now(),
///         },
///         FileChurnMetrics {
///             path: PathBuf::from("src/lib.rs"),
///             relative_path: "src/lib.rs".to_string(),
///             commit_count: 8,
///             unique_authors: vec!["dev1".to_string()],
///             additions: 60,
///             deletions: 20,
///             churn_score: 0.45,
///             last_modified: Utc::now(),
///             first_seen: Utc::now(),
///         },
///     ],
///     summary: ChurnSummary {
///         total_commits: 23,
///         total_files_changed: 2,
///         hotspot_files: vec![PathBuf::from("src/main.rs")],
///         stable_files: vec![PathBuf::from("src/lib.rs")],
///         author_contributions: [("dev1".to_string(), 15), ("dev2".to_string(), 8)].iter().cloned().collect(),
///         mean_churn_score: 0.6,
///         variance_churn_score: 0.0225,
///         stddev_churn_score: 0.15,
///     },
/// };
///
/// // Testing that the data structure compiles correctly
/// assert!(analysis.files.len() == 2);
/// assert_eq!(analysis.period_days, 30);
/// assert_eq!(analysis.summary.total_files_changed, 2);
/// ```
// Helper function to format churn analysis as summary
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_churn_as_summary(
    analysis: &crate::models::churn::CodeChurnAnalysis,
) -> Result<String> {
    let mut output = String::new();

    write_summary_header(&mut output, analysis)?;
    write_summary_top_files(&mut output, analysis)?;
    write_summary_hotspot_files(&mut output, &analysis.summary)?;
    write_summary_stable_files(&mut output, &analysis.summary)?;
    write_summary_top_contributors(&mut output, &analysis.summary)?;

    Ok(output)
}

// Helper function to write summary header
fn write_summary_header(
    output: &mut String,
    analysis: &crate::models::churn::CodeChurnAnalysis,
) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;

    writeln!(output, "{}{}Code Churn Analysis Summary{}\n", c::BOLD, c::UNDERLINE, c::RESET)?;
    writeln!(output, "  {}Period:{} {}{}{}", c::BOLD, c::RESET, c::BOLD_WHITE, analysis.period_days, c::RESET)?;
    writeln!(
        output,
        "  {}Total commits:{} {}{}{}",
        c::BOLD, c::RESET, c::BOLD_WHITE, analysis.summary.total_commits, c::RESET
    )?;
    writeln!(
        output,
        "  {}Files changed:{} {}{}{}",
        c::BOLD, c::RESET, c::BOLD_WHITE, analysis.summary.total_files_changed, c::RESET
    )?;
    Ok(())
}

// Helper function to write top files by churn
fn write_summary_top_files(
    output: &mut String,
    analysis: &crate::models::churn::CodeChurnAnalysis,
) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;

    if !analysis.files.is_empty() {
        writeln!(output, "\n{}Top Files by Churn{}\n", c::BOLD, c::RESET)?;

        // Sort files by churn score or commit count (descending)
        let mut sorted_files: Vec<_> = analysis.files.iter().collect();
        sorted_files.sort_unstable_by(|a, b| {
            // Primary sort by commit count, secondary by churn score
            match b.commit_count.cmp(&a.commit_count) {
                std::cmp::Ordering::Equal => b
                    .churn_score
                    .partial_cmp(&a.churn_score)
                    .unwrap_or(std::cmp::Ordering::Equal),
                other => other,
            }
        });

        for (i, file) in sorted_files.iter().take(10).enumerate() {
            let filename = file
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&file.relative_path);
            let score_color = if file.churn_score > 0.5 {
                c::RED
            } else if file.churn_score > 0.3 {
                c::YELLOW
            } else {
                c::GREEN
            };
            writeln!(
                output,
                "  {}. {}{}{} - {}{}{} commits, {} authors, score: {}{:.2}{}",
                i + 1,
                c::CYAN, filename, c::RESET,
                c::BOLD_WHITE, file.commit_count, c::RESET,
                file.unique_authors.len(),
                score_color, file.churn_score, c::RESET
            )?;
        }
    }
    Ok(())
}

// Helper function to write hotspot files
fn write_summary_hotspot_files(
    output: &mut String,
    summary: &crate::models::churn::ChurnSummary,
) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;

    if !summary.hotspot_files.is_empty() {
        writeln!(output, "\n{}Hotspot Files (High Churn){}\n", c::BOLD, c::RESET)?;
        for (i, file) in summary.hotspot_files.iter().take(10).enumerate() {
            writeln!(output, "  {}. {}{}{}", i + 1, c::CYAN, file.display(), c::RESET)?;
        }
    }
    Ok(())
}

// Helper function to write stable files
fn write_summary_stable_files(
    output: &mut String,
    summary: &crate::models::churn::ChurnSummary,
) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;

    if !summary.stable_files.is_empty() {
        writeln!(output, "\n{}Stable Files (Low Churn){}\n", c::BOLD, c::RESET)?;
        for (i, file) in summary.stable_files.iter().take(10).enumerate() {
            writeln!(output, "  {}. {}{}{}", i + 1, c::CYAN, file.display(), c::RESET)?;
        }
    }
    Ok(())
}

// Helper function to write top contributors
fn write_summary_top_contributors(
    output: &mut String,
    summary: &crate::models::churn::ChurnSummary,
) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;

    if !summary.author_contributions.is_empty() {
        writeln!(output, "\n{}Top Contributors{}\n", c::BOLD, c::RESET)?;
        let mut authors: Vec<_> = summary.author_contributions.iter().collect();
        authors.sort_unstable_by(|a, b| b.1.cmp(a.1));
        for (author, files) in authors.iter().take(10) {
            writeln!(output, "  {}{}{}: {}{}{} files", c::CYAN, author, c::RESET, c::BOLD_WHITE, files, c::RESET)?;
        }
    }
    Ok(())
}

// Helper function to format churn analysis as markdown
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
/// Format churn as markdown.
pub fn format_churn_as_markdown(
    analysis: &crate::models::churn::CodeChurnAnalysis,
) -> Result<String> {
    let mut output = String::new();

    write_markdown_header(&mut output, analysis)?;
    write_markdown_summary_table(&mut output, &analysis.summary)?;
    write_markdown_file_details(&mut output, &analysis.files)?;
    write_markdown_author_contributions(&mut output, &analysis.summary)?;
    write_markdown_recommendations(&mut output)?;

    Ok(output)
}

// Helper function to write markdown header
fn write_markdown_header(
    output: &mut String,
    analysis: &crate::models::churn::CodeChurnAnalysis,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "# Code Churn Analysis Report\n")?;
    writeln!(
        output,
        "Generated: {}",
        analysis.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
    )?;
    writeln!(output, "Repository: {}", analysis.repository_root.display())?;
    writeln!(output, "Analysis Period: {} days\n", analysis.period_days)?;
    Ok(())
}

// Helper function to write markdown summary table
fn write_markdown_summary_table(
    output: &mut String,
    summary: &crate::models::churn::ChurnSummary,
) -> Result<()> {
    write_markdown_table_header(output)?;
    write_summary_data_rows(output, summary)?;
    Ok(())
}

/// Write the markdown table header for summary statistics
fn write_markdown_table_header(output: &mut String) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "## Summary Statistics\n")?;
    writeln!(output, "| Metric | Value |")?;
    writeln!(output, "|--------|-------|")?;
    Ok(())
}

/// Write all summary data rows to the markdown table
fn write_summary_data_rows(
    output: &mut String,
    summary: &crate::models::churn::ChurnSummary,
) -> Result<()> {
    write_commits_row(output, summary.total_commits)?;
    write_files_changed_row(output, summary.total_files_changed)?;
    write_hotspot_files_row(output, summary.hotspot_files.len())?;
    write_stable_files_row(output, summary.stable_files.len())?;
    write_authors_row(output, summary.author_contributions.len())?;
    Ok(())
}

/// Write total commits row
fn write_commits_row(output: &mut String, total_commits: usize) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "| Total Commits | {total_commits} |")?;
    Ok(())
}

/// Write files changed row
fn write_files_changed_row(output: &mut String, files_changed: usize) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "| Files Changed | {files_changed} |")?;
    Ok(())
}

/// Write hotspot files row
fn write_hotspot_files_row(output: &mut String, hotspot_count: usize) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "| Hotspot Files | {hotspot_count} |")?;
    Ok(())
}

/// Write stable files row
fn write_stable_files_row(output: &mut String, stable_count: usize) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "| Stable Files | {stable_count} |")?;
    Ok(())
}

/// Write contributing authors row
fn write_authors_row(output: &mut String, author_count: usize) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "| Contributing Authors | {author_count} |")?;
    Ok(())
}

// Helper function to write markdown file details
fn write_markdown_file_details(
    output: &mut String,
    files: &[crate::models::churn::FileChurnMetrics],
) -> Result<()> {
    use std::fmt::Write;

    if !files.is_empty() {
        writeln!(output, "\n## File Churn Details\n")?;
        writeln!(
            output,
            "| File | Commits | Authors | Additions | Deletions | Churn Score | Last Modified |"
        )?;
        writeln!(
            output,
            "|------|---------|---------|-----------|-----------|-------------|----------------|"
        )?;

        // Sort by churn score descending
        let mut sorted_files = files.to_vec();
        sorted_files.sort_unstable_by(|a, b| {
            b.churn_score
                .partial_cmp(&a.churn_score)
                .expect("NaN values should not occur in churn scores")
        });

        for file in sorted_files.iter().take(20) {
            writeln!(
                output,
                "| {} | {} | {} | {} | {} | {:.2} | {} |",
                file.relative_path,
                file.commit_count,
                file.unique_authors.len(),
                file.additions,
                file.deletions,
                file.churn_score,
                file.last_modified.format("%Y-%m-%d")
            )?;
        }
    }
    Ok(())
}

// Helper function to write markdown author contributions
fn write_markdown_author_contributions(
    output: &mut String,
    summary: &crate::models::churn::ChurnSummary,
) -> Result<()> {
    use std::fmt::Write;

    if !summary.author_contributions.is_empty() {
        writeln!(output, "\n## Author Contributions\n")?;
        writeln!(output, "| Author | Files Modified |")?;
        writeln!(output, "|--------|----------------|")?;

        let mut authors: Vec<_> = summary.author_contributions.iter().collect();
        authors.sort_unstable_by(|a, b| b.1.cmp(a.1));

        for (author, count) in authors.iter().take(15) {
            writeln!(output, "| {author} | {count} |")?;
        }
    }
    Ok(())
}

// Helper function to write markdown recommendations
fn write_markdown_recommendations(output: &mut String) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "\n## Recommendations\n")?;
    writeln!(
        output,
        "1. **Review Hotspot Files**: Files with high churn scores may benefit from refactoring"
    )?;
    writeln!(
        output,
        "2. **Add Tests**: High-churn files should have comprehensive test coverage"
    )?;
    writeln!(
        output,
        "3. **Code Review**: Frequently modified files may indicate design issues"
    )?;
    writeln!(
        output,
        "4. **Documentation**: Document the reasons for frequent changes in hotspot files"
    )?;
    Ok(())
}

// Helper function to format churn analysis as CSV
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
/// Format churn as csv.
pub fn format_churn_as_csv(analysis: &crate::models::churn::CodeChurnAnalysis) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();

    writeln!(&mut output, "file_path,relative_path,commit_count,unique_authors,additions,deletions,churn_score,last_modified,first_seen")?;

    for file in &analysis.files {
        writeln!(
            &mut output,
            "{},{},{},{},{},{},{:.3},{},{}",
            file.path.display(),
            file.relative_path,
            file.commit_count,
            file.unique_authors.len(),
            file.additions,
            file.deletions,
            file.churn_score,
            file.last_modified.to_rfc3339(),
            file.first_seen.to_rfc3339()
        )?;
    }

    Ok(output)
}

// Helper function to write output
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn write_churn_output(content: String, output: Option<PathBuf>) -> Result<()> {
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("✅ Churn analysis written to: {}", output_path.display());
    } else {
        println!("{content}");
    }
    Ok(())
}

// Helper functions for handle_analyze_churn
// Toyota Way Extract Method: Reduce complexity by separating filtering and formatting logic

/// Applies file filtering and sorting to churn analysis results
/// Toyota Way: Extract Method - reduce complexity by extracting file processing logic
fn apply_churn_file_filtering(
    analysis: &mut crate::models::churn::CodeChurnAnalysis,
    top_files: usize,
) {
    // Apply top_files limit if specified (0 means show all)
    if top_files > 0 && analysis.files.len() > top_files {
        // Sort files by commit count descending
        analysis
            .files
            .sort_unstable_by_key(|b| std::cmp::Reverse(b.commit_count));
        analysis.files.truncate(top_files);
    }
}

/// Formats churn analysis based on requested format
/// Toyota Way: Extract Method - reduce complexity by extracting format selection logic
fn format_churn_content(
    analysis: &crate::models::churn::CodeChurnAnalysis,
    format: crate::models::churn::ChurnOutputFormat,
) -> Result<String> {
    use crate::models::churn::ChurnOutputFormat;

    match format {
        ChurnOutputFormat::Json => format_churn_as_json(analysis),
        ChurnOutputFormat::Summary => format_churn_as_summary(analysis),
        ChurnOutputFormat::Markdown => format_churn_as_markdown(analysis),
        ChurnOutputFormat::Csv => format_churn_as_csv(analysis),
    }
}

