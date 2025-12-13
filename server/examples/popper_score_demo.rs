//! Popper Falsifiability Score Example
//!
//! This example demonstrates how to use pmat's Popper Falsifiability Score
//! to evaluate a project's scientific rigor and falsifiability against
//! Karl Popper's standards for empirical science.
//!
//! Run with: `cargo run --example popper_score_demo`

use anyhow::Result;
use pmat::services::popper_score::{score_project, PopperScore};
use std::path::Path;

fn main() -> Result<()> {
    println!("🔬 Popper Falsifiability Score Demo\n");
    println!("{}", "=".repeat(60));

    // Example 1: Score current project
    println!("\nExample 1: Scoring current project");
    println!("{}", "-".repeat(40));

    let score = score_project(Path::new("."))?;
    print_score_summary(&score);

    // Example 2: Demonstrate gateway logic
    println!("\nExample 2: Understanding the Falsifiability Gateway");
    println!("{}", "-".repeat(40));
    demonstrate_gateway(&score);

    // Example 3: Category breakdown
    println!("\nExample 3: Category Breakdown");
    println!("{}", "-".repeat(40));
    print_categories(&score);

    // Example 4: Recommendations
    println!("\nExample 4: Improvement Recommendations");
    println!("{}", "-".repeat(40));
    print_recommendations(&score);

    println!("\n{}", "=".repeat(60));
    println!("🎉 Popper Score demo completed!");

    Ok(())
}

fn print_score_summary(score: &PopperScore) {
    println!(
        "  Gateway Status: {}",
        if score.gateway_passed {
            "✅ PASSED"
        } else {
            "❌ FAILED"
        }
    );
    println!(
        "  Raw Score: {:.1}/{:.0}",
        score.raw_score, score.max_available
    );
    println!("  Normalized: {:.1}%", score.normalized_score);
    println!("  Grade: {}", score.grade);
    println!("  Verdict: {}", score.analysis.verdict);
}

fn demonstrate_gateway(score: &PopperScore) {
    let falsifiability_pct = score.categories.falsifiability.percentage();

    println!(
        "  Falsifiability Score: {:.1}/{:.0} ({:.1}%)",
        score.categories.falsifiability.earned,
        score.categories.falsifiability.max,
        falsifiability_pct
    );

    println!("\n  The Falsifiability Gateway requires Category A >= 60%");
    println!("  This implements Karl Popper's demarcation criterion:");
    println!("  \"A theory that is not refutable by any conceivable event");
    println!("   is non-scientific.\" (The Logic of Scientific Discovery)\n");

    if score.gateway_passed {
        println!(
            "  ✅ Your project passes the gateway ({:.1}% >= 60%)",
            falsifiability_pct
        );
        println!("     The final score reflects all categories.");
    } else {
        println!(
            "  ❌ Your project fails the gateway ({:.1}% < 60%)",
            falsifiability_pct
        );
        println!("     The final score is 0 until falsifiability improves.");
        println!("\n  To improve, add:");
        println!("     - Explicit falsifiable claims in README");
        println!("     - Measurable success/failure criteria");
        println!("     - Comprehensive test coverage");
    }
}

fn print_categories(score: &PopperScore) {
    let categories = [
        (
            "A. Falsifiability & Testability",
            &score.categories.falsifiability,
            true,
        ),
        (
            "B. Reproducibility Infrastructure",
            &score.categories.reproducibility,
            false,
        ),
        (
            "C. Transparency & Openness",
            &score.categories.transparency,
            false,
        ),
        (
            "D. Statistical Rigor",
            &score.categories.statistical_rigor,
            false,
        ),
        (
            "E. Historical Integrity",
            &score.categories.historical_integrity,
            false,
        ),
        (
            "F. ML/AI Reproducibility",
            &score.categories.ml_reproducibility,
            false,
        ),
    ];

    for (name, category, is_gateway) in categories {
        if category.is_not_applicable {
            println!("  ⚪ {}: N/A", name);
            continue;
        }

        let pct = category.percentage();
        let icon = if pct >= 80.0 {
            "✅"
        } else if pct >= 60.0 {
            "⚠️"
        } else {
            "❌"
        };
        let gateway_marker = if is_gateway { " [GATEWAY]" } else { "" };

        println!(
            "  {} {}: {:.1}/{:.0} ({:.1}%){}",
            icon, name, category.earned, category.max, pct, gateway_marker
        );
    }
}

fn print_recommendations(score: &PopperScore) {
    if score.recommendations.is_empty() {
        println!("  ✅ No recommendations - excellent scientific rigor!");
        return;
    }

    for rec in &score.recommendations {
        let priority = match rec.priority {
            pmat::services::popper_score::RecommendationPriority::Critical => "🔴 Critical",
            pmat::services::popper_score::RecommendationPriority::High => "🟠 High",
            pmat::services::popper_score::RecommendationPriority::Medium => "🟡 Medium",
            pmat::services::popper_score::RecommendationPriority::Low => "🟢 Low",
        };

        println!("  [{}] {}: {}", priority, rec.category, rec.description);
        if let Some(cmd) = &rec.command {
            println!("      $ {}", cmd);
        }
    }
}
