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
/// ```
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn check_satd(project_path: &Path) -> Result<Vec<QualityViolation>> {
    check_satd_with_scope(project_path).await.map(|(v, _)| v)
}

/// As [`check_satd`], but also returns WHAT THE CHECK DECLINED TO READ.
///
/// `analyze_project` already computes a full `SkipCounts` — tests, out-of-scope
/// (`examples/`, `demo/`, fuzz, generated), minified/vendor, over the 500 KB
/// threshold, and unreadable — and `check_satd` threw every one of them away,
/// keeping only `satd_result.items`. So `pmat quality-gate --checks satd` shipped
/// `satd_violations: N` with no statement of the population N was measured over,
/// which is #1035's root cause one level above the detector: an absent finding
/// rendered identically whether the file was read and clean or never read.
///
/// It is not hypothetical, and the two surfaces disagree because of it. Over a
/// fixture holding `src/lib.rs` (a marker), `examples/demo.rs` (two markers) and
/// a 600 KB `src/big.rs` (a marker):
///
/// ```text
///   pmat analyze satd            Found 2 SATD violations in 2 files
///                                (1 file(s) not read: 1 examples/demo/fuzz/generated)
///   pmat quality-gate --checks satd   "satd_violations": 1, "files_examined": 4
/// ```
///
/// `analyze satd` reads the 600 KB file (its `skip_reason_for_analysis` declines
/// only *minified* content over 1 MB); the gate's `skip_reason` drops anything
/// over 500 KB. Whether to unify the two thresholds is a behaviour change the
/// detector deliberately defers — see `skip_reason_for_analysis` — but the gate
/// stating the scope it measured over is not, and without it a reader cannot
/// see that the two numbers describe different populations at all.
///
/// `files_examined` beside it does not close this: it is counted separately by
/// `count_examined_sources` over the whole tree (4 above, including `Cargo.toml`
/// and the `examples/` file no check read), so it is a population the gate could
/// have looked at, not the one it did.
pub async fn check_satd_with_scope(
    project_path: &Path,
) -> Result<(
    Vec<QualityViolation>,
    crate::services::satd_detector::SkipCounts,
)> {
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
        .iter()
        .map(satd_violation_from_debt)
        .collect();

    Ok((violations, satd_result.skipped.clone()))
}

/// SATD findings for ONE file, from the same detector and the same severity
/// scale `check_satd` uses.
///
/// `pmat quality-gate --file` used to run its own hardcoded regex
/// (`check_single_file_satd`) that stamped `severity:"warning"` on every marker
/// it matched, so the same `// TODO` was `info` from the project gate and
/// `warning` from the file gate — a second severity scale silently deciding
/// pass/fail once the verdict began reading severity. That regex is deleted;
/// this is the only single-file SATD check.
pub async fn check_satd_file(
    project_path: &Path,
    file_path: &Path,
) -> Result<Vec<QualityViolation>> {
    use crate::services::satd_detector::SATDDetector;

    let abs_file_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        project_path.join(file_path)
    };

    if !abs_file_path.exists() {
        return Ok(Vec::new());
    }

    let content = tokio::fs::read_to_string(&abs_file_path).await?;
    let debts = SATDDetector::new().extract_from_content(&content, &abs_file_path)?;

    Ok(debts.iter().map(satd_violation_from_debt).collect())
}

/// The ONE mapping from detector severity to gate severity.
///
/// Every SATD row every quality-gate surface reports comes through here, so
/// "which severities are verdict-bearing" (`is_verdict_bearing`) is asked of one
/// scale rather than of two that disagree.
fn satd_violation_from_debt(
    debt: &crate::services::satd_detector::TechnicalDebt,
) -> QualityViolation {
    use crate::services::satd_detector::Severity;
    QualityViolation {
        check_type: "satd".to_string(),
        severity: match debt.severity {
            Severity::Critical | Severity::High => "error",
            Severity::Medium => "warning",
            Severity::Low => ADVISORY_SEVERITY,
        }
        .to_string(),
        file: debt.file.display().to_string(),
        line: Some(debt.line as usize),
        message: format!(
            "{}: {} (at column {})",
            debt.category, debt.text, debt.column
        ),
        details: None,
    }
}
