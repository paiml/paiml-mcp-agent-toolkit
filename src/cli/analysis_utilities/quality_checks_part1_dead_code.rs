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


    // Use the same CargoDeadCodeAnalyzer that `pmat analyze dead-code` uses
    // to ensure consistent results (fixes #141).
    // Falls back gracefully if cargo analysis is unavailable (non-Rust projects,
    // missing Cargo.toml, etc.)
    Ok(check_dead_code_outcome(project_path, max_percentage).await?.violations)
}

/// What the dead-code check could and could not do for one path.
///
/// `check_dead_code` returned an empty list from a single `Err(_)` arm that
/// covered both "cargo check failed" and "no Cargo.toml", so an uncompilable
/// crate — the state a pre-commit gate most often meets — printed
/// `0 violations found` and the word `not_measured` appeared nowhere, while
/// `analyze dead-code` on the same tree said `{"not_measured": true}` at exit 5.
/// The two cases are kept apart here: a crate that could not be compiled is
/// NOT MEASURED; a directory with no manifest at or above it is NOT APPLICABLE,
/// and mapping both to the first would leave every non-Rust repository
/// permanently amber (CRUX-02, #1153).
#[derive(Debug, Default)]
pub struct DeadCodeOutcome {
    /// Findings from a measurement that ran.
    pub violations: Vec<QualityViolation>,
    /// Set when the analyzer ran and could not measure — the reason names why.
    pub not_measured: Option<UnmeasuredCheck>,
    /// Set when there is nothing to measure: no `Cargo.toml` at or above the path.
    pub not_applicable: Option<UnmeasuredCheck>,
}

/// The name this check carries in `--checks` and in the disclosure lists.
pub const DEAD_CODE_CHECK: &str = "dead_code";

/// Run the dead-code check and say which of its three outcomes happened.
///
/// # Errors
/// Never for the analyzer's own failures — those are reported as
/// `not_measured`; the `Result` is kept for the contract macro and callers.
pub async fn check_dead_code_outcome(
    project_path: &Path,
    max_percentage: f64,
) -> Result<DeadCodeOutcome> {
    use crate::services::cargo_dead_code_analyzer::CargoDeadCodeAnalyzer;
    let mut outcome = DeadCodeOutcome::default();
    let shown = project_path.display().to_string();
    if crate::services::cargo_dead_code_analyzer::enclosing_crate_root(project_path).is_none() {
        outcome.not_applicable = Some(UnmeasuredCheck {
            check: DEAD_CODE_CHECK.to_string(),
            path: shown,
            reason: format!(
                "no Cargo.toml at or above {} — dead-code detection needs a crate `cargo check` \
                 can compile, so there is nothing to measure here (not a failure)",
                project_path.display()
            ),
        });
        return Ok(outcome);
    }
    let analyzer = CargoDeadCodeAnalyzer::new(project_path);
    let report = match analyzer.analyze().await {
        Ok(r) => r,
        Err(e) => {
            outcome.not_measured = Some(UnmeasuredCheck {
                check: DEAD_CODE_CHECK.to_string(),
                path: shown,
                reason: dead_code_unmeasured_reason(&e),
            });
            return Ok(outcome);
        }
    };
    // A reduced scan — the compiler layer refused (lockfile) or was suppressed
    // (PMAT_DEAD_CODE_SKIP) — measured only explicit `allow(dead_code)`
    // admissions. That is not a measurement of dead code, and it must not
    // render as one: disclose it, and keep whatever the reduced scan found.
    if let Some(scan) = report
        .compiler_scan
        .as_ref()
        .filter(|s| s.verdict == crate::models::dead_code::COMPILER_SCAN_REDUCED)
    {
        outcome.not_measured = Some(UnmeasuredCheck {
            check: DEAD_CODE_CHECK.to_string(),
            path: shown,
            reason: format!(
                "could not measure: rustc's dead-code lint did not run ({}) — {}",
                scan.reason, scan.detail
            ),
        });
    }
    outcome.violations = dead_code_violations(project_path, max_percentage, &report);
    Ok(outcome)
}

/// One reason string per analyzer failure class, each naming what happened:
/// a compile failure quotes cargo's first error line, a timeout says so.
fn dead_code_unmeasured_reason(e: &anyhow::Error) -> String {
    let msg = e.to_string();
    if let Some(rest) = msg.strip_prefix("Cargo check failed:") {
        let first_error = rest
            .lines()
            .map(str::trim)
            .find(|l| l.starts_with("error"))
            .unwrap_or_else(|| rest.trim().lines().next().unwrap_or("").trim());
        format!(
            "could not compile: `cargo check` failed ({first_error}) — dead code cannot be \
             detected in a crate that does not build"
        )
    } else if msg.contains("timed out") {
        format!("could not measure: {msg}")
    } else {
        format!("could not measure: {msg}")
    }
}

