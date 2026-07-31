#![cfg_attr(coverage_nightly, coverage(off))]
//! TDG output formatting: score, comparison, and output helpers
//!
//! Handles formatting of TDG scores and comparisons for various output
//! formats (Table, JSON, Markdown, SARIF).

use super::{format_grade, TdgCommandConfig};
use crate::cli::colors as c;
use crate::cli::TdgOutputFormat;
use anyhow::Result;

/// Format TDG output based on config (cognitive complexity ≤3)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn format_tdg_output(
    score: &crate::tdg::TdgScore,
    git_context: Option<&crate::models::git_context::GitContext>,
    config: &TdgCommandConfig,
) -> Result<String> {
    if config.quiet {
        Ok(format!("{:.1}", score.total))
    } else {
        format_tdg_score(
            score.clone(),
            git_context,
            config.format.clone(),
            config.include_components,
        )
    }
}

/// Write TDG output to file or stdout (cognitive complexity ≤3)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn write_tdg_output(output_str: &str, config: &TdgCommandConfig) -> Result<()> {
    if let Some(output_path) = &config.output {
        std::fs::write(output_path, output_str)?;
    } else {
        println!("{output_str}");
    }
    Ok(())
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
pub(crate) fn format_tdg_score(
    score: crate::tdg::TdgScore,
    git_context: Option<&crate::models::git_context::GitContext>,
    format: TdgOutputFormat,
    include_components: bool,
) -> Result<String> {
    match format {
        TdgOutputFormat::Table => format_tdg_score_table(&score, git_context, include_components),
        TdgOutputFormat::Json => format_tdg_score_json(&score, git_context, include_components),
        TdgOutputFormat::Markdown => {
            format_tdg_score_markdown(&score, git_context, include_components)
        }
        // Issue #669: this arm used to `return format!("{:.1}", score.total)`,
        // so `tdg --format sarif` emitted exactly `84.0\n` — the same 5 bytes
        // as `tdg -q`. A SARIF uploader fed that gets a scalar. Reuse the
        // SARIF 2.1.0 emitter that `analyze tdg --format sarif` already uses.
        TdgOutputFormat::Sarif => Ok(serde_json::to_string_pretty(
            &crate::cli::handlers::new_tdg_handler::create_file_sarif_output(&score),
        )?),
    }
}

/// Emit SARIF for the top-level `tdg` command (issue #669).
///
/// A directory is re-scored per file so every SARIF result names a real source
/// file; formatting the project *average* would have produced a single finding
/// with no `file_path` (rendered as the invented uri `"unknown"`).
pub(crate) async fn emit_tdg_sarif(
    analyzer: &crate::tdg::TdgAnalyzer,
    config: &TdgCommandConfig,
) -> Result<()> {
    use crate::cli::handlers::new_tdg_handler::{create_file_sarif_output, create_sarif_output};

    let (sarif, score) = if config.path.is_dir() {
        let project = analyzer.analyze_project(&config.path).await?;
        let average = project.average();
        (create_sarif_output(&project), average)
    } else {
        let score = analyzer.analyze_file(&config.path).await?;
        (create_file_sarif_output(&score), score)
    };

    super::quality_gates::validate_minimum_grade(&score, config)?;
    write_tdg_output(&serde_json::to_string_pretty(&sarif)?, config)
}

/// Format TDG score as table
fn format_tdg_score_table(
    score: &crate::tdg::TdgScore,
    git_context: Option<&crate::models::git_context::GitContext>,
    include_components: bool,
) -> Result<String> {
    let mut output = String::new();

    // Header
    output.push_str("╭─────────────────────────────────────────────────╮\n");
    if let Some(file_path) = &score.file_path {
        output.push_str(&format!(
            "│  TDG Score Report: {}              │\n",
            c::path(&file_path.display().to_string())
        ));
    } else {
        output.push_str("│  TDG Score Report                              │\n");
    }
    output.push_str("├─────────────────────────────────────────────────┤\n");

    // Overall score
    let grade_str = format_grade(score.grade);
    output.push_str(&format!(
        "│  Overall Score: {}/100 ({})                  │\n",
        c::number(&format!("{:.1}", score.total)),
        c::grade(&grade_str)
    ));
    output.push_str(&format!(
        "│  Language: {:?} (confidence: {}%)             │\n",
        score.language,
        c::number(&format!("{:.0}", score.confidence * 100.0))
    ));

    // Sprint 65: Git context (if available)
    if let Some(git) = git_context {
        output.push_str("│                                                 │\n");
        output.push_str("│  🔗 Git Context:                                │\n");
        output.push_str(&format!(
            "│  ├─ Commit:  {}                     │\n",
            c::number(&git.commit_sha_short)
        ));
        output.push_str(&format!(
            "│  ├─ Branch:  {}                               │\n",
            c::path(&git.branch)
        ));
        output.push_str(&format!(
            "│  └─ Author:  {}                          │\n",
            &git.author_name
        ));
    }

    if include_components {
        output.push_str("│                                                 │\n");
        output.push_str("│  📊 Breakdown:                                  │\n");
        output.push_str(&format!(
            "│  ├─ Structural:     {}/25                    │\n",
            c::score(f64::from(score.structural_complexity), 25.0, 70.0, 40.0)
        ));
        output.push_str(&format!(
            "│  ├─ Semantic:       {}/20                    │\n",
            c::score(f64::from(score.semantic_complexity), 20.0, 70.0, 40.0)
        ));
        output.push_str(&format!(
            "│  ├─ Duplication:    {}/20                    │\n",
            c::score(f64::from(score.duplication_ratio), 20.0, 70.0, 40.0)
        ));
        output.push_str(&format!(
            "│  ├─ Coupling:       {}/15                    │\n",
            c::score(f64::from(score.coupling_score), 15.0, 70.0, 40.0)
        ));
        output.push_str(&format!(
            "│  ├─ Documentation:  {}/10                    │\n",
            c::score(f64::from(score.doc_coverage), 10.0, 70.0, 40.0)
        ));
        output.push_str(&format!(
            "│  └─ Consistency:    {}/10                    │\n",
            c::score(f64::from(score.consistency_score), 10.0, 70.0, 40.0)
        ));
    }

    output.push_str("╰─────────────────────────────────────────────────╯\n");
    Ok(output)
}

