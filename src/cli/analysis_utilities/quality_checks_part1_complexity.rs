// Complexity checking functions - extracted from quality_checks_part1.rs (CB-040)

// Quality check functions

/// Checks code complexity in a project and returns violations.
///
/// # Arguments
///
/// * `project_path` - Path to the project directory to analyze
/// * `max_complexity` - Maximum allowed cyclomatic complexity
///
/// # Returns
///
/// A vector of quality violations for functions exceeding the complexity threshold
///
/// # Examples
///
/// ```no_run
/// # use std::path::Path;
/// # use pmat::cli::analysis_utilities::{check_complexity, QualityViolation};
/// # async fn example() -> anyhow::Result<()> {
/// let violations = check_complexity(Path::new("."), 10).await?;
/// for violation in violations {
///     println!("Complex function: {} in {}", violation.message, violation.file);
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Property Tests
///
/// ```rust,no_run
/// # tokio_test::block_on(async {
/// use std::path::Path;
/// use pmat::cli::analysis_utilities::check_complexity;
///
/// // Test with a specific threshold
/// let threshold = 10u32;
/// let violations = check_complexity(Path::new("."), threshold).await.unwrap();
///
/// // Property: All violations should have complexity > threshold
/// for violation in violations {
///     // Extract complexity from message
///     if let Some(complexity_str) = violation.message
///         .split("complexity ")
///         .nth(1)
///         .and_then(|s| s.split(' ').next())
///         .and_then(|s| s.parse::<u32>().ok()) {
///         assert!(complexity_str > threshold);
///     }
/// }
/// # });
/// ```
pub async fn check_complexity(
    project_path: &Path,
    _max_complexity: u32,
) -> Result<Vec<QualityViolation>> {
    use crate::services::complexity::aggregate_results_with_thresholds;
    use crate::services::configuration_service::configuration;

    let mut violations = Vec::new();

    // Get thresholds from configuration service - SINGLE SOURCE OF TRUTH
    let config_service = configuration();
    let config = config_service.get_config()?;
    let max_cyclomatic = config.quality.max_complexity;
    let max_cognitive = config.quality.max_cognitive_complexity;

    // Load exclude_paths from .pmat-metrics.toml for filtering generated files
    let mut exclude_globs = load_exclude_paths(project_path);
    // Built-in excludes: non-production code where high complexity is expected
    for pattern in &[
        "**/examples/**", "**/benches/**", "**/scripts/**",
        "**/tests/**", "**/*_tests.rs", "**/*_tests_*.rs", "**/*tests_part*.rs",
        "**/fixtures/**",
        // Lint rule implementations have inherent pattern-matching complexity
        "**/comply_cb_detect/**", "**/comply_cb_detect.rs",
        // Language analysis infrastructure: inherently complex pattern matching
        "**/dead_code_multi_language.rs",
        "**/mcp_integration/**",
        // MCP tool functions: thin dispatch wrappers with many match arms
        "**/mcp_pmcp/tool_functions/**",
    ] {
        if let Ok(p) = glob::Pattern::new(pattern) {
            exclude_globs.push(p);
        }
    }

    // Use the existing analyze_project_files function - the ONE implementation
    let file_metrics = analyze_project_files(
        project_path,
        None, // Auto-detect toolchain
        &[],  // Empty include pattern means all files
        max_cyclomatic as u16,
        max_cognitive as u16,
    )
    .await?;

    // Check for violations using the same logic as analyze complexity
    let report = aggregate_results_with_thresholds(
        file_metrics,
        Some(max_cyclomatic as u16),
        Some(max_cognitive as u16),
    );

    // Convert violations to QualityViolation format
    // ONLY count actual violations where complexity exceeds threshold
    for violation in &report.violations {
        if !is_violation_excluded(violation, &exclude_globs) {
            process_complexity_violation(violation, &mut violations);
        }
    }

    Ok(violations)
}

/// Load exclude_paths globs from `.pmat-metrics.toml`.
fn load_exclude_paths(project_path: &Path) -> Vec<glob::Pattern> {
    let config_path = project_path.join(".pmat-metrics.toml");
    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let table: toml::Table = match content.parse() {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    table
        .get("exclude_paths")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .filter_map(|s| glob::Pattern::new(s).ok())
                .collect()
        })
        .unwrap_or_default()
}

/// Check if a complexity violation's file matches any exclude_paths glob.
fn is_violation_excluded(
    violation: &crate::services::complexity::Violation,
    exclude_globs: &[glob::Pattern],
) -> bool {
    use crate::services::complexity::Violation;
    let file_path = match violation {
        Violation::Error { file, .. } | Violation::Warning { file, .. } => file,
    };
    exclude_globs
        .iter()
        .any(|pat| pat.matches(file_path) || pat.matches_path(std::path::Path::new(file_path)))
}

/// Process a single complexity violation into `QualityViolation` format
fn process_complexity_violation(
    violation: &crate::services::complexity::Violation,
    violations: &mut Vec<QualityViolation>,
) {
    use crate::services::complexity::Violation;

    let (file, line, function, rule, message, value, threshold, severity) = match violation {
        Violation::Error {
            file,
            line,
            function,
            rule,
            message,
            value,
            threshold,
        } => (
            file, line, function, rule, message, value, threshold, "error",
        ),
        Violation::Warning {
            file,
            line,
            function,
            rule,
            message,
            value,
            threshold,
        } => (
            file, line, function, rule, message, value, threshold, "warning",
        ),
    };

    // Only add if this is an actual threshold violation
    if value > threshold {
        violations.push(QualityViolation {
            check_type: "complexity".to_string(),
            severity: severity.to_string(),
            file: file.clone(),
            line: Some(*line as usize),
            message: format!(
                "{}: {} - {} (complexity: {}, threshold: {})",
                function.as_deref().unwrap_or("global"),
                rule,
                message,
                value,
                threshold
            ),
            details: None,
        });
    }
}
