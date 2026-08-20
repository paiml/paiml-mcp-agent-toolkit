//! Falsification tests for the mutation-adequacy gate (EV-4, #1034).
//!
//! Both fixtures are **verbatim `outcomes.json` files written by cargo-mutants
//! 27.0.0**, not hand-written approximations, produced by the experiment
//! recorded in [`falsify_mut_1`] below.

use super::*;

const ALL_CAUGHT: &str = include_str!("mutation_gate_fixtures/all_caught.outcomes.json");
const ALL_MISSED: &str = include_str!("mutation_gate_fixtures/all_missed.outcomes.json");

fn excludes() -> Vec<String> {
    DEFAULT_EXCLUDE_GLOBS
        .iter()
        .map(|s| (*s).to_string())
        .collect()
}

fn scope_touching(paths: &[&str]) -> DiffScope {
    DiffScope::MutableRustSource(paths.iter().map(|p| (*p).to_string()).collect())
}

// ── parsing the real artifact ──────────────────────────────────────────────

#[test]
fn parses_a_real_cargo_mutants_27_artifact() {
    let o = parse_outcomes(ALL_CAUGHT).expect("the fixture is real cargo-mutants output");
    assert_eq!(o.total_mutants, 7);
    assert_eq!(o.caught, 7);
    assert_eq!(o.missed, 0);
    assert_eq!(o.version.as_deref(), Some("27.0.0"));
    // 7 mutants + 1 baseline.
    assert_eq!(o.entries.len(), 8);
    assert_eq!(
        o.entries
            .iter()
            .filter(|e| e.kind == ScenarioKind::Baseline)
            .count(),
        1
    );
    // Every mutant names the file it mutated and carries phase evidence.
    for e in o.entries.iter().filter(|e| e.kind == ScenarioKind::Mutant) {
        assert_eq!(e.file.as_deref(), Some("src/lib.rs"));
        assert!(e.phases.iter().any(|p| p.phase == "Test"));
    }
}

#[test]
fn a_real_all_caught_run_passes_the_gate() {
    let o = parse_outcomes(ALL_CAUGHT).expect("parse");
    let v = evaluate_mutation_gate(Some(&o), &scope_touching(&["src/lib.rs"]));
    assert!(v.passed, "expected a pass, got {:?}", v.findings);
    assert_eq!(v.summary, "7/7 mutants caught in the changed code");
}

// ── FALSIFY-MUT-1 ──────────────────────────────────────────────────────────

/// FALSIFY-MUT-1 (#1034): weaken one assertion in a covered function AND add
/// compensating tests that restore identical line coverage. Coverage does not
/// move; the mutation score collapses; the gate must fail.
///
/// The two fixtures are the two halves of that experiment, run with
/// cargo-mutants 27.0.0 and cargo-llvm-cov over one crate whose only function is
///
/// ```ignore
/// pub fn fee(cents: u32) -> u32 {
///     if cents > 1000 { cents / 10 } else { 50 }
/// }
/// ```
///
/// | | line coverage | functions | mutants caught |
/// |---|---|---|---|
/// | 5 exact assertions            | **100.00%** (8/8) | 100.00% (6/6) | **7/7** |
/// | 1 weakened + 4 compensating   | **100.00%** (8/8) | 100.00% (6/6) | **0/7** |
///
/// The compensating tests are the whole point. Without them the weakened suite
/// stops executing lines and a coverage gate fires first, so the falsifier would
/// prove nothing about mutation adequacy. With them, coverage is bit-identical
/// and only this gate can tell the two suites apart.
#[test]
fn falsify_mut_1_weakened_assertions_at_identical_coverage_fail_the_gate() {
    let strong = parse_outcomes(ALL_CAUGHT).expect("parse");
    let weak = parse_outcomes(ALL_MISSED).expect("parse");

    // Same crate, same file, same number of mutants: the only thing that
    // changed is whether the tests assert anything.
    assert_eq!(strong.total_mutants, weak.total_mutants);

    let scope = scope_touching(&["src/lib.rs"]);
    let before = evaluate_mutation_gate(Some(&strong), &scope);
    let after = evaluate_mutation_gate(Some(&weak), &scope);

    assert!(before.passed, "the strong suite must pass: {before:?}");
    assert!(!after.passed, "the weakened suite must fail");
    assert_eq!(
        after.fired(),
        vec!["INV-MUT-1"],
        "only the surviving-mutant invariant may fire — if anything else fires, the falsifier is \
         proving something other than mutation adequacy: {:?}",
        after.findings
    );
    assert!(
        after.summary.contains("7 mutant(s) survived"),
        "summary must name the survivors: {}",
        after.summary
    );
}

