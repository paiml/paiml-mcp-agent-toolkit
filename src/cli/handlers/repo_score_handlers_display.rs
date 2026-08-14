// Display/formatting functions for repo-score output
// Included from repo_score_handlers.rs — no `use` imports or inner attributes

/// Every category the report knows about, in report order.
///
/// One list, used by the text, markdown and machine renderers alike, so a
/// display filter cannot apply to some of them and not the others.
fn scored_categories(
    score: &RepoScore,
) -> [(
    &'static str,
    &'static str,
    &crate::services::repo_score::CategoryScore,
); 6] {
    [
        (
            "Documentation",
            "documentation",
            &score.categories.documentation,
        ),
        (
            "Pre-commit Hooks",
            "precommit_hooks",
            &score.categories.precommit_hooks,
        ),
        (
            "Repository Hygiene",
            "repository_hygiene",
            &score.categories.repository_hygiene,
        ),
        (
            "Build/Test Automation",
            "build_test_automation",
            &score.categories.build_test_automation,
        ),
        (
            "Continuous Integration",
            "continuous_integration",
            &score.categories.continuous_integration,
        ),
        (
            "PMAT Compliance",
            "pmat_compliance",
            &score.categories.pmat_compliance,
        ),
    ]
}

/// `--failures-only` keeps the rows a reader has to act on.
///
/// It is a DISPLAY filter and nothing else: the score, the grade and the
/// category totals are the ones the full run measured, and are still printed.
/// Hiding a row must never change a number — the flag was once wired to
/// `ScorerConfig.skip_slow_checks` and produced 99.0 where the plain run
/// produced 96.0 on the same repository.
fn keep_category(
    category: &crate::services::repo_score::CategoryScore,
    failures_only: bool,
) -> bool {
    !failures_only
        || !matches!(
            category.status,
            crate::services::repo_score::ScoreStatus::Pass
        )
}

/// Format score as human-readable text
fn format_text(score: &RepoScore, verbose: bool, failures_only: bool) -> String {
    use crate::cli::colors as c;

    let mut output = String::new();

    // Header
    output.push_str(&format!("{}\n", c::rule()));
    output.push_str(&format!("{}\n", c::header("Repository Health Score")));
    output.push_str(&format!("{}\n", c::rule()));
    output.push('\n');

    // Summary
    output.push_str(&format!("{}\n", c::label("Summary")));
    output.push_str(&format!(
        "  Score: {}\n",
        c::score(score.total_score, 100.0, 80.0, 60.0)
    ));
    output.push_str(&format!("  Grade: {}\n", c::grade(score.grade.as_str())));
    output.push('\n');

    // Categories
    output.push_str(&format!("{}\n", c::label("Categories")));
    let all = scored_categories(score);
    let mut shown = 0usize;
    for (name, _, category) in all {
        if !keep_category(category, failures_only) {
            continue;
        }
        shown += 1;
        output.push_str(&format_category(name, category, verbose));
    }
    if shown == 0 {
        output.push_str(&format!(
            "  {}\n",
            c::dim("No category failed or warned (--failures-only).")
        ));
    }
    // A filtered list that looks complete is how "1 of 6" gets read as "1".
    if shown < all.len() {
        output.push_str(&format!(
            "  {}\n",
            c::dim(&format!(
                "Showing {shown} of {} categories (--failures-only).",
                all.len()
            ))
        ));
    }
    output.push('\n');

    // Recommendations
    if !score.recommendations.is_empty() {
        output.push_str(&format!("{}\n", c::label("Recommendations")));
        for rec in &score.recommendations {
            use crate::services::repo_score::Priority;
            let priority_text = match rec.priority {
                Priority::Critical => format!("{}P0{}", c::seq(c::RED), c::seq(c::RESET)),
                Priority::High => format!("{}P1{}", c::seq(c::RED), c::seq(c::RESET)),
                Priority::Medium => format!("{}P2{}", c::seq(c::YELLOW), c::seq(c::RESET)),
                Priority::Low => format!("{}P3{}", c::seq(c::GREEN), c::seq(c::RESET)),
            };
            output.push_str(&format!(
                "  {} {}: {}\n",
                priority_text, rec.category, rec.description
            ));
        }
        output.push('\n');
    }

    // Footer
    output.push_str(&format!("{}\n", c::rule()));

    output
}

