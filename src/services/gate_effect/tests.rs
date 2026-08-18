//! Falsification suite for CB-1411 (gate-effect verification).
//!
//! Every fixture here is a workflow that a naive implementation calls
//! compliant. Each one must FAIL. A test that only shows the happy path passing
//! proves nothing about a rule whose entire job is to refuse to be fooled.

use super::required::{ContextSource, RequiredContexts};
use super::resolve::{resolve_context, Resolution};
use super::workflow::{load_workflows, parse_workflow, TriState};
use super::{analyze_with_contexts, error_severity_rules, GateEffectReport};
use crate::models::comply_config::ComplyConfig;
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

fn required(contexts: &[&str]) -> RequiredContexts {
    RequiredContexts {
        contexts: contexts.iter().map(|s| (*s).to_string()).collect(),
        source: ContextSource::Env,
    }
}

fn run(dir: &TempDir, contexts: &[&str]) -> GateEffectReport {
    analyze_with_contexts(dir.path(), &ComplyConfig::default(), &required(contexts))
}

fn why(report: &GateEffectReport) -> String {
    format!(
        "holes={:?} unreachable={:?} enforcing={:?}",
        report.holes,
        report.unreachable_rules,
        report.enforcing().collect::<Vec<_>>()
    )
}

/// The control. Without a fixture that PASSES, every FAIL below could be an
/// implementation that always fails, which would prove nothing at all.
const HEALTHY: &str = r#"
name: CI
jobs:
  quality:
    name: quality
    runs-on: ubuntu-latest
    steps:
      - name: comply
        run: pmat comply check
"#;

#[test]
fn control_a_plain_enforcing_job_passes() {
    let dir = fixture(&[(".github/workflows/ci.yml", HEALTHY)]);
    let report = run(&dir, &["quality"]);
    assert!(report.passed(), "control must pass: {}", why(&report));
    assert_eq!(report.enforcing().count(), 1, "{}", why(&report));
}

#[test]
fn control_roster_is_not_empty() {
    // If the roster were empty, INV-1400-1 would be vacuous and every fixture
    // below would "pass" for the wrong reason.
    let rules = error_severity_rules(&ComplyConfig::default());
    assert!(
        rules.len() >= 2,
        "expected a non-trivial error-severity roster, got {rules:?}"
    );
}

// ── FALSIFY-1400-1: job carries continue-on-error ───────────────────────────

#[test]
fn falsify_1400_1_job_level_continue_on_error_fails() {
    let wf = r#"
name: CI
jobs:
  quality:
    name: quality
    runs-on: ubuntu-latest
    continue-on-error: true
    steps:
      - name: comply
        run: pmat comply check
"#;
    let dir = fixture(&[(".github/workflows/ci.yml", wf)]);
    let report = run(&dir, &["quality"]);
    assert!(!report.passed(), "{}", why(&report));
    let cited = report
        .neutered()
        .flat_map(|i| i.suppressions.iter())
        .any(|s| s.contains("job `quality`") && s.contains("continue-on-error"));
    assert!(cited, "must cite the job and the key: {}", why(&report));
}

#[test]
fn falsify_1400_1b_step_level_continue_on_error_fails() {
    let wf = r#"
name: CI
jobs:
  quality:
    name: quality
    runs-on: ubuntu-latest
    steps:
      - name: Ladder gate
        continue-on-error: true
        run: pmat comply check --failures-only
"#;
    let dir = fixture(&[(".github/workflows/ci.yml", wf)]);
    let report = run(&dir, &["quality"]);
    assert!(!report.passed(), "{}", why(&report));
    let cited = report
        .neutered()
        .flat_map(|i| i.suppressions.iter())
        .any(|s| s.contains("Ladder gate") && s.contains("continue-on-error"));
    assert!(cited, "must cite the step: {}", why(&report));
}

#[test]
fn continue_on_error_expression_is_not_proof_of_propagation() {
    // `${{ ... }}` cannot be evaluated statically. "Might be false" is not
    // "provably propagates".
    let wf = r#"
name: CI
jobs:
  quality:
    runs-on: ubuntu-latest
    continue-on-error: ${{ github.event_name == 'push' }}
    steps:
      - run: pmat comply check
"#;
    let dir = fixture(&[(".github/workflows/ci.yml", wf)]);
    let report = run(&dir, &["quality"]);
    assert!(!report.passed(), "{}", why(&report));
}

// ── FALSIFY-1400-2: `pmat comply check || true` ─────────────────────────────

#[test]
fn falsify_1400_2_or_true_fails() {
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
    assert!(
        report
            .neutered()
            .flat_map(|i| i.suppressions.iter())
            .any(|s| s.contains("|| true")),
        "{}",
        why(&report)
    );
}

#[test]
fn falsify_1400_2b_exit_code_captured_never_compared_fails() {
    let wf = r#"
name: CI
jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - run: |
          set +e
          pmat comply check
          echo "comply finished"
"#;
    let dir = fixture(&[(".github/workflows/ci.yml", wf)]);
    let report = run(&dir, &["quality"]);
    assert!(!report.passed(), "{}", why(&report));
}

