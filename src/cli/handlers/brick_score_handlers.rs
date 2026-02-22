#![cfg_attr(coverage_nightly, coverage(off))]
//! CLI handler for `pmat brick-score` command (PMAT-446)
//!
//! Calculates ComputeBrick Profiling Score (0-100 scale) for
//! trueno/realizar ecosystem projects using BrickProfiler data.

use crate::cli::RepoScoreOutputFormat;
use crate::services::brick_score::{
    default_brick_budgets, find_profiler_files, load_hardware_capability, load_profiler_json,
    scale_budgets_for_hardware, score_brick_profiler, BrickScore,
};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Handle the brick-score command
///
/// Analyzes BrickProfiler output and calculates a 100-point score:
/// - Performance (40 pts): Throughput vs budget
/// - Efficiency (25 pts): Backend utilization
/// - Correctness (20 pts): Assertions passing
/// - Stability (15 pts): CV < 5%
///
/// PMAT-448: If ~/.pmat/hardware.toml exists, budgets are scaled based on
/// detected SIMD capability and memory bandwidth.
#[allow(clippy::too_many_arguments)]
pub async fn handle_brick_score(
    path: &Path,
    input_file: Option<&Path>,
    format: &RepoScoreOutputFormat,
    verbose: bool,
    failures_only: bool,
    threshold: u32,
    output: Option<&Path>,
    hardware_path: Option<&Path>,
) -> Result<()> {
    // Validate path exists
    if !path.exists() {
        anyhow::bail!("Path not found: {}", path.display());
    }

    // Load hardware capability (PMAT-448)
    let hardware = load_hardware_capability(hardware_path);

    // Find or use provided profiler JSON
    let profiler_output = if let Some(input) = input_file {
        load_profiler_json(input)
            .with_context(|| format!("Failed to load profiler JSON from {}", input.display()))?
    } else {
        // Search for profiler files
        let files = find_profiler_files(path);
        if files.is_empty() {
            anyhow::bail!(
                "No BrickProfiler JSON found in {}. \n\
                 Run your benchmark with profiling enabled, then pass the JSON file:\n\
                 \n\
                 # Option 1: Run cbtop with profiling\n\
                 cbtop --model <model> --headless --output brick_profile.json\n\
                 \n\
                 # Option 2: Pass existing JSON\n\
                 pmat brick-score --input profiler.json",
                path.display()
            );
        }

        // Use first found file
        load_profiler_json(&files[0])
            .with_context(|| format!("Failed to load profiler JSON from {}", files[0].display()))?
    };

    // Load default budgets, scaled for hardware if available
    let base_budgets = default_brick_budgets();
    let budgets = match &hardware {
        Some(hw) => scale_budgets_for_hardware(&base_budgets, hw),
        None => base_budgets,
    };

    // Calculate score
    let score = score_brick_profiler(&profiler_output, &budgets, path, hardware.as_ref());

    // Format output
    let output_text = match format {
        RepoScoreOutputFormat::Text => format_text(&score, verbose, failures_only),
        RepoScoreOutputFormat::Json => format_json(&score)?,
        RepoScoreOutputFormat::Markdown => format_markdown(&score, verbose, failures_only),
        RepoScoreOutputFormat::Yaml => format_yaml(&score)?,
    };

    // Write output
    if let Some(output_path) = output {
        fs::write(output_path, &output_text)
            .with_context(|| format!("Failed to write to {}", output_path.display()))?;
        println!("Brick score written to: {}", output_path.display());
    } else {
        print!("{}", output_text);
    }

    // Check threshold
    if score.total_score < threshold as f64 {
        anyhow::bail!(
            "Brick score {:.1} is below threshold {} (grade: {})",
            score.total_score,
            threshold,
            score.grade
        );
    }

    Ok(())
}