/// The findings for a report the analyzer DID produce.
fn dead_code_violations(
    project_path: &Path,
    max_percentage: f64,
    report: &crate::services::cargo_dead_code_analyzer::AccurateDeadCodeReport,
) -> Vec<QualityViolation> {
    let mut violations = Vec::new();

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

    violations
}

#[cfg(test)]
mod dead_code_outcome_tests {
    use super::*;

    /// A one-file crate under a temp dir; `body` is `src/lib.rs`.
    fn crate_with(body: &str) -> tempfile::TempDir {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"fx\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
        )
        .expect("manifest");
        std::fs::create_dir_all(tmp.path().join("src")).expect("src");
        std::fs::write(tmp.path().join("src/lib.rs"), body).expect("lib");
        tmp
    }

    /// CRUX-02 leg 1: a crate `cargo check` cannot compile is NOT MEASURED,
    /// and the reason says "could not compile" — where the gate used to print
    /// `0 violations found`.
    #[test]
    #[serial_test::serial(dead_code_env)]
    fn a_crate_that_does_not_compile_is_reported_as_not_measured() {
        let tmp = crate_with("pub fn broken( {\n");
        let rt = tokio::runtime::Runtime::new().expect("rt");
        let o = rt.block_on(check_dead_code_outcome(tmp.path(), 15.0)).expect("outcome");
        // The whole outcome is in the message so a red run names the path the
        // analyzer took. An assert carries it: the ratchet counts every literal
        // panic-macro call site in src/, comments included.
        assert!(
            o.not_measured.is_some(),
            "not_measured must be set for an uncompilable crate; outcome was: \
             violations={:?} not_applicable={:?}",
            o.violations,
            o.not_applicable
        );
        let u = o.not_measured.expect("checked above");
        assert_eq!(u.check, "dead_code");
        assert!(u.reason.contains("could not compile"), "{}", u.reason);
        assert!(o.not_applicable.is_none());
        assert!(o.violations.is_empty());
    }

    /// Control A: the same crate with the syntax error removed is measured —
    /// no `dead_code` entry in either list.
    #[test]
    #[serial_test::serial(dead_code_env)]
    fn the_same_crate_compiling_is_measured_with_no_disclosure() {
        let tmp = crate_with("pub fn fine() {}\n");
        let rt = tokio::runtime::Runtime::new().expect("rt");
        let o = rt.block_on(check_dead_code_outcome(tmp.path(), 15.0)).expect("outcome");
        assert!(o.not_measured.is_none(), "{:?}", o.not_measured);
        assert!(o.not_applicable.is_none(), "{:?}", o.not_applicable);
    }

    /// Control B (not-applicable ≠ not-measured): a directory with no
    /// `Cargo.toml` at or above it reports NOT APPLICABLE, never not_measured
    /// — otherwise every non-Rust repository is permanently amber.
    #[test]
    #[serial_test::serial(dead_code_env)]
    fn a_directory_without_a_manifest_is_not_applicable_not_unmeasured() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(tmp.path().join("main.py"), "print(1)\n").expect("py");
        let rt = tokio::runtime::Runtime::new().expect("rt");
        let o = rt.block_on(check_dead_code_outcome(tmp.path(), 15.0)).expect("outcome");
        let n = o.not_applicable.expect("not_applicable must be set");
        assert_eq!(n.check, "dead_code");
        assert!(n.reason.contains("no Cargo.toml"), "{}", n.reason);
        assert!(o.not_measured.is_none(), "{:?}", o.not_measured);
    }

    /// The reason classifier quotes cargo's first error line and never
    /// collapses a timeout into a compile failure.
    #[test]
    fn unmeasured_reasons_name_their_cause() {
        let compile = anyhow::anyhow!("Cargo check failed: warning: x\nerror: expected one of `:`\n");
        assert!(dead_code_unmeasured_reason(&compile).contains("could not compile: `cargo check` failed (error: expected one of `:`)"));
        let timeout = anyhow::anyhow!("Dead code analysis timed out after 1 seconds");
        let r = dead_code_unmeasured_reason(&timeout);
        assert!(r.starts_with("could not measure:") && r.contains("timed out"), "{r}");
    }
}
