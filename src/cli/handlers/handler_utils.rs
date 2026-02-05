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

/// Get top N items from a slice, or all items if limit is 0
#[must_use]
pub fn get_top_n<T: Clone>(items: &[T], limit: usize) -> Vec<T> {
    if limit == 0 {
        items.to_vec()
    } else {
        items.iter().take(limit).cloned().collect()
    }
}

/// Write content to file or stdout based on output path
///
/// # Errors
///
/// Returns error if file write fails
pub fn output_to_file_or_stdout(
    output: Option<&std::path::Path>,
    content: &str,
) -> std::io::Result<()> {
    if let Some(path) = output {
        std::fs::write(path, content)
    } else {
        println!("{content}");
        Ok(())
    }
}

/// Format a count with appropriate pluralization
#[must_use]
pub fn pluralize(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("{count} {singular}")
    } else {
        format!("{count} {plural}")
    }
}

/// Format a duration in a human-readable way
#[must_use]
pub fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();

    if secs >= 60 {
        let mins = secs / 60;
        let remaining_secs = secs % 60;
        format!("{}m {}s", mins, remaining_secs)
    } else if secs > 0 {
        format!("{}.{}s", secs, millis / 100)
    } else {
        format!("{}ms", millis)
    }
}

/// Calculate percentage with bounds (0-100)
#[must_use]
pub fn calculate_percentage(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        ((numerator as f64 / denominator as f64) * 100.0).clamp(0.0, 100.0)
    }
}

/// Check if content has heavy cfg/target_feature attributes (SIMD/arch-specific code)
///
/// Files with many cfg attributes are often SIMD or architecture-specific code
/// that only compiles on certain platforms, leading to false positives in dead code analysis.
#[must_use]
pub fn is_heavily_cfg_gated(content: &str) -> bool {
    let cfg_count = content.matches("#[cfg(target").count()
        + content.matches("#[target_feature").count()
        + content.matches("#[cfg(feature").count();
    cfg_count > 3
}

/// Check if a path should be excluded from production code analysis
///
/// Excludes test files, falsification modules, SIMD-heavy directories
#[must_use]
pub fn is_excluded_from_analysis(path_str: &str) -> bool {
    path_str.ends_with("_tests.rs")
        || path_str.contains("/tests/")
        || path_str.contains("/falsification/")
        || path_str.contains("_falsification")
        || path_str.contains("/quantize/")
        || path_str.contains("/simd/")
        || path_str.contains("/benches/")
        || path_str.contains("/examples/")
}

/// Check if a line looks like a test module marker
#[must_use]
pub fn is_test_module_marker(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed == "#[cfg(test)]" || trimmed.starts_with("#[cfg(test)]")
}

/// Check if a line is a dead code annotation
#[must_use]
pub fn is_dead_code_annotation(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("#[allow(dead_code)]") || trimmed.starts_with("#[allow(unused")
}

/// Check if a line is a code item declaration (fn, struct, enum, etc.)
#[must_use]
pub fn is_code_item_declaration(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("pub fn ")
        || trimmed.starts_with("fn ")
        || trimmed.starts_with("pub struct ")
        || trimmed.starts_with("struct ")
        || trimmed.starts_with("pub enum ")
        || trimmed.starts_with("enum ")
        || trimmed.starts_with("pub const ")
        || trimmed.starts_with("const ")
        || trimmed.starts_with("pub static ")
        || trimmed.starts_with("static ")
        || trimmed.starts_with("pub type ")
        || trimmed.starts_with("type ")
        || trimmed.starts_with("pub trait ")
        || trimmed.starts_with("trait ")
        || trimmed.starts_with("impl ")
}

