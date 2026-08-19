//! CB-2101 falsification suite for the IMPURE half of threshold coherence:
//! the driver that reads this repository's own `.pmat-metrics.toml`, the comply
//! registration, and the contract's own honesty.
//!
//! `config_tests.rs` falsifies the pure evaluator over synthetic rosters. This
//! file falsifies the things a pure test cannot reach, and the reason it exists
//! is the same reason `drive_tests.rs` does: CB-2101's evaluator shipped on
//! release/3.32.0 with no caller, no contract, and no rule id anything could
//! address. A classifier nobody calls classifies nothing.

use super::config::{Outcome, METRICS_FILE};
use super::kernel::Classification;
use std::path::Path;

const CONTRACT: &str = "contracts/comply-threshold-coherence-v1.yaml";

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn contract_text() -> String {
    std::fs::read_to_string(repo_root().join(CONTRACT)).expect("the contract is committed")
}

// ── registration: a rule nothing can address is not a rule ──────────────────

/// The defect that kept CB-2101 inert for a release. A rule `.pmat.yaml` cannot
/// address is a rule nobody can configure, one CB-2100's enforcement ledger
/// cannot list, and — as this module was — an evaluator with no caller.
#[test]
fn cb_2101_is_in_the_comply_rule_registry() {
    let ids =
        crate::cli::handlers::comply_handlers::check_evidence_gates::enumerate_comply_rule_ids(
            repo_root(),
        )
        .expect("the registry enumerates");
    assert!(
        ids.contains("cb-2101"),
        "cb-2101 is not in the comply rule registry, so it cannot appear in the enforcement \
         ledger and .pmat.yaml cannot address it"
    );
}

/// A registered rule with no declared severity is a rule that cannot fail.
/// `get_severity` answers `Warning` for an id nobody declares, and
/// `should_fail(Warning, strict = false)` is `false` — so an unconfigured
/// CB-2101 would be reported, counted, and completely inert.
#[test]
fn cb_2101_is_declared_error_not_left_at_the_warning_default() {
    let cfg = crate::models::comply_config::ComplyConfig::default();
    assert_eq!(
        cfg.get_severity("cb-2101"),
        crate::models::comply_config::CheckSeverity::Error,
        "an unconfigured rule defaults to Warning, which does not fail a default comply run"
    );
    assert!(cfg.is_check_enabled("cb-2101"));
    assert!(
        cfg.should_fail(cfg.get_severity("cb-2101"), false),
        "CB-2101 must fail a non-strict `pmat comply check`, or it is a report, not a gate"
    );
}

/// The contract must be on disk. Both module headers in this directory have
/// named `comply-threshold-coherence-v1.yaml` since the evaluator was written;
/// for a whole release the file did not exist, and nothing noticed, because a
/// doc comment is not compiled.
#[test]
fn the_coherence_contract_is_committed() {
    let path = repo_root().join(CONTRACT);
    assert!(
        path.is_file(),
        "{CONTRACT} is named by src/services/metrics_ratchet/{{mod,kernel,config}}.rs and does \
         not exist"
    );
    let text = contract_text();
    assert!(
        text.contains("kind: kernel"),
        "the contract must declare metadata.kind: kernel"
    );
}

// ── the driver, against this repository's own committed config ──────────────

/// Totality (`INV-2101-3`) on real data: every threshold this repository writes
/// down in a declared threshold section comes back with exactly one of FIRING /
/// VIOLATED / VACUOUS. This is the DoD, checked against the file rather than a
/// fixture.
#[test]
fn every_committed_threshold_is_classified() {
    let report = super::run_coherence(repo_root()).expect("the coherence audit runs at HEAD");
    assert!(
        !report.thresholds.is_empty(),
        "{METRICS_FILE} produced no classified thresholds — an empty audit cannot fail and is \
         not a gate"
    );
    for t in &report.thresholds {
        assert!(
            matches!(
                t.classification,
                Classification::Firing | Classification::Violated | Classification::Vacuous
            ),
            "{} carries no classification",
            t.key
        );
    }
    assert!(
        report.undeclared_sections.is_empty(),
        "undeclared sections of {METRICS_FILE}: {:?} — each one is a threshold nobody bound",
        report.undeclared_sections
    );
}