// ── INV-MUT-1 ──────────────────────────────────────────────────────────────

#[test]
fn inv_mut_1_a_single_survivor_fails() {
    let mut o = parse_outcomes(ALL_CAUGHT).expect("parse");
    // Flip exactly one caught mutant to missed, keeping the artifact coherent.
    let e = o
        .entries
        .iter_mut()
        .find(|e| e.summary == "CaughtMutant")
        .expect("a caught mutant");
    e.summary = "MissedMutant".to_string();
    o.caught -= 1;
    o.missed += 1;

    let v = evaluate_mutation_gate(Some(&o), &scope_touching(&["src/lib.rs"]));
    assert!(!v.passed);
    assert_eq!(v.fired(), vec!["INV-MUT-1"]);
}

#[test]
fn inv_mut_1_a_timeout_is_not_a_kill() {
    let mut o = parse_outcomes(ALL_CAUGHT).expect("parse");
    let e = o
        .entries
        .iter_mut()
        .find(|e| e.summary == "CaughtMutant")
        .expect("a caught mutant");
    e.summary = "Timeout".to_string();
    o.caught -= 1;
    o.timeout += 1;

    let v = evaluate_mutation_gate(Some(&o), &scope_touching(&["src/lib.rs"]));
    assert!(!v.passed, "a mutant that timed out was never killed");
    assert_eq!(v.fired(), vec!["INV-MUT-1"]);
}

// ── INV-MUT-2 ──────────────────────────────────────────────────────────────

/// The measured behaviour that makes this invariant necessary: cargo-mutants
/// 27.0.0 run as `--in-diff D`, where `D` changes no Rust source, **exits 0 and
/// writes no `mutants.out/` at all**, leaving any previous one untouched. A gate
/// that trusts the exit code, or reads the directory blindly, passes on a run
/// that tested nothing.
#[test]
fn inv_mut_2_a_missing_artifact_is_a_failure_not_a_pass() {
    let v = evaluate_mutation_gate(None, &scope_touching(&["src/services/foo.rs"]));
    assert!(!v.passed);
    assert_eq!(v.fired(), vec!["INV-MUT-2"]);
    assert!(v.summary.contains("UNMEASURED"), "{}", v.summary);
}

#[test]
fn inv_mut_2_a_missing_artifact_with_an_unknown_scope_is_a_failure() {
    let v = evaluate_mutation_gate(None, &DiffScope::Unknown);
    assert!(!v.passed, "unknown scope must fail closed, not pass");
    assert_eq!(v.fired(), vec!["INV-MUT-2"]);
}

#[test]
fn inv_mut_2_a_missing_artifact_is_only_ok_when_the_diff_touches_no_mutable_rust() {
    let v = evaluate_mutation_gate(None, &DiffScope::NoMutableRustSource);
    assert!(v.passed, "{:?}", v.findings);
}

#[test]
fn inv_mut_2_zero_mutants_against_a_rust_diff_is_a_failure() {
    let o = MutationOutcomes {
        total_mutants: 0,
        caught: 0,
        missed: 0,
        timeout: 0,
        unviable: 0,
        version: Some("27.0.0".into()),
        entries: vec![OutcomeEntry {
            kind: ScenarioKind::Baseline,
            summary: "Success".into(),
            file: None,
            phases: vec![PhaseResult {
                phase: "Test".into(),
                succeeded: true,
            }],
        }],
    };
    let v = evaluate_mutation_gate(Some(&o), &scope_touching(&["src/services/foo.rs"]));
    assert!(!v.passed, "an empty mutant set is not a pass");
    assert_eq!(v.fired(), vec!["INV-MUT-2"]);
}

