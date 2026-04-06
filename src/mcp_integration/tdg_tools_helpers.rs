// Included from tdg_tools.rs — helper functions for TDG tools

// Helper function to determine if severity should be included
fn should_include_severity(severity: &str, min_severity: &str) -> bool {
    debug_assert!(!severity.is_empty(), "severity must not be empty");
    debug_assert!(!min_severity.is_empty(), "min_severity must not be empty");
    let severity_levels = ["low", "medium", "high", "critical"];
    let min_index = severity_levels
        .iter()
        .position(|&s| s == min_severity)
        .unwrap_or(1);
    let severity_index = severity_levels
        .iter()
        .position(|&s| s == severity)
        .unwrap_or(0);
    severity_index >= min_index
}

// Helper function to generate actionable suggestions
fn generate_suggestion(reason: &str, category: &str) -> String {
    debug_assert!(!category.is_empty(), "category must not be empty");
    let reason_lower = reason.to_lowercase();

    if reason_lower.contains("cyclomatic complexity") || reason_lower.contains("complexity") {
        "Consider breaking down complex functions into smaller, single-responsibility functions. Extract nested logic into helper methods.".to_string()
    } else if reason_lower.contains("nesting") || reason_lower.contains("deep") {
        "Reduce nesting depth by using early returns, guard clauses, or extracting nested logic into separate functions.".to_string()
    } else if reason_lower.contains("duplication") {
        "Extract duplicated code into reusable functions or modules. Consider using design patterns like Template Method or Strategy.".to_string()
    } else if reason_lower.contains("documentation") || reason_lower.contains("doc") {
        "Add comprehensive documentation including function descriptions, parameter explanations, and usage examples.".to_string()
    } else if reason_lower.contains("coupling") {
        "Reduce coupling by using dependency injection, interfaces, or event-driven architecture. Apply SOLID principles.".to_string()
    } else if reason_lower.contains("consistency") {
        "Improve code consistency by following established style guides and naming conventions. Use automated formatters.".to_string()
    } else {
        format!(
            "Review {} and apply refactoring techniques to improve code quality.",
            category
        )
    }
}
