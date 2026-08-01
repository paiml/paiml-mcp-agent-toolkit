// Included from rust_project_score_handlers.rs — NO use imports, NO #! attributes

/// Format score as human-readable text
fn format_text(
    score: &crate::services::rust_project_score::orchestrator::ProjectScore,
    recommendations: &[String],
    verbose: bool,
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
    #[allow(clippy::cast_precision_loss)]
    let points_percentage = if applicable_possible > 0.0 {
        (applicable_earned / applicable_possible) * 100.0
    } else {
        0.0
    };
    output.push_str(&format!(
        "  Points: {} of possible points\n",
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
    // the json/yaml/markdown renderers)
    let categories = aggregation::sorted_categories(&score.categories);

    for (name, category) in categories {
        if !category.applicable {
            output.push_str(&format!("  {}  {}: N/A{}\n", c::DIM, name, c::RESET));
            continue;
        }

        let percentage = category.percentage();

        let icon = if percentage >= 90.0 {
            format!("{}✓{}", c::GREEN, c::RESET)
        } else if percentage >= 70.0 {
            format!("{}⚠{}", c::YELLOW, c::RESET)
        } else {
            format!("{}✗{}", c::RED, c::RESET)
        };

        output.push_str(&format!(
            "  {} {}: {} ({})\n",
            icon,
            name,
            c::score(category.earned, category.max, 80.0, 60.0),
            c::pct(percentage, 80.0, 60.0)
        ));
    }
    output.push('\n');

    // Recommendations
    if !recommendations.is_empty() {
        output.push_str(&format!("{}\n", c::label("Recommendations")));
        for rec in recommendations {
            output.push_str(&format!("  {}{}{}\n", c::DIM_WHITE, rec, c::RESET));
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
    #[allow(clippy::cast_precision_loss)]
    let points_percentage = if applicable_possible > 0.0 {
        (applicable_earned / applicable_possible) * 100.0
    } else {
        0.0
    };
    output.push_str(&format!(
        "- **Percentage**: {:.1}% (mean of category percentages — the grade is derived from this)\n",
        score.percentage
    ));
    output.push_str(&format!(
        "- **Points**: {points_percentage:.1}% of possible points ({applicable_earned:.1}/{applicable_possible:.0})\n"
    ));
    output.push_str(&format!("- **Grade**: {}\n\n", score.grade));

    // Categories
    output.push_str("## Categories\n\n");
    output.push_str("| Category | Score | Percentage |\n");
    output.push_str("|----------|-------|------------|\n");

    let categories = aggregation::sorted_categories(&score.categories);

    for (name, category) in categories {
        let percentage = category.percentage();

        let icon = if percentage >= 90.0 {
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
