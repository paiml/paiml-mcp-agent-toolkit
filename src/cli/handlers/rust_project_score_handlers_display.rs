// Included from rust_project_score_handlers.rs — NO use imports, NO #! attributes

/// Format score as human-readable text
fn format_text(
    score: &crate::services::rust_project_score::orchestrator::ProjectScore,
    recommendations: &[String],
    verbose: bool,
    failures_only: bool,
) -> String {
    // GH-46: --verbose not yet implemented for project score text output
    if verbose {
        eprintln!("Warning: --verbose is not yet implemented for project score text output. Flag ignored.");
    }
    use crate::cli::colors as c;
    use crate::services::rust_project_score::orchestrator::SPEC_VERSION;

    let mut output = String::new();

    // Header
    output.push_str(&format!("{}\n", c::rule()));
    output.push_str(&format!(
        "{}\n",
        c::header(&format!("Rust Project Score v{}", SPEC_VERSION))
    ));
    output.push_str(&format!("{}\n", c::rule()));
    output.push('\n');

    // Summary — exclude N/A categories from totals (#237).
    // #687: folded via `aggregation` (name-sorted, rounded) rather than over
    // `HashMap::values()`, so text cannot disagree with json/yaml about the
    // same number.
    use crate::services::rust_project_score::aggregation;
    let applicable_earned = aggregation::applicable_earned(&score.categories);
    let applicable_possible = aggregation::applicable_possible(&score.categories);
    output.push_str(&format!("{}\n", c::label("Summary")));
    output.push_str(&format!(
        "  Score: {}\n",
        c::score(applicable_earned, applicable_possible, 80.0, 60.0)
    ));
    output.push_str(&format!(
        "  Normalized: {} (avg of category %)\n",
        c::pct(score.percentage, 80.0, 60.0)
    ));
    // The points ratio, spelled out. "Score: 236.9/289" above and "Normalized:
    // 87.2%" here are two different quantities; naming the third makes the
    // relationship checkable instead of leaving the reader to divide and
    // conclude one of them is wrong.
    //
    // #717: this is also the figure the grade is derived from, so say so — the
    // grade used to come from "Normalized" instead, which is a different number.
    let points_percentage =
        crate::services::rust_project_score::orchestrator::points_percentage(&score.categories);
    output.push_str(&format!(
        "  Points: {} of possible points (grade basis)\n",
        c::pct(points_percentage, 80.0, 60.0)
    ));
    output.push_str(&format!(
        "  Grade: {}\n",
        c::grade(&score.grade.to_string())
    ));
    output.push('\n');

    // Categories
    output.push_str(&format!("{}\n", c::label("Categories")));

    // Sort categories by name for consistent output (#687: same ordering as
    // the json/yaml/markdown renderers). #943: `--failures-only` drops the
    // passing ones, using the single predicate in `aggregation`.
    let categories = aggregation::sorted_categories_filtered(&score.categories, failures_only);
    let hidden = score.categories.len() - categories.len();

    for (name, category) in categories {
        if !category.applicable {
            output.push_str(&format!("  {}  {}: N/A{}\n", c::seq(c::DIM), name, c::seq(c::RESET)));
            continue;
        }

        let percentage = category.percentage();

        let icon = if percentage >= 90.0 {
            format!("{}✓{}", c::seq(c::GREEN), c::seq(c::RESET))
        } else if percentage >= 70.0 {
            format!("{}⚠{}", c::seq(c::YELLOW), c::seq(c::RESET))
        } else {
            format!("{}✗{}", c::seq(c::RED), c::seq(c::RESET))
        };

        output.push_str(&format!(
            "  {} {}: {} ({})\n",
            icon,
            name,
            c::score(category.earned, category.max, 80.0, 60.0),
            c::pct(percentage, 80.0, 60.0)
        ));
    }
    // A filtered list must say what it left out, or the reader cannot tell a
    // clean project from a truncated report.
    if hidden > 0 {
        output.push_str(&format!(
            "  {}({hidden} passing categor{} hidden by --failures-only; totals above cover all of them){}\n",
            c::seq(c::DIM),
            if hidden == 1 { "y" } else { "ies" },
            c::seq(c::RESET)
        ));
    }
    output.push('\n');

    // Recommendations
    if !recommendations.is_empty() {
        output.push_str(&format!("{}\n", c::label("Recommendations")));
        for rec in recommendations {
            output.push_str(&format!("  {}{}{}\n", c::seq(c::DIM_WHITE), rec, c::seq(c::RESET)));
        }
        output.push('\n');
    }

    // Footer
    output.push_str(&format!("{}\n", c::rule()));

    output
}

