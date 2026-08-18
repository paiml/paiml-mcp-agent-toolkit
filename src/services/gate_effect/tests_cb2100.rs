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
use super::{analyze_with_contexts, roster, GateEffectReport};
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
    let carries = report.context_carries();
    assert_eq!(
        carries,
        vec![
            ("docs build".to_string(), false),
            ("quality".to_string(), true)
        ],
        "each root must be attributed separately even though the verdict unions them"
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
