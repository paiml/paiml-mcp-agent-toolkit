#![cfg_attr(coverage_nightly, coverage(off))]
//! Demo Score CLI handlers
//!
//! Category G: Demo Quality scoring (0-10 scale)
//! Based on docs/specifications/demo-and-book-scoring.md

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::cli::RepoScoreOutputFormat;
use crate::services::repo_score::scorers::{DemoScorer, Scorer, ScorerConfig};

/// Handle the demo-score command
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_demo_score(
    path: &Path,
    format: &RepoScoreOutputFormat,
    verbose: bool,
    failures_only: bool,
    output: Option<&Path>,
) -> Result<()> {
    // Validate path exists
    if !path.exists() {
        anyhow::bail!("Path not found: {}", path.display());
    }

    // Validate it's a directory
    if !path.is_dir() {
        anyhow::bail!("Path is not a directory: {}", path.display());
    }

    // Run Demo scoring
    let scorer = DemoScorer::new();
    let config = ScorerConfig::default();
    let demo_score = scorer
        .score(path, &config)
        .await
        .context("Failed to calculate demo score")?;

    // Format output
    let output_text = match format {
        RepoScoreOutputFormat::Text => format_text(&demo_score, verbose, failures_only),
        RepoScoreOutputFormat::Json => format_json(&demo_score)?,
        RepoScoreOutputFormat::Markdown => format_markdown(&demo_score, verbose, failures_only),
        RepoScoreOutputFormat::Yaml => format_yaml(&demo_score)?,
    };

    // Write output
    if let Some(output_path) = output {
        fs::write(output_path, &output_text)
            .with_context(|| format!("Failed to write to {}", output_path.display()))?;
        println!("Demo score written to: {}", output_path.display());
    } else {
        print!("{}", output_text);
    }

    Ok(())
}

use crate::services::repo_score::models::CategoryScore;

/// Format score as human-readable text
///
/// `--format text` is documented as "Text format with colored output
/// (default)", but this renderer emitted no escape at all, so `demo-score
/// --color always` was byte-identical to `--color never`. Colour comes from the
/// shared helpers, which honour `--color always` on a pipe and stay plain under
/// `--color never`, `NO_COLOR` and a redirected stdout.
fn format_text(score: &CategoryScore, verbose: bool, failures_only: bool) -> String {
    use crate::cli::colors as c;

    let mut output = String::new();
    output.push_str(&format!("{}\n", c::rule()));
    output.push_str(&format!(
        "{}\n",
        c::subheader("📚  Demo Quality Score (Category G)")
    ));
    output.push_str(&format!("{}\n\n", c::rule()));
    let percentage = (score.score / score.max_score) * 100.0;
    let grade = grade_from_percentage(percentage);
    output.push_str(&format!(
        "{} {:.1}/{:.1} ({}) - {} {}\n\n",
        c::label("Score:"),
        score.score,
        score.max_score,
        c::pct(percentage, 80.0, 50.0),
        c::label("Grade:"),
        c::grade(grade)
    ));
    output.push_str(&format!("{}\n", c::subheader("Categories:")));
    for sub in &score.subcategories {
        format_text_subcategory(sub, verbose, failures_only, &mut output);
    }
    output.push_str("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str("Category Breakdown:\n");
    output.push_str("  G1: Time-to-Interaction (3 pts) - Quick-start, examples\n");
    output.push_str("  G2: Error Gracefulness (3 pts) - Proper error handling\n");
    output.push_str("  G3: Visual Stability (2 pts) - Rich output formatting\n");
    output.push_str("  G4: Wow Factor (2 pts) - Demo GIF, badges, web demo\n");
    output
}

fn subcategory_status(
    sub: &crate::services::repo_score::models::SubcategoryScore,
) -> (&'static str, f64, bool) {
    let is_na = sub.max_score == 0.0;
    let pct = if is_na {
        0.0
    } else {
        (sub.score / sub.max_score) * 100.0
    };
    let icon = if is_na {
        "➖"
    } else if pct >= 80.0 {
        "✅"
    } else if pct >= 50.0 {
        "⚠️"
    } else {
        "❌"
    };
    (icon, pct, is_na)
}

fn finding_icon_text(severity: crate::services::repo_score::models::Severity) -> &'static str {
    match severity {
        crate::services::repo_score::models::Severity::Success => "✓",
        crate::services::repo_score::models::Severity::Info => "ℹ",
        crate::services::repo_score::models::Severity::Warning => "⚠",
        crate::services::repo_score::models::Severity::Error => "✗",
    }
}

