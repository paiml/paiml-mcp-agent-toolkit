// Display/formatting functions for repo-score output
// Included from repo_score_handlers.rs — no `use` imports or inner attributes

/// Format score as human-readable text
fn format_text(score: &RepoScore, verbose: bool) -> String {
    let mut output = String::new();

    // Header
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str("📊  Repository Health Score\n");
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push('\n');

    // Summary
    output.push_str("📌  Summary\n");
    output.push_str(&format!("  Score: {:.1}/100\n", score.total_score));
    output.push_str(&format!("  Grade: {}\n", score.grade.as_str()));
    output.push('\n');

    // Categories
    output.push_str("📂  Categories\n");
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
        output.push_str("💡  Recommendations\n");
        for rec in &score.recommendations {
            use crate::services::repo_score::Priority;
            output.push_str(&format!(
                "  {} {}: {}\n",
                match rec.priority {
                    Priority::Critical => "🔴",
                    Priority::High => "🔴",
                    Priority::Medium => "🟡",
                    Priority::Low => "🟢",
                },
                rec.category,
                rec.description
            ));
        }
        output.push('\n');
    }

    // Footer
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    output
}

/// Format a single category
fn format_category(
    name: &str,
    category: &crate::services::repo_score::CategoryScore,
    verbose: bool,
) -> String {
    let mut output = String::new();
    let status_icon = match category.status {
        crate::services::repo_score::ScoreStatus::Pass => "✅",
        crate::services::repo_score::ScoreStatus::Warning => "⚠️",
        crate::services::repo_score::ScoreStatus::Fail => "❌",
    };

    output.push_str(&format!(
        "  {} {:<25} {:.1}/{:.1} ({:.1}%)\n",
        status_icon, name, category.score, category.max_score, category.percentage
    ));

    if verbose && !category.findings.is_empty() {
        for finding in &category.findings {
            output.push_str(&format!("     • {}\n", finding.message));
        }
    }

    output
}

/// Format score as JSON
fn format_json(score: &RepoScore) -> Result<String> {
    serde_json::to_string_pretty(score).context("Failed to serialize to JSON")
}

/// Format score as YAML
fn format_yaml(score: &RepoScore) -> Result<String> {
    serde_yaml_ng::to_string(score).context("Failed to serialize to YAML")
}

/// Format score as Markdown
fn format_markdown(score: &RepoScore) -> String {
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
            crate::services::repo_score::ScoreStatus::Pass => "✅ Pass",
            crate::services::repo_score::ScoreStatus::Warning => "⚠️ Warning",
            crate::services::repo_score::ScoreStatus::Fail => "❌ Fail",
        };
        output.push_str(&format!(
            "| {} | {:.1} | {:.1} | {:.1}% | {} |\n",
            name, cat.score, cat.max_score, cat.percentage, status
        ));
    }

    output.push_str("\n## Recommendations\n\n");
    if score.recommendations.is_empty() {
        output.push_str("No recommendations - excellent work! 🎉\n");
    } else {
        for rec in &score.recommendations {
            output.push_str(&format!("- **{}**: {}\n", rec.category, rec.description));
        }
    }

    output
}
