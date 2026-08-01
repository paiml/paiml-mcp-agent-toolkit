// Entropy checking functions - extracted from quality_checks_part1.rs (CB-040)

/// Check code entropy (diversity) across the project
///
/// This function analyzes code entropy to detect low-diversity code that might
/// indicate copy-paste programming, lack of abstraction, or potential defects.
///
/// # Arguments
/// * `project_path` - Root directory to analyze
/// * `min_entropy` - Minimum required pattern diversity on a 0.0-1.0 scale.
///   `0.0` means "no diversity required" and therefore never reports a violation.
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
///
/// // A zero requirement can never be unmet
/// assert!(check_entropy(Path::new("."), 0.0).await?.is_empty());
/// # Ok(())
/// # }
/// ```
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn check_entropy(
    project_path: &Path,
    min_entropy: f64,
) -> Result<Vec<QualityViolation>> {
    check_entropy_with_excludes(project_path, min_entropy, &[]).await
}

/// Check entropy with configurable threshold and exclude paths (#194, #195).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn check_entropy_with_excludes(
    project_path: &Path,
    min_entropy: f64,
    extra_exclude_paths: &[String],
) -> Result<Vec<QualityViolation>> {
    use crate::entropy::EntropyAnalyzer;

    let config = build_entropy_config(project_path, min_entropy, extra_exclude_paths);
    let analyzer = EntropyAnalyzer::with_config(config);

    // Run AST-based entropy analysis
    let report = analyzer.analyze(project_path).await?;

    // #683: the threshold must actually gate the check. Previously `min_entropy`
    // only reached `EntropyConfig::min_pattern_diversity`, which is consulted by
    // exactly one of four detectors, so `--min-entropy 0.0` ("require zero
    // diversity", which must always pass) reported the same FAILED / 8 violations
    // as `--min-entropy 0.99`.
    let Some(measured) = report.entropy_metrics.pattern_diversity else {
        // Nothing measurable ⇒ nothing to compare against ⇒ nothing to report.
        return Ok(vec![]);
    };
    if measured >= min_entropy {
        return Ok(vec![]);
    }
    // #683: every emitted violation names what was compared, so a reader can tell
    // which threshold produced it. The old text ("ApiCall pattern repeated 10
    // times (saves 302 lines)") never mentioned a threshold at all.
    let threshold_context = format!(
        " [pattern diversity {:.1}% < required {:.1}% (--min-entropy {:.2})]",
        measured * 100.0,
        min_entropy * 100.0,
        min_entropy
    );

    // Convert actionable violations to QualityViolation format
    Ok(report
        .actionable_violations
        .into_iter()
        .map(|violation| to_quality_violation(violation, &threshold_context))
        .collect())
}

/// Build the entropy analyzer config used by the quality gate.
fn build_entropy_config(
    project_path: &Path,
    min_entropy: f64,
    extra_exclude_paths: &[String],
) -> crate::entropy::EntropyConfig {
    // TOYOTA WAY FIX: Replace Shannon entropy with AST pattern-based entropy
    // Sprint 98: Fix for 5831 false positive entropy violations
    use crate::entropy::violation_detector::Severity;
    use crate::entropy::EntropyConfig;

    let mut config = EntropyConfig {
        min_severity: Severity::Medium, // Only report medium+ severity
        // Use CLI/TOML-provided threshold instead of hardcoded 0.3 (#194)
        min_pattern_diversity: min_entropy,
        // Load max_pattern_repetition from config files (#219)
        max_pattern_repetition: load_max_pattern_repetition(project_path),
        ..Default::default()
    };
    for pattern in [
        "**/target/**",
        "**/node_modules/**",
        "**/*.test.rs",
        "**/*_tests.rs",
        "**/*_tests_*.rs",
        "**/*tests_part*.rs",
        "**/tests/**",
        "**/examples/**",
        "**/benches/**",
    ] {
        config.exclude_paths.push(pattern.to_string());
    }

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
    config.with_project_ignores(project_path)
}

/// Render one actionable entropy violation as a quality-gate violation.
///
/// `threshold_context` is appended to the message so the row always states the
/// threshold that produced it (#683).
fn to_quality_violation(
    violation: crate::entropy::ActionableViolation,
    threshold_context: &str,
) -> QualityViolation {
    use crate::entropy::violation_detector::Severity;

    QualityViolation {
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
            "{} (saves {} lines) - Fix: {}{}",
            violation.message,
            violation.estimated_loc_reduction,
            violation.fix_suggestion,
            threshold_context
        ),
        details: Some(ViolationDetails {
            affected_files: violation
                .affected_files
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
            example_code: Some(violation.pattern.example_code.clone()),
            fix_suggestion: Some(violation.fix_suggestion.clone()),
            score_factors: vec![
                format!("pattern_type: {:?}", violation.pattern.pattern_type),
                format!("repetitions: {}", violation.pattern.repetitions),
                format!("variation_score: {:.2}", violation.pattern.variation_score),
            ],
        }),
    }
}

/// Load max_pattern_repetition from config files (#219, #227).
/// Priority: `.pmat-gates.toml` > `.pmat-metrics.toml` > `pmat.toml [quality]` > default (5).
fn load_max_pattern_repetition(project_path: &Path) -> usize {
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