fn format_text_subcategory(
    sub: &crate::services::repo_score::models::SubcategoryScore,
    verbose: bool,
    failures_only: bool,
    output: &mut String,
) {
    use crate::cli::colors as c;

    let (icon, pct, is_na) = subcategory_status(sub);
    // PMAT-688: `--failures-only` is a display filter — a subcategory at or
    // above the 80% pass line is not shown, in text and markdown alike.
    if failures_only && !is_na && pct >= 80.0 {
        return;
    }
    if is_na {
        output.push_str(&format!("  {} {}: {}\n", icon, sub.name, c::dim("N/A")));
    } else {
        output.push_str(&format!(
            "  {} {}: {:.1}/{:.1} ({})\n",
            icon,
            sub.name,
            sub.score,
            sub.max_score,
            c::colored(c::threshold_color(pct, 80.0, 50.0), &format!("{pct:.0}%"))
        ));
    }
    if !verbose {
        return;
    }
    for finding in &sub.findings {
        if failures_only
            && finding.severity == crate::services::repo_score::models::Severity::Success
        {
            continue;
        }
        output.push_str(&format!(
            "      {} {}\n",
            finding_icon_text(finding.severity),
            finding.message
        ));
    }
}

/// Format score as JSON
fn format_json(score: &CategoryScore) -> Result<String> {
    serde_json::to_string_pretty(&score).context("Failed to serialize to JSON")
}

/// Format score as Markdown
fn format_markdown(score: &CategoryScore, verbose: bool, failures_only: bool) -> String {
    let mut output = String::new();
    output.push_str("# Demo Quality Score (Category G)\n\n");
    let percentage = (score.score / score.max_score) * 100.0;
    let grade = grade_from_percentage(percentage);
    output.push_str(&format!(
        "**Score:** {:.1}/{:.1} ({:.1}%) - **Grade:** {}\n\n",
        score.score, score.max_score, percentage, grade
    ));
    output.push_str("## Categories\n\n");
    output.push_str("| Category | Score | Max | Percentage |\n");
    output.push_str("|----------|-------|-----|------------|\n");
    for sub in &score.subcategories {
        let (icon, pct, is_na) = subcategory_status(sub);
        if failures_only && !is_na && pct >= 80.0 {
            continue;
        }
        if is_na {
            output.push_str(&format!("| {} {} | N/A | N/A | N/A |\n", icon, sub.name));
        } else {
            output.push_str(&format!(
                "| {} {} | {:.1} | {:.1} | {:.0}% |\n",
                icon, sub.name, sub.score, sub.max_score, pct
            ));
        }
    }
    if verbose {
        format_md_findings(score, failures_only, &mut output);
    }
    output
}

fn format_md_findings(score: &CategoryScore, failures_only: bool, output: &mut String) {
    output.push_str("\n## Findings\n\n");
    for sub in &score.subcategories {
        output.push_str(&format!("### {}\n\n", sub.name));
        for finding in &sub.findings {
            if failures_only
                && finding.severity == crate::services::repo_score::models::Severity::Success
            {
                continue;
            }
            let icon = match finding.severity {
                crate::services::repo_score::models::Severity::Success => "✅",
                crate::services::repo_score::models::Severity::Info => "ℹ️",
                crate::services::repo_score::models::Severity::Warning => "⚠️",
                crate::services::repo_score::models::Severity::Error => "❌",
            };
            output.push_str(&format!("- {} {}\n", icon, finding.message));
        }
        output.push('\n');
    }
}

/// Format score as YAML
fn format_yaml(score: &CategoryScore) -> Result<String> {
    serde_yaml_ng::to_string(&score).context("Failed to serialize to YAML")
}