/// Format score as Markdown
fn format_markdown(
    score: &crate::services::rust_project_score::orchestrator::ProjectScore,
    recommendations: &[String],
    verbose: bool,
    failures_only: bool,
) -> String {
    // GH-46: --verbose not yet implemented for project score markdown output
    if verbose {
        eprintln!("Warning: --verbose is not yet implemented for project score markdown output. Flag ignored.");
    }
    use crate::services::rust_project_score::orchestrator::SPEC_VERSION;

    let mut output = String::new();

    // Header
    output.push_str(&format!("# Rust Project Score v{}\n\n", SPEC_VERSION));

    // Summary — exclude N/A categories from totals (#237).
    // #687: same deterministic fold as text/json/yaml.
    use crate::services::rust_project_score::aggregation;
    let applicable_earned = aggregation::applicable_earned(&score.categories);
    let applicable_possible = aggregation::applicable_possible(&score.categories);
    output.push_str("## Summary\n\n");
    output.push_str(&format!(
        "- **Score**: {:.1}/{:.0}\n",
        applicable_earned, applicable_possible
    ));
    // ARITHMETIC SANITY: `percentage` does NOT follow from the two numbers on
    // the line above it — it is the unweighted mean of the 11 category
    // percentages, not `earned/possible`. On pmat's own tree that is 87.2% for
    // a 236.9/289 score whose points ratio is 82.0%. Markdown used to print the
    // two adjacent with no disclaimer at all (only the text renderer said
    // "avg of category %"), so a reader had every reason to check the division
    // and conclude one of them was wrong. Both are now named.
    //
    // #717: and the grade no longer comes from `percentage` — it comes from the
    // points ratio below, the one that follows from the Score line.
    let points_percentage =
        crate::services::rust_project_score::orchestrator::points_percentage(&score.categories);
    output.push_str(&format!(
        "- **Percentage**: {:.1}% (mean of category percentages)\n",
        score.percentage
    ));
    output.push_str(&format!(
        "- **Points**: {points_percentage:.1}% of possible points ({applicable_earned:.1}/{applicable_possible:.0}) — the grade is derived from this\n"
    ));
    output.push_str(&format!("- **Grade**: {}\n\n", score.grade));

    // Categories
    output.push_str("## Categories\n\n");
    output.push_str("| Category | Score | Percentage |\n");
    output.push_str("|----------|-------|------------|\n");

    let categories = aggregation::sorted_categories_filtered(&score.categories, failures_only);
    let hidden = score.categories.len() - categories.len();

    for (name, category) in categories {
        let percentage = category.percentage();

        let icon = if percentage >= aggregation::PASSING_PERCENTAGE {
            "Pass"
        } else if percentage >= 70.0 {
            "Warning"
        } else {
            "Fail"
        };

        output.push_str(&format!(
            "| {} {} | {:.1}/{:.0} | {:.1}% |\n",
            icon, name, category.earned, category.max, percentage
        ));
    }
    if hidden > 0 {
        output.push_str(&format!(
            "\n_{hidden} passing categories hidden by `--failures-only`; the totals above cover all of them._\n"
        ));
    }
    output.push('\n');

    // Recommendations
    if !recommendations.is_empty() {
        output.push_str("## Recommendations\n\n");
        for rec in recommendations {
            output.push_str(&format!("- {}\n", rec));
        }
        output.push('\n');
    }

    output
}
