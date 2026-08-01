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

/// Render the one analysis in the declared format (issue #669, second round).
///
/// Every format reads the SAME `TdgAnalysis`, so no two renderers of `pmat tdg`
/// can report different numbers for the same run.
pub(crate) fn format_tdg_analysis(
    analysis: &super::quality_gates::TdgAnalysis,
    git_context: Option<&crate::models::git_context::GitContext>,
    config: &TdgCommandConfig,
) -> Result<String> {
    if matches!(config.format, TdgOutputFormat::Sarif) {
        // `--quiet` does NOT downgrade SARIF to a bare number. That is issue
        // #669 verbatim: `tdg -q --format sarif` wrote `84.0`, which any SARIF
        // uploader rejects. A declared machine format must produce that format.
        return Ok(serde_json::to_string_pretty(&sarif_for(analysis))?);
    }
    format_tdg_output(&analysis.score, git_context, config)
}

/// SARIF for the analysis: project document for a directory, file document for
/// a single file. Both are built from the already-computed scores.
fn sarif_for(analysis: &super::quality_gates::TdgAnalysis) -> serde_json::Value {
    use crate::cli::handlers::new_tdg_handler::{create_file_sarif_output, create_sarif_output};

    match &analysis.project {
        Some(project) => create_sarif_output(project, &analysis.root),
        None => create_file_sarif_output(&analysis.score),
    }
}

/// Format TDG score as table
///
/// Rows are padded from their MEASURED width (colour escapes excluded); the
/// literal space runs this used to carry made the right border drift with the
/// content, e.g. `(A+)` and `(F)` rows closing in different columns.
fn format_tdg_score_table(
    score: &crate::tdg::TdgScore,
    git_context: Option<&crate::models::git_context::GitContext>,
    include_components: bool,
) -> Result<String> {
    use crate::tdg::formatters::boxdraw::{box_blank, box_bottom, box_row, box_separator, box_top};
    let mut output = String::new();
    let mut line = |text: String| {
        output.push_str(&text);
        output.push('\n');
    };

    // Header
    line(box_top());
    match &score.file_path {
        Some(file_path) => line(box_row(&format!(
            "TDG Score Report: {}",
            c::path(&file_path.display().to_string())
        ))),
        None => line(box_row("TDG Score Report")),
    }
    line(box_separator());

    // Overall score
    let grade_str = format_grade(score.grade);
    line(box_row(&format!(
        "Overall Score: {}/100 ({})",
        c::number(&format!("{:.1}", score.total)),
        c::grade(&grade_str)
    )));
    line(box_row(&format!(
        "Language: {:?} (confidence: {}%)",
        score.language,
        c::number(&format!("{:.0}", score.confidence * 100.0))
    )));

    // Sprint 65: Git context (if available)
    if let Some(git) = git_context {
        line(box_blank());
        line(box_row("🔗 Git Context:"));
        line(box_row(&format!(
            "├─ Commit:  {}",
            c::number(&git.commit_sha_short)
        )));
        line(box_row(&format!("├─ Branch:  {}", c::path(&git.branch))));
        line(box_row(&format!("└─ Author:  {}", &git.author_name)));
    }

    if include_components {
        line(box_blank());
        line(box_row("📊 Breakdown:"));
        for (label, value, max) in [
            ("├─ Structural:    ", score.structural_complexity, 25.0),
            ("├─ Semantic:      ", score.semantic_complexity, 20.0),
            ("├─ Duplication:   ", score.duplication_ratio, 20.0),
            ("├─ Coupling:      ", score.coupling_score, 15.0),
            ("├─ Documentation: ", score.doc_coverage, 10.0),
            ("└─ Consistency:   ", score.consistency_score, 10.0),
        ] {
            // c::score already renders "earned/max", so the row must NOT append
            // another "/25": the old template printed "25.0/25.0/25".
            line(box_row(&format!(
                "{label} {}",
                c::score(f64::from(value), max, 70.0, 40.0)
            )));
        }
    }

    line(box_bottom());
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
        use crate::tdg::formatters::boxdraw::{box_bottom, box_row, box_separator, box_top};
        let mut output = String::new();
        let mut line = |text: String| {
            output.push_str(&text);
            output.push('\n');
        };
        line(box_top());
        line(box_row("TDG Comparison"));
        line(box_separator());
        let grade1 = format_grade(comparison.source1.grade);
        let grade2 = format_grade(comparison.source2.grade);
        line(box_row(&format!(
            "Source 1: {} ({})",
            c::number(&format!("{:.1}", comparison.source1.total)),
            c::grade(&grade1)
        )));
        line(box_row(&format!(
            "Source 2: {} ({})",
            c::number(&format!("{:.1}", comparison.source2.total)),
            c::grade(&grade2)
        )));
        line(box_row(&format!(
            "Difference: {}",
            c::delta(f64::from(comparison.delta))
        )));
        line(box_row(&format!(
            "Winner: {}",
            c::label(&comparison.winner)
        )));
        line(box_bottom());
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
