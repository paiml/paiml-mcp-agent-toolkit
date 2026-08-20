//! CB-2100 falsification suite.
//!
//! `tests.rs` next door covers the three invariants v1 shipped with. This file
//! covers what v1 could not see: the union of roots (INV-2100-1), a verdict
//! that disagrees with its exit code (INV-2100-4), an invocation that can never
//! run (INV-2100-5), a compile that is mistaken for an execution (INV-2100-6),
//! and the rule's own freedom from hardcoded gate names (INV-2100-7).
//!
//! Each falsifier is a fixture. The three case files the backlog named —
//! `pmat quality-gate --perf` exiting 0 while printing a failure,
//! `post-release.yml`'s `--all-features` build that could never succeed, and
//! `feature-matrix.yml` compiling tests it never ran — were all fixed before
//! this rule existed, so none of them reproduces against HEAD. Reconstructing
//! their *shape* as a fixture is what keeps the invariant falsifiable after the
//! defect is gone; a falsifier that passes because the bug was fixed elsewhere
//! tests nothing.

use super::kernel::{gates, reachable, select_by_context, Edge};
use super::ledger::{self, Status};
use super::required::{ContextSource, RequiredContexts};
use super::resolve::{resolve_context, Resolution};
use super::workflow::load_workflows;
use super::{analyze_with_contexts, roster, GateEffectReport, RootEffect};
use crate::models::comply_config::{CheckSeverity, ComplyConfig};
use std::path::Path;
use tempfile::TempDir;

fn fixture(files: &[(&str, &str)]) -> TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    for (rel, body) in files {
        let path = dir.path().join(rel);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&path, body).expect("write");
    }
    dir
}

fn run(dir: &TempDir, contexts: &[&str]) -> GateEffectReport {
    analyze_with_contexts(
        dir.path(),
        &ComplyConfig::default(),
        &RequiredContexts {
            contexts: contexts.iter().map(|s| (*s).to_string()).collect(),
            source: ContextSource::Env,
        },
    )
}

fn why(report: &GateEffectReport) -> String {
    format!(
        "holes={:?} unreachable={:?} enforcing={:?} neutered={:?}",
        report.holes,
        report.unreachable_rules,
        report.enforcing().collect::<Vec<_>>(),
        report.neutered().collect::<Vec<_>>()
    )
}

fn suppression_mentions(report: &GateEffectReport, needle: &str) -> bool {
    report
        .neutered()
        .flat_map(|i| i.suppressions.iter())
        .any(|s| s.contains(needle))
}

// ── the rule's identity ─────────────────────────────────────────────────────

/// CB-2100, not CB-1411. The id moved because the seven-invariant rule is a
/// different rule from the three-invariant one, and because a band audit found
/// CB-21xx free while CB-14xx is dense with live rules.
#[test]
fn the_rule_is_registered_as_cb_2100() {
    let config = ComplyConfig::default();
    let entry = config
        .checks
        .get("cb-2100")
        .expect("cb-2100 must be a declared comply check");
    assert!(entry.enabled, "CB-2100 must ship enabled");
    assert!(
        matches!(
            entry.severity,
            CheckSeverity::Error | CheckSeverity::Critical
        ),
        "CB-2100 must be severity=error — a gate-effect rule that is only a warning \
         cannot fail anything, which is the defect it exists to find"
    );
}

