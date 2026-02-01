//! Handler utility functions - unit testable pure functions extracted from CLI handlers
//!
//! This module contains pure functions that were extracted from CLI handlers
//! to improve unit testability and coverage. These functions have no side effects
//! and can be easily tested without CLI argument parsing overhead.

use crate::cli;

/// Convert deep context DAG type to standard DAG type
///
/// # Examples
///
/// ```
/// use pmat::cli::handlers::handler_utils::convert_deep_context_dag_type;
/// use pmat::cli::enums::{DeepContextDagType, DagType};
///
/// let result = convert_deep_context_dag_type(DeepContextDagType::CallGraph);
/// assert_eq!(result, DagType::CallGraph);
/// ```
#[must_use]
pub fn convert_deep_context_dag_type(dag_type: cli::DeepContextDagType) -> cli::DagType {
    match dag_type {
        cli::DeepContextDagType::CallGraph => cli::DagType::CallGraph,
        cli::DeepContextDagType::ImportGraph => cli::DagType::ImportGraph,
        cli::DeepContextDagType::Inheritance => cli::DagType::Inheritance,
        cli::DeepContextDagType::FullDependency => cli::DagType::FullDependency,
    }
}

/// Convert cache strategy enum to string representation
///
/// # Examples
///
/// ```
/// use pmat::cli::handlers::handler_utils::convert_cache_strategy;
/// use pmat::cli::enums::DeepContextCacheStrategy;
///
/// assert_eq!(convert_cache_strategy(DeepContextCacheStrategy::Normal), "normal");
/// assert_eq!(convert_cache_strategy(DeepContextCacheStrategy::ForceRefresh), "force-refresh");
/// assert_eq!(convert_cache_strategy(DeepContextCacheStrategy::Offline), "offline");
/// ```
#[must_use]
pub fn convert_cache_strategy(strategy: cli::DeepContextCacheStrategy) -> String {
    match strategy {
        cli::DeepContextCacheStrategy::Normal => "normal".to_string(),
        cli::DeepContextCacheStrategy::ForceRefresh => "force-refresh".to_string(),
        cli::DeepContextCacheStrategy::Offline => "offline".to_string(),
    }
}

/// Parse threshold value with bounds checking
///
/// Ensures threshold is within valid range (0.0 - 1.0 or 0 - 100 depending on format)
#[must_use]
pub fn normalize_threshold(threshold: f64, is_percentage: bool) -> f64 {
    let normalized = if is_percentage {
        threshold / 100.0
    } else {
        threshold
    };
    normalized.clamp(0.0, 1.0)
}

/// Format file path for display, truncating long paths
#[must_use]
pub fn format_display_path(path: &std::path::Path, max_len: usize) -> String {
    let path_str = path.to_string_lossy();
    if path_str.len() <= max_len {
        path_str.to_string()
    } else {
        format!("...{}", &path_str[path_str.len() - max_len + 3..])
    }
}

/// Validate output format string and return canonical form
#[must_use]
pub fn normalize_output_format(format: &str) -> &'static str {
    match format.to_lowercase().as_str() {
        "json" | "j" => "json",
        "markdown" | "md" | "m" => "markdown",
        "text" | "txt" | "t" | "" => "text",
        "yaml" | "yml" | "y" => "yaml",
        "html" | "h" => "html",
        "csv" | "c" => "csv",
        _ => "text",
    }
}