/// Arm (a), on this repository, with no fixture in the way: whatever
/// `.pmat-metrics.toml` currently says about unwraps, the verdict must be
/// derived from a measurement and must be `Fail` exactly when the declared
/// limit is under the measured count. A threshold that is breached and reports
/// anything other than VIOLATED is the defect CB-2101 exists to catch.
#[test]
fn arm_a_max_unwrap_calls_is_judged_against_a_live_measurement() {
    let report = super::run_coherence(repo_root()).expect("the coherence audit runs at HEAD");
    let v = report
        .thresholds
        .iter()
        .find(|t| t.key == "quality_gates.max_unwrap_calls")
        .expect("quality_gates.max_unwrap_calls is classified");
    let measured = v
        .measured
        .expect("the unwrap gate must carry a live measurement, not a remembered number");
    let limit: i64 = v.configured.parse().expect("an integer limit");
    let breached = measured > limit;
    assert_eq!(
        v.classification == Classification::Violated,
        breached,
        "limit {limit} vs measured {measured}: classification {:?} disagrees with the bound",
        v.classification
    );
    if breached {
        assert_eq!(
            v.outcome,
            Outcome::Fail,
            "a breached limit on a green build must FAIL, not warn"
        );
    }
}

/// Every `enforced_by` path and every `metric` id a committed binding names
/// must exist. A binding that names a deleted file records a belief about this
/// repository, not an enforcement of it — and until the existence check was
/// added, the audit reported that belief as a fact.
#[test]
fn every_committed_binding_names_something_real() {
    let cfg = super::config::RatchetConfig::load(repo_root()).expect("the ratchet file parses");
    let mut wrong = Vec::new();
    for (key, b) in &cfg.coherence.binding {
        if let Some(p) = b.enforced_by.as_deref() {
            if !repo_root().join(p).exists() {
                wrong.push(format!("{key}: enforced_by '{p}' does not exist"));
            }
        }
        if let Some(m) = b.metric.as_deref() {
            if !cfg.metric.contains_key(m) {
                wrong.push(format!("{key}: metric '{m}' has no [metric.*] baseline"));
            }
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// An audit that classified nothing folds to `Ok` and passes every run forever.
/// This is not hypothetical: `evaluate_ratchet` shipped with exactly that bug —
/// an empty metric map folded `Outcome::Ok` — and it survived because an empty
/// gate looks identical to a passing one from the outside. The coherence audit
/// refuses the same shape rather than inheriting it.
#[test]
fn an_empty_audit_is_a_failure_not_a_clean_sheet() {
    use super::config::{
        evaluate_coherence, CoherenceConfig, EnforcerIndex, Measurements, MetricsRoster,
    };
    use std::collections::BTreeMap;

    // A roster whose only section is declared as carrying no thresholds: the
    // audit is empty for an entirely legitimate reason.
    let roster = MetricsRoster::parse("[enforcement]\nfail_on_x = true\n")
        .expect("the fixture parses");
    let cfg = CoherenceConfig {
        threshold_sections: vec!["quality_gates".into()],
        non_threshold_sections: BTreeMap::from([(
            "enforcement".to_string(),
            "switches".to_string(),
        )]),
        binding: BTreeMap::new(),
    };
    let report = evaluate_coherence(
        &roster,
        &cfg,
        &Measurements::new(),
        &BTreeMap::new(),
        &EnforcerIndex::default(),
    );
    assert!(
        report.thresholds.is_empty(),
        "this fixture is meaningless unless the audit really is empty"
    );

    // The pure evaluator reports Ok, honestly: nothing it was asked about is
    // wrong. Refusing the empty audit is the DRIVER's job, and this is the test
    // that says which layer owns it.
    assert_eq!(report.outcome, super::config::Outcome::Ok);

    let check = crate::cli::handlers::comply_handlers::check_handlers::check_threshold_coherence(
        &empty_project(),
    );
    assert_eq!(
        check.status,
        crate::cli::handlers::comply_handlers::check_handlers::types::CheckStatus::Skip,
        "a project with no ratchet file declares no bindings and is skipped, not passed"
    );
}

/// A scratch project with neither config file and no git history — the "never
/// adopted" door.
fn empty_project() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pmat-cb2101-empty-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default()
    ));
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

/// The DoD, checked on the artefact a CI job actually consumes: every threshold
/// appears as `<key>=<CLASS>` in the message CB-2101 puts into
/// `pmat comply check --format json`. A roll-up that truncates when the file
/// grows would satisfy the rule on this repository and fail it on the next one.
#[test]
fn the_json_output_classifies_every_threshold() {
    let report = super::run_coherence(repo_root()).expect("the coherence audit runs at HEAD");
    let check = crate::cli::handlers::comply_handlers::check_handlers::check_threshold_coherence(
        repo_root(),
    );
    let json = serde_json::to_string(&check).expect("the check serialises");
    for t in &report.thresholds {
        let expected = format!("{}={}", t.key, t.classification.as_str());
        assert!(
            check.message.contains(&expected),
            "the comply message does not classify {}: expected `{expected}`",
            t.key
        );
        assert!(json.contains(&t.key), "{} is absent from the JSON", t.key);
    }
}

// ── the contract's own honesty ──────────────────────────────────────────────

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

/// A named harness must exist. `#[cfg(kani)]` code is never compiled by `cargo
/// test`, so a rename would silently orphan the contract's claim and nothing at
/// all would notice.
#[test]
fn every_kani_harness_the_contract_names_exists() {
    let contract = contract_text();
    let kernel =
        std::fs::read_to_string(repo_root().join("src/services/metrics_ratchet/kernel.rs"))
            .expect("the kernel is committed");
    let named: Vec<String> = contract
        .lines()
        .filter_map(|l| l.trim().strip_prefix("kani_harness:"))
        .map(|v| v.trim().to_string())
        .collect();
    assert!(
        !named.is_empty(),
        "a kernel contract must name its harnesses"
    );
    for h in &named {
        assert!(
            kernel.contains(&format!("fn {h}(")),
            "the contract names kani harness `{h}`, which does not exist in kernel.rs"
        );
    }
}

/// `kind: kernel` plus a `kani_harness` per equation reads as a discharged
/// proof. In this repository nothing has ever run one, and with kani installed
/// nothing can. The contract has to say so in a field, not leave a reader to
/// infer a proof that never happened.
#[test]
fn the_contract_states_whether_its_kani_harnesses_discharge() {
    let contract = contract_text();
    assert!(
        contract.contains("kani_status:"),
        "a kernel contract naming kani harnesses must record whether they discharge"
    );
}

/// And the claim must match the repository, in both directions: the honest note
/// becomes a dishonest one the day somebody wires kani up and leaves it.
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

/// The meta-guard, and the one CB-2102's contract does not have: every
/// `test:` a falsifier names must be a test that exists. A contract listing a
/// falsifier nobody wrote is the CB-2103 defect class in its purest form — a
/// document asserting coverage that no compiler and no runner ever checks.
#[test]
fn every_falsifier_names_a_test_that_exists() {
    let contract = contract_text();
    let mut missing = Vec::new();
    for line in contract.lines() {
        let Some(v) = line.trim().strip_prefix("test:") else {
            continue;
        };
        let v = v.trim();
        let Some((file, name)) = v.split_once("::") else {
            missing.push(format!("{v} is not <file>::<test>"));
            continue;
        };
        let Ok(text) = std::fs::read_to_string(repo_root().join(file)) else {
            missing.push(format!("{v}: {file} does not exist"));
            continue;
        };
        if !text.contains(&format!("fn {name}(")) {
            missing.push(format!("{v}: no `fn {name}(` in {file}"));
        }
    }
    assert!(
        !missing.is_empty() || contract.contains("test:"),
        "the contract lists no falsifiers"
    );
    assert!(missing.is_empty(), "{missing:#?}");
}