/// Format a single category
fn format_category(
    name: &str,
    category: &crate::services::repo_score::CategoryScore,
    verbose: bool,
) -> String {
    use crate::cli::colors as c;

    let mut output = String::new();
    let status_icon = match category.status {
        crate::services::repo_score::ScoreStatus::Pass => {
            format!("{}✓{}", c::seq(c::GREEN), c::seq(c::RESET))
        }
        crate::services::repo_score::ScoreStatus::Warning => {
            format!("{}⚠{}", c::seq(c::YELLOW), c::seq(c::RESET))
        }
        crate::services::repo_score::ScoreStatus::Fail => {
            format!("{}✗{}", c::seq(c::RED), c::seq(c::RESET))
        }
    };

    output.push_str(&format!(
        "  {} {:<25} {} ({})\n",
        status_icon,
        name,
        c::score(category.score, category.max_score, 80.0, 60.0),
        c::pct(category.percentage, 80.0, 60.0)
    ));

    if verbose && !category.findings.is_empty() {
        for finding in &category.findings {
            output.push_str(&format!(
                "     {}{}{}\n",
                c::seq(c::DIM),
                finding.message,
                c::seq(c::RESET)
            ));
        }
    }

    output
}

/// The score document, with `--failures-only` applied to `categories`.
///
/// The filter reaches the machine formats too: a JSON consumer asking for
/// failures only and getting all six categories back cannot tell the flag was
/// honoured. `failures_only` is stamped into the document so a reader knows the
/// category map is a filtered view of a full measurement, and the totals are
/// left exactly as measured.
fn score_document(score: &RepoScore, failures_only: bool) -> Result<serde_json::Value> {
    let mut doc = serde_json::to_value(score).context("Failed to serialize score")?;
    doc["failures_only"] = serde_json::Value::Bool(failures_only);
    if !failures_only {
        return Ok(doc);
    }

    let kept: serde_json::Map<String, serde_json::Value> = scored_categories(score)
        .into_iter()
        .filter(|(_, _, category)| keep_category(category, true))
        .map(|(_, key, _)| {
            let value = doc["categories"][key].clone();
            (key.to_string(), value)
        })
        .collect();
    doc["categories"] = serde_json::Value::Object(kept);
    Ok(doc)
}

/// Format score as JSON
fn format_json(score: &RepoScore, failures_only: bool) -> Result<String> {
    serde_json::to_string_pretty(&score_document(score, failures_only)?)
        .context("Failed to serialize to JSON")
}

/// Format score as YAML
fn format_yaml(score: &RepoScore, failures_only: bool) -> Result<String> {
    serde_yaml_ng::to_string(&score_document(score, failures_only)?)
        .context("Failed to serialize to YAML")
}

/// Format score as Markdown
fn format_markdown(score: &RepoScore, failures_only: bool) -> String {
    let mut output = String::new();

    output.push_str("# Repository Health Score\n\n");

    output.push_str("## Summary\n\n");
    output.push_str(&format!("- **Score**: {:.1}/100\n", score.total_score));
    output.push_str(&format!("- **Grade**: {}\n\n", score.grade.as_str()));

    output.push_str("## Category Scores\n\n");
    output.push_str("| Category | Score | Max | Percentage | Status |\n");
    output.push_str("|----------|-------|-----|------------|--------|\n");

    let categories = scored_categories(score);
    let mut shown = 0usize;
    for (name, _, cat) in categories {
        if !keep_category(cat, failures_only) {
            continue;
        }
        shown += 1;
        let status = match cat.status {
            crate::services::repo_score::ScoreStatus::Pass => "Pass",
            crate::services::repo_score::ScoreStatus::Warning => "Warning",
            crate::services::repo_score::ScoreStatus::Fail => "Fail",
        };
        output.push_str(&format!(
            "| {} | {:.1} | {:.1} | {:.1}% | {} |\n",
            name, cat.score, cat.max_score, cat.percentage, status
        ));
    }
    if shown < categories.len() {
        output.push_str(&format!(
            "\n_Showing {shown} of {} categories (--failures-only)._\n",
            categories.len()
        ));
    }

    output.push_str("\n## Recommendations\n\n");
    if score.recommendations.is_empty() {
        output.push_str("No recommendations - excellent work!\n");
    } else {
        for rec in &score.recommendations {
            output.push_str(&format!("- **{}**: {}\n", rec.category, rec.description));
        }
    }

    output
}
