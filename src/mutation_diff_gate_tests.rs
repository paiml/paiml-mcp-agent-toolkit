//! PMAT-630 / #1034 EV-4 — tests for the WIRING of the mutation-on-diff gate.
//!
//! The verdict itself lives in `services::mutation_gate` and has its own
//! fixture suite; nothing here re-tests it. What was missing was everything
//! around it. `evaluate_mutation_gate` / `run_mutation_gate` landed complete and
//! **unreachable**: no CLI subcommand, no Makefile target, no workflow step ever
//! produced a `mutants.out/` for it to read or asked it for an answer, and
//! `cargo mutants --in-diff` appeared in no runnable file in the repository. A
//! verdict function nothing can call is the same defect as a cache nothing
//! writes — correct, and never consulted.
//!
//! So these tests pin the three things that make it live, each of which is a
//! thing a later edit could quietly remove while leaving every existing test
//! green:
//!
//!   1. the producer exists and is executable (`scripts/mutation-diff-gate.sh`);
//!   2. it delegates the verdict to the Rust gate instead of growing a second
//!      copy of the rules in shell;
//!   3. CI runs it, uncushioned, on a cadence the workflow states out loud.

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    let p = repo_root().join(rel);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{} missing: {e}", p.display()))
}

/// The file with `#` comment lines removed.
///
/// Both the producer and the workflow have to *name* the suppressions they
/// refuse to use, in the prose that explains why. Searching raw text for
/// `|| true` therefore flags the sentence promising not to write one — a check
/// that fails on its own documentation teaches people to delete the
/// documentation. Only executable lines are searched.
fn code_of(rel: &str) -> String {
    read(rel)
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
}

const SCRIPT: &str = "scripts/mutation-diff-gate.sh";
const EXAMPLE: &str = "examples/mutation_gate.rs";
const WORKFLOW: &str = ".github/workflows/mutation-diff.yml";

#[test]
fn the_producer_exists_and_is_executable() {
    let p = repo_root().join(SCRIPT);
    let meta = fs::metadata(&p).unwrap_or_else(|e| panic!("{} missing: {e}", p.display()));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert!(
            meta.permissions().mode() & 0o111 != 0,
            "{} is not executable, so the workflow step cannot run it",
            p.display()
        );
    }
    #[cfg(not(unix))]
    let _ = meta;
}

/// The whole point of the producer: it must actually run mutation testing on the
/// diff. `cargo mutants` alone is not enough — it is already in ci.yml over the
/// entire crate, which is a different question and one no PR ever waits for.
#[test]
fn the_producer_runs_cargo_mutants_scoped_to_the_diff() {
    let s = read(SCRIPT);
    assert!(
        s.contains("cargo mutants"),
        "the producer must invoke cargo-mutants"
    );
    assert!(
        s.contains("--in-diff"),
        "the producer must scope the run to the diff; a whole-crate run answers a different question"
    );
    assert!(
        s.contains("git merge-base"),
        "the diff must be taken against a merge base, not against an arbitrary ref"
    );
}

/// Measured, not assumed: with cargo-mutants 27.0.0, a `--in-diff` run that
/// selects no mutants exits 0, writes no report, and leaves any PREVIOUS
/// `mutants.out/` untouched. The Rust gate fails closed on a *missing* artifact
/// but cannot detect a *stale* one, so deleting it before every run is the one
/// invariant that has to live in the producer.
#[test]
fn the_producer_clears_a_stale_report_before_running() {
    let s = code_of(SCRIPT);
    let clear = s
        .find("rm -rf mutants.out")
        .expect("the producer must delete mutants.out before running");
    let run = s
        .find("cargo mutants \\")
        .expect("the producer must invoke cargo mutants");
    assert!(
        clear < run,
        "mutants.out is cleared after the run, which leaves the stale-report window open"
    );
}

/// One verdict, in one place. A shell reimplementation would be a second set of
/// rules to keep in sync, and the copy CI does not run is the one that drifts.
#[test]
fn the_producer_delegates_the_verdict_to_the_rust_gate() {
    let s = code_of(SCRIPT);
    assert!(
        s.contains("--example mutation_gate"),
        "the producer must hand the result to services::mutation_gate"
    );
    // A shell verdict would have to read the counts out of the report itself.
    for leak in ["total_mutants", "outcomes.json", "missed.txt"] {
        assert!(
            !s.contains(leak),
            "the producer reads `{leak}` — that is a second verdict growing in shell"
        );
    }
}

/// The anti-dead-code test. `services::mutation_gate` was complete and had no
/// caller for its whole life before this change; deleting this one file would
/// return it to that state with every other test still green.
#[test]
fn the_rust_gate_has_a_real_entry_point() {
    let e = read(EXAMPLE);
    assert!(
        e.contains("run_mutation_gate"),
        "{EXAMPLE} must call the gate, not reimplement it"
    );
    assert!(
        e.contains("ExitCode"),
        "{EXAMPLE} must turn the verdict into a process exit status; a gate that only prints is not a gate"
    );
}

/// The gate must be reachable from CI, uncushioned. A mutation gate wearing
/// `continue-on-error` is exactly the defect this backlog exists to remove —
/// ci.yml's existing `mutants` job carries two of them and has, by its own
/// comment, never executed a single mutant.
#[test]
fn the_ci_leg_runs_the_gate_and_cannot_be_neutered() {
    let w = code_of(WORKFLOW);
    assert!(
        w.contains("scripts/mutation-diff-gate.sh run"),
        "the workflow must invoke the full `run` path"
    );
    assert!(
        !w.contains("continue-on-error"),
        "the mutation gate must not be cushioned with continue-on-error"
    );
    assert!(
        !w.contains("|| true"),
        "the mutation gate must not be cushioned with `|| true`"
    );
    // Placement is a decision, and it belongs in the file rather than in a
    // commit message nobody reads.
    assert!(
        w.contains("schedule:") && w.contains("cron:"),
        "the workflow must state the cadence it runs on"
    );
    assert!(
        w.contains("fetch-depth: 0"),
        "a shallow checkout has no merge base, and a gate that cannot see the diff must not run"
    );
}

/// Neither half may suppress its own commands. `set -euo pipefail` is what makes
/// the producer's failures reach the workflow at all.
#[test]
fn neither_half_of_the_gate_suppresses_its_own_failures() {
    let s = code_of(SCRIPT);
    assert!(
        s.contains("set -euo pipefail"),
        "the producer must abort on the first failing command"
    );
    for suppression in ["|| true", "|| :", "continue-on-error", "--skip"] {
        assert!(
            !s.contains(suppression),
            "the producer contains `{suppression}`"
        );
    }
}
