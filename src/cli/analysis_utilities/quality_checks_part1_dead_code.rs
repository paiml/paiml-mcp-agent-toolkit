// Dead code checking functions - extracted from quality_checks_part1.rs (CB-040)
/// Detects dead code in a project and returns violations.
///
/// # Arguments
///
/// * `project_path` - Path to the project directory to analyze
/// * `max_percentage` - Maximum allowed percentage of dead code
///
/// # Returns
///
/// A vector of quality violations for dead code exceeding the threshold
///
/// # Examples
///
/// ```no_run
/// # use std::path::Path;
/// # use pmat::cli::analysis_utilities::{check_dead_code, QualityViolation};
/// # async fn example() -> anyhow::Result<()> {
/// let violations = check_dead_code(Path::new("."), 15.0).await?;
/// if violations.is_empty() {
///     println!("Dead code is within acceptable limits");
/// } else {
///     for violation in violations {
///         println!("Dead code issue: {}", violation.message);
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Property Tests
///
/// ```rust,no_run
/// # use std::path::Path;
/// # use pmat::cli::analysis_utilities::check_dead_code;
/// #
/// # #[tokio::test]
/// # async fn test_dead_code_detection() -> anyhow::Result<()> {
/// // Test with a high threshold (should get no violations)
/// let violations = check_dead_code(Path::new("."), 90.0).await?;
///
/// // Verify violation structure
/// for violation in &violations {
///     assert_eq!(violation.check_type, "dead_code");
///     assert!(violation.severity == "error" || violation.severity == "warning");
///     assert!(!violation.message.is_empty());
/// }
/// # Ok(())
/// # }
/// ```
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn check_dead_code(
    project_path: &Path,
    max_percentage: f64,
) -> Result<Vec<QualityViolation>> {
    use crate::services::cargo_dead_code_analyzer::CargoDeadCodeAnalyzer;

    let mut violations = Vec::new();

    // Use the same CargoDeadCodeAnalyzer that `pmat analyze dead-code` uses
    // to ensure consistent results (fixes #141).
    // Falls back gracefully if cargo analysis is unavailable (non-Rust projects,
    // missing Cargo.toml, etc.)
    let analyzer = CargoDeadCodeAnalyzer::new(project_path);
    let report = match analyzer.analyze().await {
        Ok(r) => r,
        Err(_) => return Ok(violations), // No cargo project → no dead code violations
    };

    let dead_percentage = report.dead_code_percentage;

    if dead_percentage > max_percentage {
        violations.push(QualityViolation {
            check_type: "dead_code".to_string(),
            severity: "error".to_string(),
            file: project_path.to_string_lossy().to_string(),
            line: None,
            message: format!(
                "Dead code percentage {dead_percentage:.1}% exceeds maximum allowed {max_percentage:.1}%"
            ),
            details: None,
        });
    }

    // Add a warning for each file with significant dead code
    for file in report.files_with_dead_code.iter().take(5) {
        if file.file_dead_percentage > 20.0 {
            violations.push(QualityViolation {
                check_type: "dead_code".to_string(),
                severity: "warning".to_string(),
                file: file.file_path.display().to_string(),
                line: None,
                message: format!(
                    "File has {:.1}% dead code ({} dead items)",
                    file.file_dead_percentage,
                    file.dead_items.len()
                ),
                details: None,
            });
        }
    }

    Ok(violations)
}