/// Format score as human-readable text
fn format_text(score: &BrickScore, verbose: bool, failures_only: bool) -> String {
    let mut output = String::new();
    format_text_header(score, &mut output);
    format_text_categories(score, verbose, failures_only, &mut output);
    if verbose && !failures_only {
        format_text_brick_timing(score, &mut output);
    }
    format_text_recommendations(score, &mut output);
    format_text_grading_scale(&mut output);
    output
}

fn format_text_header(score: &BrickScore, output: &mut String) {
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str(&format!(
        "🧱  ComputeBrick Score v{}\n",
        score.metadata.version
    ));
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push('\n');
    output.push_str("📌  Summary\n");
    output.push_str(&format!("  Score: {:.1}/100\n", score.total_score));
    output.push_str(&format!("  Grade: {}\n", score.grade));
    if let Some(model) = &score.metadata.model {
        output.push_str(&format!("  Model: {}\n", model));
    }
    if let Some(hw) = &score.metadata.hardware {
        output.push_str(&format!("  Hardware: {}\n", hw));
    }
    output.push_str(&format!(
        "  Bricks: {} ({} samples)\n",
        score.metadata.total_bricks, score.metadata.total_samples
    ));
    output.push('\n');
}

fn format_text_categories(
    score: &BrickScore,
    verbose: bool,
    failures_only: bool,
    output: &mut String,
) {
    output.push_str("📂  Categories\n");
    let categories = [
        (&score.performance, "A. Performance"),
        (&score.efficiency, "B. Efficiency"),
        (&score.correctness, "C. Correctness"),
        (&score.stability, "D. Stability"),
    ];
    for (category, name) in categories {
        format_single_category(category, name, verbose, failures_only, output);
    }
    output.push('\n');
}

fn format_single_category(
    category: &crate::services::brick_score::CategoryScore,
    name: &str,
    verbose: bool,
    failures_only: bool,
    output: &mut String,
) {
    let percentage = category.percentage();
    let icon = if percentage >= 80.0 {
        "✅"
    } else if percentage >= 60.0 {
        "⚠️"
    } else {
        "❌"
    };
    output.push_str(&format!(
        "  {} {}: {:.1}/{:.0} ({:.0}%)\n",
        icon, name, category.earned, category.max_points, percentage
    ));
    if !verbose {
        return;
    }
    for check in &category.checks {
        if failures_only && check.passed {
            continue;
        }
        let check_icon = if check.passed { "✓" } else { "✗" };
        output.push_str(&format!(
            "      {} {}: {:.2} {} (threshold: {:.2} {})\n",
            check_icon, check.name, check.actual, check.unit, check.threshold, check.unit
        ));
        if let Some(rec) = &check.recommendation {
            output.push_str(&format!("        → {}\n", rec));
        }
    }
}

fn format_text_brick_timing(score: &BrickScore, output: &mut String) {
    output.push_str("📊  Per-Brick Timing\n");
    output.push_str("  ┌───────────────────┬──────────┬──────────┬─────────┬───────────┐\n");
    output.push_str("  │ Brick             │ Mean µs  │ Budget   │ CV %    │ Throughput│\n");
    output.push_str("  ├───────────────────┼──────────┼──────────┼─────────┼───────────┤\n");
    for brick in &score.brick_reports {
        let status = if brick.over_budget { "❌" } else { "✅" };
        let budget_str = brick
            .budget_us
            .map(|b| format!("{:.1}", b))
            .unwrap_or_else(|| "-".to_string());
        output.push_str(&format!(
            "  │ {:<17} │ {:>7.1} │ {:>7} {} │ {:>6.1} │ {:>9.0} │\n",
            brick
                .name
                .get(..brick.name.len().min(17))
                .unwrap_or(&brick.name),
            brick.mean_us,
            budget_str,
            status,
            brick.cv_percent,
            brick.throughput / 1000.0,
        ));
    }
    output.push_str("  └───────────────────┴──────────┴──────────┴─────────┴───────────┘\n");
    output.push('\n');
    format_text_roofline(score, output);
}

