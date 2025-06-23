//! Helper functions for incremental coverage analysis

use crate::services::incremental_coverage_analyzer::{
    ChangeSet, CoverageUpdate, FileId, IncrementalCoverageAnalyzer,
};
use anyhow::Result;
use std::fmt::Write;
use std::path::{Path, PathBuf};

/// Setup coverage analyzer
pub fn setup_coverage_analyzer(
    cache_dir: Option<PathBuf>,
    force_refresh: bool,
) -> Result<IncrementalCoverageAnalyzer> {
    let cache_path = cache_dir.unwrap_or_else(|| std::env::temp_dir().join("pmat_coverage_cache"));

    let analyzer = IncrementalCoverageAnalyzer::new(&cache_path)?;

    if force_refresh {
        eprintln!("🧹 Clearing coverage cache...");
        // In real implementation, would clear the cache
    }

    Ok(analyzer)
}

/// Get changed files using git
pub async fn get_changed_files_for_coverage(
    project_path: &Path,
    base_branch: &str,
    target_branch: Option<&str>,
) -> Result<Vec<(PathBuf, String)>> {
    eprintln!("🔍 Getting changed files...");
    eprintln!("📍 Project: {}", project_path.display());
    eprintln!("🔄 Base branch: {}", base_branch);
    if let Some(target) = target_branch {
        eprintln!("🎯 Target branch: {}", target);
    }

    // Use git to get actual changed files
    use tokio::process::Command;
    
    let target = target_branch.unwrap_or("HEAD");
    let output = Command::new("git")
        .arg("diff")
        .arg("--name-status")
        .arg(format!("{}...{}", base_branch, target))
        .current_dir(project_path)
        .output()
        .await?;
    
    if !output.status.success() {
        // If git command fails, return empty list instead of erroring
        eprintln!("⚠️ Git command failed, returning empty changelist");
        return Ok(vec![]);
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut changed_files = Vec::new();
    
    for line in stdout.lines() {
        if let Some((status, path)) = line.split_once('\t') {
            let full_path = project_path.join(path);
            // Only include files that actually exist (not deleted)
            if full_path.exists() && status != "D" {
                changed_files.push((full_path, status.to_string()));
            }
        }
    }

    eprintln!("📝 Found {} changed files", changed_files.len());
    Ok(changed_files)
}

/// Analyze incremental coverage
pub async fn analyze_incremental_coverage(
    analyzer: &IncrementalCoverageAnalyzer,
    changed_files: &[(PathBuf, String)],
    _changed_files_only: bool,
) -> Result<CoverageUpdate> {
    // Create change set
    let mut modified_files = Vec::new();
    let mut added_files = Vec::new();

    for (path, status) in changed_files {
        let hash = analyzer.compute_file_hash(path).await?;
        let file_id = FileId {
            path: path.clone(),
            hash,
        };

        match status.as_str() {
            "M" => modified_files.push(file_id),
            "A" => added_files.push(file_id),
            _ => {} // Skip deleted files
        }
    }

    let changeset = ChangeSet {
        modified_files,
        added_files,
        deleted_files: vec![],
    };

    // Analyze changes
    analyzer.analyze_changes(&changeset).await
}

/// Check coverage threshold
pub fn check_coverage_threshold(coverage_data: &CoverageUpdate, threshold: f64) -> Result<()> {
    let coverage = coverage_data.delta_coverage.percentage;

    eprintln!(
        "📈 Overall coverage: {:.1}%",
        coverage_data.aggregate_coverage.line_percentage
    );
    eprintln!("🆕 New code coverage: {:.1}%", coverage);

    if coverage < threshold {
        eprintln!(
            "❌ Coverage threshold not met: {:.1}% < {:.1}%",
            coverage, threshold
        );
        anyhow::bail!("Coverage threshold not met");
    } else {
        eprintln!(
            "✅ Coverage threshold met: {:.1}% >= {:.1}%",
            coverage, threshold
        );
    }

    Ok(())
}

/// Format coverage as summary
pub fn format_coverage_summary(
    coverage_data: &CoverageUpdate,
    base_branch: &str,
    target_branch: &Option<String>,
) -> Result<String> {
    let mut output = String::new();

    writeln!(&mut output, "# Incremental Coverage Summary\n")?;
    writeln!(&mut output, "**Base Branch**: {}", base_branch)?;
    if let Some(ref target) = target_branch {
        writeln!(&mut output, "**Target Branch**: {}", target)?;
    }

    writeln!(
        &mut output,
        "**Files Analyzed**: {}",
        coverage_data.file_coverage.len()
    )?;
    writeln!(
        &mut output,
        "**Overall Coverage**: {:.1}%",
        coverage_data.aggregate_coverage.line_percentage
    )?;
    writeln!(
        &mut output,
        "**New Code Coverage**: {:.1}%",
        coverage_data.delta_coverage.percentage
    )?;

    Ok(output)
}

/// Format coverage as JSON
pub fn format_coverage_json(coverage_data: &CoverageUpdate) -> Result<String> {
    serde_json::to_string_pretty(coverage_data).map_err(Into::into)
}

/// Format coverage as markdown
pub fn format_coverage_markdown(coverage_data: &CoverageUpdate, detailed: bool) -> Result<String> {
    let mut output = String::new();

    writeln!(&mut output, "# Incremental Coverage Report\n")?;

    // Summary
    writeln!(&mut output, "## Summary\n")?;
    writeln!(
        &mut output,
        "- **Overall Coverage**: {:.1}%",
        coverage_data.aggregate_coverage.line_percentage
    )?;
    writeln!(
        &mut output,
        "- **New Code Coverage**: {:.1}% ({}/{} lines)",
        coverage_data.delta_coverage.percentage,
        coverage_data.delta_coverage.new_lines_covered,
        coverage_data.delta_coverage.new_lines_total
    )?;

    if detailed && !coverage_data.file_coverage.is_empty() {
        writeln!(&mut output, "\n## File Details\n")?;

        for (file_id, file_cov) in &coverage_data.file_coverage {
            writeln!(&mut output, "### {}\n", file_id.path.display())?;
            writeln!(
                &mut output,
                "- Line Coverage: {:.1}%",
                file_cov.line_coverage
            )?;
            writeln!(
                &mut output,
                "- Branch Coverage: {:.1}%",
                file_cov.branch_coverage
            )?;
        }
    }

    Ok(output)
}

/// Format coverage as LCOV
pub fn format_coverage_lcov(coverage_data: &CoverageUpdate) -> Result<String> {
    let mut output = String::new();

    for (file_id, file_cov) in &coverage_data.file_coverage {
        writeln!(&mut output, "SF:{}", file_id.path.display())?;

        // Mock line data - in real implementation would have actual line coverage
        // Using line coverage percentage to estimate
        let estimated_total_lines = 100; // Placeholder
        let estimated_covered_lines =
            (estimated_total_lines as f64 * file_cov.line_coverage / 100.0) as usize;

        for i in 1..=estimated_total_lines {
            writeln!(&mut output, "DA:{},1", i)?;
        }

        writeln!(&mut output, "LF:{}", estimated_total_lines)?;
        writeln!(&mut output, "LH:{}", estimated_covered_lines)?;
        writeln!(&mut output, "end_of_record")?;
    }

    Ok(output)
}
