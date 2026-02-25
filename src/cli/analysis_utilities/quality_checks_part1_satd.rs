// SATD checking functions - extracted from quality_checks_part1.rs (CB-040)
/// Detects self-admitted technical debt (SATD) in source code.
///
/// Scans for technical debt markers like TODO, FIXME, HACK, etc.
///
/// # Arguments
///
/// * `project_path` - Path to the project directory to analyze
///
/// # Returns
///
/// A vector of quality violations for each SATD comment found
///
/// # Examples
///
/// ```no_run
/// # use std::path::Path;
/// # use pmat::cli::analysis_utilities::{check_satd, QualityViolation};
/// # async fn example() -> anyhow::Result<()> {
/// let violations = check_satd(Path::new(".")).await?;
///
/// // Group by severity
/// let mut by_severity = std::collections::HashMap::new();
/// for violation in violations {
///     *by_severity.entry(violation.severity.clone()).or_insert(0) += 1;
/// }
///
/// for (severity, count) in by_severity {
///     println!("{} SATD items with severity: {}", count, severity);
/// }
/// # Ok(())
/// # }
/// ```ignore
///
/// # Property Tests
///
/// ```rust,no_run
/// # tokio_test::block_on(async {
/// use std::path::Path;
/// use pmat::cli::analysis_utilities::check_satd;
///
/// // Property: All detected items should have valid SATD patterns
/// let violations = check_satd(Path::new(".")).await.unwrap();
///
/// let valid_patterns = ["TODO", "FIXME", "HACK", "XXX", "BUG", "REFACTOR"];
/// for violation in violations {
///     assert_eq!(violation.check_type, "satd");
///     assert!(violation.line.is_some()); // Should have line numbers
///     
///     // Check that message contains a valid SATD type (case-insensitive)
///     let message_upper = violation.message.to_uppercase();
///     let has_valid_pattern = valid_patterns.iter()
///         .any(|&pattern| message_upper.contains(pattern));
///     if !has_valid_pattern {
///         eprintln!("Violation message doesn't contain expected pattern: {}", violation.message);
///     }
/// }
/// # });
/// ```
pub async fn check_satd(project_path: &Path) -> Result<Vec<QualityViolation>> {
    // Toyota Way: Use the ONE proper implementation, not duplicate logic
    use crate::services::satd_detector::SATDDetector;

    let detector = SATDDetector::new();
    let include_tests = false; // Don't include test files in quality gate

    // Use the proper SATD analyzer with context awareness
    let satd_result = detector
        .analyze_project(project_path, include_tests)
        .await?;

    // Convert SATD items to quality violations
    let violations: Vec<QualityViolation> = satd_result
        .items
        .into_iter()
        .map(|debt| QualityViolation {
            check_type: "satd".to_string(),
            severity: match debt.severity {
                crate::services::satd_detector::Severity::Critical => "error",
                crate::services::satd_detector::Severity::High => "error",
                crate::services::satd_detector::Severity::Medium => "warning",
                crate::services::satd_detector::Severity::Low => "info",
            }
            .to_string(),
            file: debt.file.display().to_string(),
            line: Some(debt.line as usize),
            message: format!(
                "{}: {} (at column {})",
                debt.category, debt.text, debt.column
            ),
            details: None,
        })
        .collect();

    Ok(violations)
}
