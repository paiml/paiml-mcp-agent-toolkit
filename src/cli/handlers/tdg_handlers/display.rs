#![cfg_attr(coverage_nightly, coverage(off))]
//! TDG display functions: history, explain, gate results, grade distribution
//!
//! Functions that format output for display (history records, explain mode,
//! quality gate results, baseline summaries).

use super::{format_grade, truncate_string, TdgCommandConfig};
use crate::cli::TdgOutputFormat;
use crate::tdg::Grade;
use anyhow::Result;

/// Format TDG history output (Sprint 65 Phase 3)
pub(crate) fn format_history_output(
    records: &[crate::tdg::storage::FullTdgRecord],
    format: TdgOutputFormat,
) -> Result<String> {
    use chrono::{DateTime, Utc};

    if format == TdgOutputFormat::Table {
        let mut output = String::new();
        output.push_str(
            "╭──────────────────────────────────────────────────────────────────────────╮\n",
        );
        output.push_str(
            "│  TDG History                                                             │\n",
        );
        output.push_str(
            "├──────────────────────────────────────────────────────────────────────────┤\n",
        );

        for record in records {
            if let Some(git_ctx) = &record.git_context {
                let timestamp: DateTime<Utc> = git_ctx.commit_timestamp;
                let date_str = timestamp.format("%Y-%m-%d %H:%M").to_string();

                output.push_str(&format!(
                    "│  📝 {} - {} ({})                                            │\n",
                    git_ctx.commit_sha_short,
                    format_grade(record.score.grade),
                    record.score.total
                ));
                output.push_str(&format!(
                    "│  ├─ Branch:  {}                                                           │\n",
                    git_ctx.branch
                ));
                output.push_str(&format!(
                    "│  ├─ Author:  {}                                                      │\n",
                    git_ctx.author_name
                ));
                output.push_str(&format!(
                    "│  ├─ Date:    {}                                                  │\n",
                    date_str
                ));
                output.push_str(&format!(
                    "│  └─ File:    {}                                                          │\n",
                    record.identity.path.display()
                ));
                output.push_str("│                                                                          │\n");
            }
        }

        output.push_str(
            "╰──────────────────────────────────────────────────────────────────────────╯\n",
        );
        Ok(output)
    } else {
        // JSON format
        let json_records: Vec<_> = records
            .iter()
            .map(|r| {
                serde_json::json!({
                    "file_path": r.identity.path,
                    "score": {
                        "total": r.score.total,
                        "grade": format_grade(r.score.grade),
                        "structural_complexity": r.score.structural_complexity,
                        "semantic_complexity": r.score.semantic_complexity,
                        "duplication_ratio": r.score.duplication_ratio,
                        "coupling_score": r.score.coupling_score,
                        "doc_coverage": r.score.doc_coverage,
                        "consistency_score": r.score.consistency_score,
                        "entropy_score": r.score.entropy_score,
                    },
                    "git_context": r.git_context.as_ref().map(|git| serde_json::json!({
                        "commit_sha": git.commit_sha,
                        "commit_sha_short": git.commit_sha_short,
                        "branch": git.branch,
                        "author_name": git.author_name,
                        "author_email": git.author_email,
                        "commit_timestamp": git.commit_timestamp,
                        "commit_message": git.commit_message,
                        "tags": git.tags,
                    })),
                })
            })
            .collect();

        Ok(serde_json::to_string_pretty(&serde_json::json!({
            "history": json_records,
            "total_records": records.len()
        }))?)
    }
}

/// Format explain mode output (Issue #78)
pub(crate) fn format_explain_output(
    explained: &crate::tdg::explain::ExplainedTDGScore,
    config: &TdgCommandConfig,
) -> Result<String> {
    match config.format {
        TdgOutputFormat::Json => Ok(serde_json::to_string_pretty(explained)?),
        TdgOutputFormat::Markdown => {
            // Markdown format uses same structure as table but in markdown
            let json = serde_json::to_string_pretty(explained)?;
            Ok(format!("```json\n{}\n```", json))
        }
        _ => format_explain_output_table(explained, config),
    }
}

/// Format explain mode output as table
fn format_explain_output_table(
    explained: &crate::tdg::explain::ExplainedTDGScore,
    config: &TdgCommandConfig,
) -> Result<String> {
    let mut output = String::new();

    // Header
    output.push_str("╭───────────────────────────────────────────────────────────────╮\n");
    output.push_str("│  TDG Explain Report                                           │\n");
    output.push_str("├───────────────────────────────────────────────────────────────┤\n");

    // Overall score
    output.push_str(&format!(
        "│  Score: {:.1}/100 ({})                                         │\n",
        explained.score.total,
        format_grade(explained.score.grade)
    ));
    output.push_str("│                                                               │\n");

    // Function breakdown
    if !explained.functions.is_empty() {
        output.push_str(&format!(
            "│  📊 Functions by Complexity (threshold: {:2})                  │\n",
            config.threshold
        ));
        output.push_str(
            "├───────────────────────────────────────────────────────────────┤\n",
        );

        for func in explained.functions.iter().take(10) {
            let severity_icon = match func.severity {
                crate::tdg::explain::ComplexitySeverity::Low => "🟢",
                crate::tdg::explain::ComplexitySeverity::Medium => "🟡",
                crate::tdg::explain::ComplexitySeverity::High => "🟠",
                crate::tdg::explain::ComplexitySeverity::Critical => "🔴",
            };
            output.push_str(&format!(
                "│  {} {:30} [line {:4}] CC={:2} TDG={:.1} │\n",
                severity_icon,
                truncate_string(&func.name, 30),
                func.line_number,
                func.cyclomatic,
                func.tdg_impact
            ));
        }

        if explained.functions.len() > 10 {
            output.push_str(&format!(
                "│  ... and {} more functions                                    │\n",
                explained.functions.len() - 10
            ));
        }
    } else {
        output.push_str(
            "│  ✅ No functions above complexity threshold                   │\n",
        );
    }

    // Recommendations
    if !explained.recommendations.is_empty() {
        output.push_str(
            "│                                                               │\n",
        );
        output.push_str(
            "│  💡 Recommendations                                           │\n",
        );
        output.push_str(
            "├───────────────────────────────────────────────────────────────┤\n",
        );

        for (i, rec) in explained.recommendations.iter().take(5).enumerate() {
            output.push_str(&format!(
                "│  {}. [+{:.1} pts] {}                     │\n",
                i + 1,
                rec.expected_impact,
                truncate_string(&rec.action, 40)
            ));
        }
    }

    output.push_str("╰───────────────────────────────────────────────────────────────╯\n");
    Ok(output)
}

