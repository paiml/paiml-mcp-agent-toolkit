// Markdown, JSON, and YAML output formatting for Popper score
// Included by popper_score_handlers.rs — do NOT add `use` imports here.

/// Format score as JSON
fn format_json(score: &PopperScore) -> Result<String> {
    serde_json::to_string_pretty(score).context("Failed to serialize to JSON")
}

/// Format a single category row for markdown table output
fn format_markdown_category_row(
    output: &mut String,
    name: &str,
    category: &crate::services::popper_score::PopperCategoryScore,
    is_gateway: bool,
) {
    if category.is_not_applicable {
        output.push_str(&format!("| {} | N/A | N/A | ⚪ N/A |\n", name));
        return;
    }

    let percentage = category.percentage();
    let icon = percentage_icon(percentage);

    let status = if is_gateway {
        format!("{} GATEWAY", icon)
    } else {
        icon.to_string()
    };

    output.push_str(&format!(
        "| {} | {:.1}/{:.0} | {:.1}% | {} |\n",
        name, category.earned, category.max, percentage, status
    ));
}

/// Append verbose detailed breakdown section for markdown
fn format_markdown_detailed_breakdown(output: &mut String, score: &PopperScore) {
    output.push_str("## 📊 Detailed Breakdown\n\n");
    for (name, category, _) in popper_category_entries(score) {
        if category.is_not_applicable {
            continue;
        }
        output.push_str(&format!("### {}\n\n", name));
        for sub in &category.sub_scores {
            output.push_str(&format!(
                "- **{}**: {:.1}/{:.0} - {}\n",
                sub.id, sub.earned, sub.max, sub.description
            ));
        }
        output.push('\n');
    }
}

/// Append markdown-formatted recommendations to the output
fn format_markdown_recommendations(output: &mut String, score: &PopperScore) {
    if score.recommendations.is_empty() {
        return;
    }
    output.push_str("## 💡 Recommendations\n\n");
    for rec in &score.recommendations {
        let priority = priority_label_markdown(&rec.priority);
        output.push_str(&format!(
            "- **[{}]** {}: {}\n",
            priority, rec.category, rec.description
        ));
        if let Some(cmd) = &rec.command {
            output.push_str(&format!("  ```bash\n  {}\n  ```\n", cmd));
        }
    }
    output.push('\n');
}

/// Format score as Markdown
fn format_markdown(score: &PopperScore, verbose: bool, _failures_only: bool) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&format!(
        "# 🔬 Popper Falsifiability Score v{}\n\n",
        score.metadata.version
    ));

    // Gateway status
    if score.gateway_passed {
        output.push_str("> ✅ **Gateway PASSED**: Falsifiability >= 60%\n\n");
    } else {
        output.push_str("> ❌ **Gateway FAILED**: Falsifiability < 60%\n");
        output.push_str("> Without falsifiable claims, the total score is 0.\n\n");
    }

    // Summary
    output.push_str("## 📌 Summary\n\n");
    output.push_str(&format!(
        "- **Score**: {:.1}/{:.0}\n",
        score.raw_score, score.max_available
    ));
    output.push_str(&format!(
        "- **Normalized**: {:.1}%\n",
        score.normalized_score
    ));
    output.push_str(&format!("- **Grade**: {}\n\n", score.grade));

    // Categories table
    output.push_str("## 📂 Categories\n\n");
    output.push_str("| Category | Score | Percentage | Status |\n");
    output.push_str("|----------|-------|------------|--------|\n");

    for (name, category, is_gateway) in popper_category_entries(score) {
        format_markdown_category_row(&mut output, name, category, is_gateway);
    }
    output.push('\n');

    // Detailed sub-scores in verbose mode
    if verbose {
        format_markdown_detailed_breakdown(&mut output, score);
    }

    // Verdict
    output.push_str("## 📋 Verdict\n\n");
    output.push_str(&format!("{}\n\n", score.analysis.verdict));

    // Recommendations
    format_markdown_recommendations(&mut output, score);

    output
}

/// Format score as YAML
fn format_yaml(score: &PopperScore) -> Result<String> {
    serde_yaml_ng::to_string(score).context("Failed to serialize to YAML")
}
