// Display/formatting functions for repo-score output
// Included from repo_score_handlers.rs — no `use` imports or inner attributes

/// Format score as human-readable text
fn format_text(score: &RepoScore, verbose: bool) -> String {
    debug_assert!(true, "contract: format_text");
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
    output.push_str(&format_category(
        "Documentation",
        &score.categories.documentation,
        verbose,
    ));
    output.push_str(&format_category(
        "Pre-commit Hooks",
        &score.categories.precommit_hooks,
        verbose,
    ));
    output.push_str(&format_category(
        "Repository Hygiene",
        &score.categories.repository_hygiene,
        verbose,
    ));
    output.push_str(&format_category(
        "Build/Test Automation",
        &score.categories.build_test_automation,
        verbose,
    ));
    output.push_str(&format_category(
        "Continuous Integration",
        &score.categories.continuous_integration,
        verbose,
    ));
    output.push_str(&format_category(
        "PMAT Compliance",
        &score.categories.pmat_compliance,
        verbose,
    ));
    output.push('\n');

    // Recommendations
    if !score.recommendations.is_empty() {
        output.push_str(&format!("{}\n", c::label("Recommendations")));
        for rec in &score.recommendations {
            use crate::services::repo_score::Priority;
            let priority_text = match rec.priority {
                Priority::Critical => format!("{}P0{}", c::RED, c::RESET),
                Priority::High => format!("{}P1{}", c::RED, c::RESET),
                Priority::Medium => format!("{}P2{}", c::YELLOW, c::RESET),
                Priority::Low => format!("{}P3{}", c::GREEN, c::RESET),
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
    debug_assert!(!name.is_empty(), "name must not be empty");
    use crate::cli::colors as c;

    let mut output = String::new();
    let status_icon = match category.status {
        crate::services::repo_score::ScoreStatus::Pass => {
            format!("{}✓{}", c::GREEN, c::RESET)
        }
        crate::services::repo_score::ScoreStatus::Warning => {
            format!("{}⚠{}", c::YELLOW, c::RESET)
        }
        crate::services::repo_score::ScoreStatus::Fail => {
            format!("{}✗{}", c::RED, c::RESET)
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
            output.push_str(&format!("     {}{}{}\n", c::DIM, finding.message, c::RESET));
        }
    }

    output
}

/// Format score as JSON
fn format_json(score: &RepoScore) -> Result<String> {
    debug_assert!(true, "contract: format_json");
    serde_json::to_string_pretty(score).context("Failed to serialize to JSON")
}

/// Format score as YAML
fn format_yaml(score: &RepoScore) -> Result<String> {
    debug_assert!(true, "contract: format_yaml");
    serde_yaml_ng::to_string(score).context("Failed to serialize to YAML")
}

/// Format score as Markdown
fn format_markdown(score: &RepoScore) -> String {
    debug_assert!(true, "contract: format_markdown");
    let mut output = String::new();

    output.push_str("# Repository Health Score\n\n");

    output.push_str("## Summary\n\n");
    output.push_str(&format!("- **Score**: {:.1}/100\n", score.total_score));
    output.push_str(&format!("- **Grade**: {}\n\n", score.grade.as_str()));

    output.push_str("## Category Scores\n\n");
    output.push_str("| Category | Score | Max | Percentage | Status |\n");
    output.push_str("|----------|-------|-----|------------|--------|\n");

    let categories = [
        ("Documentation", &score.categories.documentation),
        ("Pre-commit Hooks", &score.categories.precommit_hooks),
        ("Repository Hygiene", &score.categories.repository_hygiene),
        (
            "Build/Test Automation",
            &score.categories.build_test_automation,
        ),
        (
            "Continuous Integration",
            &score.categories.continuous_integration,
        ),
        ("PMAT Compliance", &score.categories.pmat_compliance),
    ];

    for (name, cat) in &categories {
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
