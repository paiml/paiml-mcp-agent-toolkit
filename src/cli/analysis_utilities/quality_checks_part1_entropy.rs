// Entropy checking functions - extracted from quality_checks_part1.rs (CB-040)

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
    min_entropy: f64,
) -> Result<Vec<QualityViolation>> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    check_entropy_with_excludes(project_path, min_entropy, &[]).await
}

/// Check entropy with configurable threshold and exclude paths (#194, #195).
pub async fn check_entropy_with_excludes(
    project_path: &Path,
    min_entropy: f64,
    extra_exclude_paths: &[String],
) -> Result<Vec<QualityViolation>> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    // TOYOTA WAY FIX: Replace Shannon entropy with AST pattern-based entropy
    // Sprint 98: Fix for 5831 false positive entropy violations
    use crate::entropy::violation_detector::Severity;
    use crate::entropy::{EntropyAnalyzer, EntropyConfig};

    // Load max_pattern_repetition from config files (#219)
    let max_rep = load_max_pattern_repetition(project_path);

    // Create entropy analyzer with tuned config to reduce false positives
    let mut config = EntropyConfig {
        min_severity: Severity::Medium, // Only report medium+ severity
        // Use CLI/TOML-provided threshold instead of hardcoded 0.3 (#194)
        min_pattern_diversity: min_entropy,
        max_pattern_repetition: max_rep,
        ..Default::default()
    };
    config.exclude_paths.push("**/target/**".to_string());
    config.exclude_paths.push("**/node_modules/**".to_string());
    config.exclude_paths.push("**/*.test.rs".to_string());
    config.exclude_paths.push("**/*_tests.rs".to_string());
    config.exclude_paths.push("**/*_tests_*.rs".to_string());
    config.exclude_paths.push("**/*tests_part*.rs".to_string());
    config.exclude_paths.push("**/tests/**".to_string());
    config.exclude_paths.push("**/examples/**".to_string());
    config.exclude_paths.push("**/benches/**".to_string());

    // Apply extra exclude paths from .pmat-metrics.toml [exclude] (#195)
    for path in extra_exclude_paths {
        let pattern = if path.contains('*') {
            path.clone()
        } else {
            format!("{}**", path.trim_end_matches('/').to_owned() + "/")
        };
        config.exclude_paths.push(pattern);
    }

    // Also load .pmatignore patterns
    config = config.with_project_ignores(project_path);

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
            details: Some(ViolationDetails {
                affected_files: violation.affected_files.iter().map(|p| p.to_string_lossy().to_string()).collect(),
                example_code: Some(violation.pattern.example_code.clone()),
                fix_suggestion: Some(violation.fix_suggestion.clone()),
                score_factors: vec![
                    format!("pattern_type: {:?}", violation.pattern.pattern_type),
                    format!("repetitions: {}", violation.pattern.repetitions),
                    format!("variation_score: {:.2}", violation.pattern.variation_score),
                ],
            }),
        })
        .collect();

    Ok(violations)
}

/// Load max_pattern_repetition from config files (#219, #227).
/// Priority: `.pmat-gates.toml` > `.pmat-metrics.toml` > `pmat.toml [quality]` > default (5).
fn load_max_pattern_repetition(project_path: &Path) -> usize {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    // Highest priority: .pmat-gates.toml and .pmat-metrics.toml [entropy] section
    for filename in &[".pmat-gates.toml", ".pmat-metrics.toml"] {
        let path = project_path.join(filename);
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(table) = content.parse::<toml::Table>() {
                if let Some(val) = table
                    .get("entropy")
                    .and_then(|t| t.get("max_pattern_repetition"))
                    .and_then(|v| v.as_integer())
                {
                    return val.max(1) as usize;
                }
            }
        }
    }
    // Lowest priority: pmat.toml [quality] section (#227)
    if let Ok(content) = std::fs::read_to_string(project_path.join("pmat.toml")) {
        if let Ok(table) = content.parse::<toml::Table>() {
            if let Some(val) = table
                .get("quality")
                .and_then(|t| t.get("max_pattern_repetition"))
                .and_then(|v| v.as_integer())
            {
                return val.max(1) as usize;
            }
        }
    }
    5 // default: same as EntropyConfig::default()
}