/// INV-2100-7. The roots come from branch protection; nothing in the rule may
/// know the name of a gate. A rule that hardcodes `gate` reports a repository
/// as compliant on the day it renames a job — it fails its own INV-2100-3.
#[test]
fn inv_2100_7_no_gate_name_is_hardcoded_in_the_rule() {
    // The falsifier's own data, deliberately NOT read by the rule: these are
    // this repository's live required contexts as of the branch-protection API.
    const FORBIDDEN: &[&str] = &[
        "ci / gate",
        "feature-gate",
        "docs build (docs.rs environment)",
        "pmat score",
        "quality-gate / score",
    ];
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sources = [
        "src/services/gate_effect/mod.rs",
        "src/services/gate_effect/effect.rs",
        "src/services/gate_effect/graph.rs",
        "src/services/gate_effect/invocation.rs",
        "src/services/gate_effect/kernel.rs",
        "src/services/gate_effect/ledger.rs",
        "src/services/gate_effect/reach.rs",
        "src/services/gate_effect/required.rs",
        "src/services/gate_effect/resolve.rs",
        "src/services/gate_effect/roster.rs",
        "src/services/gate_effect/workflow.rs",
        "src/cli/handlers/comply_handlers/check_handlers/check_gate_effect.rs",
        "src/cli/handlers/comply_handlers/check_handlers/check_builders_gate_effect.rs",
    ];
    for rel in sources {
        let text = std::fs::read_to_string(root.join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"));
        for (n, line) in text.lines().enumerate() {
            for literal in string_literals(line) {
                assert!(
                    !FORBIDDEN.iter().any(|f| literal.contains(f)) && literal != "gate",
                    "{rel}:{} hardcodes the gate name `{literal}` — roots must come from \
                     branch protection, never from a literal",
                    n + 1
                );
            }
        }
    }
}

/// String literals on a line, with `//` comments removed first. Prose about
/// gates is fine; a *literal* naming one is not.
fn string_literals(line: &str) -> Vec<String> {
    let code = match line.find("//") {
        Some(i) => &line[..i],
        None => line,
    };
    code.split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

// ── the roster the rule is asked about ──────────────────────────────────────

/// A configuration that declares rules and grades none of them `error` is the
/// vacuous case wearing a disguise: the checks run, they print, and they cannot
/// fail. The hole must say so, and must say why — an id absent from `checks:`
/// resolves to `Warning`, not to its default severity, so a partial map demotes
/// every rule it omits.
#[test]
fn a_config_whose_checks_are_all_sub_error_is_a_hole_with_a_diagnosis() {
    let mut config = ComplyConfig::default();
    for check in config.checks.values_mut() {
        check.severity = CheckSeverity::Warning;
    }
    let dir = fixture(&[(
        ".github/workflows/ci.yml",
        "name: CI\njobs:\n  quality:\n    steps:\n      - run: pmat comply check\n",
    )]);
    let report = analyze_with_contexts(
        dir.path(),
        &config,
        &RequiredContexts {
            contexts: vec!["quality".to_string()],
            source: ContextSource::Env,
        },
    );
    assert!(!report.passed(), "{}", why(&report));
    let hole = report.holes.join(" | ");
    assert!(hole.contains("not one of them is severity=error"), "{hole}");
    assert!(hole.contains("resolves to Warning"), "{hole}");
}

// ── INV-2100-1: the roots are unioned ───────────────────────────────────────

const TWO_CONTEXTS: &str = r#"
name: CI
jobs:
  docs:
    name: docs build
    runs-on: ubuntu-latest
    steps:
      - run: cargo doc
  quality:
    name: quality
    runs-on: ubuntu-latest
    steps:
      - run: pmat comply check
"#;

#[test]
fn inv_2100_1_any_root_may_carry_the_roster() {
    let dir = fixture(&[(".github/workflows/ci.yml", TWO_CONTEXTS)]);
    // The first root reaches nothing; the union still reaches the roster.
    let report = run(&dir, &["docs build", "quality"]);
    assert!(report.passed(), "{}", why(&report));
    assert_eq!(
        report.context_effects(),
        vec![
            ("docs build".to_string(), RootEffect::ReachesNothing),
            ("quality".to_string(), RootEffect::Carries)
        ],
        "each root must be attributed separately even though the verdict unions them — and \
         `docs build` is a job this repository READ, so it is a measured zero, not a hole"
    );
}

#[test]
fn inv_2100_1_a_root_that_carries_nothing_is_named() {
    let dir = fixture(&[(".github/workflows/ci.yml", TWO_CONTEXTS)]);
    let report = run(&dir, &["docs build"]);
    assert!(
        !report.passed(),
        "a required check that reaches no rule must not pass: {}",
        why(&report)
    );
}

// ── F-1: continue-on-error on the comply job ────────────────────────────────

#[test]
fn f1_job_level_continue_on_error_fails_citing_job_and_key() {
    let wf = r#"
name: CI
jobs:
  quality:
    runs-on: ubuntu-latest
    continue-on-error: true
    steps:
      - run: pmat comply check
"#;
    let dir = fixture(&[(".github/workflows/ci.yml", wf)]);
    let report = run(&dir, &["quality"]);
    assert!(!report.passed(), "{}", why(&report));
    assert!(
        suppression_mentions(&report, "job `quality`")
            && suppression_mentions(&report, "continue-on-error"),
        "{}",
        why(&report)
    );
}

// ── F-2: `|| true` ──────────────────────────────────────────────────────────

#[test]
fn f2_or_true_fails() {
    let wf = r#"
name: CI
jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - run: pmat comply check || true
"#;
    let dir = fixture(&[(".github/workflows/ci.yml", wf)]);
    let report = run(&dir, &["quality"]);
    assert!(!report.passed(), "{}", why(&report));
    assert!(suppression_mentions(&report, "|| true"), "{}", why(&report));
}

// ── F-3: display name equal to a required context ───────────────────────────

/// The sharpest form of INV-2100-3: a job whose *display name* is literally the
/// required context string, in a workflow where the real context is something
/// else. A display-name matcher reports this repository compliant. The context
/// matcher must not.
#[test]
fn f3_a_display_name_equal_to_the_required_context_is_not_a_match() {
    let caller = r#"
name: CI
jobs:
  ci:
    uses: ./.github/workflows/reusable.yml
  decoy:
    name: ci / quality
    runs-on: ubuntu-latest
    steps:
      - run: pmat comply check
"#;
    let callee = r#"
name: Reusable
on:
  workflow_call:
jobs:
  quality:
    name: quality
    runs-on: ubuntu-latest
    steps:
      - run: cargo test --lib
"#;
    let dir = fixture(&[
        (".github/workflows/ci.yml", caller),
        (".github/workflows/reusable.yml", callee),
    ]);
    let set = load_workflows(dir.path());

    // Both the decoy job and the real callee claim the string `ci / quality`:
    // one as a display name, one as a context. Only the context counts.
    assert_eq!(
        resolve_context(&set, "ci / quality"),
        Resolution::Job {
            workflow: Path::new(".github/workflows/reusable.yml").to_path_buf(),
            job_id: "quality".into(),
        },
        "the required context must resolve to the callee job, not to the decoy"
    );
    let report = run(&dir, &["ci / quality"]);
    assert!(
        !report.passed(),
        "the decoy's comply invocation must not count: {}",
        why(&report)
    );
}

// ── F-4: zero jobs — fail closed ────────────────────────────────────────────

#[test]
fn f4_zero_jobs_fails_closed() {
    let dir = fixture(&[(".github/workflows/ci.yml", "name: CI\non:\n  push: {}\n")]);
    let report = run(&dir, &["anything"]);
    assert!(!report.passed(), "{}", why(&report));
    assert!(
        report.holes.iter().any(|h| h.contains("zero jobs")),
        "{}",
        why(&report)
    );
}

// ── F-5: branch protection unfetchable — fail closed ────────────────────────

/// A directory with no manifest, no override and no answerable API must be an
/// error. "We could not find out what gates this repository" is a failure, and
/// it must never degrade into a pass.
///
/// The override is passed as an argument rather than through the environment:
/// `set_var` is unsafe and racy, and a test that has to disarm the environment
/// to reach its assertion is a test whose assertion gets deleted the first time
/// it flakes.
#[test]
fn f5_unresolvable_required_contexts_fail_closed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let err = super::required::resolve_with_override(None, dir.path())
        .expect_err("an unresolvable required-check list must be an error");
    assert!(
        err.contains("no required status check contexts could be resolved"),
        "the error must say what could not be measured, got: {err}"
    );
}

#[test]
fn f5b_an_empty_override_is_a_failure_not_an_empty_pass() {
    let dir = tempfile::tempdir().expect("tempdir");
    super::required::resolve_with_override(Some(" , ,"), dir.path())
        .expect_err("an empty context list must not resolve");
}

/// And the rule end to end: when the roots cannot be resolved, `analyze` marks
/// every rule unreachable instead of reporting that there was nothing to do.
///
/// The fixture's job id is deliberately not a plausible required-check name, so
/// the assertion holds whether the resolution failed outright (no manifest, no
/// API) or produced somebody else's context list from the environment.
#[test]
fn f5c_an_unresolvable_root_list_marks_every_rule_unreachable() {
    let dir = fixture(&[(
        ".github/workflows/ci.yml",
        "name: CI\njobs:\n  cb2100-fixture-9f3:\n    steps:\n      - run: pmat comply check\n",
    )]);
    let config = ComplyConfig::default();
    let report = super::analyze(dir.path(), &config);
    assert!(!report.passed(), "{}", why(&report));
    assert_eq!(
        report.unreachable_rules,
        super::error_severity_rules(&config),
        "an unresolvable root list must condemn the whole roster: {}",
        why(&report)
    );
}

// ── F-6: the live configuration, as a PASSING fixture ───────────────────────

/// The shape this repository actually has: a required context `ci / gate`
/// produced by a job inside a called workflow, **plus** an unrequired top-level
/// job literally named `gate`. v1 of this rule matched display names and would
/// have answered on the wrong job. This fixture pins that defect as a permanent
/// regression test — and it must PASS, so the suite is not just an
/// implementation that fails everything.
#[test]
fn f6_required_context_beside_an_unrequired_job_named_the_same_thing_passes() {
    let caller = r#"
name: CI
jobs:
  ci:
    uses: ./.github/workflows/reusable.yml
  gate:
    name: gate
    runs-on: ubuntu-latest
    needs: [ci]
    steps:
      - run: echo "not the required check"
"#;
    let callee = r#"
name: Reusable
on:
  workflow_call:
jobs:
  gate:
    name: gate
    runs-on: ubuntu-latest
    steps:
      - run: pmat comply check
"#;
    let dir = fixture(&[
        (".github/workflows/ci.yml", caller),
        (".github/workflows/reusable.yml", callee),
    ]);
    let report = run(&dir, &["ci / gate"]);
    assert!(
        report.passed(),
        "the live shape must pass, or every FAIL above proves nothing: {}",
        why(&report)
    );
    assert_eq!(
        report.enforcing().count(),
        1,
        "exactly the callee's invocation carries it: {}",
        why(&report)
    );
    // The unrequired top-level `gate` job is a different context entirely.
    let set = load_workflows(dir.path());
    assert_eq!(
        resolve_context(&set, "gate"),
        Resolution::Job {
            workflow: Path::new(".github/workflows/ci.yml").to_path_buf(),
            job_id: "gate".into(),
        }
    );
}

// ── INV-2100-4: prints a failure verdict, exits 0 ───────────────────────────

/// The `pmat quality-gate --perf` shape, reconstructed. At HEAD the real
/// command exits 1, so the defect cannot be reproduced against this tree — the
/// fixture is a wrapper script with the same shape, which is what keeps the
/// invariant falsifiable now that the original is fixed.
#[test]
fn inv_2100_4_a_wrapper_that_prints_failed_and_exits_zero_does_not_gate() {
    let wf = r#"
name: CI
jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - run: ./scripts/gate.sh
"#;
    let script = "pmat comply check\necho \"❌ FAILED: complexity p99 over budget\"\nexit 0\n";
    let dir = fixture(&[
        (".github/workflows/ci.yml", wf),
        ("scripts/gate.sh", script),
    ]);
    let report = run(&dir, &["quality"]);
    assert!(!report.passed(), "{}", why(&report));
    assert!(
        suppression_mentions(&report, "prints a failure verdict"),
        "{}",
        why(&report)
    );
}

/// The same invariant as a kernel property, over the pair the rule actually
/// decides on. Proved for all inputs by `KANI-2100-4`.
#[test]
fn inv_2100_4_kernel_gates_only_on_the_exit_code() {
    assert!(!gates(true, 0), "printed FAILED and exited 0 — not a gate");
    assert!(gates(true, 1));
    assert!(gates(false, 0));
    assert!(gates(false, 3));
}

/// And the executable form: a stub command with exactly the defect's shape.
/// Running it is the only way to show the predicate is about real processes
/// rather than about strings.
#[test]
fn inv_2100_4_a_stub_command_that_prints_failed_and_exits_zero_is_caught() {
    let dir = fixture(&[(
        "stub.sh",
        "#!/bin/sh\necho 'FAILED: 3 violations'\nexit 0\n",
    )]);
    let path = dir.path().join("stub.sh");
    let out = std::process::Command::new("sh")
        .arg(&path)
        .output()
        .expect("run stub");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let printed_failure = stdout.contains("FAILED");
    let code = out.status.code().unwrap_or(-1);

    assert!(
        printed_failure && code == 0,
        "stub shape: {stdout} / {code}"
    );
    assert!(
        !gates(printed_failure, code),
        "a command that announces failure and exits 0 gates nothing"
    );
}

// ── INV-2100-5: an invocation that can never succeed ────────────────────────

/// The `post-release.yml --all-features` shape: a job in the closure can never
/// succeed, so the required check can never go green and the invocation
/// downstream of it never produces a verdict anybody acts on.
#[test]
fn inv_2100_5_a_job_that_can_never_succeed_does_not_gate() {
    let wf = r#"
name: CI
jobs:
  quarantine:
    runs-on: ubuntu-latest
    steps:
      - run: |
          echo "the all-features build the fleet audit found could never link"
          false
  quality:
    runs-on: ubuntu-latest
    needs: [quarantine]
    steps:
      - run: pmat comply check
"#;
    let dir = fixture(&[(".github/workflows/ci.yml", wf)]);
    let report = run(&dir, &["quality"]);
    assert!(!report.passed(), "{}", why(&report));
    assert!(
        suppression_mentions(&report, "can never succeed"),
        "{}",
        why(&report)
    );
}

#[test]
fn inv_2100_5_a_dead_line_after_an_unconditional_failure_does_not_gate() {
    let wf = r#"
name: CI
jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - run: |
          exit 1
          pmat comply check
"#;
    let dir = fixture(&[(".github/workflows/ci.yml", wf)]);
    let report = run(&dir, &["quality"]);
    assert!(!report.passed(), "{}", why(&report));
    assert!(
        suppression_mentions(&report, "never runs"),
        "{}",
        why(&report)
    );
}

/// The control for INV-2100-5, and the one that keeps it honest: `exit 1`
/// inside an `if` is a working gate, not a job that can never succeed. Without
/// this, the detector would fail every correctly-written workflow in the fleet.
#[test]
fn inv_2100_5_a_guarded_exit_is_a_gate_not_an_impossibility() {
    let wf = r#"
name: CI
jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - run: |
          pmat comply check
          if [ ! -f report.json ]; then
            exit 1
          fi
"#;
    let dir = fixture(&[(".github/workflows/ci.yml", wf)]);
    let report = run(&dir, &["quality"]);
    assert!(report.passed(), "{}", why(&report));
}

// ── INV-2100-6: compiling is not executing ──────────────────────────────────

/// The `feature-matrix.yml` shape: a job that built the test binaries and never
/// ran them, while the ledger counted it as coverage.
#[test]
fn inv_2100_6_compiling_a_test_is_not_running_it() {
    let job = super::workflow::parse_workflow(
        Path::new(".github/workflows/feature.yml"),
        "jobs:\n  feature-tests:\n    steps:\n      - run: cargo test --all-features --no-run\n",
    )
    .expect("parse")
    .jobs
    .remove(0);

    let found = super::invocation::find_in_job(Path::new("."), &job, &["cargo test"], &[]);
    assert_eq!(found.len(), 1, "the invocation is there: {found:?}");
    assert!(
        !found[0].is_enforcing(),
        "but it compiles without executing: {found:?}"
    );
    assert!(
        found[0].suppressions.iter().any(|s| s.contains("--no-run")),
        "the reason must name the flag: {found:?}"
    );
}

#[test]
fn inv_2100_6_actually_running_the_test_does_establish_reachability() {
    let job = super::workflow::parse_workflow(
        Path::new(".github/workflows/feature.yml"),
        "jobs:\n  feature-tests:\n    steps:\n      - run: cargo test --all-features\n",
    )
    .expect("parse")
    .jobs
    .remove(0);
    let found = super::invocation::find_in_job(Path::new("."), &job, &["cargo test"], &[]);
    assert_eq!(found.len(), 1);
    assert!(found[0].is_enforcing(), "{found:?}");
}

// ── the kernels ─────────────────────────────────────────────────────────────

#[test]
fn kernel_reachability_unions_the_roots() {
    // 0,1 roots; 2 reachable only from root 1.
    let edges = [Edge::live(1, 2)];
    assert!(reachable(3, &edges, &[0, 1], 2));
    assert!(!reachable(3, &edges, &[0], 2));
}

#[test]
fn kernel_a_dead_edge_is_not_an_edge() {
    let edges = [Edge::dead(0, 1), Edge::live(1, 2)];
    assert!(
        !reachable(3, &edges, &[0], 2),
        "neutering the first hop must disconnect everything behind it"
    );
}

#[test]
fn kernel_no_roots_reaches_nothing() {
    let edges = [Edge::live(0, 1)];
    assert!(!reachable(2, &edges, &[], 1));
    assert!(!reachable(2, &edges, &[], 0));
}

#[test]
fn kernel_selects_on_context_never_on_display() {
    let candidates = [
        ("gate".to_string(), "ci / gate".to_string()),
        ("ci / gate".to_string(), "gate".to_string()),
    ];
    assert_eq!(
        select_by_context(&candidates, &"ci / gate".to_string()),
        Some(1),
        "index 0's DISPLAY name equals the required context; that must not match"
    );
}

// ── the ledger ──────────────────────────────────────────────────────────────

/// The ledger's population is the *registry* — the clause ids the builders
/// register, which is what CB-1703 already holds the documentation to. Two
/// implementations of "which rules exist" would give two answers, and this
/// asserts there is only one.
#[test]
fn the_roster_is_exactly_the_registered_rule_set() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let registered =
        crate::cli::handlers::comply_handlers::check_evidence_gates::enumerate_comply_rule_ids(
            root,
        )
        .expect("the comply rule registry must be enumerable");
    let rules = roster::collect(root);
    assert!(
        registered.len() > 100,
        "the registry is the population; got {}",
        registered.len()
    );
    let keys: Vec<String> = rules.iter().map(|r| r.config_key()).collect();
    let mut sorted_registered: Vec<String> = registered.iter().cloned().collect();
    sorted_registered.sort();
    let mut sorted_keys = keys.clone();
    sorted_keys.sort();
    assert_eq!(
        sorted_keys, sorted_registered,
        "the ledger must cover exactly the registered rules — no invented rows, no omissions"
    );
    assert!(
        rules.iter().any(|r| r.id == "CB-2100"),
        "the rule must appear in its own ledger — a gate that exempts itself is the \
         defect this rule exists to find"
    );
    for r in &rules {
        assert!(
            r.has_citation() && r.file.starts_with("src/"),
            "every registered rule in this repository has a definition site, got {r:?}"
        );
    }
}

