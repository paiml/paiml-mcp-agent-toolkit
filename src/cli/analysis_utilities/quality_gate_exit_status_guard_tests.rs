//! `pmat quality-gate` must gate — proven in the one target CI executes.
//!
//! ## Why this file exists twice over
//!
//! The behaviour under guard is the 3.32.0 fix: a gate that printed
//! "⚠️ Quality gate found 35 blocking violations" and exited **0** unless the
//! caller opted in with `--fail-on-violation`. `Makefile:2239` is
//! `pmat quality-gate --perf --max-complexity-p99 20 || (echo "❌ …" && exit 1)`,
//! so that `||` arm could never run: the repo's own gate was decorative, and so
//! was every other `gate || fail` line anyone had written.
//!
//! The end-to-end proof of that fix lives in `tests/modules/quality_gate_exit_status.rs`,
//! which spawns `env!("CARGO_BIN_EXE_pmat")` and reads the real process exit
//! code. That file belongs to the `all` integration target — and **CI never
//! builds it**. The org's reusable workflow
//! (`paiml/.github/.github/workflows/sovereign-ci.yml`, called from
//! `.github/workflows/ci.yml`) pins the test scope to
//! `TEST_SCOPE: ${{ inputs.test_workspace && '--workspace --lib' || '--lib' }}`
//! and pmat does not set `test_workspace`, so the two commands CI runs are
//! `cargo test --lib` (the `test` job) and `cargo llvm-cov test --lib …` (the
//! `coverage` job). `--lib` cannot see a `tests/*.rs` target. A guard CI cannot
//! see does not guard anything, so the same behaviour is pinned here, in the
//! lib target, where `cargo test --lib` runs it.
//!
//! ## How a lib test observes a process exit code
//!
//! `std::process::exit(1)` is the whole subject, and it is a property of a
//! process, not of a function — so an in-process assertion on a helper's return
//! value would prove nothing about the thing that regressed. `CARGO_BIN_EXE_pmat`
//! is not available to a lib target, and `cargo test --lib` does not even build
//! the binary, so spawning `target/…/pmat` from here would read whatever stale
//! artifact happened to be on disk.
//!
//! So the test binary re-executes **itself**: `std::env::current_exe()` with
//! `--exact <this test's own name>` and [`CHILD_ENV`] set. The child re-enters
//! the same `#[test]` function, takes the child branch, and drives the real
//! production route — `Cli::try_parse_from(argv)` →
//! `CommandDispatcher::execute_command` (what `cli::run` calls) — against a
//! throwaway fixture. Whatever exit code that produces is what the parent reads.
//!
//! Two sentinels make the answer unambiguous:
//!
//! | child exit | meaning |
//! |---|---|
//! | `1` | the gate called `std::process::exit(1)` — it gated |
//! | [`GATE_RETURNED`] | the dispatcher returned normally — the gate did **not** gate |
//! | anything else | the child never got that far (see [`ChildRun::explain`]) |
//!
//! `0` is deliberately not a sentinel: a `--exact` filter that matches no test
//! also exits 0, and "the guard silently tested nothing" must not read as
//! "the gate behaved". Every child run is checked for `running 1 test` for the
//! same reason.

use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Carries the fixture path and the argv into the re-executed child.
const CHILD_ENV: &str = "PMAT_GATE_EXIT_STATUS_GUARD";

/// Field separator inside [`CHILD_ENV`] — a unit separator cannot occur in the
/// paths or flags being passed.
const SEP: char = '\u{1f}';

/// The child reached the end of `execute_command` without the gate exiting.
///
/// In the shipped binary this is the path that ends in exit **0**; the number
/// differs only because the child has to distinguish "returned" from "the
/// filter matched nothing", which also exits 0.
const GATE_RETURNED: i32 = 17;

// ── fixtures ────────────────────────────────────────────────────────────────

/// A crate whose only source file carries one SATD marker the gate classifies
/// `severity: "error"`, i.e. exactly one blocking violation.
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

// ── the child half ──────────────────────────────────────────────────────────

/// `Some(argv)` when this process is the re-executed child, `None` in the
/// ordinary `cargo test --lib` run.
fn child_argv() -> Option<Vec<String>> {
    let raw = std::env::var(CHILD_ENV).ok()?;
    Some(raw.split(SEP).map(str::to_string).collect())
}