#[test]
fn falsify_1400_2c_piped_invocation_loses_its_status() {
    // GitHub runs `run:` under `bash -e`, which does NOT set pipefail.
    let wf = r#"
name: CI
jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - run: pmat comply check | tee comply.log
"#;
    let dir = fixture(&[(".github/workflows/ci.yml", wf)]);
    let report = run(&dir, &["quality"]);
    assert!(!report.passed(), "{}", why(&report));
}

#[test]
fn a_pipeline_with_pipefail_still_propagates() {
    let wf = r#"
name: CI
jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - run: |
          set -o pipefail
          pmat comply check | tee comply.log
"#;
    let dir = fixture(&[(".github/workflows/ci.yml", wf)]);
    let report = run(&dir, &["quality"]);
    assert!(report.passed(), "{}", why(&report));
}

#[test]
fn an_if_wrapper_that_exits_nonzero_still_propagates() {
    let wf = r#"
name: CI
jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - run: |
          if ! pmat comply check; then
            echo "::error::comply failed"
            exit 1
          fi
"#;
    let dir = fixture(&[(".github/workflows/ci.yml", wf)]);
    let report = run(&dir, &["quality"]);
    assert!(report.passed(), "{}", why(&report));
}

// ── FALSIFY-1400-3: display name vs context string (INV-1400-3) ─────────────

/// The subtle one, and this repository is the proof.
///
/// A top-level job whose *display name* is `gate` reports as `gate`. The
/// required context is `ci / quality`, produced by a job inside the reusable
/// workflow that job `ci` calls. Matching display names finds the enforcing
/// `gate` job and calls the repo compliant — on a check nobody requires.
#[test]
fn falsify_1400_3_display_name_match_is_not_context_match() {
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

    // The naive answer: a job whose display name is `gate` runs comply, enforcing.
    let set = load_workflows(dir.path());
    let by_display = set
        .jobs()
        .find(|j| j.context() == "gate")
        .expect("a job named `gate` exists");
    assert!(
        by_display
            .run_scripts()
            .any(|s| s.contains("pmat comply check")),
        "the display-name match really does look compliant — that is the trap"
    );

    // The correct answer: `ci / quality` is a different job, in a different
    // file, and it never runs comply.
    let report = run(&dir, &["ci / quality"]);
    assert!(!report.passed(), "{}", why(&report));
    assert_eq!(
        resolve_context(&set, "ci / quality"),
        Resolution::Job {
            workflow: Path::new(".github/workflows/reusable.yml").to_path_buf(),
            job_id: "quality".into(),
        }
    );
    // ...and the unrequired top-level job is a *different* context.
    assert_eq!(
        resolve_context(&set, "gate"),
        Resolution::Job {
            workflow: Path::new(".github/workflows/ci.yml").to_path_buf(),
            job_id: "gate".into(),
        }
    );
}

#[test]
fn falsify_1400_3b_external_reusable_workflow_is_opaque_not_compliant() {
    let wf = r#"
name: CI
jobs:
  ci:
    uses: paiml/.github/.github/workflows/sovereign-ci.yml@main
  gate:
    runs-on: ubuntu-latest
    needs: [ci]
    steps:
      - run: pmat comply check
"#;
    let dir = fixture(&[(".github/workflows/ci.yml", wf)]);
    let report = run(&dir, &["ci / gate"]);
    assert!(!report.passed(), "{}", why(&report));
    assert!(
        report.holes.iter().any(|h| h.contains("not readable")),
        "an unreadable callee must be a hole, never a pass: {}",
        why(&report)
    );
}

// ── FALSIFY-1400-4: zero jobs ───────────────────────────────────────────────

#[test]
fn falsify_1400_4_workflow_with_zero_jobs_fails() {
    let dir = fixture(&[(".github/workflows/ci.yml", "name: CI\non:\n  push: {}\n")]);
    let report = run(&dir, &["ci / gate"]);
    assert!(!report.passed(), "{}", why(&report));
    assert!(
        report.holes.iter().any(|h| h.contains("zero jobs")),
        "{}",
        why(&report)
    );
}

#[test]
fn a_phantom_required_context_fails() {
    let dir = fixture(&[(".github/workflows/ci.yml", HEALTHY)]);
    let report = run(&dir, &["a-check-nothing-reports"]);
    assert!(!report.passed(), "{}", why(&report));
    assert!(
        report.holes.iter().any(|h| h.contains("phantom gate")),
        "{}",
        why(&report)
    );
}

#[test]
fn an_unparsable_workflow_is_a_hole() {
    let dir = fixture(&[
        (".github/workflows/ci.yml", HEALTHY),
        (".github/workflows/broken.yml", "jobs:\n  - [unbalanced\n"),
    ]);
    let report = run(&dir, &["quality"]);
    assert!(!report.passed(), "{}", why(&report));
    assert!(
        report.holes.iter().any(|h| h.contains("did not parse")),
        "{}",
        why(&report)
    );
}