#[test]
fn inv_mut_2_zero_mutants_with_an_unknown_scope_is_a_failure() {
    let o = MutationOutcomes {
        total_mutants: 0,
        caught: 0,
        missed: 0,
        timeout: 0,
        unviable: 0,
        version: Some("27.0.0".into()),
        entries: vec![OutcomeEntry {
            kind: ScenarioKind::Baseline,
            summary: "Success".into(),
            file: None,
            phases: vec![PhaseResult {
                phase: "Test".into(),
                succeeded: true,
            }],
        }],
    };
    assert!(!evaluate_mutation_gate(Some(&o), &DiffScope::Unknown).passed);
    assert!(
        evaluate_mutation_gate(Some(&o), &DiffScope::NoMutableRustSource).passed,
        "0 mutants IS legitimate when the diff changes nothing mutable"
    );
}

/// FALSIFY-MUT-2b (B3, #1034/PMAT-630): the doc comment on
/// `examples/mutation_gate.rs` says "no diff means `DiffScope::Unknown`, which
/// the gate treats as a failure ... being unable to see the diff is not
/// permission to pass" — but `evaluate_mutation_gate` only enforced that when
/// NO artifact existed at all. A real, non-empty, all-caught `outcomes.json` —
/// exactly the shape `mutation-diff-gate.sh`'s own header warns a STALE
/// `mutants.out/` can leave behind — walked straight past every INV-MUT-2
/// branch when `total_mutants != 0`: the `o.total_mutants == 0` guard never
/// ran, so the `DiffScope::Unknown` arm inside it never fired, and nothing
/// else in the function looks at `scope` unless it is
/// `MutableRustSource`. Result: an unreadable diff sitting next to *any*
/// nonempty, self-consistent, all-caught artifact was credited as a pass —
/// the fail-OPEN behaviour the doc comment explicitly disclaims.
#[test]
fn falsify_mut_2b_unknown_scope_is_not_saved_by_a_stale_all_caught_artifact() {
    let stale = parse_outcomes(ALL_CAUGHT).expect("real cargo-mutants fixture");
    assert!(
        stale.total_mutants > 0,
        "the fixture must be non-empty to falsify this"
    );

    let v = evaluate_mutation_gate(Some(&stale), &DiffScope::Unknown);
    assert!(
        !v.passed,
        "an unreadable diff must fail closed even when mutants.out/ (however real, however \
         all-caught) sits on disk — it may describe a different change entirely: {:?}",
        v.findings
    );
    assert_eq!(v.fired(), vec!["INV-MUT-2"]);
}

#[test]
fn inv_mut_2_mutants_that_never_compiled_are_not_a_pass() {
    let o = MutationOutcomes {
        total_mutants: 2,
        caught: 0,
        missed: 0,
        timeout: 0,
        unviable: 2,
        version: Some("27.0.0".into()),
        entries: vec![
            OutcomeEntry {
                kind: ScenarioKind::Baseline,
                summary: "Success".into(),
                file: None,
                phases: vec![PhaseResult {
                    phase: "Test".into(),
                    succeeded: true,
                }],
            },
            OutcomeEntry {
                kind: ScenarioKind::Mutant,
                summary: "Unviable".into(),
                file: Some("src/lib.rs".into()),
                phases: vec![PhaseResult {
                    phase: "Build".into(),
                    succeeded: false,
                }],
            },
            OutcomeEntry {
                kind: ScenarioKind::Mutant,
                summary: "Unviable".into(),
                file: Some("src/lib.rs".into()),
                phases: vec![PhaseResult {
                    phase: "Build".into(),
                    succeeded: false,
                }],
            },
        ],
    };
    let v = evaluate_mutation_gate(Some(&o), &scope_touching(&["src/lib.rs"]));
    assert!(!v.passed, "0 executed mutants is 0 evidence");
    assert_eq!(v.fired(), vec!["INV-MUT-2"]);
}

// ── INV-MUT-3 / FALSIFY-MUT-3 ──────────────────────────────────────────────

