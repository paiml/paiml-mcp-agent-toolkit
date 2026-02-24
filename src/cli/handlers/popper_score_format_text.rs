// Text output formatting for Popper score
// Included by popper_score_handlers.rs — do NOT add `use` imports here.

/// Format a single category line for text output, including optional verbose sub-scores
fn format_text_category(
    output: &mut String,
    name: &str,
    category: &crate::services::popper_score::PopperCategoryScore,
    is_gateway: bool,
    verbose: bool,
    failures_only: bool,
) {
    if category.is_not_applicable {
        output.push_str(&format!("  ⚪ {}: N/A\n", name));
        return;
    }

    let percentage = category.percentage();
    let icon = percentage_icon(percentage);
    let gateway_marker = if is_gateway { " [GATEWAY]" } else { "" };

    output.push_str(&format!(
        "  {} {}: {:.1}/{:.0} ({:.1}%){}\n",
        icon, name, category.earned, category.max, percentage, gateway_marker
    ));

    if verbose && !failures_only {
        for sub in &category.sub_scores {
            let sub_icon = if sub.earned >= sub.max * 0.8 {
                "  ✓"
            } else if sub.earned >= sub.max * 0.5 {
                "  ~"
            } else {
                "  ✗"
            };
            output.push_str(&format!(
                "    {} {}: {:.1}/{:.0} - {}\n",
                sub_icon, sub.id, sub.earned, sub.max, sub.description
            ));
        }
    }
}

/// Append text-formatted recommendations to the output
fn format_text_recommendations(output: &mut String, score: &PopperScore, failures_only: bool) {
    if score.recommendations.is_empty() || (failures_only && score.gateway_passed) {
        return;
    }
    output.push_str("💡  Recommendations\n");
    for rec in &score.recommendations {
        let icon = priority_icon_text(&rec.priority);
        output.push_str(&format!(
            "  {} [{}] {}\n",
            icon, rec.category, rec.description
        ));
        if let Some(cmd) = &rec.command {
            output.push_str(&format!("     $ {}\n", cmd));
        }
    }
    output.push('\n');
}

/// Format score as human-readable text
fn format_text(score: &PopperScore, verbose: bool, failures_only: bool) -> String {
    let mut output = String::new();

    // Header
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str(&format!(
        "🔬  Popper Falsifiability Score v{}\n",
        score.metadata.version
    ));
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push('\n');

    // Gateway status
    if score.gateway_passed {
        output.push_str("✅  Gateway: PASSED (Falsifiability >= 60%)\n");
    } else {
        output.push_str("❌  Gateway: FAILED (Falsifiability < 60%)\n");
        output.push_str("    Without falsifiable claims, score is 0.\n");
    }
    output.push('\n');

    // Summary
    output.push_str("📌  Summary\n");
    output.push_str(&format!(
        "  Score: {:.1}/{:.0}\n",
        score.raw_score, score.max_available
    ));
    output.push_str(&format!("  Normalized: {:.1}%\n", score.normalized_score));
    output.push_str(&format!("  Grade: {}\n", score.grade));
    output.push('\n');

    // Categories
    output.push_str("📂  Categories\n");
    for (name, category, is_gateway) in popper_category_entries(score) {
        format_text_category(
            &mut output,
            name,
            category,
            is_gateway,
            verbose,
            failures_only,
        );
    }
    output.push('\n');

    // Verdict
    output.push_str("📋  Verdict\n");
    output.push_str(&format!("  {}\n", score.analysis.verdict));
    output.push('\n');

    // Recommendations
    format_text_recommendations(&mut output, score, failures_only);

    // Footer
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    output
}