// ── needs-closure and `if: always()` ────────────────────────────────────────

#[test]
fn a_needed_job_carries_the_gate_when_the_edge_propagates() {
    let wf = r#"
name: CI
jobs:
  comply:
    runs-on: ubuntu-latest
    steps:
      - run: pmat comply check
  gate:
    runs-on: ubuntu-latest
    needs: [comply]
    steps:
      - run: echo ok
"#;
    let dir = fixture(&[(".github/workflows/ci.yml", wf)]);
    let report = run(&dir, &["gate"]);
    assert!(report.passed(), "{}", why(&report));
}

#[test]
fn if_always_without_a_result_check_breaks_the_edge() {
    let wf = r#"
name: CI
jobs:
  comply:
    runs-on: ubuntu-latest
    steps:
      - run: pmat comply check
  gate:
    runs-on: ubuntu-latest
    needs: [comply]
    if: always()
    steps:
      - run: echo "always green"
"#;
    let dir = fixture(&[(".github/workflows/ci.yml", wf)]);
    let report = run(&dir, &["gate"]);
    assert!(!report.passed(), "{}", why(&report));
}

#[test]
fn if_always_that_inspects_the_result_keeps_the_edge() {
    let wf = r#"
name: CI
jobs:
  comply:
    runs-on: ubuntu-latest
    steps:
      - run: pmat comply check
  gate:
    runs-on: ubuntu-latest
    needs: [comply]
    if: always()
    steps:
      - run: |
          if [ "${{ needs.comply.result }}" != "success" ]; then
            exit 1
          fi
"#;
    let dir = fixture(&[(".github/workflows/ci.yml", wf)]);
    let report = run(&dir, &["gate"]);
    assert!(report.passed(), "{}", why(&report));
}

// ── one hop of indirection ──────────────────────────────────────────────────

#[test]
fn a_makefile_hop_is_followed() {
    let dir = fixture(&[
        (
            ".github/workflows/ci.yml",
            r#"
name: CI
jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - run: make comply
"#,
        ),
        ("Makefile", "comply:\n\t@pmat comply check\n"),
    ]);
    let report = run(&dir, &["quality"]);
    assert!(report.passed(), "{}", why(&report));
}

#[test]
fn a_makefile_hop_with_a_dash_prefix_is_neutered() {
    let dir = fixture(&[
        (
            ".github/workflows/ci.yml",
            r#"
name: CI
jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - run: make comply
"#,
        ),
        ("Makefile", "comply:\n\t-pmat comply check\n"),
    ]);
    let report = run(&dir, &["quality"]);
    assert!(!report.passed(), "{}", why(&report));
}

#[test]
fn a_rule_subsetting_invocation_cannot_stand_for_the_whole_roster() {
    let wf = r#"
name: CI
jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - run: pmat comply check --checks cb-050
"#;
    let dir = fixture(&[(".github/workflows/ci.yml", wf)]);
    let report = run(&dir, &["quality"]);
    assert!(!report.passed(), "{}", why(&report));
}

#[test]
fn a_commented_out_invocation_is_not_an_invocation() {
    let wf = r#"
name: CI
jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - run: |
          # pmat comply check
          echo skipped
"#;
    let dir = fixture(&[(".github/workflows/ci.yml", wf)]);
    let report = run(&dir, &["quality"]);
    assert!(!report.passed(), "{}", why(&report));
    assert_eq!(report.invocations.len(), 0, "{}", why(&report));
}

// ── parser ──────────────────────────────────────────────────────────────────

#[test]
fn parser_reads_needs_as_string_or_sequence() {
    let wf = parse_workflow(
        Path::new("x.yml"),
        "jobs:\n  a:\n    needs: b\n  c:\n    needs: [b, a]\n",
    )
    .expect("parse");
    let a = wf.jobs.iter().find(|j| j.id == "a").expect("a");
    let c = wf.jobs.iter().find(|j| j.id == "c").expect("c");
    assert_eq!(a.needs, vec!["b".to_string()]);
    assert_eq!(c.needs, vec!["b".to_string(), "a".to_string()]);
}

#[test]
fn parser_distinguishes_the_three_continue_on_error_states() {
    let wf = parse_workflow(
        Path::new("x.yml"),
        "jobs:\n  a: {}\n  b:\n    continue-on-error: true\n  c:\n    continue-on-error: ${{ x }}\n",
    )
    .expect("parse");
    let get = |id: &str| {
        wf.jobs
            .iter()
            .find(|j| j.id == id)
            .expect("job")
            .continue_on_error
    };
    assert_eq!(get("a"), TriState::No);
    assert_eq!(get("b"), TriState::Yes);
    assert_eq!(get("c"), TriState::Unknown);
}
