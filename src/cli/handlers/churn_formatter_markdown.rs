/// Toyota Way: Single Responsibility - Format churn analysis as markdown
/// Extracted from stubs.rs to reduce complexity and improve maintainability
///
/// # Parameters
///
/// * `summary` - Churn analysis summary data
///
/// # Returns
///
/// * `Ok(String)` - Formatted markdown output
/// * `Err(anyhow::Error)` - Formatting failed
pub fn format_churn_markdown(analysis: &crate::models::churn::CodeChurnAnalysis) -> Result<String> {
    let mut output = String::new();

    // Header
    writeln!(&mut output, "# Code Churn Analysis Report\n")?;
    writeln!(
        &mut output,
        "Generated: {}",
        analysis.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
    )?;
    writeln!(
        &mut output,
        "Repository: {}",
        analysis.repository_root.display()
    )?;
    writeln!(
        &mut output,
        "Analysis Period: {} days\n",
        analysis.period_days
    )?;

    // Summary statistics table
    write_markdown_summary_table(&mut output, &analysis.summary)?;

    // File details if available
    write_markdown_file_details(&mut output, &analysis.files)?;

    // Author contributions if available
    if !analysis.summary.author_contributions.is_empty() {
        write_author_contributions(&mut output, &analysis.summary)?;
    }

    Ok(output)
}

/// Toyota Way: Extract Method - Write markdown summary table (complexity ≤8)
/// Creates a summary statistics table in markdown format
pub fn write_markdown_summary_table(
    output: &mut String,
    summary: &crate::models::churn::ChurnSummary,
) -> Result<()> {
    writeln!(output, "## Summary Statistics\n")?;
    writeln!(output, "| Metric | Value |")?;
    writeln!(output, "|--------|-------|")?;
    writeln!(output, "| Total Commits | {} |", summary.total_commits)?;
    writeln!(
        output,
        "| Files Changed | {} |",
        summary.total_files_changed
    )?;
    writeln!(
        output,
        "| Hotspot Files | {} |",
        summary.hotspot_files.len()
    )?;
    writeln!(output, "| Stable Files | {} |", summary.stable_files.len())?;
    writeln!(
        output,
        "| Contributing Authors | {} |",
        summary.author_contributions.len()
    )?;
    Ok(())
}

/// Toyota Way: Extract Method - Write markdown file details (complexity ≤8)
fn write_markdown_file_details(
    output: &mut String,
    files: &[crate::models::churn::FileChurnMetrics],
) -> Result<()> {
    if files.is_empty() {
        return Ok(());
    }

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
    sorted_files.sort_by(|a, b| {
        b.churn_score
            .partial_cmp(&a.churn_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Write top 20 files
    for file in sorted_files.iter().take(20) {
        write_file_row(output, file)?;
    }

    Ok(())
}

/// Toyota Way: Extract Method - Write single file row (complexity ≤3)
fn write_file_row(
    output: &mut String,
    file: &crate::models::churn::FileChurnMetrics,
) -> Result<()> {
    writeln!(
        output,
        "| {} | {} | {} | {} | {} | {:.2} | {} |",
        file.relative_path,
        file.commit_count,
        file.unique_authors.len(),
        file.additions,
        file.deletions,
        file.churn_score,
        file.last_modified.format("%Y-%m-%d"),
    )?;
    Ok(())
}

/// Toyota Way: Extract Method - Write author contributions (complexity ≤8)
fn write_author_contributions(
    output: &mut String,
    summary: &crate::models::churn::ChurnSummary,
) -> Result<()> {
    writeln!(output, "\n## Author Contributions\n")?;
    writeln!(output, "| Author | Files Modified |")?;
    writeln!(output, "|--------|----------------|")?;

    // Sort by file count descending
    let mut sorted_authors: Vec<_> = summary.author_contributions.iter().collect();
    sorted_authors.sort_by(|a, b| b.1.cmp(a.1));

    // Write top 15 authors
    for (author, count) in sorted_authors.iter().take(15) {
        writeln!(output, "| {author} | {count} |")?;
    }

    Ok(())
}