/// Drive the real CLI route in this process and never come back.
///
/// `cli::run` does exactly this — `parse_with_suggestions()` then
/// `CommandDispatcher::execute_command(cli.command, server)` — so a regression
/// anywhere between the clap flag and `handle_quality_gate_exit_status` is
/// visible here, including the two dispatch arms that decide what to pass for
/// `exit_on_violation`.
fn run_as_child(argv: Vec<String>) -> ! {
    let fixture = PathBuf::from(&argv[0]);
    let argv: Vec<String> = argv[1..].to_vec();

    // The gate resolves config from the CWD; run from the fixture so this
    // repo's own pmat.toml cannot decide the verdict.
    std::env::set_current_dir(&fixture).expect("cd into fixture");
    // Belt and braces: nothing spawned from here may re-enter child mode.
    std::env::remove_var(CHILD_ENV);

    let outcome = crate::cli::commands::on_big_stack(move || {
        use clap::Parser;
        let cli = crate::cli::Cli::try_parse_from(&argv)
            .unwrap_or_else(|e| panic!("child argv must parse, got: {e}"));
        // `worker_threads`/`thread_stack_size` mirror what the binary gets from
        // `#[tokio::main]` on an 8MB main stack; the default 2MB worker stack
        // overflows in the analysis pipeline.
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_stack_size(8 * 1024 * 1024)
            .enable_all()
            .build()
            .expect("build tokio runtime");
        rt.block_on(async move {
            let server = std::sync::Arc::new(
                crate::stateless_server::StatelessTemplateServer::new()
                    .expect("stateless template server"),
            );
            crate::cli::command_dispatcher::CommandDispatcher::execute_command(cli.command, server)
                .await
        })
    });

    // Still here ⇒ the gate did not exit. Surface a dispatch error rather than
    // reporting it as "the gate declined to gate".
    outcome.expect("the quality gate must not error out");
    std::process::exit(GATE_RETURNED);
}

// ── the parent half ─────────────────────────────────────────────────────────

/// What one re-executed child answered.
struct ChildRun {
    code: Option<i32>,
    stdout: String,
    stderr: String,
    report: serde_json::Value,
}

impl ChildRun {
    /// `results.blocking_violations` from the JSON report the child wrote.
    ///
    /// Read rather than assumed: a test that only checks an exit code cannot
    /// tell "the gate failed" from "the process died", and both are non-zero.
    fn blocking_violations(&self) -> i64 {
        self.report["results"]["blocking_violations"]
            .as_i64()
            .unwrap_or_else(|| {
                panic!(
                    "results.blocking_violations must be present\nreport: {}\n{}",
                    self.report,
                    self.explain()
                )
            })
    }

    fn explain(&self) -> String {
        format!(
            "child exit: {:?}\n--- child stdout ---\n{}\n--- child stderr ---\n{}",
            self.code, self.stdout, self.stderr
        )
    }
}

/// libtest's own name for a `#[test]` in this module — `pmat::` stripped, the
/// way the harness prints it.
fn test_name(function: &str) -> String {
    let module = module_path!();
    let module = module.split_once("::").map_or(module, |(_, rest)| rest);
    format!("{module}::{function}")
}

/// Re-execute this test binary, running only `function`, in child mode.
fn respawn(function: &str, fixture: &Path, extra: &[&str]) -> ChildRun {
    let out_dir = TempDir::new().expect("tempdir for the report");
    let report_path = out_dir.path().join("gate.json");

    let mut payload = vec![
        fixture.to_str().expect("utf-8 fixture path").to_string(),
        "pmat".to_string(),
        "quality-gate".to_string(),
        "--project-path".to_string(),
        fixture.to_str().expect("utf-8 fixture path").to_string(),
        "--checks".to_string(),
        "satd".to_string(),
        "--format".to_string(),
        "json".to_string(),
        "--output".to_string(),
        report_path.to_str().expect("utf-8 report path").to_string(),
    ];
    payload.extend(extra.iter().map(|s| (*s).to_string()));

    // `--nocapture` is load-bearing, not noise: libtest buffers a test's stdout
    // and stderr and prints them when the test ENDS, and the child never ends —
    // it calls `std::process::exit`. Without it the gate's own
    // "❌ Quality gate FAILED" dies in that buffer and the parent sees an empty
    // stderr beside the exit code it is trying to attribute.
    let out = Command::new(std::env::current_exe().expect("current_exe"))
        .args([
            "--exact",
            &test_name(function),
            "--test-threads=1",
            "--nocapture",
        ])
        .env(CHILD_ENV, payload.join(&SEP.to_string()))
        .output()
        .expect("re-exec the test binary");

    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(
        stdout.contains("running 1 test"),
        "the `--exact {}` filter must select exactly this test — a filter that \
         matches nothing also exits 0, and would make every assertion below \
         vacuous\n--- child stdout ---\n{stdout}\n--- child stderr ---\n{stderr}",
        test_name(function)
    );

    let report = std::fs::read_to_string(&report_path).map_or(serde_json::Value::Null, |s| {
        serde_json::from_str(&s).unwrap_or(serde_json::Value::Null)
    });

    ChildRun {
        code: out.status.code(),
        stdout,
        stderr,
        report,
    }
}

// ── the guards ──────────────────────────────────────────────────────────────

