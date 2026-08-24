//! CB-2104 — the vacuity guard and the self-test fixture.
//!
//! These tests exist because of one measured fact about the researched design:
//! its report for *"I analysed 12,693 numbers and found nothing"* was
//! byte-identical to *"`git ls-files` returned nothing and I analysed
//! nothing."* Everything below is an attempt to make those two outcomes
//! impossible to confuse.

use std::path::{Path, PathBuf};

use super::census::{self, Vacuity};
use super::cohort::CohortConfig;
use super::corpus;
use super::{Census, CorpusFile, RuleId, Status};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// A census that would pass every plausibility rule, as a base for mutation.
fn plausible() -> Census {
    Census {
        files_scanned: 1_200,
        files_tracked: 3_252,
        r1_framed_numerals: 7_295,
        r2_mentions: 3_883,
        ..Census::default()
    }
}

// ---------------------------------------------------------------------------
// The self-test fixture — the control the researched designs lacked
// ---------------------------------------------------------------------------

/// RED without the self-test runner.
///
/// Four planted defects, one per rule family, must come back on every
/// invocation. A rule that has silently stopped firing and a repository that is
/// genuinely clean produce the same empty output; only a corpus that MUST fire
/// separates them.
#[test]
fn the_self_test_recovers_all_four_planted_defects() {
    let st = census::self_test();
    assert_eq!(
        st.planted,
        census::PLANTED.len(),
        "the fixture declares {} planted defects",
        census::PLANTED.len()
    );
    assert_eq!(
        st.recovered, st.planted,
        "self-test did not recover every planted defect; missed: {:?}",
        st.missed
    );
    assert!(st.passed, "self-test must pass: {st:?}");
    assert_eq!(
        census::PLANTED.len(),
        4,
        "one planted defect per rule family"
    );

    let rules: Vec<RuleId> = census::PLANTED.iter().map(|p| p.rule).collect();
    for want in [RuleId::R1, RuleId::C1, RuleId::C4, RuleId::C5] {
        assert!(
            rules.contains(&want),
            "{want:?} has no planted defect, so its silence proves nothing"
        );
    }
}

/// RED without the innocent half of the fixture being scanned at all.
///
/// The 26 innocent numbers and 10 correct derivations travel in the same corpus
/// as the planted defects, so a rule that got louder shows up here rather than
/// on a user's repository.
#[test]
fn the_self_test_flags_none_of_the_innocent_numbers() {
    let st = census::self_test();
    assert!(
        st.false_positives.is_empty(),
        "innocent numbers were flagged: {:?}",
        st.false_positives
    );
    assert_eq!(
        st.innocent_items,
        census::INNOCENT_ITEMS,
        "the innocent half must be counted, or 0/36 means nothing"
    );

    // The innocent half is not quiet because it is small: every one of the 26
    // classes is replicated across eight files with a 6/2 value split, which is
    // exactly the shape R1 hunts.
    let innocent = census::innocent_corpus();
    assert!(
        innocent.len() >= 32,
        "the innocent half must be replicated to be a control, got {} files",
        innocent.len()
    );
}

/// The control on the control.
///
/// A self-test that cannot fail is decoration. Remove one planted defect from
/// the corpus and the runner must report it missing — otherwise "4/4 recovered"
/// is a constant, not a measurement.
#[test]
fn removing_a_planted_defect_fails_the_self_test() {
    let full = census::fixture_corpus();
    let baseline = census::self_test_over(&full);
    assert!(baseline.passed, "control: the intact fixture must pass");

    for planted in census::PLANTED {
        let ablated: Vec<CorpusFile> = full
            .iter()
            .filter(|f| !f.path.starts_with(planted.file_prefix))
            .cloned()
            .collect();
        assert!(
            ablated.len() < full.len(),
            "ablation removed nothing for {:?} — the prefix {:?} matches no fixture file",
            planted.rule,
            planted.file_prefix
        );
        let st = census::self_test_over(&ablated);
        assert!(
            !st.passed,
            "deleting the {:?} defect left the self-test green, so it does not test {:?}",
            planted.rule, planted.rule
        );
    }
}