/// FALSIFY-MUT-3 (#1034): a backend stubbed to report "all caught" must not be
/// able to pass. This is the artifact such a stub produces — correct headline
/// numbers, self-consistent counts, and nothing whatsoever behind them.
#[test]
fn falsify_mut_3_a_stubbed_backend_claiming_all_caught_cannot_pass() {
    let stub = r#"{
      "outcomes": [
        {"scenario": {"Mutant": {"file": "src/lib.rs"}}, "summary": "CaughtMutant"},
        {"scenario": {"Mutant": {"file": "src/lib.rs"}}, "summary": "CaughtMutant"}
      ],
      "total_mutants": 2, "caught": 2, "missed": 0, "timeout": 0, "unviable": 0
    }"#;
    let o = parse_outcomes(stub).expect("parse");
    // The stub's own arithmetic is impeccable: nothing missed, nothing timed
    // out, counts agree with the list. INV-MUT-1 and INV-MUT-2 are both silent.
    assert_eq!((o.caught, o.missed, o.timeout), (2, 0, 0));

    let v = evaluate_mutation_gate(Some(&o), &scope_touching(&["src/lib.rs"]));
    assert!(!v.passed, "a stub must not be able to report success");
    assert!(
        v.fired().iter().all(|i| *i == "INV-MUT-3"),
        "only the backend-integrity invariant can catch this: {:?}",
        v.findings
    );
    let joined = v.summary.clone();
    assert!(
        joined.contains("no unmutated baseline"),
        "must say the baseline is missing: {joined}"
    );
    assert!(
        joined.contains("no build/test phase evidence"),
        "must say the verdicts were asserted, not executed: {joined}"
    );
    assert!(
        joined.contains("cargo_mutants_version"),
        "must say the result is unattributable: {joined}"
    );
}

#[test]
fn inv_mut_3_a_failing_baseline_invalidates_every_kill() {
    let mut o = parse_outcomes(ALL_CAUGHT).expect("parse");
    o.entries
        .iter_mut()
        .find(|e| e.kind == ScenarioKind::Baseline)
        .expect("baseline")
        .summary = "Failure".to_string();
    let v = evaluate_mutation_gate(Some(&o), &scope_touching(&["src/lib.rs"]));
    assert!(!v.passed);
    assert_eq!(v.fired(), vec!["INV-MUT-3"]);
}

#[test]
fn inv_mut_3_headline_counts_that_contradict_the_outcome_list_fail() {
    let mut o = parse_outcomes(ALL_MISSED).expect("parse");
    // Rewrite only the headline, the way a wrapper "summarising" a run would.
    o.caught = 7;
    o.missed = 0;
    let v = evaluate_mutation_gate(Some(&o), &scope_touching(&["src/lib.rs"]));
    assert!(
        !v.passed,
        "the headline must not be able to overrule the list"
    );
    assert!(v.fired().contains(&"INV-MUT-3"));
}

/// The stale-artifact failure mode measured against cargo-mutants 27.0.0: an
/// `--in-diff` run that selects nothing leaves the previous run's `mutants.out`
/// in place. Cross-checking mutant files against the diff is what catches it.
#[test]
fn inv_mut_3_an_artifact_describing_files_the_diff_never_touched_fails() {
    let o = parse_outcomes(ALL_CAUGHT).expect("parse"); // all mutants in src/lib.rs
    let v = evaluate_mutation_gate(Some(&o), &scope_touching(&["src/services/other.rs"]));
    assert!(!v.passed, "a stale artifact must not pass");
    assert!(v.fired().contains(&"INV-MUT-3"));
    assert!(v.summary.contains("may be stale"), "{}", v.summary);
}

// ── diff classification ────────────────────────────────────────────────────

#[test]
fn classify_diff_reads_the_plus_plus_plus_headers() {
    let diff = "diff --git a/src/lib.rs b/src/lib.rs\n\
                --- a/src/lib.rs\n\
                +++ b/src/lib.rs\n\
                @@ -1 +1 @@\n-a\n+b\n";
    assert_eq!(
        classify_diff(diff, &excludes()),
        scope_touching(&["src/lib.rs"])
    );
}

#[test]
fn classify_diff_ignores_non_rust_and_excluded_paths() {
    let diff = "+++ b/README.md\n+++ b/tests/integration.rs\n\
                +++ b/benches/bench.rs\n+++ b/build.rs\n+++ b/src/foo_tests.rs\n";
    assert_eq!(
        classify_diff(diff, &excludes()),
        DiffScope::NoMutableRustSource,
        "cargo-mutants mutates none of these, so an empty mutant set is legitimate"
    );
}