/// An empty registry is a failure, not a clean sheet — and `collect` returns
/// the empty vector that makes the ledger say so.
#[test]
fn an_unenumerable_registry_yields_an_empty_roster() {
    let empty = tempfile::tempdir().expect("tempdir");
    assert!(roster::collect(empty.path()).is_empty());
}

#[test]
fn ledger_rows_carry_a_status_and_a_citation_for_every_rule() {
    let dir = fixture(&[(
        ".github/workflows/ci.yml",
        "name: CI\njobs:\n  quality:\n    steps:\n      - run: pmat comply check\n",
    )]);
    let report = run(&dir, &["quality"]);
    // The roster is read from this repository, the verdict from the fixture.
    let rows = ledger::rows(
        Path::new(env!("CARGO_MANIFEST_DIR")),
        &report,
        &ComplyConfig::default(),
    )
    .expect("roster is not empty");
    assert!(rows.len() > 100);
    assert!(rows.iter().all(|r| r.status == Status::Enforced));
    assert!(rows.iter().all(|r| !r.carrier.is_empty()));
}

#[test]
fn ledger_marks_every_rule_unreachable_when_nothing_gates() {
    let dir = fixture(&[(
        ".github/workflows/ci.yml",
        "name: CI\njobs:\n  quality:\n    steps:\n      - run: cargo test\n",
    )]);
    let report = run(&dir, &["quality"]);
    let rows = ledger::rows(
        Path::new(env!("CARGO_MANIFEST_DIR")),
        &report,
        &ComplyConfig::default(),
    )
    .expect("roster is not empty");
    assert!(
        rows.iter().all(|r| r.status == Status::Unreachable),
        "a repo whose required check never runs comply has no enforced rules"
    );
}