/// A planted number in the fixture must never reach a user's report.
///
/// `tests/fixtures/` is excluded from the real corpus, so the four defects this
/// check plants in its own repository are invisible to it. If that exclusion
/// rots, pmat starts reporting its own test data.
#[test]
fn the_fixture_is_excluded_from_the_real_corpus() {
    for f in census::fixture_corpus() {
        let on_disk = format!("{}/{}", census::FIXTURE_ROOT, f.path);
        assert_eq!(
            corpus::path_exclusion(&on_disk),
            Some(corpus::Exclusion::FixtureTree),
            "{on_disk} would be scanned as if it were a claim about this repository"
        );
    }
}

// ---------------------------------------------------------------------------
// R-13  the vacuity guard
// ---------------------------------------------------------------------------

/// RED without the vacuity guard.
///
/// A genuinely empty repository is UNMEASURABLE, not clean: there is nothing to
/// say, and saying "no contradictions" would be a claim the run did not earn.
#[test]
fn r13_unmeasurable_corpus_exits_two() {
    let cfg = CohortConfig::default();

    let not_a_repo = tempfile::tempdir().expect("tempdir");
    std::fs::write(not_a_repo.path().join("a.md"), "There are 70 crates.\n").expect("write");
    let report = census::run(not_a_repo.path(), &cfg);
    assert_eq!(
        report.status,
        Status::Unmeasurable,
        "a non-git directory must not report a clean tree"
    );
    assert_eq!(report.exit_code(), 2);
    assert!(
        report.findings.is_empty(),
        "an unmeasurable run must print no findings"
    );

    let empty = tempfile::tempdir().expect("tempdir");
    let init = std::process::Command::new("git")
        .arg("init")
        .arg("-q")
        .arg(empty.path())
        .status()
        .expect("git init");
    assert!(init.success(), "git init failed");
    let report = census::run(empty.path(), &cfg);
    assert_eq!(
        report.status,
        Status::Unmeasurable,
        "an empty git repository has nothing to say, which is not the same as being clean"
    );
    assert_eq!(report.exit_code(), 2);
}

/// RED without the plausibility rules.
///
/// Stated in `.pmat-ratchet.toml`'s idiom: a metric that measures 0 against a
/// baseline above 0 is UNMEASURABLE, not passed.
#[test]
fn plausibility_separates_nothing_to_say_from_nothing_measured() {
    assert!(
        census::plausibility(&plausible()).is_none(),
        "a healthy census must be measured"
    );

    let empty = Census {
        files_scanned: 0,
        ..plausible()
    };
    assert_eq!(census::plausibility(&empty), Some(Vacuity::EmptyCorpus));

    let no_framed = Census {
        r1_framed_numerals: 0,
        ..plausible()
    };
    assert_eq!(
        census::plausibility(&no_framed),
        Some(Vacuity::NoFramedNumerals),
        "1,200 files and not one framed numeral means the extractor rotted"
    );

    let no_mentions = Census {
        r2_mentions: 0,
        ..plausible()
    };
    assert_eq!(
        census::plausibility(&no_mentions),
        Some(Vacuity::NoMentions)
    );

    // Counter-control: a genuinely small corpus is not a rotted one. Three
    // files with no framed numerals is a plausible three files.
    let tiny = Census {
        files_scanned: 3,
        r1_framed_numerals: 0,
        r2_mentions: 0,
        ..plausible()
    };
    assert!(
        census::plausibility(&tiny).is_none(),
        "the guard must not fire on a corpus that is merely small"
    );
}

// ---------------------------------------------------------------------------
// R-12  the user's hard constraint
// ---------------------------------------------------------------------------

/// Findings exit 0. Always. This check warns; it never blocks.
#[test]
fn r12_exit_code_is_zero_with_findings() {
    let report = census::run(&repo_root(), &CohortConfig::default());
    assert_eq!(
        report.status,
        Status::Ok,
        "this repository must be measurable: {:?}",
        report.warnings
    );
    assert_eq!(
        report.exit_code(),
        0,
        "a WARN check exits 0 whether or not it found something"
    );
    assert_eq!(report.severity, "warn");
    assert_eq!(report.check, "CB-2104");

    // And the guarantee stated over a report that definitely carries findings,
    // so this test cannot pass merely because the tree happens to be clean.
    let mut with_findings = report;
    with_findings.findings = census::self_test_over(&census::fixture_corpus()).findings;
    assert!(
        !with_findings.findings.is_empty(),
        "the fixture must produce findings, or the next assertion is vacuous"
    );
    assert_eq!(
        with_findings.exit_code(),
        0,
        "findings must never block: exit 0"
    );
}

