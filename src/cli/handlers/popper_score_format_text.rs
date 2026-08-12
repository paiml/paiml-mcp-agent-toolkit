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
    use crate::cli::colors as c;

    if category.is_not_applicable {
        output.push_str(&format!("  {}  {}: N/A{}\n", c::seq(c::DIM), name, c::seq(c::RESET)));
        return;
    }

    let percentage = category.percentage();
    let icon = percentage_icon(percentage);
    let gateway_marker = if is_gateway {
        format!(" {}[GATEWAY]{}", c::seq(c::BOLD_YELLOW), c::seq(c::RESET))
    } else {
        String::new()
    };

    output.push_str(&format!(
        "  {} {}: {} ({}){}\n",
        icon,
        name,
        c::score(category.earned, category.max, 80.0, 60.0),
        c::pct(percentage, 80.0, 60.0),
        gateway_marker
    ));

    if verbose && !failures_only {
        for sub in &category.sub_scores {
            let sub_icon = if sub.earned >= sub.max * 0.8 {
                format!("  {}✓{}", c::seq(c::GREEN), c::seq(c::RESET))
            } else if sub.earned >= sub.max * 0.5 {
                format!("  {}~{}", c::seq(c::YELLOW), c::seq(c::RESET))
            } else {
                format!("  {}✗{}", c::seq(c::RED), c::seq(c::RESET))
            };
            output.push_str(&format!(
                "    {} {}: {} - {}\n",
                sub_icon,
                sub.id,
                c::score(sub.earned, sub.max, 80.0, 50.0),
                sub.description
            ));
        }
    }
}

/// Append text-formatted recommendations to the output
fn format_text_recommendations(output: &mut String, score: &PopperScore, failures_only: bool) {
    use crate::cli::colors as c;

    if score.recommendations.is_empty() || (failures_only && score.gateway_passed) {
        return;
    }
    output.push_str(&format!("{}\n", c::label("Recommendations")));
    for rec in &score.recommendations {
        let icon = priority_icon_text(&rec.priority);
        output.push_str(&format!(
            "  {} [{}] {}\n",
            icon, rec.category, rec.description
        ));
        if let Some(cmd) = &rec.command {
            output.push_str(&format!(
                "     {}$ {}{}\n",
                c::seq(c::DIM_CYAN),
                cmd,
                c::seq(c::RESET)
            ));
        }
    }
    output.push('\n');
}

/// Format score as human-readable text
fn format_text(score: &PopperScore, verbose: bool, failures_only: bool) -> String {
    use crate::cli::colors as c;

    let mut output = String::new();

    // Header
    output.push_str(&format!("{}\n", c::rule()));
    output.push_str(&format!(
        "{}\n",
        c::header(&format!(
            "Popper Falsifiability Score v{}",
            score.metadata.version
        ))
    ));
    output.push_str(&format!("{}\n", c::rule()));
    output.push('\n');

    // Gateway status
    if score.gateway_passed {
        output.push_str(&format!(
            "{} Gateway: PASSED (Falsifiability >= 60%)\n",
            c::pass("")
        ));
    } else {
        output.push_str(&format!(
            "{} Gateway: FAILED (Falsifiability < 60%)\n",
            c::fail("")
        ));
        output.push_str(&format!(
            "    {}Without falsifiable claims, score is 0.{}\n",
            c::seq(c::DIM), c::seq(c::RESET)
        ));
    }
    output.push('\n');

    // Summary
    output.push_str(&format!("{}\n", c::label("Summary")));
    output.push_str(&format!(
        "  Score: {}\n",
        c::score(score.raw_score, score.max_available, 80.0, 60.0)
    ));
    output.push_str(&format!(
        "  Normalized: {}\n",
        c::pct(score.normalized_score, 80.0, 60.0)
    ));
    output.push_str(&format!("  Grade: {}\n", c::grade(&score.grade.to_string())));
    output.push('\n');

    // Categories
    output.push_str(&format!("{}\n", c::label("Categories")));
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
    output.push_str(&format!("{}\n", c::label("Verdict")));
    output.push_str(&format!("  {}\n", score.analysis.verdict));
    output.push('\n');

    // Recommendations
    format_text_recommendations(&mut output, score, failures_only);

    // Footer
    output.push_str(&format!("{}\n", c::rule()));

    output
}