#[test]
fn ledger_over_an_empty_roster_is_an_error_not_a_clean_sheet() {
    let empty = tempfile::tempdir().expect("tempdir");
    let dir = fixture(&[(
        ".github/workflows/ci.yml",
        "name: CI\njobs:\n  quality:\n    steps:\n      - run: pmat comply check\n",
    )]);
    let report = run(&dir, &["quality"]);
    let err = ledger::rows(empty.path(), &report, &ComplyConfig::default())
        .expect_err("an empty roster must fail closed");
    assert!(err.contains("vacuous"), "{err}");
}

#[test]
fn the_rendered_ledger_is_deterministic() {
    let dir = fixture(&[(
        ".github/workflows/ci.yml",
        "name: CI\njobs:\n  quality:\n    steps:\n      - run: pmat comply check\n",
    )]);
    let report = run(&dir, &["quality"]);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let a = ledger::render(root, &report, &ComplyConfig::default()).expect("render");
    let b = ledger::render(root, &report, &ComplyConfig::default()).expect("render");
    assert_eq!(a, b, "a ledger that changes between runs cannot be diffed");
    assert!(a.contains("| CB-2100 |"), "{}", &a[..400.min(a.len())]);
}

// ── the ledger's identity ───────────────────────────────────────────────────

/// A ledger that is checked by byte-comparing its rendering reddens for reasons
/// that have nothing to do with enforcement. These four tests fix what drift
/// *means*: the data, never the presentation.
const LEDGER_FIXTURE: &str =
    "name: CI\njobs:\n  quality:\n    steps:\n      - run: pmat comply check\n";

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Rewrite every `` `path:N` `` citation to `` `path:N+by` ``, which is exactly
/// what inserting `by` lines above a rule declaration does to the ledger.
fn shift_citation_lines(document: &str, by: usize) -> String {
    document
        .split('`')
        .enumerate()
        .map(|(i, part)| {
            if i % 2 == 0 {
                return part.to_string();
            }
            match part.rsplit_once(':') {
                Some((head, tail)) if tail.parse::<usize>().is_ok() => {
                    format!("{head}:{}", tail.parse::<usize>().expect("digits") + by)
                }
                _ => part.to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join("`")
}

/// FINDING 1. An edit *above* a rule declaration moves its citation and nothing
/// else. A two-line clippy restructure did exactly this and turned CB-2100 red,
/// which teaches people to regenerate the ledger reflexively — and a reflex
/// carries no signal.
#[test]
fn an_edit_above_a_rule_declaration_is_not_ledger_drift() {
    let committed = ledger::committed(repo_root()).expect("this repository commits its ledger");
    let moved = shift_citation_lines(&committed, 2);
    assert_ne!(
        committed, moved,
        "control: the two documents must really differ, or this test proves nothing"
    );
    assert!(
        !ledger::drifted(&committed, &moved),
        "moving a line number is not a change to what this repository enforces"
    );
}

/// FINDING 2. `PMAT_REQUIRED_STATUS_CHECKS` supplying the IDENTICAL contexts the
/// manifest supplies changes one provenance label and nothing else. Comparing
/// the rendering made that a drift failure.
#[test]
fn the_provenance_of_the_root_list_is_not_ledger_drift() {
    let dir = fixture(&[(".github/workflows/ci.yml", LEDGER_FIXTURE)]);
    let config = ComplyConfig::default();
    let mut from_manifest = run(&dir, &["quality"]);
    from_manifest.context_source = Some(ContextSource::Manifest.label().to_string());
    let mut from_env = from_manifest.clone();
    from_env.context_source = Some(ContextSource::Env.label().to_string());

    let a = ledger::render(repo_root(), &from_manifest, &config).expect("render");
    let b = ledger::render(repo_root(), &from_env, &config).expect("render");
    assert_ne!(
        a, b,
        "control: the renderings must really differ, which is why the byte compare fired"
    );
    assert!(
        !ledger::drifted(&a, &b),
        "the same four roots read from a different place are the same four roots"
    );
}

/// The control for both. Loosening drift must not make it blind: a status, a
/// carrier or the FILE a rule lives in changing is still drift.
#[test]
fn a_changed_status_carrier_or_file_is_still_ledger_drift() {
    let base = "| CB-2100 | Comply Gate Effect | error | ENFORCED | ci.yml:gate | `src/a.rs:10` |";
    for mutated in [
        "| CB-2100 | Comply Gate Effect | error | NEUTERED | ci.yml:gate | `src/a.rs:10` |",
        "| CB-2100 | Comply Gate Effect | error | ENFORCED | ci.yml:lint | `src/a.rs:10` |",
        "| CB-2100 | Comply Gate Effect | error | ENFORCED | ci.yml:gate | `src/b.rs:10` |",
        "| CB-2100 | Comply Gate Effect | warning | ENFORCED | ci.yml:gate | `src/a.rs:10` |",
        "| CB-2100 | Renamed | error | ENFORCED | ci.yml:gate | `src/a.rs:10` |",
        "| CB-2101 | Comply Gate Effect | error | ENFORCED | ci.yml:gate | `src/a.rs:10` |",
    ] {
        assert!(
            ledger::drifted(base, mutated),
            "this is a real change and must still be drift: {mutated}"
        );
    }
}

/// A row dropped or added is drift, whatever the line numbers say. The
/// masking must not collapse two documents of different length.
#[test]
fn a_missing_row_is_still_ledger_drift() {
    let committed = ledger::committed(repo_root()).expect("this repository commits its ledger");
    let truncated: String = committed
        .lines()
        .filter(|l| !l.starts_with("| CB-2100 |"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        ledger::drifted(&committed, &truncated),
        "a ledger missing a rule has drifted, however its citations are spelled"
    );
}

// ── no row may be blank ─────────────────────────────────────────────────────

fn repo_ledger_rows() -> Vec<ledger::Row> {
    let dir = fixture(&[(".github/workflows/ci.yml", LEDGER_FIXTURE)]);
    let report = run(&dir, &["quality"]);
    ledger::rows(repo_root(), &report, &ComplyConfig::default()).expect("roster is not empty")
}

/// FINDING 3. A row with no title tells the reader nothing. Six of them shipped
/// blank, and "blank" is indistinguishable from "we know it has no title" and
/// from "the scanner lost it" — which is exactly what had happened to two.
#[test]
fn the_ledger_never_renders_a_blank_title() {
    let dir = fixture(&[(".github/workflows/ci.yml", LEDGER_FIXTURE)]);
    let report = run(&dir, &["quality"]);
    let doc = ledger::render(repo_root(), &report, &ComplyConfig::default()).expect("render");
    let blank: Vec<&str> = doc
        .lines()
        .filter(|l| l.starts_with("| CB-"))
        .filter(|l| l.split('|').nth(2).is_some_and(|c| c.trim().is_empty()))
        .collect();
    assert!(
        blank.is_empty(),
        "a ledger row must never have an empty Title cell — say UNIDENTIFIED and why: {blank:#?}"
    );
}

/// The scan of a handler file stops at `#[cfg(test)]` so that fixture ids like
/// `CB-001: test issue` are not mistaken for declarations. The guard was a
/// substring match, so a file that merely *mentions* the attribute — in a
/// comment, or in a string literal it greps for — lost every declaration below
/// the mention. `check_commit_enforcement_p2.rs` says `#[cfg(test)]` in a
/// comment near its top and declares CB-1334 and CB-1336 hundreds of lines
/// later, so both arrived in the ledger titleless.
#[test]
fn a_cfg_test_mention_in_a_comment_does_not_truncate_the_roster_scan() {
    let rules = roster::collect(repo_root());
    for (id, title) in [
        ("CB-1334", "Hook Atomic Writes"),
        ("CB-1336", "Hook No Injection"),
    ] {
        let rule = rules
            .iter()
            .find(|r| r.id == id)
            .unwrap_or_else(|| panic!("{id} is a registered rule"));
        assert_eq!(
            rule.title, title,
            "{id} declares itself `{id}: {title}`; the roster must not lose it to a \
             `#[cfg(test)]` mentioned in a comment above"
        );
    }
}

/// One comply check, as a function of the project it audits.
type CheckFn = fn(&Path) -> crate::cli::handlers::comply_handlers::ComplianceCheck;

/// A check that never says its own id cannot be correlated with its ledger row
/// by anything a reader or a SARIF consumer can see. Two of them did not.
#[test]
fn every_rule_that_owns_a_cb_id_reports_it_at_runtime() {
    let dir = tempfile::tempdir().expect("tempdir");
    let p = dir.path();
    let cases: [(&str, CheckFn); 4] = [
        (
            "CB-040",
            crate::cli::handlers::comply_handlers::check_handlers::check_extended::check_file_health,
        ),
        (
            "CB-060",
            crate::cli::handlers::comply_handlers::check_compute_brick,
        ),
        (
            "CB-120",
            crate::cli::handlers::comply_handlers::check_oip_tarantula_patterns,
        ),
        (
            "CB-125",
            crate::cli::handlers::comply_handlers::check_coverage_quality_patterns,
        ),
    ];
    for (id, check) in cases {
        let reported = check(p).name;
        assert!(
            reported.contains(id),
            "the check registered as `{}` reports itself as {reported:?}, which names no CB id \
             at all — nothing can tie that finding back to its ledger row",
            id.to_lowercase()
        );
    }
}

/// The control for the scan fix: loosening the `#[cfg(test)]` guard must not
/// start citing lines that do not declare the rule they are attributed to.
#[test]
fn every_citation_points_at_a_line_that_names_its_rule() {
    for rule in roster::collect(repo_root()) {
        if !rule.has_citation() {
            continue;
        }
        let text = std::fs::read_to_string(repo_root().join(&rule.file))
            .unwrap_or_else(|e| panic!("{} is citable: {e}", rule.file.display()));
        let line = text
            .lines()
            .nth(rule.line - 1)
            .unwrap_or_else(|| panic!("{} has no line {}", rule.file.display(), rule.line));
        assert!(
            line.to_uppercase().contains(&rule.id) || line.contains(&rule.config_key()),
            "{} cites {} but that line does not name the rule: {line}",
            rule.id,
            rule.citation()
        );
    }
}

/// The control for the UNIDENTIFIED cell: a rule that genuinely has no title
/// must say so, and say why, rather than render an empty cell.
#[test]
fn an_untitled_rule_renders_as_unidentified_with_a_reason() {
    let untitled = roster::Rule {
        id: "CB-9999".into(),
        title: String::new(),
        file: std::path::PathBuf::from("src/x.rs"),
        line: 7,
    };
    let cell = ledger::title_cell(&untitled);
    assert!(!cell.trim().is_empty(), "never blank");
    assert!(cell.contains("UNIDENTIFIED"), "{cell}");
    assert!(cell.contains("cb-9999"), "the reason names the key: {cell}");
    let titled = roster::Rule {
        title: "Comply Gate Effect".into(),
        ..untitled
    };
    assert_eq!(ledger::title_cell(&titled), "Comply Gate Effect");
}

/// And the whole-roster consequence: after the scan fix and the four checks
/// that now name themselves, this repository has no unidentified rule at all.
#[test]
fn this_repository_has_no_unidentified_rule() {
    let blank: Vec<String> = repo_ledger_rows()
        .iter()
        .filter(|r| r.rule.title.is_empty())
        .map(|r| format!("{} at {}", r.rule.id, r.rule.citation()))
        .collect();
    assert!(
        blank.is_empty(),
        "every registered rule declares `CB-nnn: <title>` somewhere a reader can find: {blank:#?}"
    );
}

// ── a hole is not a zero ────────────────────────────────────────────────────

/// One required context resolves into a reusable workflow hosted in another
/// repository, and one is read in full and reaches nothing. The live shape:
/// `ci / gate` calls `paiml/.github`, and `docs build` is an honest zero.
const OPAQUE_AND_BARREN: &str = r#"
name: CI
jobs:
  ci:
    uses: paiml/.github/.github/workflows/sovereign-ci.yml@main
  docs:
    name: docs build
    runs-on: ubuntu-latest
    steps:
      - run: cargo doc
"#;

/// The rendered roots row for one required context.
fn root_row(doc: &str, context: &str) -> String {
    doc.lines()
        .find(|l| l.starts_with(&format!("| `{context}` |")))
        .unwrap_or_else(|| panic!("no roots row for `{context}` in:\n{doc}"))
        .to_string()
}

fn rendered_for(contexts: &[&str], workflow: &str) -> String {
    let dir = fixture(&[(".github/workflows/ci.yml", workflow)]);
    let report = run(&dir, contexts);
    ledger::render(repo_root(), &report, &ComplyConfig::default()).expect("render")
}

/// FINDING 4. The roots table printed ONE phrase for every context that did not
/// carry a rule — "this required check gates nothing in the CB roster" — which
/// conflates a required check that was read in full and genuinely reaches
/// nothing with one whose steps this repository cannot read at all. The second
/// is a HOLE, and the whole point of the Holes section is that an unmeasured
/// thing is not a measured zero.
#[test]
fn an_opaque_root_is_not_rendered_as_a_measured_zero() {
    let doc = rendered_for(&["ci / gate", "docs build"], OPAQUE_AND_BARREN);
    let opaque = root_row(&doc, "ci / gate");
    let barren = root_row(&doc, "docs build");
    assert_ne!(
        opaque.split_once('|').map(|x| x.1),
        barren.split_once('|').map(|x| x.1),
        "an unreadable required check and a measured zero must not render identically\n\
         opaque: {opaque}\nbarren: {barren}"
    );
    assert!(
        opaque.contains("sovereign-ci.yml"),
        "the opaque row must name the workflow it cannot read: {opaque}"
    );
    assert!(
        !opaque.contains("gates nothing"),
        "an unreadable check is not known to gate nothing — that is precisely what is \
         unmeasured: {opaque}"
    );
    assert!(
        barren.contains("gates nothing") || barren.contains("reaches no"),
        "a root that WAS read and reaches nothing must still say so plainly: {barren}"
    );
}

/// The third case the single phrase also swallowed: a context no job in
/// `.github/workflows` produces at all. It cannot turn green on its own, and
/// what it would have reached is unmeasured — not zero.
#[test]
fn a_phantom_root_is_not_rendered_as_a_measured_zero() {
    let doc = rendered_for(&["nobody-reports-this", "docs build"], OPAQUE_AND_BARREN);
    let phantom = root_row(&doc, "nobody-reports-this");
    let barren = root_row(&doc, "docs build");
    assert_ne!(
        phantom.split_once('|').map(|x| x.1),
        barren.split_once('|').map(|x| x.1),
        "a phantom gate and a measured zero must not render identically\n\
         phantom: {phantom}\nbarren: {barren}"
    );
    assert!(
        !phantom.contains("gates nothing"),
        "nothing is known about what a phantom check gates: {phantom}"
    );
}

/// The control: a root that really does carry the roster must still read as a
/// plain yes. A fix that renders every root "unknown" would satisfy the two
/// tests above and destroy the table.
#[test]
fn a_root_that_carries_the_roster_still_reads_as_yes() {
    let doc = rendered_for(&["quality"], LEDGER_FIXTURE);
    let row = root_row(&doc, "quality");
    assert!(
        row.contains("yes"),
        "a required check that reaches an enforcing invocation carries the roster: {row}"
    );
    assert!(!row.contains("UNKNOWN") && !row.contains("HOLE"), "{row}");
}

// ── the proofs the contract names ───────────────────────────────────────────

const CONTRACT: &str = "contracts/comply-gate-effect-v1.yaml";

fn contract_text() -> String {
    std::fs::read_to_string(repo_root().join(CONTRACT)).expect("the contract is committed")
}

/// Every `kani_harness:` scalar in the contract.
fn named_harnesses(contract: &str) -> Vec<String> {
    contract
        .lines()
        .filter_map(|l| l.trim().strip_prefix("kani_harness:"))
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect()
}

/// Does this repository have anything at all that RUNS kani?
///
/// Deliberately generous: a Makefile target, a workflow step, or a script. If
/// none of them mentions kani, no harness in the tree has ever been discharged
/// here, whatever the contract says.
fn has_a_kani_runner() -> bool {
    let root = repo_root();
    let mut candidates: Vec<std::path::PathBuf> = vec![root.join("Makefile")];
    for dir in ["\u{2e}github/workflows", "scripts"] {
        if let Ok(entries) = std::fs::read_dir(root.join(dir)) {
            candidates.extend(entries.flatten().map(|e| e.path()));
        }
    }
    candidates.iter().any(|p| {
        std::fs::read_to_string(p)
            .map(|t| t.contains("cargo kani") || t.contains("cargo-kani"))
            .unwrap_or(false)
    })
}

/// A named harness must exist. `#[cfg(kani)]` code is never compiled by
/// `cargo test`, so a rename would silently orphan the contract's claim and
/// nothing would notice.
#[test]
fn every_kani_harness_the_contract_names_exists() {
    let contract = contract_text();
    let names = named_harnesses(&contract);
    assert!(
        names.len() >= 3,
        "the contract must still name its kernel harnesses, found {names:?}"
    );
    let sources: String = ["kernel.rs", "graph.rs", "resolve.rs", "effect.rs"]
        .iter()
        .filter_map(|f| {
            std::fs::read_to_string(repo_root().join("src/services/gate_effect").join(f)).ok()
        })
        .collect();
    for name in &names {
        assert!(
            sources.contains(&format!("fn {name}()")),
            "the contract names kani harness `{name}`, which no source in \
             src/services/gate_effect/ defines"
        );
        assert!(
            sources.contains("#[kani::proof]"),
            "the harnesses must be real `#[kani::proof]` functions"
        );
    }
}

/// FINDING 5. `metadata.kind: kernel` plus a `kani_harness:` per equation reads
/// as "these predicates are proved". They are not. Measured at HEAD with kani
/// 0.67.0 installed, `cargo kani` cannot even start on this crate. A contract
/// that names a proof which does not discharge is precisely the defect class
/// this backlog exists to add a rule for, so the contract must say so in its
/// own text rather than leaving the reader to infer it.
#[test]
fn the_contract_states_whether_its_kani_harnesses_discharge() {
    let contract = contract_text();
    assert!(
        contract.contains("kani_status:"),
        "the contract names {} kani harness(es) and never says whether any of them is \
         discharged — `kind: kernel` plus a harness name reads as a proof that ran",
        named_harnesses(&contract).len()
    );
}

/// And the claim must match the repository. If the contract ever says the
/// harnesses ARE discharged, something in the tree has to run them — otherwise
/// the honest note becomes a dishonest one the day someone edits it.
#[test]
fn a_discharged_claim_requires_something_that_actually_runs_kani() {
    let contract = contract_text();
    let claims_discharged = contract
        .lines()
        .filter_map(|l| l.trim().strip_prefix("kani_status:"))
        .any(|v| v.trim() == "discharged");
    if claims_discharged {
        assert!(
            has_a_kani_runner(),
            "the contract claims its kani harnesses are discharged, but no Makefile target, \
             workflow step or script in this repository runs kani"
        );
    } else {
        assert!(
            !has_a_kani_runner(),
            "this repository now runs kani; the contract must stop saying its harnesses are \
             not discharged"
        );
    }
}
