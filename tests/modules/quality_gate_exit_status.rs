//! `pmat quality-gate` must gate. Its exit code is the only thing a shell
//! caller reads, and it used to be 0 no matter what the gate found.
//!
//! Measured on this repo with the literal `make dogfood-all` invocation:
//!
//! ```text
//! $ pmat quality-gate --perf --max-complexity-p99 20
//! ⚠️ Quality gate found 35 blocking violations (37 total findings)
//! $ echo $?
//! 0
//! ```
//!
//! `Makefile:2239` is `pmat quality-gate … || (echo "❌ Quality gate failed" &&
//! exit 1)`, so that `||` arm could never run: the repo's own gate was
//! decorative, and so was every other `gate || fail` line anyone had written.
//! Exit 1 lived behind an opt-in `--fail-on-violation`, which meant the command
//! whose NAME is a gate delivered a report by DEFAULT, and the two were
//! indistinguishable to any caller that checks only the exit code. "36 blocking
//! violations" that do not block is a contradiction in terms.
//!
//! The gate now exits non-zero on blocking violations by default;
//! `--report-only` (alias `--no-fail`) is the opt-out for the report use case.
//!
//! Everything here drives `env!("CARGO_BIN_EXE_pmat")` — the artifact cargo just
//! built — because an exit code is a property of the process, not of a function.

use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Clap's usage-error exit code. Distinguished from the gate's own 1 so a test
/// cannot pass because a flag was rejected.
const CLAP_USAGE_ERROR: i32 = 2;

/// A crate whose only source file carries one SATD marker the gate classifies
/// `severity: "error"`, i.e. one blocking violation.
///
/// The marker is assembled rather than written out: this file lives inside the
/// tree pmat gates, and a literal marker here would be a finding there — the
/// detector cannot tell a fixture from a confession.
fn write_dirty_fixture(dir: &Path) -> PathBuf {
    let marker = format!("{}ME", "FIX");
    write_crate(
        dir,
        &format!(
            "/// Adds two numbers.\npub fn add(a: i32, b: i32) -> i32 {{\n\
             \x20   // {marker}: handle overflow\n    a + b\n}}\n"
        ),
    )
}

/// The same crate with the marker removed: zero violations for `--checks satd`.
fn write_clean_fixture(dir: &Path) -> PathBuf {
    write_crate(
        dir,
        "/// Adds two numbers.\npub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    )
}

fn write_crate(dir: &Path, lib_rs: &str) -> PathBuf {
    std::fs::create_dir_all(dir.join("src")).expect("mkdir src");
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"gate-fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write Cargo.toml");
    let lib = dir.join("src").join("lib.rs");
    std::fs::write(&lib, lib_rs).expect("write lib.rs");
    lib
}

