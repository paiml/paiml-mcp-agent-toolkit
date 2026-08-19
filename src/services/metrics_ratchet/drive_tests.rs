//! CB-2102 falsification suite for the IMPURE half: measurement, git history,
//! write-back, registration, and the contract's own honesty.
//!
//! `kernel_tests.rs` and `config_tests.rs` falsify the pure comparators. This
//! file falsifies everything those two deliberately cannot reach — and the
//! reason it exists at all is that, until now, nothing reached them either:
//! the whole module had no caller in the tree.

use super::config::{
    Measurement, Measurements, MetricBaseline, Outcome, RatchetConfig, RATCHET_FILE,
};
use super::history::{prior_version, Prior};
use super::{measure, rewrite, run, status, RatchetStatus};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const CONTRACT: &str = "contracts/comply-ratchet-v1.yaml";

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn contract_text() -> String {
    std::fs::read_to_string(repo_root().join(CONTRACT)).expect("the contract is committed")
}

// ── the measurement runner ──────────────────────────────────────────────────

/// The single most dangerous shape in the whole design: `<producer> | wc -l`
/// where the producer fails. Without `pipefail` the pipeline exits 0 and `wc`
/// prints a perfectly plausible `0` — and a ratchet, which only ever looks
/// upward, greets zero as perfection and the lowering pass makes it permanent.
#[test]
fn a_failing_producer_in_a_pipeline_is_unavailable_not_a_zero() {
    match measure::measure(repo_root(), "exit 7 | wc -l") {
        Measurement::Unavailable(_) => {}
        Measurement::Value(v) => {
            panic!("a broken producer was read as a count of {v}")
        }
    }
}

/// The control: an honest pipeline of exactly the same shape does measure.
#[test]
fn an_honest_pipeline_measures() {
    assert_eq!(
        measure::measure(repo_root(), "printf 'a\\nb\\nc\\n' | wc -l"),
        Measurement::Value(3)
    );
}

/// `grep` exits 1 for "no matches", which is a real count of zero. Treating it
/// as an error would mean a metric could never legitimately reach zero — the
/// one place every ratchet is trying to get to.
#[test]
fn no_matches_is_a_zero_not_a_failure() {
    assert_eq!(
        measure::measure(repo_root(), "printf 'a\\n' | grep -c zzzzz"),
        Measurement::Value(0)
    );
}

/// A command that prints prose, or nothing, has not measured anything.
#[test]
fn non_numeric_output_is_not_a_measurement() {
    assert!(matches!(
        measure::parse_count("error: no such file"),
        Measurement::Unavailable(_)
    ));
    assert!(matches!(
        measure::parse_count(""),
        Measurement::Unavailable(_)
    ));
    assert_eq!(measure::parse_count("  12  \n"), Measurement::Value(12));
    // The answer of the `| wc -l` idiom is the LAST line, not the first.
    assert_eq!(
        measure::parse_count("progress...\n42\n"),
        Measurement::Value(42)
    );
    // A metric with no command is a baseline nobody can recompute.
    assert!(matches!(
        measure::measure(repo_root(), "   "),
        Measurement::Unavailable(_)
    ));
}

// ── git history: what the file used to say ──────────────────────────────────

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pmat-cb2102-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn git(dir: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@example.com")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@example.com")
        .output()
        .expect("git runs");
    assert!(out.status.success(), "git {args:?}: {out:?}");
}