/// Display gate result in table format
pub(crate) fn display_gate_result_table(result: &crate::tdg::GateResult) {
    println!("\n{}", result.message);

    if !result.violations.is_empty() {
        println!("\n📋 Violations:");
        println!("┌────────────────────────────────┬──────────────┬──────────┬────────────────────────────────┐");
        println!("│ File                           │ Type         │ Severity │ Message                        │");
        println!("├────────────────────────────────┼──────────────┼──────────┼────────────────────────────────┤");

        for violation in &result.violations {
            let path = format!("{}", violation.path.display());
            let vtype = format!("{:?}", violation.violation_type);
            let sev = format!("{:?}", violation.severity);
            println!(
                "│ {:<30} │ {:<12} │ {:<8} │ {:<30} │",
                path.get(..path.len().min(30)).unwrap_or(&path),
                vtype.get(..vtype.len().min(12)).unwrap_or(&vtype),
                sev.get(..sev.len().min(8)).unwrap_or(&sev),
                violation
                    .message
                    .get(..violation.message.len().min(30))
                    .unwrap_or(&violation.message)
            );
        }
        println!("└────────────────────────────────┴──────────────┴──────────┴────────────────────────────────┘");
    }
}

/// Display gate result in the requested format
pub(crate) fn display_gate_result(
    result: &crate::tdg::GateResult,
    format: &crate::cli::TdgOutputFormat,
) -> Result<()> {
    match format {
        crate::cli::TdgOutputFormat::Table => display_gate_result_table(result),
        crate::cli::TdgOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(result)?);
        }
        crate::cli::TdgOutputFormat::Sarif => {
            println!("SARIF format not yet implemented for quality gates");
        }
        crate::cli::TdgOutputFormat::Markdown => {
            println!("Markdown format not yet implemented for quality gates");
        }
    }
    Ok(())
}

/// Display grade distribution histogram
pub(in crate::cli::handlers::tdg_handlers) fn display_grade_distribution(
    baseline: &crate::tdg::TdgBaseline,
) {
    let mut grade_counts: std::collections::HashMap<Grade, usize> =
        std::collections::HashMap::new();
    let mut f_grade_files: Vec<String> = Vec::new();

    for (path, entry) in &baseline.files {
        *grade_counts.entry(entry.score.grade).or_insert(0) += 1;
        if entry.score.grade == Grade::F {
            f_grade_files.push(format!(
                "     {} ({:.1})",
                path.display(),
                entry.score.total
            ));
        }
    }

    println!("\n📊 Grade Distribution:");
    let grade_order = [
        Grade::APLus,
        Grade::A,
        Grade::AMinus,
        Grade::BPlus,
        Grade::B,
        Grade::BMinus,
        Grade::CPlus,
        Grade::C,
        Grade::CMinus,
        Grade::D,
        Grade::F,
    ];
    for grade in grade_order {
        let count = grade_counts.get(&grade).unwrap_or(&0);
        if *count > 0 {
            let bar = "█".repeat((*count).min(30));
            println!("   {:>3}: {:>4} {}", grade, count, bar);
        }
    }

    display_f_grade_warning(&f_grade_files);
}

/// Display F-grade warning if any files have F grades
fn display_f_grade_warning(f_grade_files: &[String]) {
    if f_grade_files.is_empty() {
        return;
    }
    println!(
        "\n⚠️  F-Grade Warning: {} file(s) with F grade:",
        f_grade_files.len()
    );
    for file in f_grade_files.iter().take(10) {
        println!("{}", file);
    }
    if f_grade_files.len() > 10 {
        println!("     ... and {} more", f_grade_files.len() - 10);
    }
    println!("\n   F-grades cap project score at B. Fix these to improve project grade.");
}

/// Display baseline in table format
pub(in crate::cli::handlers::tdg_handlers) fn display_baseline_table(
    path: &std::path::Path,
    baseline: &crate::tdg::TdgBaseline,
) {
    println!("📝 {}", path.display());
    println!("   Version: {}", baseline.version);
    println!(
        "   Created: {}",
        baseline.created_at.format("%Y-%m-%d %H:%M:%S")
    );
    println!("   Files: {}", baseline.summary.total_files);
    println!("   Avg Score: {:.1}", baseline.summary.avg_score);
    if let Some(git_ctx) = &baseline.git_context {
        println!("   Git: {} on {}", git_ctx.commit_sha_short, git_ctx.branch);
    }
    println!();
}