// ---------------------------------------------------------------------------
// R-14 / R-15  the census
// ---------------------------------------------------------------------------

/// RED without the census.
///
/// Every path emits a census, and on a real tree its counters are non-zero. A
/// "clean" result that does not carry `files_scanned`, `r1_framed_numerals`,
/// `r2_mentions` and the suppression counters is a bug, not a pass.
#[test]
fn r14_census_is_always_emitted() {
    let report = census::run(&repo_root(), &CohortConfig::default());
    let c = &report.census;
    assert!(c.files_scanned > 20, "files_scanned {}", c.files_scanned);
    assert!(
        c.files_tracked >= c.files_scanned,
        "tracked {} < scanned {}",
        c.files_tracked,
        c.files_scanned
    );
    assert!(
        c.r1_files_scanned > 0 && c.r1_files_scanned <= c.files_scanned,
        "R1 reads a subset of the corpus: {} of {}",
        c.r1_files_scanned,
        c.files_scanned
    );
    assert!(c.r1_framed_numerals > 0, "R1 framed nothing");
    assert!(c.r2_mentions > 0, "R2 extracted nothing");
    assert!(
        c.raw_numeric_literals > c.r1_framed_numerals + c.r2_mentions,
        "the coverage denominator must exceed what the rules read: {} vs {} + {}",
        c.raw_numeric_literals,
        c.r1_framed_numerals,
        c.r2_mentions
    );
}

/// RED without the suppression counters.
///
/// A guard that can hide a finding must say how often it did. G1 is exercised
/// by a generated copy of the planted R1 template; the derivation guard by the
/// ten correct derivations.
#[test]
fn r15_suppression_counters_are_reported() {
    let mut files = census::fixture_corpus();
    files.push(CorpusFile::new(
        "planted/crate-h/README.md",
        "// Auto-generated — DO NOT EDIT.\n\nPart of the Sample monorepo — 99 workspace crates.\n",
    ));
    let (_, c, _) = census::run_corpus(files, &CohortConfig::default());
    assert!(
        c.suppressed_generated >= 1,
        "G1 suppressed nothing over a corpus containing a generated file"
    );
    assert!(
        c.suppressed_derivation >= 1,
        "the derivation guard suppressed nothing over the ten derivations"
    );
}

// ---------------------------------------------------------------------------
// R-16  the one knob that destroys the check
// ---------------------------------------------------------------------------

/// `--min-sites 3` measured 1/10 precision on the reference corpus, so lowering
/// it must never be silent.
#[test]
fn r16_min_sites_below_default_warns() {
    let cfg = CohortConfig {
        min_sites: 3,
        ..CohortConfig::default()
    };
    let (_, _, warnings) = census::run_corpus(census::fixture_corpus(), &cfg);
    assert!(
        warnings.iter().any(|w| w.contains("--min-sites 3")),
        "lowering the floor must warn: {warnings:?}"
    );

    let (_, _, quiet) = census::run_corpus(census::fixture_corpus(), &CohortConfig::default());
    assert!(
        !quiet.iter().any(|w| w.contains("--min-sites")),
        "the default floor must not warn: {quiet:?}"
    );
}

// ---------------------------------------------------------------------------
// The corpus split: R2 reads JSON, R1 does not
// ---------------------------------------------------------------------------

/// JSON is interchange, not authored prose. R2 reads it because a config key is
/// a config key whatever the syntax; R1 does not, because including it pulled
/// 3.3M numerals out of 3,318 machine-written `contract.json` files.
#[test]
fn r2_reads_json_and_r1_does_not() {
    assert!(
        corpus::R2_PATHSPECS.contains(&"*.json"),
        "R2's corpus must include JSON"
    );
    assert!(
        !corpus::R1_PATHSPECS.contains(&"*.json"),
        "R1's corpus must not include JSON"
    );
    for spec in corpus::R1_PATHSPECS {
        assert!(
            corpus::R2_PATHSPECS.contains(spec),
            "R2 must read everything R1 reads, missing {spec}"
        );
    }
    assert_eq!(
        corpus::R2_PATHSPECS.len(),
        corpus::R1_PATHSPECS.len() + 1,
        "R2's corpus is R1's plus JSON, nothing else"
    );

    let files = vec![
        CorpusFile::new("a.json", "{\n  \"max_size\": 4096\n}\n"),
        CorpusFile::new("a.md", "There are 70 crates in the workspace.\n"),
    ];
    let (_, c, _) = census::run_corpus(files, &CohortConfig::default());
    assert_eq!(c.files_scanned, 2, "R2 sees both files");
    assert_eq!(c.r1_files_scanned, 1, "R1 sees only the markdown");
}