/// Calculate severity level from numeric score
#[must_use]
pub fn score_to_severity(score: f64) -> &'static str {
    match score {
        s if s >= 0.9 => "critical",
        s if s >= 0.7 => "high",
        s if s >= 0.4 => "medium",
        _ => "low",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_deep_context_dag_type_call_graph() {
        let result = convert_deep_context_dag_type(cli::DeepContextDagType::CallGraph);
        assert_eq!(result, cli::DagType::CallGraph);
    }

    #[test]
    fn test_convert_deep_context_dag_type_import_graph() {
        let result = convert_deep_context_dag_type(cli::DeepContextDagType::ImportGraph);
        assert_eq!(result, cli::DagType::ImportGraph);
    }

    #[test]
    fn test_convert_deep_context_dag_type_inheritance() {
        let result = convert_deep_context_dag_type(cli::DeepContextDagType::Inheritance);
        assert_eq!(result, cli::DagType::Inheritance);
    }

    #[test]
    fn test_convert_deep_context_dag_type_full_dependency() {
        let result = convert_deep_context_dag_type(cli::DeepContextDagType::FullDependency);
        assert_eq!(result, cli::DagType::FullDependency);
    }

    #[test]
    fn test_convert_cache_strategy_normal() {
        assert_eq!(
            convert_cache_strategy(cli::DeepContextCacheStrategy::Normal),
            "normal"
        );
    }

    #[test]
    fn test_convert_cache_strategy_force_refresh() {
        assert_eq!(
            convert_cache_strategy(cli::DeepContextCacheStrategy::ForceRefresh),
            "force-refresh"
        );
    }

    #[test]
    fn test_convert_cache_strategy_offline() {
        assert_eq!(
            convert_cache_strategy(cli::DeepContextCacheStrategy::Offline),
            "offline"
        );
    }

    #[test]
    fn test_normalize_threshold_percentage() {
        assert!((normalize_threshold(50.0, true) - 0.5).abs() < 0.001);
        assert!((normalize_threshold(100.0, true) - 1.0).abs() < 0.001);
        assert!((normalize_threshold(0.0, true) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_normalize_threshold_ratio() {
        assert!((normalize_threshold(0.5, false) - 0.5).abs() < 0.001);
        assert!((normalize_threshold(1.0, false) - 1.0).abs() < 0.001);
        assert!((normalize_threshold(0.0, false) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_normalize_threshold_clamping() {
        assert!((normalize_threshold(150.0, true) - 1.0).abs() < 0.001);
        assert!((normalize_threshold(-10.0, true) - 0.0).abs() < 0.001);
        assert!((normalize_threshold(1.5, false) - 1.0).abs() < 0.001);
        assert!((normalize_threshold(-0.5, false) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_format_display_path_short() {
        let path = std::path::Path::new("src/main.rs");
        assert_eq!(format_display_path(path, 50), "src/main.rs");
    }

    #[test]
    fn test_format_display_path_long() {
        let path = std::path::Path::new("very/long/path/to/some/deeply/nested/file.rs");
        let formatted = format_display_path(path, 20);
        assert!(formatted.starts_with("..."));
        assert!(formatted.len() <= 20);
    }

    #[test]
    fn test_normalize_output_format_json() {
        assert_eq!(normalize_output_format("json"), "json");
        assert_eq!(normalize_output_format("JSON"), "json");
        assert_eq!(normalize_output_format("j"), "json");
    }

    #[test]
    fn test_normalize_output_format_markdown() {
        assert_eq!(normalize_output_format("markdown"), "markdown");
        assert_eq!(normalize_output_format("md"), "markdown");
        assert_eq!(normalize_output_format("m"), "markdown");
    }

    #[test]
    fn test_normalize_output_format_text() {
        assert_eq!(normalize_output_format("text"), "text");
        assert_eq!(normalize_output_format("txt"), "text");
        assert_eq!(normalize_output_format(""), "text");
    }

    #[test]
    fn test_normalize_output_format_yaml() {
        assert_eq!(normalize_output_format("yaml"), "yaml");
        assert_eq!(normalize_output_format("yml"), "yaml");
    }

    #[test]
    fn test_normalize_output_format_html() {
        assert_eq!(normalize_output_format("html"), "html");
        assert_eq!(normalize_output_format("h"), "html");
    }

    #[test]
    fn test_normalize_output_format_csv() {
        assert_eq!(normalize_output_format("csv"), "csv");
        assert_eq!(normalize_output_format("c"), "csv");
    }

    #[test]
    fn test_normalize_output_format_unknown() {
        assert_eq!(normalize_output_format("xyz"), "text");
        assert_eq!(normalize_output_format("invalid"), "text");
    }

    #[test]
    fn test_score_to_severity_critical() {
        assert_eq!(score_to_severity(0.95), "critical");
        assert_eq!(score_to_severity(0.9), "critical");
        assert_eq!(score_to_severity(1.0), "critical");
    }

    #[test]
    fn test_score_to_severity_high() {
        assert_eq!(score_to_severity(0.85), "high");
        assert_eq!(score_to_severity(0.7), "high");
        assert_eq!(score_to_severity(0.89), "high");
    }

    #[test]
    fn test_score_to_severity_medium() {
        assert_eq!(score_to_severity(0.5), "medium");
        assert_eq!(score_to_severity(0.4), "medium");
        assert_eq!(score_to_severity(0.69), "medium");
    }

    #[test]
    fn test_score_to_severity_low() {
        assert_eq!(score_to_severity(0.3), "low");
        assert_eq!(score_to_severity(0.0), "low");
        assert_eq!(score_to_severity(0.39), "low");
    }
}