#[cfg_attr(coverage_nightly, coverage(off))]
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

    #[test]
    fn test_get_top_n_with_limit() {
        let items = vec![1, 2, 3, 4, 5];
        assert_eq!(get_top_n(&items, 3), vec![1, 2, 3]);
    }

    #[test]
    fn test_get_top_n_no_limit() {
        let items = vec![1, 2, 3, 4, 5];
        assert_eq!(get_top_n(&items, 0), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_get_top_n_limit_exceeds() {
        let items = vec![1, 2, 3];
        assert_eq!(get_top_n(&items, 10), vec![1, 2, 3]);
    }

    #[test]
    fn test_get_top_n_empty() {
        let items: Vec<i32> = vec![];
        assert!(get_top_n(&items, 5).is_empty());
    }

    #[test]
    fn test_pluralize_singular() {
        assert_eq!(pluralize(1, "file", "files"), "1 file");
        assert_eq!(pluralize(1, "item", "items"), "1 item");
    }

    #[test]
    fn test_pluralize_plural() {
        assert_eq!(pluralize(0, "file", "files"), "0 files");
        assert_eq!(pluralize(2, "file", "files"), "2 files");
        assert_eq!(pluralize(100, "item", "items"), "100 items");
    }

    #[test]
    fn test_format_duration_milliseconds() {
        let d = std::time::Duration::from_millis(500);
        assert_eq!(format_duration(d), "500ms");
    }

    #[test]
    fn test_format_duration_seconds() {
        let d = std::time::Duration::from_secs(5);
        assert_eq!(format_duration(d), "5.0s");
    }

    #[test]
    fn test_format_duration_seconds_with_millis() {
        let d = std::time::Duration::from_millis(5500);
        assert_eq!(format_duration(d), "5.5s");
    }

    #[test]
    fn test_format_duration_minutes() {
        let d = std::time::Duration::from_secs(125);
        assert_eq!(format_duration(d), "2m 5s");
    }

    #[test]
    fn test_calculate_percentage_normal() {
        assert!((calculate_percentage(50, 100) - 50.0).abs() < 0.001);
        assert!((calculate_percentage(25, 100) - 25.0).abs() < 0.001);
        assert!((calculate_percentage(1, 4) - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_percentage_zero_denominator() {
        assert!((calculate_percentage(10, 0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_percentage_zero_numerator() {
        assert!((calculate_percentage(0, 100) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_percentage_full() {
        assert!((calculate_percentage(100, 100) - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_is_heavily_cfg_gated_true() {
        let content = r#"
            #[cfg(target_arch = "x86_64")]
            fn foo() {}
            #[cfg(target_feature = "avx2")]
            fn bar() {}
            #[cfg(feature = "simd")]
            fn baz() {}
            #[target_feature(enable = "sse2")]
            fn qux() {}
        "#;
        assert!(is_heavily_cfg_gated(content));
    }

    #[test]
    fn test_is_heavily_cfg_gated_false() {
        let content = r#"
            fn normal_function() {}
            #[cfg_attr(coverage_nightly, coverage(off))]
            #[cfg(test)]
            mod tests {}
        "#;
        assert!(!is_heavily_cfg_gated(content));
    }

    #[test]
    fn test_is_excluded_from_analysis_tests() {
        assert!(is_excluded_from_analysis("src/foo_tests.rs"));
        assert!(is_excluded_from_analysis("src/tests/mod.rs"));
        assert!(is_excluded_from_analysis("/path/to/tests/foo.rs"));
    }

    #[test]
    fn test_is_excluded_from_analysis_falsification() {
        assert!(is_excluded_from_analysis("src/falsification/mod.rs"));
        assert!(is_excluded_from_analysis("src/foo_falsification.rs"));
    }

    #[test]
    fn test_is_excluded_from_analysis_simd() {
        assert!(is_excluded_from_analysis("src/quantize/mod.rs"));
        assert!(is_excluded_from_analysis("src/simd/avx.rs"));
    }

    #[test]
    fn test_is_excluded_from_analysis_production() {
        assert!(!is_excluded_from_analysis("src/lib.rs"));
        assert!(!is_excluded_from_analysis("src/main.rs"));
        assert!(!is_excluded_from_analysis("src/services/foo.rs"));
    }

    #[test]
    fn test_is_test_module_marker_true() {
        assert!(is_test_module_marker("#[cfg(test)]"));
        assert!(is_test_module_marker("  #[cfg(test)]"));
        assert!(is_test_module_marker("#[cfg(test)] // comment"));
    }

    #[test]
    fn test_is_test_module_marker_false() {
        assert!(!is_test_module_marker("fn test() {}"));
        assert!(!is_test_module_marker("#[test]"));
        assert!(!is_test_module_marker("#[cfg(feature = \"test\")]"));
    }

    #[test]
    fn test_is_dead_code_annotation_true() {
        assert!(is_dead_code_annotation("#[allow(dead_code)]"));
        assert!(is_dead_code_annotation("  #[allow(dead_code)]"));
        assert!(is_dead_code_annotation("#[allow(unused)]"));
        assert!(is_dead_code_annotation("#[allow(unused_imports)]"));
    }

    #[test]
    fn test_is_dead_code_annotation_false() {
        assert!(!is_dead_code_annotation("fn dead_code() {}"));
        assert!(!is_dead_code_annotation("// #[allow(dead_code)]"));
        assert!(!is_dead_code_annotation("#[derive(Debug)]"));
    }

    #[test]
    fn test_is_code_item_declaration_functions() {
        assert!(is_code_item_declaration("fn foo() {}"));
        assert!(is_code_item_declaration("pub fn bar() {}"));
        assert!(is_code_item_declaration("  fn baz() {}"));
    }

    #[test]
    fn test_is_code_item_declaration_types() {
        assert!(is_code_item_declaration("struct Foo {}"));
        assert!(is_code_item_declaration("pub struct Bar {}"));
        assert!(is_code_item_declaration("enum Baz {}"));
        assert!(is_code_item_declaration("pub enum Qux {}"));
        assert!(is_code_item_declaration("type Alias = i32;"));
        assert!(is_code_item_declaration("pub type PublicAlias = i32;"));
    }

    #[test]
    fn test_is_code_item_declaration_traits_impls() {
        assert!(is_code_item_declaration("trait Foo {}"));
        assert!(is_code_item_declaration("pub trait Bar {}"));
        assert!(is_code_item_declaration("impl Foo for Bar {}"));
    }

    #[test]
    fn test_is_code_item_declaration_consts() {
        assert!(is_code_item_declaration("const FOO: i32 = 1;"));
        assert!(is_code_item_declaration("pub const BAR: i32 = 2;"));
        assert!(is_code_item_declaration("static BAZ: i32 = 3;"));
        assert!(is_code_item_declaration("pub static QUX: i32 = 4;"));
    }

    #[test]
    fn test_is_code_item_declaration_false() {
        assert!(!is_code_item_declaration("let x = 1;"));
        assert!(!is_code_item_declaration("// fn comment"));
        assert!(!is_code_item_declaration("use crate::foo;"));
    }
}
