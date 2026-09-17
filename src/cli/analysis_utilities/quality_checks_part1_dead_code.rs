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
        crate_at(tmp.path(), "fx", body);
        tmp
    }

    /// A one-file crate named `name` at `dir`; `body` is `src/lib.rs`.
    fn crate_at(dir: &Path, name: &str, body: &str) {
        std::fs::create_dir_all(dir.join("src")).expect("src");
        std::fs::write(
            dir.join("Cargo.toml"),
            format!("[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[workspace]\n\n[lib]\npath = \"src/lib.rs\"\n"),
        )
        .expect("manifest");
        std::fs::write(dir.join("src/lib.rs"), body).expect("lib");
        // The analyzer passes `--locked` (#1076), so a fixture with no
        // `Cargo.lock` is refused for the MISSING LOCKFILE — not for the
        // reason each test below names. That made the uncompilable-crate leg
        // pass locally for the wrong reason and fail in `ci / test`, where the
        // refusal took a different shape (run 34123854548).
        crate::services::cargo_dead_code_analyzer::write_fixture_lockfile(dir);
    }

    /// `cargo check --lib --bins` exactly as the analyzer spells it, run in
    /// `dir` with the ambient environment; returns whether it exited 0.
    fn plain_cargo_check_succeeds(dir: &Path) -> bool {
        std::process::Command::new("cargo")
            .current_dir(dir)
            .args(["check", "--message-format=json", "--lib", "--bins"])
            .output()
            .expect("cargo check runs")
            .status
            .success()
    }

    /// #1305 (duplicate report #1284): the planted form of the `ci / test`
    /// flake, deterministic.
    ///
    /// cargo fingerprints a workspace-root package by its path RELATIVE to the
    /// workspace root, so two different crates that share a package name share
    /// one fingerprint in a shared target directory — and freshness is decided
    /// by mtime. A source older than the other crate's last check is "fresh":
    /// cargo replays that crate's (empty) diagnostics and exits 0 without
    /// compiling a line of it. `ci / test` sets `CARGO_TARGET_DIR` to a per-PR
    /// directory that `ci / coverage` mounts too, and every nested `cargo
    /// check` in the suite inherits it, so the uncompilable `fx` fixture above
    /// was intermittently answered by another run's compilable `fx`.
    ///
    /// Planted without touching process-global state: a `.cargo/config.toml`
    /// above both crates shares their target dir (an inherited
    /// `CARGO_TARGET_DIR` overrides it and is shared just the same), the
    /// package name is unique to this test so no concurrent process can move
    /// the fingerprint, and the broken source is backdated instead of slept on.
    #[test]
    fn a_broken_crate_sharing_a_target_dir_with_a_same_named_crate_is_not_measured() {
        let parent = tempfile::tempdir().expect("tempdir");
        let shared = parent.path().join("shared-target");
        std::fs::create_dir_all(parent.path().join(".cargo")).expect(".cargo");
        std::fs::write(
            parent.path().join(".cargo/config.toml"),
            format!("[build]\ntarget-dir = {:?}\n", shared.display().to_string()),
        )
        .expect("config");
        let suffix: String = parent
            .path()
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default()
            .chars()
            .filter(char::is_ascii_alphanumeric)
            .collect();
        let name = format!("fx_shared_{suffix}");
        let broken = parent.path().join("broken");
        let fine = parent.path().join("fine");
        crate_at(&broken, &name, "pub fn broken( {\n");
        let an_hour_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
        std::fs::File::options()
            .write(true)
            .open(broken.join("src/lib.rs"))
            .and_then(|f| f.set_modified(an_hour_ago))
            .expect("backdate the broken source");
        crate_at(&fine, &name, "pub fn fine() {}\n");

        // The premise, measured rather than assumed: plain cargo IS fooled.
        // If this ever fails, cargo stopped sharing fingerprints across roots
        // and the plant below no longer tests anything.
        assert!(plain_cargo_check_succeeds(&fine), "the compilable crate must check");
        assert!(
            plain_cargo_check_succeeds(&broken),
            "premise: plain `cargo check` in {} was expected to exit 0 by replaying the \
             same-named crate's fingerprint from the shared target dir; it did not, so this \
             test no longer plants the #1305 condition",
            broken.display()
        );

        let rt = tokio::runtime::Runtime::new().expect("rt");
        let measured = rt.block_on(check_dead_code_outcome(&fine, 15.0)).expect("outcome");
        assert!(measured.not_measured.is_none(), "{:?}", measured.not_measured);
        let o = rt.block_on(check_dead_code_outcome(&broken, 15.0)).expect("outcome");
        assert!(
            o.not_measured.is_some(),
            "an uncompilable crate was reported as MEASURED (violations={:?}) because its \
             `cargo check` replayed a same-named crate's fingerprint from a shared target dir; \
             CARGO_TARGET_DIR={:?}",
            o.violations,
            std::env::var_os("CARGO_TARGET_DIR")
        );
        let reason = o.not_measured.expect("checked above").reason;
        assert!(reason.contains("could not compile"), "{reason}");
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
        // This test's whole premise is "no Cargo.toml at or above". A tempdir
        // gives no such guarantee — it lives wherever TMPDIR points, and if
        // that is inside a cargo workspace then `enclosing_crate_root` walks UP
        // and finds one (#1361). The sibling tests fix that with an empty
        // `[workspace]` table in the fixture's own manifest; this one has no
        // manifest to put it in, by construction. So it asserts its premise
        // instead of assuming it: an unmeetable precondition is `not_measured`,
        // and the right thing is to say so in words the reader can act on.
        assert!(
            crate::services::cargo_dead_code_analyzer::enclosing_crate_root(tmp.path()).is_none(),
            "this test needs a directory with NO Cargo.toml at or above it, and TMPDIR \
             ({}) is inside a cargo workspace — point TMPDIR outside one (#1361)",
            tmp.path().display()
        );
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
