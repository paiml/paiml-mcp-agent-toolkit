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

    // #683: the threshold must actually gate. It reaches
    // `EntropyConfig::min_pattern_diversity`, which is what decides whether the
    // low-diversity violation is raised — so raising or lowering `--min-entropy`
    // changes the violation set, as the flag's help promises.
    //
    // What it must NOT do is act as a master switch over unrelated findings.
    // Returning early when `measured >= min_entropy` suppressed everything:
    // `--min-entropy 0.3` on a tree measured at 62.6% diversity hid six
    // High-severity concrete repetition violations that `analyze entropy`
    // reports on the same tree with file lists, and the default gate therefore
    // certified "Total violations: 0" for code the sibling command flags 14
    // times. Repetition, cross-file duplication and inconsistency are measured
    // against their own thresholds and are reported regardless of diversity.
    let threshold_context = diversity_context(report.entropy_metrics.pattern_diversity, min_entropy);

    // Convert actionable violations to QualityViolation format
    Ok(report
        .actionable_violations
        .into_iter()
        .map(|violation| to_quality_violation(violation, &threshold_context))
        .collect())
}

/// Annotation appended to the low-diversity row naming the comparison that
/// produced it (#683).
///
/// `min_entropy` is printed with `{}` (shortest round-trip form), not `{:.2}`:
/// `--min-entropy 0.999` rendered as "required 99.9% (--min-entropy 1.00)",
/// where the two halves of one sentence named different values and the echo
/// named a number the user never passed.
fn diversity_context(measured: Option<f64>, min_entropy: f64) -> String {
    measured.map_or_else(String::new, |m| {
        format!(
            " [pattern diversity {:.1}% < required {:.1}% (--min-entropy {})]",
            m * 100.0,
            min_entropy * 100.0,
            min_entropy
        )
    })
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
        // Exactly the list `analyze entropy` uses. This function used to build a
        // narrower one (`**/*_tests.rs` and friends where analyze had
        // `**/*test*.rs`), so the gate walked 1328 files where analyze walked 939
        // and the two commands answered the same question differently.
        exclude_paths: EntropyConfig::analysis_excludes(false),
        ..Default::default()
    };

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
/// `diversity_context` is appended only to the project-level diversity row — the
/// row `--min-entropy` actually decides. Stamping "[pattern diversity 62.6% <
/// required 30%]" onto an `ApiCall pattern repeated 17 times` row told the reader
/// that a repetition finding came from the diversity threshold, which it did not.
fn to_quality_violation(
    violation: crate::entropy::ActionableViolation,
    diversity_context: &str,
) -> QualityViolation {
    use crate::entropy::violation_detector::Severity;

    let context = if violation.pattern.is_none() {
        diversity_context
    } else {
        ""
    };

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
            "{} (saves {}) - Fix: {}{}",
            violation.message,
            violation.render_loc_reduction(),
            violation.fix_suggestion,
            context
        ),
        details: Some(ViolationDetails {
            affected_files: violation
                .affected_files
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
            // Absent for project-level findings rather than the old placeholder
            // "Various repetitive patterns" / "repetitions: 0" / a variation_score
            // that was just `1 - diversity` restated (#650).
            example_code: violation.pattern.as_ref().map(|p| p.example_code.clone()),
            fix_suggestion: Some(violation.fix_suggestion.clone()),
            score_factors: violation.pattern.as_ref().map_or_else(
                || vec!["scope: project-wide pattern distribution".to_string()],
                |p| {
                    vec![
                        format!("pattern_type: {:?}", p.pattern_type),
                        format!("repetitions: {}", p.repetitions),
                        format!("variation_score: {:.2}", p.variation_score),
                    ]
                },
            ),
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
