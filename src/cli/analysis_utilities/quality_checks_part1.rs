// Quality check functions - extracted for file health (CB-040)
/// Check if path is a build artifact that should be excluded from duplicate detection
pub fn is_build_artifact(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains("/target/")
        || path_str.contains("/build/")
        || path_str.contains("/out/")
        || path_str.contains("/.cargo/")
        || path_str.contains("/node_modules/")
        || path_str.contains("/dist/")
        || path_str.contains("/.git/")
        || path_str.contains("/generated/")
        || path_str.starts_with("./target/")
        || path_str.starts_with("target/")
}

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
/// ```ignore
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
        process_complexity_violation(violation, &mut violations);
    }

    Ok(violations)
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
        });
    }
}

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
/// ```ignore
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
            });
        }
    }

    Ok(violations)
}

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
        })
        .collect();

    Ok(violations)
}

/// Check code entropy (diversity) across the project
///
/// This function analyzes code entropy to detect low-diversity code that might
/// indicate copy-paste programming, lack of abstraction, or potential defects.
///
/// # Arguments
/// * `project_path` - Root directory to analyze
/// * `min_entropy` - Minimum acceptable entropy (typically 0.5-0.9)
///
/// # Example
///
/// ```rust,no_run
/// # use std::path::Path;
/// # use pmat::cli::analysis_utilities::QualityViolation;
/// #
/// # #[tokio::test]
/// # async fn test_entropy_check() -> anyhow::Result<()> {
/// // Check for low entropy (repetitive) code
/// let violations = check_entropy(Path::new("."), 0.7).await?;
///
/// for violation in &violations {
///     assert_eq!(violation.check_type, "entropy");
///     println!("Low diversity in {}: {}", violation.file, violation.message);
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Property Tests
///
/// ```rust,no_run
/// # use std::path::Path;
/// #
/// # #[tokio::test]
/// # async fn test_entropy_threshold() -> anyhow::Result<()> {
/// // Test with different thresholds
/// let low_threshold = check_entropy(Path::new("."), 0.3).await?;
/// let high_threshold = check_entropy(Path::new("."), 0.9).await?;
///
/// // Higher threshold should find more violations
/// assert!(high_threshold.len() >= low_threshold.len());
/// # Ok(())
/// # }
/// ```
pub async fn check_entropy(
    project_path: &Path,
    _min_entropy: f64,
) -> Result<Vec<QualityViolation>> {
    // TOYOTA WAY FIX: Replace Shannon entropy with AST pattern-based entropy
    // Sprint 98: Fix for 5831 false positive entropy violations
    use crate::entropy::violation_detector::Severity;
    use crate::entropy::{EntropyAnalyzer, EntropyConfig};

    // Create entropy analyzer with tuned config to reduce false positives
    let mut config = EntropyConfig {
        min_severity: Severity::Medium, // Only report medium+ severity
        ..Default::default()
    };
    config.exclude_paths.push("**/target/**".to_string());
    config.exclude_paths.push("**/node_modules/**".to_string());
    config.exclude_paths.push("**/*.test.rs".to_string());
    config.exclude_paths.push("**/tests/**".to_string());

    let analyzer = EntropyAnalyzer::with_config(config);

    // Run AST-based entropy analysis
    let report = analyzer.analyze(project_path).await?;

    // Convert actionable violations to QualityViolation format
    let violations: Vec<QualityViolation> = report
        .actionable_violations
        .into_iter()
        .map(|violation| QualityViolation {
            check_type: "entropy".to_string(),
            severity: match violation.severity {
                Severity::Low => "info".to_string(),
                Severity::Medium => "warning".to_string(),
                Severity::High => "error".to_string(),
            },
            file: violation.affected_files.first().map_or_else(
                || "project".to_string(),
                |p| p.to_string_lossy().to_string(),
            ),
            line: None, // Pattern violations span multiple lines
            message: format!(
                "{} (saves {} lines) - Fix: {}",
                violation.message, violation.estimated_loc_reduction, violation.fix_suggestion
            ),
        })
        .collect();

    Ok(violations)
}

// NOTE: Shannon entropy functions removed in Sprint 98
// These were replaced by AST pattern-based entropy detection
// in the entropy module. The old character-based approach
// generated 5,831 false positives and has been deprecated.

async fn check_security(project_path: &Path) -> Result<Vec<QualityViolation>> {
    let mut violations = Vec::new();
    let patterns = get_security_patterns();

    use tokio::fs;

    if let Ok(mut entries) = fs::read_dir(project_path).await {
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && is_source_file(&path) {
                check_file_security(&path, &patterns, &mut violations).await?;
            }
        }
    }

    Ok(violations)
}