/// `None` legitimately means "this is the initial capture, nothing was raised".
/// An unreadable history must NOT borrow that meaning, or an unjustified raise
/// becomes invisible on exactly the machines where history is hardest to read.
#[test]
fn an_unreadable_prior_is_a_hole_not_an_initial_capture() {
    let dir = scratch("nogit");
    match prior_version(&dir, RATCHET_FILE, None) {
        Prior::Unavailable(_) => {}
        other => panic!("a directory that is not a git repository gave {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// The control: in a real repository the previous COMMITTED version is found,
/// even while the working copy is dirty.
#[test]
fn the_prior_version_is_the_newest_committed_text_that_differs() {
    let dir = scratch("prior");
    git(&dir, &["init", "-q"]);
    std::fs::write(dir.join(RATCHET_FILE), "first\n").expect("write");
    git(&dir, &["add", RATCHET_FILE]);
    git(&dir, &["commit", "-qm", "one"]);
    std::fs::write(dir.join(RATCHET_FILE), "second\n").expect("write");

    let current = std::fs::read_to_string(dir.join(RATCHET_FILE)).ok();
    assert_eq!(
        prior_version(&dir, RATCHET_FILE, current.as_deref()),
        Prior::Content("first\n".to_string())
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// The obvious way to pass a gate is to delete its input. `.pmat-ratchet.toml`
/// is absent in a project that never had one — a Skip — and absent in a project
/// that just deleted one, which is a Fail. Git tells the two apart.
#[test]
fn deleting_the_ratchet_file_is_not_a_way_of_passing() {
    assert_eq!(status(repo_root()), RatchetStatus::Present);

    let dir = scratch("deleted");
    git(&dir, &["init", "-q"]);
    assert_eq!(
        status(&dir),
        RatchetStatus::Absent,
        "a project that never had a ratchet has nothing to enforce"
    );
    std::fs::write(dir.join(RATCHET_FILE), "version = 1\n").expect("write");
    git(&dir, &["add", RATCHET_FILE]);
    git(&dir, &["commit", "-qm", "add ratchet"]);
    std::fs::remove_file(dir.join(RATCHET_FILE)).expect("remove");
    assert_eq!(
        status(&dir),
        RatchetStatus::Deleted,
        "deleting a committed ratchet is a finding, not a Skip"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

// ── the lowering pass ───────────────────────────────────────────────────────

fn baseline(v: i64, justification: Option<&str>) -> MetricBaseline {
    MetricBaseline {
        baseline: v,
        unit: "count".into(),
        band: 10,
        includes_test_files: true,
        command: "printf '1\\n'".into(),
        description: "d".into(),
        justification: justification.map(str::to_string),
        zero_is_reachable: false,
    }
}

const SAMPLE: &str = "\
# a comment that must survive
version = 1

[meta]
captured_at_commit = \"abc\"
captured_at = \"2026-08-19\"

[metric.u]
# why this metric exists
baseline = 100
unit = \"count\"
band = 10
includes_test_files = true
command = \"printf '97\\\\n'\"
description = \"d\"
justification = \"a reason that no longer applies\"

[coherence]
threshold_sections = []

[coherence.non_threshold_sections]
";

/// F-2's write-back half, plus the two things a scheduled editor must never
/// do: destroy the file's documentation, or write a number it did not re-read.
#[test]
fn lowering_preserves_comments_and_verifies_what_it_wrote() {
    let mut want = BTreeMap::new();
    want.insert("u".to_string(), 97i64);
    let out = rewrite::apply(SAMPLE, &want).expect("rewrite succeeds");

    assert!(out.contains("# a comment that must survive"));
    assert!(out.contains("# why this metric exists"));
    assert!(out.contains("baseline = 97"));
    assert!(
        !out.contains("justification"),
        "a justification that outlived the change it justified pre-authorises the next raise"
    );

    let parsed = RatchetConfig::parse(&out).expect("the rewritten file re-parses");
    assert_eq!(parsed.metric["u"].baseline, 97);
    assert_eq!(parsed.metric["u"].justification, None);
}

/// INV-2102-2 at the file level: the pass can never raise, whatever it measures.
#[test]
fn lowering_never_raises_a_baseline() {
    let mut metrics = BTreeMap::new();
    metrics.insert("u".to_string(), baseline(100, None));
    let mut measured = Measurements::new();
    measured.insert("u".to_string(), Measurement::Value(100_000));
    assert!(
        rewrite::lowered_baselines(&metrics, &measured).is_empty(),
        "a measurement above the baseline is a FAIL, never a new baseline"
    );

    // And an unmeasured metric is never lowered either: baking a failure into
    // the baseline would convert it into the new truth.
    metrics.insert("v".to_string(), baseline(100, None));
    assert!(!rewrite::lowered_baselines(&metrics, &measured).contains_key("v"));
}

/// A metric named for lowering whose `baseline =` line cannot be found must be
/// an error, not a silent no-op that reports success.
#[test]
fn lowering_a_metric_with_no_baseline_line_is_an_error() {
    let mut want = BTreeMap::new();
    want.insert("missing".to_string(), 1i64);
    let err = rewrite::apply(SAMPLE, &want).unwrap_err();
    assert!(err.contains("no `baseline =` line"), "{err}");
}

// ── this repository's own ratchet ───────────────────────────────────────────

/// The live assertion. Every command in the committed `.pmat-ratchet.toml`
/// runs here and must still be within its baseline.
///
/// Deliberately a `--lib` test and not only a comply rule: the enforcement
/// ledger CB-2100 generates records that no required status check currently
/// reaches the CB roster, so a rule living only in `pmat comply check` gates
/// nothing today. `cargo test --lib` is reached.
#[test]
fn the_committed_ratchet_holds_at_head() {
    let report = run(repo_root()).expect("the repository's own ratchet file must load");
    if report.outcome != Outcome::Ok {
        let mut lines = report.holes.clone();
        lines.extend(report.unjustified_raises.clone());
        lines.extend(
            report
                .metrics
                .iter()
                .filter(|m| m.outcome == Outcome::Fail)
                .map(|m| format!("{}: {}", m.metric, m.detail)),
        );
        panic!("{RATCHET_FILE} is red at HEAD:\n  {}", lines.join("\n  "));
    }
}

/// Each metric's `command` is the whole contract of its baseline, so a command
/// that no longer runs is a baseline that has already rotted — even while the
/// number still looks perfectly plausible.
#[test]
fn every_committed_metric_command_still_measures() {
    let cfg = RatchetConfig::load(repo_root()).expect("the ratchet file parses");
    assert!(!cfg.metric.is_empty(), "an empty ratchet is not a gate");
    for (id, m) in &cfg.metric {
        match measure::measure(repo_root(), &m.command) {
            Measurement::Value(_) => {}
            Measurement::Unavailable(why) => {
                panic!(
                    "metric `{id}` no longer measures: {why} (command: {})",
                    m.command
                )
            }
        }
    }
}

/// The scope predicate is the load-bearing decision, and the file must pin it
/// unambiguously: the same "unwrap count" for this tree has been quoted as 570,
/// 11,002, 20,326 and 20,378 by people who each meant a different predicate.
/// Two metrics differing only in scope must therefore differ in command too.
#[test]
fn no_two_metrics_share_a_command() {
    let cfg = RatchetConfig::load(repo_root()).expect("the ratchet file parses");
    let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
    for (id, m) in &cfg.metric {
        if let Some(other) = seen.insert(m.command.as_str(), id.as_str()) {
            panic!("metrics `{other}` and `{id}` share a command, so one of them is a copy");
        }
    }
}

/// The baselines must be the values the tree actually has, not values it has
/// already beaten — otherwise the ratchet has slack in it and a regression can
/// hide inside the gap. Deliberately computed, never applied: a test that
/// edits a tracked file is a test that changes the thing it is measuring.
#[test]
fn the_committed_baselines_have_no_slack_left_in_them() {
    let cfg = RatchetConfig::load(repo_root()).expect("the ratchet file parses");
    let measurements = measure::measure_all(repo_root(), &cfg.metric);
    let slack = rewrite::lowered_baselines(&cfg.metric, &measurements);
    assert!(
        slack.is_empty(),
        "the tree has improved since capture and the baselines still carry the old numbers \
         ({slack:?}) — run `pmat comply ratchet --lower` and commit the result"
    );
}

// ── the contract's own honesty ──────────────────────────────────────────────

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
fn has_a_kani_runner() -> bool {
    let root = repo_root();
    let mut candidates: Vec<PathBuf> = vec![root.join("Makefile")];
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

/// A named harness must exist. `#[cfg(kani)]` code is never compiled by `cargo
/// test`, so a rename would silently orphan the contract's claim and nothing
/// would notice.
#[test]
fn every_kani_harness_the_contract_names_exists() {
    let contract = contract_text();
    let names = named_harnesses(&contract);
    assert!(
        !names.is_empty(),
        "the contract must still name its kernel harnesses"
    );
    let source =
        std::fs::read_to_string(repo_root().join("src/services/metrics_ratchet/kernel.rs"))
            .expect("the kernel is committed");
    assert!(
        source.contains("#[kani::proof]"),
        "the harnesses must be real `#[kani::proof]` functions"
    );
    for name in &names {
        assert!(
            source.contains(&format!("fn {name}()")),
            "the contract names kani harness `{name}`, which \
             src/services/metrics_ratchet/kernel.rs does not define"
        );
    }
}

/// `kind: kernel` plus a `kani_harness:` per equation reads as "these
/// predicates are proved". They are not: with kani 0.67.0 installed, `cargo
/// kani` cannot start on this crate. The contract must say so in its own text.
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

/// And the claim must match the repository, in both directions: the honest
/// note becomes a dishonest one the day somebody wires kani up and leaves it.
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
            "the contract claims its kani harnesses are discharged, but nothing in this \
             repository runs kani"
        );
    } else {
        assert!(
            !has_a_kani_runner(),
            "this repository now runs kani; the contract must stop saying its harnesses are \
             not discharged"
        );
    }
}

/// A registered rule with no declared severity is a rule that cannot fail.
/// `get_severity` answers `Warning` for an id nobody declares, and
/// `should_fail(Warning, strict = false)` is `false` — so an unconfigured
/// CB-2102 would be reported, counted, and completely inert on a default `pmat
/// comply check`. It would also sit outside the severity=error roster CB-2100
/// checks reachability for, so nothing would notice.
#[test]
fn cb_2102_is_declared_error_not_left_at_the_warning_default() {
    let cfg = crate::models::comply_config::ComplyConfig::default();
    assert_eq!(
        cfg.get_severity("cb-2102"),
        crate::models::comply_config::CheckSeverity::Error,
        "an unconfigured rule defaults to Warning, which does not fail a default comply run"
    );
    assert!(cfg.is_check_enabled("cb-2102"));
    assert!(
        cfg.should_fail(cfg.get_severity("cb-2102"), false),
        "CB-2102 must fail a non-strict `pmat comply check`, or it is a report, not a gate"
    );
}

/// The defect this whole change closes. A rule `.pmat.yaml` cannot address is
/// a rule nobody can configure, one CB-2100's enforcement ledger cannot list,
/// and — as this module was for an entire release — an engine with no caller.
#[test]
fn cb_2102_is_in_the_comply_rule_registry() {
    let ids =
        crate::cli::handlers::comply_handlers::check_evidence_gates::enumerate_comply_rule_ids(
            repo_root(),
        )
        .expect("the registry enumerates");
    assert!(
        ids.contains("cb-2102"),
        "cb-2102 is not in the comply rule registry, so it cannot appear in the enforcement \
         ledger and .pmat.yaml cannot address it"
    );
}

// ── the silent zero the pipefail guard does not catch ───────────────────────

/// The defect CB-2101's F-3 falsifier found in CB-2102's measurement layer.
///
/// `measure` guards the shape it documents at length — `<producer> | wc -l`
/// where the producer *fails* — by running under `bash -o pipefail` and
/// rejecting exit codes outside {0, 1}. It does not, and cannot, guard the
/// shape where the producer SUCCEEDS over an empty input set. A rotted
/// pathspec and a genuine zero are byte-identical at the shell:
///
/// ```text
/// $ git grep -oF 'TOKEN'       -- 'no/such/path/*.rs' | wc -l  ->  0, exit 1, no stderr
/// $ git grep -oF 'NOT_PRESENT' -- 'src/*.rs'          | wc -l  ->  0, exit 1, no stderr
/// ```
///
/// `TOKEN` stands in for the real pattern deliberately: writing the literal
/// this repository actually ratchets would move the number this guard protects,
/// because the metric counts occurrences in prose about itself. The ratchet
/// caught exactly that on the commit that introduced this test.
///
/// So the exit code cannot tell them apart and the guard has to be the
/// baseline. Measured against the shipped binary before this test existed: a
/// metric whose pathspec was edited to `no/such/path/*.rs` reported
/// `FIRING  measured 0 count against limit 100` and the audit exited 0 — and
/// the ratchet, which only ever looks upward, read `0 <= 20390` as a Pass. Both
/// gates greeted a broken predicate as the best day in the project's history.
#[test]
fn a_zero_against_a_nonzero_baseline_is_a_hole_not_the_best_day_in_project_history() {
    let mut m = baseline(20_390, None);
    // A `git grep -oF <pattern> -- <pathspec> | wc -l` whose PATHSPEC has rotted.
    // The pattern is deliberately not one this repository ratchets: what is under
    // test is the empty input set, not the token, and using a real ratcheted
    // literal here would move the metric this test defends.
    m.command = "git grep -oF 'TOKEN' -- 'no/such/path/*.rs' | wc -l".into();
    let got = measure::measure_metric(repo_root(), &m);
    let Measurement::Unavailable(why) = &got else {
        unreachable!("a rotted pathspec produced {got:?}, which was accepted against 20390")
    };
    assert!(
        why.contains('0') && why.contains("20390"),
        "the message must carry both numbers, got: {why}"
    );
}

/// The control, in three parts. Without it the guard above is satisfied by
/// rejecting every zero, which would mean a metric could never reach the one
/// number every ratchet is trying to get to.
#[test]
fn zero_is_still_reachable_when_it_is_declared_or_already_the_baseline() {
    // 1. A metric already AT zero measures zero and is fine.
    let mut at_zero = baseline(0, None);
    at_zero.command = "printf '0\\n'".into();
    assert_eq!(
        measure::measure_metric(repo_root(), &at_zero),
        Measurement::Value(0)
    );

    // 2. A metric that declares zero reachable measures zero and is fine. This
    //    is the deliberate, auditable override: one word in the committed file,
    //    reviewed like any other change, not a flag on the command line.
    let mut declared = baseline(20_390, None);
    declared.command = "printf '0\\n'".into();
    declared.zero_is_reachable = true;
    assert_eq!(
        measure::measure_metric(repo_root(), &declared),
        Measurement::Value(0)
    );

    // 3. A non-zero measurement is untouched by the guard whatever it is.
    let mut ordinary = baseline(20_390, None);
    ordinary.command = "printf '7\\n'".into();
    assert_eq!(
        measure::measure_metric(repo_root(), &ordinary),
        Measurement::Value(7)
    );
}