/// What one gate run answered: (exit code, stdout, stderr).
struct GateRun {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl GateRun {
    /// `results.blocking_violations` from the JSON report on stdout.
    ///
    /// Read rather than assumed: a test that only checks an exit code cannot
    /// tell "the gate failed" from "the process died", and both are non-zero.
    fn blocking_violations(&self) -> i64 {
        let parsed: serde_json::Value = serde_json::from_str(&self.stdout).unwrap_or_else(|e| {
            panic!(
                "gate must emit parseable JSON on stdout ({e})\nstdout: {}\nstderr: {}",
                self.stdout, self.stderr
            )
        });
        parsed["results"]["blocking_violations"]
            .as_i64()
            .expect("results.blocking_violations must be present")
    }
}

/// `pmat quality-gate --project-path <project> --checks satd --format json <extra…>`
fn run_gate(project: &Path, extra: &[&str]) -> GateRun {
    let out = Command::new(env!("CARGO_BIN_EXE_pmat"))
        .args([
            "quality-gate",
            "--project-path",
            project.to_str().expect("utf-8 fixture path"),
            "--checks",
            "satd",
            "--format",
            "json",
        ])
        .args(extra)
        // The gate resolves config from the CWD; run from the fixture so this
        // repo's own pmat.toml cannot decide the verdict.
        .current_dir(project)
        .output()
        .expect("failed to spawn pmat");
    GateRun {
        code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

/// THE regression: no flags, blocking violations, non-zero exit.
#[test]
fn test_blocking_violations_exit_non_zero_by_default() {
    let tmp = TempDir::new().expect("tempdir");
    write_dirty_fixture(tmp.path());

    let run = run_gate(tmp.path(), &[]);

    assert_eq!(
        run.blocking_violations(),
        1,
        "fixture assumption: exactly one blocking SATD violation"
    );
    assert_eq!(
        run.code,
        Some(1),
        "`pmat quality-gate` reporting a blocking violation must exit 1, so that \
         `gate || fail` callers — Makefile:2239 among them — can see it\n\
         stdout: {}\nstderr: {}",
        run.stdout,
        run.stderr
    );
    assert!(
        run.stderr.contains("Quality gate FAILED"),
        "the non-zero exit must be the gate's verdict, not an unrelated error: {}",
        run.stderr
    );
}

/// The opt-out: same fixture, same findings, exit 0.
#[test]
fn test_report_only_reports_the_same_findings_without_failing() {
    let tmp = TempDir::new().expect("tempdir");
    write_dirty_fixture(tmp.path());

    let default = run_gate(tmp.path(), &[]);
    let report = run_gate(tmp.path(), &["--report-only"]);

    assert_eq!(
        report.code,
        Some(0),
        "--report-only must exit 0 on a failing tree\nstdout: {}\nstderr: {}",
        report.stdout,
        report.stderr
    );
    assert_eq!(
        report.blocking_violations(),
        default.blocking_violations(),
        "--report-only opts out of the VERDICT, not out of the checks: it must \
         still report every finding the gating run reports"
    );
    assert_eq!(
        report.blocking_violations(),
        1,
        "…and that is a non-zero number, or this test proves nothing"
    );
}

/// `--no-fail` is the same opt-out under its documented alias.
#[test]
fn test_no_fail_alias_matches_report_only() {
    let tmp = TempDir::new().expect("tempdir");
    write_dirty_fixture(tmp.path());

    let run = run_gate(tmp.path(), &["--no-fail"]);

    assert_eq!(
        run.code,
        Some(0),
        "--no-fail is a visible alias of --report-only\nstderr: {}",
        run.stderr
    );
    assert_eq!(run.blocking_violations(), 1);
}

/// Existing `--fail-on-violation` callers must keep working — and keep gating.
#[test]
fn test_fail_on_violation_is_still_accepted_and_still_gates() {
    let tmp = TempDir::new().expect("tempdir");
    write_dirty_fixture(tmp.path());

    let run = run_gate(tmp.path(), &["--fail-on-violation"]);

    assert_ne!(
        run.code,
        Some(CLAP_USAGE_ERROR),
        "the flag must still parse, not be rejected: {}",
        run.stderr
    );
    assert_eq!(
        run.code,
        Some(1),
        "--fail-on-violation still means what it said\nstderr: {}",
        run.stderr
    );
}

/// Asking for a report AND a failure is refused rather than silently resolved.
#[test]
fn test_report_only_conflicts_with_fail_on_violation() {
    let tmp = TempDir::new().expect("tempdir");
    write_dirty_fixture(tmp.path());

    let run = run_gate(tmp.path(), &["--report-only", "--fail-on-violation"]);

    assert_eq!(
        run.code,
        Some(CLAP_USAGE_ERROR),
        "contradictory flags must produce a usage error, not a guess\n\
         stdout: {}\nstderr: {}",
        run.stdout,
        run.stderr
    );
}

/// A clean tree exits 0 whichever way it is asked — the change must not turn
/// "nothing found" into a failure.
#[test]
fn test_clean_fixture_exits_zero_with_and_without_the_opt_out() {
    let tmp = TempDir::new().expect("tempdir");
    write_clean_fixture(tmp.path());

    for extra in [vec![], vec!["--report-only"], vec!["--fail-on-violation"]] {
        let run = run_gate(tmp.path(), &extra);
        assert_eq!(
            run.blocking_violations(),
            0,
            "fixture assumption: a clean tree has no blocking violations ({extra:?})"
        );
        assert_eq!(
            run.code,
            Some(0),
            "a clean tree must pass with {extra:?}\nstdout: {}\nstderr: {}",
            run.stdout,
            run.stderr
        );
    }
}

/// `--file` has its own exit-status call site, so it gets its own proof.
#[test]
fn test_single_file_mode_gates_by_default_and_reports_under_opt_out() {
    let tmp = TempDir::new().expect("tempdir");
    let lib = write_dirty_fixture(tmp.path());
    let file = lib.to_str().expect("utf-8 fixture path").to_string();

    let gated = run_gate(tmp.path(), &["--file", &file]);
    assert_eq!(
        gated.code,
        Some(1),
        "`quality-gate --file` must gate too\nstdout: {}\nstderr: {}",
        gated.stdout,
        gated.stderr
    );

    let reported = run_gate(tmp.path(), &["--file", &file, "--report-only"]);
    assert_eq!(
        reported.code,
        Some(0),
        "…and honour the same opt-out\nstdout: {}\nstderr: {}",
        reported.stdout,
        reported.stderr
    );
}
