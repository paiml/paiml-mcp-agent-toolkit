//! Perfection Score CLI Handlers (master-plan-pmat-work-system.md)
//!
//! Implements P-001 through P-010 acceptance criteria.

use crate::cli::commands::PerfectionScoreOutputFormat;
use crate::services::perfection_score::{PerfectionScoreCalculator, PerfectionScoreResult};
use std::fs;
use std::path::Path;

/// Handle perfection-score command
pub async fn handle_perfection_score(
    path: &Path,
    breakdown: bool,
    target: Option<u16>,
    format: PerfectionScoreOutputFormat,
    output: Option<&Path>,
    fast: bool,
) -> anyhow::Result<()> {
    let calculator = PerfectionScoreCalculator::new().fast_mode(fast);
    let mut result = calculator.calculate(path).await?;

    if let Some(t) = target {
        result = result.with_target(t);
    }

    let output_text = match format {
        PerfectionScoreOutputFormat::Text => format_text(&result, breakdown),
        PerfectionScoreOutputFormat::Json => format_json(&result)?,
        PerfectionScoreOutputFormat::Markdown => format_markdown(&result, breakdown),
    };

    if let Some(output_path) = output {
        fs::write(output_path, &output_text)?;
        println!("✅ Perfection score written to {}", output_path.display());
    } else {
        println!("{}", output_text);
    }

    Ok(())
}

fn format_text(result: &PerfectionScoreResult, breakdown: bool) -> String {
    let mut out = String::new();

    out.push_str("🏆 PMAT Perfection Score\n");
    out.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");

    // Main score display
    out.push_str(&format!(
        "  Total: {:.1}/{} points\n",
        result.total_score, result.max_score
    ));
    out.push_str(&format!("  Grade: {}\n", result.grade));

    if let Some(gap) = result.target_gap {
        if gap > 0.0 {
            out.push_str(&format!("  Target Gap: {:.1} points needed\n", gap));
        } else {
            out.push_str("  ✅ Target achieved!\n");
        }
    }

    out.push('\n');

    // Category breakdown
    if breakdown {
        out.push_str("📊 Category Breakdown\n");
        out.push_str("═══════════════════════════════════════════════════════\n\n");

        for cat in &result.categories {
            let progress_bar = create_progress_bar(cat.earned_points, f64::from(cat.max_points), 20);
            out.push_str(&format!(
                "  {:25} {} {:.1}/{} pts ({})\n",
                cat.name, progress_bar, cat.earned_points, cat.max_points, cat.grade
            ));
            if let Some(details) = &cat.details {
                if !details.is_empty() {
                    out.push_str(&format!("    └─ {}\n", details));
                }
            }
        }
        out.push('\n');
    }

    // Recommendations
    if !result.recommendations.is_empty() {
        out.push_str("💡 Recommendations\n");
        out.push_str("───────────────────────────────────────────────────────\n");
        for rec in &result.recommendations {
            out.push_str(&format!("  {}\n", rec));
        }
    }

    out.push_str("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    out
}

fn format_json(result: &PerfectionScoreResult) -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(result)?)
}

fn format_markdown(result: &PerfectionScoreResult, breakdown: bool) -> String {
    let mut out = String::new();

    out.push_str("# PMAT Perfection Score Report\n\n");

    out.push_str("## Summary\n\n");
    out.push_str(&format!(
        "| Metric | Value |\n|--------|-------|\n| **Total Score** | {:.1}/{} |\n| **Grade** | {} |\n",
        result.total_score, result.max_score, result.grade
    ));

    if let Some(gap) = result.target_gap {
        if gap > 0.0 {
            out.push_str(&format!("| **Target Gap** | {:.1} points |\n", gap));
        } else {
            out.push_str("| **Target** | ✅ Achieved |\n");
        }
    }

    out.push('\n');

    if breakdown {
        out.push_str("## Category Breakdown\n\n");
        out.push_str("| Category | Score | Max | Grade |\n");
        out.push_str("|----------|-------|-----|-------|\n");

        for cat in &result.categories {
            out.push_str(&format!(
                "| {} | {:.1} | {} | {} |\n",
                cat.name, cat.earned_points, cat.max_points, cat.grade
            ));
        }
        out.push('\n');
    }

    if !result.recommendations.is_empty() {
        out.push_str("## Recommendations\n\n");
        for rec in &result.recommendations {
            out.push_str(&format!("- {}\n", rec));
        }
    }

    out
}

fn create_progress_bar(current: f64, max: f64, width: usize) -> String {
    let percentage = (current / max).clamp(0.0, 1.0);
    let filled = (percentage * width as f64) as usize;
    let empty = width - filled;

    let color = if percentage >= 0.8 {
        "\x1b[32m" // Green
    } else if percentage >= 0.6 {
        "\x1b[33m" // Yellow
    } else {
        "\x1b[31m" // Red
    };

    format!(
        "[{}{}{}{}]",
        color,
        "█".repeat(filled),
        "░".repeat(empty),
        "\x1b[0m"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::perfection_score::CategoryScore;

    #[test]
    fn test_format_text_output() {
        let categories = vec![
            CategoryScore::new("Test Category", 80.0, 40),
        ];
        let result = PerfectionScoreResult::new(categories);
        let text = format_text(&result, true);

        assert!(text.contains("PMAT Perfection Score"));
        assert!(text.contains("Test Category"));
    }

    #[test]
    fn test_format_json_output() {
        let categories = vec![
            CategoryScore::new("Test", 75.0, 30),
        ];
        let result = PerfectionScoreResult::new(categories);
        let json = format_json(&result).unwrap();

        assert!(json.contains("total_score"));
        assert!(json.contains("grade"));
    }

    #[test]
    fn test_format_markdown_output() {
        let categories = vec![
            CategoryScore::new("Test", 70.0, 25),
        ];
        let result = PerfectionScoreResult::new(categories);
        let md = format_markdown(&result, true);

        assert!(md.contains("# PMAT Perfection Score Report"));
        assert!(md.contains("| Category |"));
    }

    #[test]
    fn test_progress_bar() {
        let bar = create_progress_bar(32.0, 40.0, 10); // 80%
        assert!(bar.contains("█"));
        assert!(bar.contains("░"));
    }
}