/// Format TDG score as JSON
fn format_tdg_score_json(
    score: &crate::tdg::TdgScore,
    git_context: Option<&crate::models::git_context::GitContext>,
    include_components: bool,
) -> Result<String> {
    let json_value = serde_json::json!({
        "file": score.file_path.as_ref().map(|p| p.to_string_lossy().to_string()),
        "language": format!("{:?}", score.language),
        "confidence": score.confidence,
        "score": {
            "total": score.total,
            "grade": format_grade(score.grade),
            "breakdown": if include_components {
                Some(serde_json::json!({
                    "structural_complexity": score.structural_complexity,
                    "semantic_complexity": score.semantic_complexity,
                    "duplication": score.duplication_ratio,
                    "coupling": score.coupling_score,
                    "documentation": score.doc_coverage,
                    "consistency": score.consistency_score,
                }))
            } else {
                None
            }
        },
        "git_context": git_context.map(|git| serde_json::json!({
            "commit_sha": git.commit_sha,
            "commit_sha_short": git.commit_sha_short,
            "branch": git.branch,
            "author_name": git.author_name,
            "author_email": git.author_email,
            "commit_timestamp": git.commit_timestamp.to_rfc3339(),
            "commit_message": git.commit_message,
            "tags": git.tags,
            "is_clean": git.is_clean,
            "uncommitted_files": git.uncommitted_files,
        }))
    });
    Ok(serde_json::to_string_pretty(&json_value)?)
}

/// Format TDG score as Markdown
fn format_tdg_score_markdown(
    score: &crate::tdg::TdgScore,
    _git_context: Option<&crate::models::git_context::GitContext>,
    include_components: bool,
) -> Result<String> {
    let mut output = String::new();

    output.push_str("# TDG Score Report\n\n");
    if let Some(file_path) = &score.file_path {
        output.push_str(&format!("**File**: `{}`\n\n", file_path.display()));
    }

    output.push_str(&format!(
        "**Overall Score**: {:.1}/100 ({})\n",
        score.total,
        format_grade(score.grade)
    ));
    output.push_str(&format!(
        "**Language**: {:?} (confidence: {:.0}%)\n\n",
        score.language,
        score.confidence * 100.0
    ));

    if include_components {
        output.push_str("## Component Breakdown\n\n");
        output.push_str("| Component | Score | Max |\n");
        output.push_str("|-----------|-------|-----|\n");
        output.push_str(&format!(
            "| Structural Complexity | {:.1} | 25 |\n",
            score.structural_complexity
        ));
        output.push_str(&format!(
            "| Semantic Complexity | {:.1} | 20 |\n",
            score.semantic_complexity
        ));
        output.push_str(&format!(
            "| Duplication | {:.1} | 20 |\n",
            score.duplication_ratio
        ));
        output.push_str(&format!(
            "| Coupling | {:.1} | 15 |\n",
            score.coupling_score
        ));
        output.push_str(&format!(
            "| Documentation | {:.1} | 10 |\n",
            score.doc_coverage
        ));
        output.push_str(&format!(
            "| Consistency | {:.1} | 10 |\n",
            score.consistency_score
        ));
    }

    Ok(output)
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn format_comparison(
    comparison: crate::tdg::Comparison,
    format: TdgOutputFormat,
) -> Result<String> {
    if format == TdgOutputFormat::Table {
        let mut output = String::new();
        output.push_str("╭─────────────────────────────────────────────────╮\n");
        output.push_str("│  TDG Comparison                                 │\n");
        output.push_str("├─────────────────────────────────────────────────┤\n");
        let grade1 = format_grade(comparison.source1.grade);
        let grade2 = format_grade(comparison.source2.grade);
        output.push_str(&format!(
            "│  Source 1: {} ({})                           │\n",
            c::number(&format!("{:.1}", comparison.source1.total)),
            c::grade(&grade1)
        ));
        output.push_str(&format!(
            "│  Source 2: {} ({})                           │\n",
            c::number(&format!("{:.1}", comparison.source2.total)),
            c::grade(&grade2)
        ));
        output.push_str(&format!(
            "│  Difference: {}                             │\n",
            c::delta(f64::from(comparison.delta))
        ));

        output.push_str(&format!(
            "│  Winner: {}                                      │\n",
            c::label(&comparison.winner)
        ));

        output.push_str("╰─────────────────────────────────────────────────╯\n");
        Ok(output)
    } else {
        // For other formats, output as JSON
        let json_value = serde_json::json!({
            "source1": {
                "total": comparison.source1.total,
                "grade": format_grade(comparison.source1.grade),
            },
            "source2": {
                "total": comparison.source2.total,
                "grade": format_grade(comparison.source2.grade),
            },
            "difference": comparison.delta,
            "winner": comparison.winner
        });
        Ok(serde_json::to_string_pretty(&json_value)?)
    }
}
