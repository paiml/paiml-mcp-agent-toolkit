// Included from rust_project_score_handlers.rs — NO use imports, NO #! attributes

/// Format score as human-readable text
fn format_text(
    score: &crate::services::rust_project_score::orchestrator::ProjectScore,
    recommendations: &[String],
    _verbose: bool,
) -> String {
    use crate::services::rust_project_score::orchestrator::SPEC_VERSION;

    let mut output = String::new();

    // Header
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str(&format!("🦀  Rust Project Score v{}\n", SPEC_VERSION));
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push('\n');

    // Summary — exclude N/A categories from totals (#237)
    let applicable_earned: f64 = score
        .categories
        .values()
        .filter(|cat| cat.applicable)
        .map(|cat| cat.earned)
        .sum();
    let applicable_possible: f64 = score
        .categories
        .values()
        .filter(|cat| cat.applicable)
        .map(|cat| cat.max)
        .sum();
    output.push_str("📌  Summary\n");
    output.push_str(&format!(
        "  Score: {:.1}/{:.0}\n",
        applicable_earned, applicable_possible
    ));
    output.push_str(&format!(
        "  Normalized: {:.1}% (avg of category %)\n",
        score.percentage
    ));
    output.push_str(&format!("  Grade: {}\n", score.grade));
    output.push('\n');

    // Categories
    output.push_str("📂  Categories\n");

    // Sort categories by name for consistent output
    let mut categories: Vec<_> = score.categories.iter().collect();
    categories.sort_by_key(|(name, _)| *name);

    for (name, category) in categories {
        if !category.applicable {
            output.push_str(&format!("  ⬚  {}: N/A\n", name));
            continue;
        }

        let percentage = category.percentage();

        let icon = if percentage >= 90.0 {
            "✅"
        } else if percentage >= 70.0 {
            "⚠️"
        } else {
            "❌"
        };

        output.push_str(&format!(
            "  {} {}: {:.1}/{:.0} ({:.1}%)\n",
            icon, name, category.earned, category.max, percentage
        ));
    }
    output.push('\n');

    // Recommendations
    if !recommendations.is_empty() {
        output.push_str("💡  Recommendations\n");
        for rec in recommendations {
            output.push_str(&format!("  • {}\n", rec));
        }
        output.push('\n');
    }

    // Footer
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    output
}

/// Format score as Markdown
fn format_markdown(
    score: &crate::services::rust_project_score::orchestrator::ProjectScore,
    recommendations: &[String],
    _verbose: bool,
) -> String {
    use crate::services::rust_project_score::orchestrator::SPEC_VERSION;

    let mut output = String::new();

    // Header
    output.push_str(&format!("# 🦀 Rust Project Score v{}\n\n", SPEC_VERSION));

    // Summary — exclude N/A categories from totals (#237)
    let applicable_earned: f64 = score
        .categories
        .values()
        .filter(|cat| cat.applicable)
        .map(|cat| cat.earned)
        .sum();
    let applicable_possible: f64 = score
        .categories
        .values()
        .filter(|cat| cat.applicable)
        .map(|cat| cat.max)
        .sum();
    output.push_str("## 📌 Summary\n\n");
    output.push_str(&format!(
        "- **Score**: {:.1}/{:.0}\n",
        applicable_earned, applicable_possible
    ));
    output.push_str(&format!("- **Percentage**: {:.1}%\n", score.percentage));
    output.push_str(&format!("- **Grade**: {}\n\n", score.grade));

    // Categories
    output.push_str("## 📂 Categories\n\n");
    output.push_str("| Category | Score | Percentage |\n");
    output.push_str("|----------|-------|------------|\n");

    let mut categories: Vec<_> = score.categories.iter().collect();
    categories.sort_by_key(|(name, _)| *name);

    for (name, category) in categories {
        let percentage = category.percentage();

        let icon = if percentage >= 90.0 {
            "✅"
        } else if percentage >= 70.0 {
            "⚠️"
        } else {
            "❌"
        };

        output.push_str(&format!(
            "| {} {} | {:.1}/{:.0} | {:.1}% |\n",
            icon, name, category.earned, category.max, percentage
        ));
    }
    output.push('\n');

    // Recommendations
    if !recommendations.is_empty() {
        output.push_str("## 💡 Recommendations\n\n");
        for rec in recommendations {
            output.push_str(&format!("- {}\n", rec));
        }
        output.push('\n');
    }

    output
}