#[test]
fn classify_diff_skips_deletions() {
    let diff = "--- a/src/gone.rs\n+++ /dev/null\n";
    assert_eq!(
        classify_diff(diff, &excludes()),
        DiffScope::NoMutableRustSource
    );
}

#[test]
fn matches_exclude_glob_handles_the_two_shapes_cargo_mutants_uses() {
    assert!(matches_exclude_glob("src/tests/a.rs", "**/tests/**"));
    assert!(!matches_exclude_glob("src/testsuite/a.rs", "**/tests/**"));
    assert!(matches_exclude_glob("crates/x/build.rs", "**/build.rs"));
    assert!(!matches_exclude_glob("crates/x/rebuild.rs", "**/build.rs"));
    assert!(matches_exclude_glob("src/a_tests.rs", "**/*_tests.rs"));
    assert!(!matches_exclude_glob("src/a_testsx.rs", "**/*_tests.rs"));
}

/// Anti-drift: the gate's idea of "mutable Rust source" is read from the same
/// `mutants.toml` cargo-mutants reads, so the two cannot disagree about whether
/// an empty mutant set was legitimate.
#[test]
fn exclude_globs_are_read_from_the_projects_mutants_toml() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("mutants.toml"),
        "# a comment\nexclude_globs = [\n    \"**/tests/**\",\n    \"**/vendored/**\",\n]\n",
    )
    .expect("write");
    let globs = load_exclude_globs(dir.path());
    assert_eq!(globs, vec!["**/tests/**", "**/vendored/**"]);
    assert!(!is_mutable_rust_source("src/vendored/a.rs", &globs));
    assert!(is_mutable_rust_source("src/benches_like.rs", &globs));
}

#[test]
fn exclude_globs_fall_back_to_the_defaults_when_no_mutants_toml_exists() {
    let dir = tempfile::tempdir().expect("tempdir");
    assert_eq!(
        load_exclude_globs(dir.path()).len(),
        DEFAULT_EXCLUDE_GLOBS.len()
    );
}

/// This repository's committed `mutants.toml` must keep excluding test-only
/// trees, or the gate would start demanding mutants for files cargo-mutants
/// never mutates and fail every PR that touches a test.
#[test]
fn this_repos_mutants_toml_still_excludes_the_test_only_trees() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let globs = load_exclude_globs(root);
    for expected in ["**/tests/**", "**/benches/**", "**/*_tests.rs"] {
        assert!(
            globs.iter().any(|g| g == expected),
            "mutants.toml lost `{expected}`; the gate and the tool would now disagree about \
             which files are mutable. globs = {globs:?}"
        );
    }
}

// ── end-to-end over a directory ────────────────────────────────────────────

#[test]
fn run_mutation_gate_fails_closed_when_the_artifact_directory_is_absent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let diff = dir.path().join("pr.diff");
    std::fs::write(&diff, "+++ b/src/lib.rs\n").expect("write");
    let v = run_mutation_gate(dir.path(), Some(&diff));
    assert!(!v.passed);
    assert_eq!(v.fired(), vec!["INV-MUT-2"]);
}

#[test]
fn run_mutation_gate_reads_a_real_artifact_directory() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("mutants.out")).expect("mkdir");
    std::fs::write(
        dir.path().join("mutants.out").join("outcomes.json"),
        ALL_MISSED,
    )
    .expect("write");
    let diff = dir.path().join("pr.diff");
    std::fs::write(&diff, "+++ b/src/lib.rs\n").expect("write");

    let v = run_mutation_gate(dir.path(), Some(&diff));
    assert!(!v.passed);
    assert_eq!(v.fired(), vec!["INV-MUT-1"]);
}

#[test]
fn an_unreadable_diff_is_an_unknown_scope_not_an_empty_one() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("does-not-exist.diff");
    let v = run_mutation_gate(dir.path(), Some(&missing));
    assert!(
        !v.passed,
        "a diff we cannot read must never be read as 'nothing changed'"
    );
    assert_eq!(v.fired(), vec!["INV-MUT-2"]);
}
