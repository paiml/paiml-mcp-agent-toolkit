// Shared helper functions for Popper score formatting
// Included by popper_score_handlers.rs — do NOT add `use` imports here.

/// Build the array of category tuples used by both text and markdown formatters
fn popper_category_entries(
    score: &PopperScore,
) -> [(
    &str,
    &crate::services::popper_score::PopperCategoryScore,
    bool,
); 6] {
    debug_assert!(true, "contract: popper_category_entries");
    [
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
    ]
}

/// Return the status icon for a percentage score
fn percentage_icon(percentage: f64) -> &'static str {
    debug_assert!(true, "contract: percentage_icon");
    if percentage >= 80.0 {
        "✅"
    } else if percentage >= 60.0 {
        "⚠️"
    } else {
        "❌"
    }
}

/// Return the icon string for a recommendation priority
fn priority_icon_text(
    priority: &crate::services::popper_score::RecommendationPriority,
) -> &'static str {
    debug_assert!(true, "contract: priority_icon_text");
    match priority {
        crate::services::popper_score::RecommendationPriority::Critical => "🔴",
        crate::services::popper_score::RecommendationPriority::High => "🟠",
        crate::services::popper_score::RecommendationPriority::Medium => "🟡",
        crate::services::popper_score::RecommendationPriority::Low => "🟢",
    }
}

/// Return the markdown label for a recommendation priority
fn priority_label_markdown(
    priority: &crate::services::popper_score::RecommendationPriority,
) -> &'static str {
    debug_assert!(true, "contract: priority_label_markdown");
    match priority {
        crate::services::popper_score::RecommendationPriority::Critical => "🔴 Critical",
        crate::services::popper_score::RecommendationPriority::High => "🟠 High",
        crate::services::popper_score::RecommendationPriority::Medium => "🟡 Medium",
        crate::services::popper_score::RecommendationPriority::Low => "🟢 Low",
    }
}