fn format_text_roofline(score: &BrickScore, output: &mut String) {
    if !score.brick_reports.iter().any(|b| b.bottleneck.is_some()) {
        return;
    }
    output.push_str("📈  Roofline Analysis\n");
    output.push_str("  ┌───────────────────┬────────┬────────────┐\n");
    output.push_str("  │ Brick             │ AI     │ Bottleneck │\n");
    output.push_str("  ├───────────────────┼────────┼────────────┤\n");
    for brick in &score.brick_reports {
        let ai_str = brick
            .arithmetic_intensity
            .map(|ai| format!("{:.2}", ai))
            .unwrap_or_else(|| "-".to_string());
        let bottleneck_str = brick
            .bottleneck
            .map(|b| match b {
                crate::services::brick_score::Bottleneck::Memory => "🔴 Memory",
                crate::services::brick_score::Bottleneck::Compute => "🟢 Compute",
            })
            .unwrap_or("-");
        output.push_str(&format!(
            "  │ {:<17} │ {:>6} │ {:>10} │\n",
            brick
                .name
                .get(..brick.name.len().min(17))
                .unwrap_or(&brick.name),
            ai_str,
            bottleneck_str
        ));
    }
    output.push_str("  └───────────────────┴────────┴────────────┘\n");
    output.push_str("  AI = Arithmetic Intensity (FLOP/byte)\n");
    output.push_str("  🔴 Memory-bound: Optimize memory access patterns\n");
    output.push_str("  🟢 Compute-bound: Optimize SIMD/GPU utilization\n");
    output.push('\n');
}

fn format_text_recommendations(score: &BrickScore, output: &mut String) {
    let recommendations: Vec<_> = [
        &score.performance.checks,
        &score.efficiency.checks,
        &score.stability.checks,
    ]
    .iter()
    .flat_map(|checks| checks.iter())
    .filter_map(|c| c.recommendation.as_ref())
    .collect();
    if recommendations.is_empty() {
        return;
    }
    output.push_str("💡  Recommendations\n");
    for (i, rec) in recommendations.iter().take(5).enumerate() {
        output.push_str(&format!("  {}. {}\n", i + 1, rec));
    }
    if recommendations.len() > 5 {
        output.push_str(&format!("  ... and {} more\n", recommendations.len() - 5));
    }
    output.push('\n');
}

fn format_text_grading_scale(output: &mut String) {
    output.push_str("📋  Grading Scale\n");
    output.push_str("  A (90-100): Production Ready\n");
    output.push_str("  B (80-89):  Optimization Needed\n");
    output.push_str("  C (70-79):  Functional but Slow\n");
    output.push_str("  D (60-69):  Unstable/Inefficient\n");
    output.push_str("  F (<60):    Do Not Merge\n");
}

/// Format score as JSON
fn format_json(score: &BrickScore) -> Result<String> {
    serde_json::to_string_pretty(score).context("Failed to serialize to JSON")
}

/// Format score as Markdown
fn format_markdown(score: &BrickScore, verbose: bool, failures_only: bool) -> String {
    let mut output = String::new();
    output.push_str("# ComputeBrick Score Report\n\n");
    format_md_summary(score, &mut output);
    format_md_categories(score, &mut output);
    if verbose {
        format_md_brick_details(score, failures_only, &mut output);
    }
    output
}

fn format_md_summary(score: &BrickScore, output: &mut String) {
    output.push_str("## Summary\n\n");
    output.push_str("| Metric | Value |\n");
    output.push_str("|--------|-------|\n");
    output.push_str(&format!("| **Score** | {:.1}/100 |\n", score.total_score));
    output.push_str(&format!("| **Grade** | {} |\n", score.grade));
    if let Some(model) = &score.metadata.model {
        output.push_str(&format!("| Model | {} |\n", model));
    }
    output.push_str(&format!(
        "| Bricks | {} ({} samples) |\n",
        score.metadata.total_bricks, score.metadata.total_samples
    ));
    output.push('\n');
}