/// Convert percentage to letter grade
fn grade_from_percentage(pct: f64) -> &'static str {
    match pct as u32 {
        90..=100 => "A+",
        85..=89 => "A",
        80..=84 => "A-",
        75..=79 => "B+",
        70..=74 => "B",
        65..=69 => "B-",
        60..=64 => "C+",
        55..=59 => "C",
        50..=54 => "C-",
        45..=49 => "D+",
        40..=44 => "D",
        _ => "F",
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Create a minimal README
        fs::write(
            repo_path.join("README.md"),
            "# Test Project\n\n## Quick Start\n\n```bash\ncargo run\n```",
        )
        .unwrap();

        // Create examples directory
        fs::create_dir_all(repo_path.join("examples")).unwrap();
        fs::write(
            repo_path.join("examples/basic.rs"),
            "fn main() { println!(\"Hello\"); }",
        )
        .unwrap();

        temp_dir
    }

    #[tokio::test]
    async fn test_handle_demo_score_text() {
        let temp_dir = create_test_repo();
        let result = handle_demo_score(
            temp_dir.path(),
            &RepoScoreOutputFormat::Text,
            false,
            false,
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_demo_score_json() {
        let temp_dir = create_test_repo();
        let result = handle_demo_score(
            temp_dir.path(),
            &RepoScoreOutputFormat::Json,
            false,
            false,
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_demo_score_markdown() {
        let temp_dir = create_test_repo();
        let result = handle_demo_score(
            temp_dir.path(),
            &RepoScoreOutputFormat::Markdown,
            true,
            false,
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_demo_score_yaml() {
        let temp_dir = create_test_repo();
        let result = handle_demo_score(
            temp_dir.path(),
            &RepoScoreOutputFormat::Yaml,
            false,
            false,
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_demo_score_invalid_path() {
        let result = handle_demo_score(
            Path::new("/nonexistent/path"),
            &RepoScoreOutputFormat::Text,
            false,
            false,
            None,
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_handle_demo_score_file_not_dir() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("file.txt");
        fs::write(&file_path, "test").unwrap();

        let result =
            handle_demo_score(&file_path, &RepoScoreOutputFormat::Text, false, false, None).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a directory"));
    }

    #[test]
    fn test_grade_from_percentage() {
        assert_eq!(grade_from_percentage(95.0), "A+");
        assert_eq!(grade_from_percentage(85.0), "A");
        assert_eq!(grade_from_percentage(75.0), "B+");
        assert_eq!(grade_from_percentage(65.0), "B-");
        assert_eq!(grade_from_percentage(55.0), "C");
        assert_eq!(grade_from_percentage(30.0), "F");
    }

    fn mixed_category() -> CategoryScore {
        use crate::services::repo_score::models::{
            Finding, ScoreStatus, Severity, SubcategoryScore,
        };
        let sub = |id: &str, name: &str, score: f64, max: f64, severity: Severity, msg: &str| {
            SubcategoryScore {
                id: id.to_string(),
                name: name.to_string(),
                score,
                max_score: max,
                findings: vec![Finding {
                    severity,
                    category: "G".to_string(),
                    message: msg.to_string(),
                    location: None,
                    impact_points: 0.0,
                }],
            }
        };
        CategoryScore {
            score: 4.0,
            max_score: 6.0,
            percentage: 66.7,
            status: ScoreStatus::Fail,
            subcategories: vec![
                sub(
                    "G1",
                    "Time-to-Interaction",
                    1.0,
                    3.0,
                    Severity::Error,
                    "no quick start",
                ),
                sub(
                    "G2",
                    "Error Gracefulness",
                    3.0,
                    3.0,
                    Severity::Success,
                    "errors are handled",
                ),
            ],
            findings: Vec::new(),
        }
    }

    /// PMAT-688: `--failures-only` only filtered findings, and only under
    /// `--verbose`, so the swept `demo-score --failures-only` reproduced the
    /// baseline. A subcategory at or above the 80% pass line is not a failure
    /// and must not be listed.
    #[test]
    fn failures_only_hides_passing_subcategories_in_text() {
        let score = mixed_category();
        let all = format_text(&score, false, false);
        let failing = format_text(&score, false, true);
        // The static "Category Breakdown" legend names every category; the
        // row is what the filter removes.
        assert!(all.contains("Error Gracefulness: 3.0/3.0"), "{all}");
        assert!(
            !failing.contains("Error Gracefulness: 3.0/3.0"),
            "{failing}"
        );
        assert!(
            failing.contains("Time-to-Interaction: 1.0/3.0"),
            "{failing}"
        );
    }

    #[test]
    fn failures_only_hides_passing_subcategories_in_markdown() {
        let score = mixed_category();
        let all = format_markdown(&score, false, false);
        let failing = format_markdown(&score, false, true);
        assert!(all.contains("Error Gracefulness"), "{all}");
        assert!(!failing.contains("Error Gracefulness"), "{failing}");
        assert!(failing.contains("Time-to-Interaction"), "{failing}");
    }
}
