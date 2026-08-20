//! Transport gate: the **CLI** interface, exercised against the shipped binary.
//!
//! Declared by `[package.metadata.transports] cli = { e2e = "e2e_cli_t" }` in
//! `Cargo.toml`. The table names this target so the release protocol can run
//! `cargo test --test e2e_cli_t` and get a hard exit 101 if it is missing —
//! a bare `cargo test` cannot prove a transport is covered, because cargo
//! silently skips targets whose `required-features` are off.
//!
//! Every assertion here drives `env!("CARGO_BIN_EXE_pmat")`, the artifact cargo
//! just built, as a real child process. Calling the library instead would prove
//! nothing about reachability: a sibling repo's four-way parity suite stayed
//! green for months while two of its transports had no caller from `main.rs`.
//! Spawning the binary is what makes "wired into the entry point" observable.
//!
//! `tests/modules/cli_smoke_test.rs` covers a much wider command matrix, but its
//! main case is `#[ignore]`d as >240s, so it is not a gate. This target is the
//! fast, always-run core: process starts, parses argv, does real work, exits 0.

use std::path::Path;
use std::process::{Command, Output};

/// Run the shipped binary with `args` and return the raw output.
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pmat"))
        .args(args)
        // Keep stderr free of log noise so a failure message stays readable.
        .env("RUST_LOG", "error")
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn the pmat binary for `{args:?}`: {e}"))
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn assert_ok(out: &Output, what: &str) {
    assert!(
        out.status.success(),
        "`pmat {what}` must exit 0, got {:?}\nstderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The process starts and reports the version cargo built it from.
///
/// Asserting against `CARGO_PKG_VERSION` rather than a literal ties the check to
/// the crate being released: a binary left over from an older build, picked up
/// by a hand-written path, would fail here instead of quietly passing.
#[test]
fn version_round_trip() {
    let out = run(&["--version"]);
    assert_ok(&out, "--version");
    let stdout = stdout_of(&out);
    assert!(
        stdout.contains("pmat"),
        "--version must name the binary, got: {stdout}"
    );
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "--version must report {}, got: {stdout}",
        env!("CARGO_PKG_VERSION")
    );
}

/// argv parsing is reachable: the top-level command table renders.
#[test]
fn help_lists_subcommands() {
    let out = run(&["--help"]);
    assert_ok(&out, "--help");
    let stdout = stdout_of(&out);
    assert!(
        stdout.contains("Commands:"),
        "--help must list the subcommand table, got: {stdout}"
    );
    for expected in ["analyze", "context", "quality-gate"] {
        assert!(
            stdout.contains(expected),
            "--help must advertise the `{expected}` subcommand, got: {stdout}"
        );
    }
}

/// A subcommand that does real work, on a fixture whose answer is known.
///
/// `--version`/`--help` only prove clap is wired. This drives the analysis
/// pipeline end to end through the process boundary and checks the numbers,
/// so a dispatcher that parses the command and then routes nowhere fails.
#[test]
fn analyze_complexity_produces_the_expected_numbers() {
    let dir = tempfile::tempdir().expect("tempdir");
    // One function, one branch: cyclomatic 2 by the standard definition.
    std::fs::write(
        dir.path().join("main.rs"),
        "fn main() {\n    let x = 1;\n    if x > 0 {\n        println!(\"hi\");\n    }\n}\n",
    )
    .expect("write fixture");

    let path = dir.path().to_str().expect("utf-8 tempdir path");
    let out = run(&["analyze", "complexity", "--project-path", path]);
    assert_ok(&out, "analyze complexity");
    let stdout = stdout_of(&out);

    for expected in [
        "Files analyzed: 1",
        "Total functions: 1",
        "Max Cyclomatic: 2",
    ] {
        assert!(
            stdout.contains(expected),
            "analyze complexity must report `{expected}` for the fixture, got:\n{stdout}"
        );
    }
    assert!(
        stdout.contains("main"),
        "the analysed function must be named in the report, got:\n{stdout}"
    );
}

/// Machine-readable output is a real interface too — agents consume it.
#[test]
fn json_output_is_parseable() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("main.rs"),
        "fn main() {\n    let x = 1;\n    if x > 0 {\n        println!(\"hi\");\n    }\n}\n",
    )
    .expect("write fixture");

    let path = dir.path().to_str().expect("utf-8 tempdir path");
    let out = run(&[
        "analyze",
        "complexity",
        "--project-path",
        path,
        "--format",
        "json",
    ]);
    assert_ok(&out, "analyze complexity --format json");
    let stdout = stdout_of(&out);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!("--format json must emit exactly one JSON document on stdout: {e}\ngot:\n{stdout}")
    });
    assert!(
        parsed.is_object() || parsed.is_array(),
        "--format json must emit a JSON object or array, got: {parsed}"
    );
}

/// The binary under test is the one cargo built for this run.
///
/// This repo redirects `target-dir`, and a stale artifact at a hand-written path
/// has already produced a fake "fixed" measurement here once. The rest of this
/// file is only meaningful if `CARGO_BIN_EXE_pmat` resolves to a real file.
#[test]
fn the_binary_under_test_exists() {
    let bin = Path::new(env!("CARGO_BIN_EXE_pmat"));
    assert!(
        bin.is_file(),
        "CARGO_BIN_EXE_pmat must point at the freshly built binary, got: {}",
        bin.display()
    );
}
