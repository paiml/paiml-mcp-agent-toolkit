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

/// As [`check_satd`], but also returns THE POPULATION IT MEASURED OVER.
///
/// `analyze_project` computes a full census — how many files were walked, how
/// many were read, and the rest by reason — and `check_satd` threw every one of
/// those numbers away, keeping only `satd_result.items`. So
/// `pmat quality-gate --checks satd` shipped `satd_violations: N` with no
/// statement of what N was measured over, which is #1035's root cause one level
/// above the detector: an absent finding rendered identically whether the file
/// was read and clean or never read.
///
/// It was not hypothetical, and the two surfaces DISAGREED because of it. Over a
/// fixture holding `src/lib.rs` (a marker), `examples/hello.rs` (two markers)
/// and an 800 KB `src/big.rs` (a marker), on the pre-fix build:
///
/// ```text
///   pmat analyze satd            Found 2 SATD violations in 2 files
///                                (4 file(s) not read: 1 test, 3 out of scope)
///   pmat quality-gate --checks satd   "satd_violations": 1, "files_examined": 7
/// ```
///
/// Four markers existed; one surface saw two, the other one, and neither said
/// why. Both causes are fixed at the detector: `examples/` is shipped code and
/// is analysed, and the two size thresholds are now one
/// ([`MAX_FILE_BYTES`](crate::services::satd_detector::MAX_FILE_BYTES)), so the
/// two commands read the same population. What remains is that the gate must
/// STATE that population, because `files_examined` beside it does not: that is
/// counted separately by `count_examined_sources` over the whole tree (7 above,
/// including `Cargo.toml`), so it is a population the gate could have looked at,
/// not the one it did.
pub async fn check_satd_with_scope(
    project_path: &Path,
) -> Result<(
    Vec<QualityViolation>,
    crate::services::satd_detector::FileCensus,
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

    Ok((violations, satd_result.census.clone()))
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