/// THE regression: no flags, blocking violations, `exit(1)`.
#[test]
fn a_bare_quality_gate_run_exits_1_on_blocking_violations() {
    if let Some(argv) = child_argv() {
        run_as_child(argv);
    }

    let tmp = TempDir::new().expect("tempdir");
    write_dirty_fixture(tmp.path());

    let run = respawn(
        "a_bare_quality_gate_run_exits_1_on_blocking_violations",
        tmp.path(),
        &[],
    );

    assert_eq!(
        run.blocking_violations(),
        1,
        "fixture assumption: exactly one blocking SATD violation\n{}",
        run.explain()
    );
    assert_eq!(
        run.code,
        Some(1),
        "`pmat quality-gate` reporting a blocking violation must exit 1, so that \
         `gate || fail` callers — Makefile:2239 among them — can see it; \
         {GATE_RETURNED} means the dispatcher returned instead\n{}",
        run.explain()
    );
    assert!(
        run.stderr.contains("Quality gate FAILED"),
        "the non-zero exit must be the gate's verdict, not an unrelated error\n{}",
        run.explain()
    );
}

/// The opt-out: same fixture, same findings, no exit.
#[test]
fn report_only_reports_the_same_findings_without_failing() {
    if let Some(argv) = child_argv() {
        run_as_child(argv);
    }

    let tmp = TempDir::new().expect("tempdir");
    write_dirty_fixture(tmp.path());

    let run = respawn(
        "report_only_reports_the_same_findings_without_failing",
        tmp.path(),
        &["--report-only"],
    );

    assert_eq!(
        run.code,
        Some(GATE_RETURNED),
        "--report-only must not fail the process on a failing tree\n{}",
        run.explain()
    );
    assert_eq!(
        run.blocking_violations(),
        1,
        "--report-only opts out of the VERDICT, not out of the checks: it must \
         still report the blocking violation, or this test proves nothing\n{}",
        run.explain()
    );
}

/// `--no-fail` is the same opt-out under its documented alias.
#[test]
fn no_fail_alias_matches_report_only() {
    if let Some(argv) = child_argv() {
        run_as_child(argv);
    }

    let tmp = TempDir::new().expect("tempdir");
    write_dirty_fixture(tmp.path());

    let run = respawn(
        "no_fail_alias_matches_report_only",
        tmp.path(),
        &["--no-fail"],
    );

    assert_eq!(
        run.code,
        Some(GATE_RETURNED),
        "--no-fail is a visible alias of --report-only\n{}",
        run.explain()
    );
    assert_eq!(run.blocking_violations(), 1, "{}", run.explain());
}

/// Existing `--fail-on-violation` callers must keep working — and keep gating.
#[test]
fn fail_on_violation_is_still_accepted_and_still_gates() {
    if let Some(argv) = child_argv() {
        run_as_child(argv);
    }

    let tmp = TempDir::new().expect("tempdir");
    write_dirty_fixture(tmp.path());

    let run = respawn(
        "fail_on_violation_is_still_accepted_and_still_gates",
        tmp.path(),
        &["--fail-on-violation"],
    );

    assert_eq!(
        run.code,
        Some(1),
        "--fail-on-violation still means what it said\n{}",
        run.explain()
    );
}

/// A clean tree must not exit — the change must not turn "nothing found" into
/// a failure.
#[test]
fn a_clean_tree_does_not_exit_even_though_the_gate_gates_by_default() {
    if let Some(argv) = child_argv() {
        run_as_child(argv);
    }

    let tmp = TempDir::new().expect("tempdir");
    write_clean_fixture(tmp.path());

    let run = respawn(
        "a_clean_tree_does_not_exit_even_though_the_gate_gates_by_default",
        tmp.path(),
        &[],
    );

    assert_eq!(
        run.blocking_violations(),
        0,
        "fixture assumption: a clean tree has no blocking violations\n{}",
        run.explain()
    );
    assert_eq!(
        run.code,
        Some(GATE_RETURNED),
        "a clean tree must pass\n{}",
        run.explain()
    );
}

/// `--file` has its own exit-status call site, so it gets its own proof.
#[test]
fn single_file_mode_gates_by_default() {
    if let Some(argv) = child_argv() {
        run_as_child(argv);
    }

    let tmp = TempDir::new().expect("tempdir");
    let lib = write_dirty_fixture(tmp.path());
    let file = lib.to_str().expect("utf-8 fixture path").to_string();

    let run = respawn(
        "single_file_mode_gates_by_default",
        tmp.path(),
        &["--file", &file],
    );

    assert_eq!(
        run.code,
        Some(1),
        "`quality-gate --file` must gate too\n{}",
        run.explain()
    );
}

/// …and honour the same opt-out.
#[test]
fn single_file_mode_honours_report_only() {
    if let Some(argv) = child_argv() {
        run_as_child(argv);
    }

    let tmp = TempDir::new().expect("tempdir");
    let lib = write_dirty_fixture(tmp.path());
    let file = lib.to_str().expect("utf-8 fixture path").to_string();

    let run = respawn(
        "single_file_mode_honours_report_only",
        tmp.path(),
        &["--file", &file, "--report-only"],
    );

    assert_eq!(
        run.code,
        Some(GATE_RETURNED),
        "`quality-gate --file --report-only` must report without failing\n{}",
        run.explain()
    );
}
