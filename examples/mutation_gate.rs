//! PMAT-630 / #1034 EV-4 — the entry point `services::mutation_gate` never had.
//!
//! `evaluate_mutation_gate` / `run_mutation_gate` landed in f30371ad6 with a
//! thorough test suite and **no caller anywhere outside those tests**: no CLI
//! subcommand, no Makefile target, no workflow step. A verdict function nothing
//! can invoke is the same defect as a cache nothing writes — the logic is
//! correct and never consulted, and the repository looks gated while no run is
//! ever judged. This binary is the missing half, and
//! `.github/workflows/mutation-diff.yml` is what calls it.
//!
//! Deliberately an example rather than a `[[bin]]`: `cargo install pmat` must
//! not grow a second executable for a CI-internal tool, and the crate is at 94%
//! of the crates.io size ceiling (see the `exclude` note in Cargo.toml), so this
//! file is packaged out too. `cargo run --example mutation_gate` from a checkout
//! is all CI needs, and `clippy --all-targets` still lints it.
//!
//! Usage:
//!   cargo run --example mutation_gate -- --project <dir> --diff <file>
//!
//! Exit status is the gate: 0 pass, 1 fail. There is no third state, and no
//! flag that turns a failure into a warning.

use std::path::PathBuf;
use std::process::ExitCode;

use pmat::services::mutation_gate::{diff_path_from_env, run_mutation_gate};

fn main() -> ExitCode {
    let mut project = PathBuf::from(".");
    let mut diff: Option<PathBuf> = None;

    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--project" => match args.next() {
                Some(v) => project = PathBuf::from(v),
                None => return usage("--project needs a directory"),
            },
            "--diff" => match args.next() {
                Some(v) => diff = Some(PathBuf::from(v)),
                None => return usage("--diff needs a file"),
            },
            "-h" | "--help" => {
                eprintln!("usage: mutation_gate --project <dir> --diff <unified diff>");
                return ExitCode::from(0);
            }
            other => return usage(&format!("unknown argument {other}")),
        }
    }

    // `$PMAT_MUTATION_DIFF` is the module's own documented fallback. Note that
    // *no* diff means `DiffScope::Unknown`, which the gate treats as a failure
    // rather than as "nothing changed" — being unable to see the diff is not
    // permission to pass.
    let diff = diff.or_else(diff_path_from_env);

    let verdict = run_mutation_gate(&project, diff.as_deref());

    println!("{}", verdict.summary);
    for f in &verdict.findings {
        println!("  {}: {}", f.invariant, f.message);
    }

    if verdict.passed {
        ExitCode::from(0)
    } else {
        ExitCode::from(1)
    }
}

fn usage(msg: &str) -> ExitCode {
    eprintln!("mutation_gate: {msg}");
    eprintln!("usage: mutation_gate --project <dir> --diff <unified diff>");
    // Not 1: a usage error must not be mistakable for "the gate failed", and a
    // caller that checks `!= 0` treats both as red either way.
    ExitCode::from(2)
}