// ---------------------------------------------------------------------------
// The measured baseline — spec section 3
// ---------------------------------------------------------------------------

/// What CB-2104 says about the repository it ships in.
///
/// The researched baseline is **one** finding on pmat: C5 on
/// `src/tests/binary_size.rs:40` against `.pmat-metrics.toml`. This test pins
/// the shape rather than a headline number that would flap on every edit: the
/// run is measurable, the census is populated, and no finding names a file
/// under `tests/fixtures/`.
#[test]
fn the_live_tree_is_measured_and_carries_no_fixture_findings() {
    let report = census::run(&repo_root(), &CohortConfig::default());
    assert_eq!(report.status, Status::Ok);
    assert!(report.self_test.passed, "{:?}", report.self_test);

    for f in &report.findings {
        for s in &f.sites {
            assert!(
                !s.file.contains("tests/fixtures/"),
                "a planted fixture number reached the real report: {}:{}",
                s.file,
                s.line
            );
        }
    }

    // Printed rather than asserted: the count is the measurement, and pinning
    // it here would make every legitimate fix to the tree fail this test. The
    // researched baseline is 1 (C5, binary_size.rs) at 583ea9ac2.
    println!(
        "pmat: {} finding(s), {} files scanned, {} mentions, {} framed",
        report.findings.len(),
        report.census.files_scanned,
        report.census.r2_mentions,
        report.census.r1_framed_numerals
    );
    for f in &report.findings {
        println!(
            "  [{}] {} — {}",
            f.rule.as_str(),
            f.quantity,
            f.sites
                .iter()
                .map(|s| format!("{}:{}", s.file, s.line))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

/// CB-2104 must be addressable from `.pmat.yaml`, and must not be an Error.
///
/// An id nobody declares resolves to `Warning` by default, which reads the same
/// as this — but only by accident, and `.pmat.yaml` could then never turn the
/// rule off. Declaring it `Error` would be worse: `should_fail(Error, _)` is
/// true, so an advisory check would start failing `pmat comply check`, and it
/// would join the severity=error roster CB-2100 verifies reachability for while
/// being reachable from no gate at all.
#[test]
fn cb_2104_is_registered_as_a_warning_and_can_be_disabled() {
    use crate::models::comply_config::{CheckSeverity, ComplyConfig};

    let checks = ComplyConfig::default().checks;
    let entry = checks
        .get("cb-2104")
        .expect("cb-2104 must be in the comply rule registry, or .pmat.yaml cannot address it");
    assert!(entry.enabled, "the rule ships on");
    assert_eq!(
        entry.severity,
        CheckSeverity::Warning,
        "an advisory check declared Error would block a comply run"
    );

    // The registration is load-bearing rather than decorative: the handler
    // reads it, so this entry is what `.pmat.yaml` switches.
    let mut off = ComplyConfig::default();
    off.checks
        .entry("cb-2104".to_string())
        .and_modify(|c| c.enabled = false);
    assert!(
        !off.is_check_enabled("cb-2104"),
        "disabling the rule in .pmat.yaml must actually disable it"
    );
}

/// What the JSON half of R2's corpus is actually worth, printed not inferred.
///
/// The spec gives R2 `*.json` and R1 nothing. Whether that costs or buys
/// anything on a real tree is a measurement, so it is taken rather than
/// asserted: the numbers print, and only the direction is pinned.
#[test]
fn the_json_half_of_the_corpus_is_measured_not_assumed() {
    let root = repo_root();
    let (all, _) = corpus::collect(&root, corpus::R2_PATHSPECS).expect("collect R2 corpus");
    let (no_json, _) = corpus::collect(&root, corpus::R1_PATHSPECS).expect("collect R1 corpus");
    let (_, with, _) = census::run_corpus(all, &CohortConfig::default());
    let (_, without, _) = census::run_corpus(no_json, &CohortConfig::default());
    println!(
        "JSON adds {} files and {} R2 mentions ({} -> {}); R1 reads {} of {}",
        with.files_scanned - without.files_scanned,
        with.r2_mentions as i64 - without.r2_mentions as i64,
        without.r2_mentions,
        with.r2_mentions,
        with.r1_files_scanned,
        with.files_scanned
    );
    assert!(
        with.files_scanned >= without.files_scanned,
        "R2's corpus is a superset of R1's"
    );
    assert_eq!(
        with.r1_files_scanned, without.files_scanned,
        "R1's half of the joint scan must equal a JSON-free scan, or the partition is wrong"
    );
}

// ---------------------------------------------------------------------------
// The contract must describe this code, not a plan
// ---------------------------------------------------------------------------

fn contract_text() -> String {
    std::fs::read_to_string(repo_root().join("contracts/comply-numeric-claims-v1.yaml"))
        .expect("contracts/comply-numeric-claims-v1.yaml is committed")
}

/// Every falsifier the contract names must be a test that exists.
///
/// A contract naming a test that was renamed out from under it is a claim
/// nothing can check, and it reads exactly like a claim something does — the
/// defect class CB-2100 exists to find, guarded here the way
/// `comply-ratchet-v1.yaml` guards its own kani harness names.
#[test]
fn every_falsifier_the_contract_names_exists() {
    let contract = contract_text();
    let mut checked = 0usize;
    for line in contract.lines() {
        let Some(rest) = line.trim().strip_prefix("test: ") else {
            continue;
        };
        let Some((file, name)) = rest.split_once("::") else {
            unreachable!("falsifier {rest:?} must be written as <file>::<test name>");
        };
        let path = repo_root().join(file);
        let Ok(source) = std::fs::read_to_string(&path) else {
            unreachable!("the contract names {file}, which is not readable");
        };
        assert!(
            source.contains(&format!("fn {name}()")),
            "the contract names falsifier `{name}`, which {file} does not define"
        );
        checked += 1;
    }
    assert!(
        checked >= 18,
        "the contract must still carry its falsifiers, found {checked}"
    );
}

/// Every source file the contract points at must exist.
#[test]
fn every_path_the_contract_references_exists() {
    let contract = contract_text();
    let mut in_refs = false;
    let mut checked = 0usize;
    for line in contract.lines() {
        if line.trim_end() == "  references:" {
            in_refs = true;
            continue;
        }
        if !in_refs {
            continue;
        }
        match line.trim().strip_prefix("- ") {
            Some(p) => {
                assert!(
                    repo_root().join(p).exists(),
                    "the contract references {p}, which does not exist"
                );
                checked += 1;
            }
            None => in_refs = false,
        }
    }
    assert!(
        checked >= 10,
        "the contract must still name the module it describes, found {checked}"
    );
}

/// The contract must not call this rule enforced.
///
/// Nothing runs CB-2104 inside `pmat comply check`, no CI job gates on it, and
/// it is registered at severity Warning. A contract that said `enforced` would
/// be a decorative claim about enforcement inside a repository whose flagship
/// rule is about decorative claims.
#[test]
fn the_contract_does_not_claim_to_be_enforced() {
    let contract = contract_text();
    assert!(
        contract.contains("status: advisory"),
        "the contract must declare itself advisory"
    );
    assert!(
        !contract.contains("status: enforced"),
        "CB-2104 is run by no gate; declaring it enforced would be a claim nothing backs"
    );
}

/// The exclusion counters must come from the corpus pass, not from zero.
///
/// This repository tracks a `CHANGELOG` and a `tests/fixtures/` tree, so both
/// counters are non-zero here; a zero would mean the exclusion list stopped
/// being applied, which is how a fixture's planted numbers reach a real report.
#[test]
fn the_exclusion_counters_are_filled_from_the_corpus_pass() {
    let c = census::run(&repo_root(), &CohortConfig::default()).census;
    assert!(
        c.excluded_fixture_tree > 0,
        "this repository has a tests/fixtures/ tree; the counter must see it"
    );
    assert!(
        c.excluded_changelog > 0,
        "this repository tracks a CHANGELOG; the counter must see it"
    );
    println!(
        "excluded: machine-managed {} fixture-tree {} changelog {} unreadable {}",
        c.excluded_machine_managed, c.excluded_fixture_tree, c.excluded_changelog, c.unreadable
    );
}