fn format_md_categories(score: &BrickScore, output: &mut String) {
    output.push_str("## Categories\n\n");
    output.push_str("| Category | Score | Max | % |\n");
    output.push_str("|----------|-------|-----|---|\n");
    let categories = [
        (&score.performance, "Performance"),
        (&score.efficiency, "Efficiency"),
        (&score.correctness, "Correctness"),
        (&score.stability, "Stability"),
    ];
    for (cat, name) in categories {
        let status = if cat.percentage() >= 80.0 {
            "✅"
        } else if cat.percentage() >= 60.0 {
            "⚠️"
        } else {
            "❌"
        };
        output.push_str(&format!(
            "| {} {} | {:.1} | {:.0} | {:.0}% |\n",
            status,
            name,
            cat.earned,
            cat.max_points,
            cat.percentage()
        ));
    }
    output.push('\n');
}

fn format_md_brick_details(score: &BrickScore, failures_only: bool, output: &mut String) {
    output.push_str("## Per-Brick Details\n\n");
    output.push_str("| Brick | Mean µs | Budget µs | CV % | Status |\n");
    output.push_str("|-------|---------|-----------|------|--------|\n");
    for brick in &score.brick_reports {
        if failures_only && !brick.over_budget && brick.cv_percent < 15.0 {
            continue;
        }
        let status = if brick.over_budget {
            "❌ Over budget"
        } else if brick.cv_percent >= 15.0 {
            "⚠️ Unstable"
        } else {
            "✅ OK"
        };
        let budget_str = brick
            .budget_us
            .map(|b| format!("{:.1}", b))
            .unwrap_or_else(|| "-".to_string());
        output.push_str(&format!(
            "| {} | {:.1} | {} | {:.1} | {} |\n",
            brick.name, brick.mean_us, budget_str, brick.cv_percent, status
        ));
    }
    output.push('\n');
}

/// Format score as YAML
fn format_yaml(score: &BrickScore) -> Result<String> {
    serde_yaml_ng::to_string(score).context("Failed to serialize to YAML")
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::brick_score::{BrickProfilerOutput, BrickStats};

    fn sample_profiler_output() -> BrickProfilerOutput {
        BrickProfilerOutput {
            bricks: vec![
                BrickStats {
                    name: "RmsNorm".to_string(),
                    count: 100,
                    total_ns: 800_000,
                    min_ns: 7_000,
                    max_ns: 9_000,
                    total_elements: 1_000_000,
                },
                BrickStats {
                    name: "Attention".to_string(),
                    count: 100,
                    total_ns: 2_000_000,
                    min_ns: 18_000,
                    max_ns: 22_000,
                    total_elements: 500_000,
                },
            ],
            total_tokens: 1000,
            total_ns: 2_800_000,
            model: Some("test-model".to_string()),
            hardware: Some("RTX 4090".to_string()),
        }
    }

    #[test]
    fn test_format_text() {
        let output = sample_profiler_output();
        let budgets = default_brick_budgets();
        let score = score_brick_profiler(&output, &budgets, Path::new("."), None);

        let text = format_text(&score, true, false);
        assert!(text.contains("ComputeBrick Score"));
        assert!(text.contains("Performance"));
        assert!(text.contains("Efficiency"));
    }

    #[test]
    fn test_format_json() {
        let output = sample_profiler_output();
        let budgets = default_brick_budgets();
        let score = score_brick_profiler(&output, &budgets, Path::new("."), None);

        let json = format_json(&score).unwrap();
        assert!(json.contains("total_score"));
        assert!(json.contains("grade"));
    }

    #[test]
    fn test_format_markdown() {
        let output = sample_profiler_output();
        let budgets = default_brick_budgets();
        let score = score_brick_profiler(&output, &budgets, Path::new("."), None);

        let md = format_markdown(&score, true, false);
        assert!(md.contains("# ComputeBrick Score Report"));
        assert!(md.contains("| Category |"));
    }
}
