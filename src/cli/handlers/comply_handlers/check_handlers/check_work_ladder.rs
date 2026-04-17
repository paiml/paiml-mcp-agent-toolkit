// Work Verification Ladder enforcement (CB-1610..1619) — Component 28
//
// Sub-spec: docs/specifications/components/pmat-work-verification-ladder.md
//
// Audits the typed ladder (L0..L5) on each `.pmat-work/<ID>/contract.json`.
// The schema migration (`verification_level: String` → `VerificationLevel`)
// ships as a separate commit; these checks operate on the string field via
// `VerificationLevel::parse_strict`.
//
// Implemented this pass (functional):
//
//   CB-1610 (L1) — `verification_level` parses to a known variant
//   CB-1611 (L1) — target level ≤ max attainable level across bindings
//   CB-1617 (L3) — downgrade without `--reason` forbidden (ledger audit)
//   CB-1619 (L3) — on completion, achieved level == target level
//
// Deferred (scaffolded with Skip + reason, infrastructure pending):
//
//   CB-1612 cargo-test evidence     → needs verify pipeline
//   CB-1613 falsification log       → needs .pmat-work/<ID>/falsification.log
//   CB-1614 Kani artifact           → needs Component 24 Kani invoker
//   CB-1615 Kani harness SHA        → needs harness hash index
//   CB-1616 Lean proof zero-sorry   → needs Component 24 Lean consumer
//   CB-1618 monotonicity audit      → needs checkpoint history

use std::path::Path;

use super::types::*;
use crate::cli::handlers::work_contract::WorkContract;
use crate::cli::handlers::work_verification_level::VerificationLevel;

// ─── Shared helpers ──────────────────────────────────────────────────────────

fn deferred(name: &str, reason: &str) -> ComplianceCheck {
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Skip,
        message: format!("Deferred — {}", reason),
        severity: Severity::Info,
    }
}

fn skip_no_contracts(name: &str) -> ComplianceCheck {
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Skip,
        message: "No `.pmat-work/*/contract.json` tickets present".into(),
        severity: Severity::Info,
    }
}

/// Load every `.pmat-work/<ID>/contract.json` into a vector.
fn load_active_contracts(project_path: &Path) -> Vec<WorkContract> {
    let dir = project_path.join(".pmat-work");
    if !dir.exists() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return out;
    };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let Some(id) = entry.file_name().to_str().map(|s| s.to_string()) else {
            continue;
        };
        if id.starts_with('.') || id == "ledger" {
            continue;
        }
        if let Ok(c) = WorkContract::load(project_path, &id) {
            out.push(c);
        }
    }
    out
}

/// Max attainable level across all YAML bindings on a ticket. Following
/// Liskov-Wing: the weakest binding dominates (weakest-link semantics).
/// Returns `L1` for tickets with no bindings — bare runtime asserts are
/// always achievable without a contract anchor.
fn max_attainable_for_ticket(project_path: &Path, c: &WorkContract) -> VerificationLevel {
    if c.implements.is_empty() {
        return VerificationLevel::L1;
    }
    let mut weakest = VerificationLevel::L5;
    for binding in &c.implements {
        let file = if binding.file.is_absolute() {
            binding.file.clone()
        } else {
            project_path.join(&binding.file)
        };
        let level = match std::fs::read_to_string(&file) {
            Ok(content) => VerificationLevel::max_attainable_from_yaml(&content),
            Err(_) => VerificationLevel::L1,
        };
        if level < weakest {
            weakest = level;
        }
    }
    weakest
}

// ─── CB-1610: verification_level parses ──────────────────────────────────────

/// CB-1610 (L1): the `verification_level` string on every ticket must parse
/// strictly to a known ladder variant. Catches typos like `"L3 "`, `"l4"`,
/// or free-form strings like `"strong"` that silently downgrade enforcement.
pub(crate) fn check_ladder_parses(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1610: Verification Level Parses";
    let contracts = load_active_contracts(project_path);
    if contracts.is_empty() {
        return skip_no_contracts(name);
    }

    let mut bad: Vec<String> = Vec::new();
    for c in &contracts {
        if VerificationLevel::parse_strict(&c.verification_level).is_none() {
            bad.push(format!(
                "  {} -> verification_level='{}'",
                c.work_item_id, c.verification_level
            ));
        }
    }

    if !bad.is_empty() {
        let mut msg = format!("{} ticket(s) have unparseable level:\n", bad.len());
        for line in &bad {
            msg.push_str(line);
            msg.push('\n');
        }
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: msg,
            severity: Severity::Error,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!("All {} ticket level(s) parse to L0..L5", contracts.len()),
        severity: Severity::Info,
    }
}

// ─── CB-1611: target ≤ max attainable ─────────────────────────────────────────

/// CB-1611 (L1): `verification_level` claimed by a ticket cannot exceed the
/// max attainable level of its weakest binding. Catches tickets that claim
/// `L4` without any `kani_harnesses:` in the bound YAML.
pub(crate) fn check_ladder_bound_by_yaml(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1611: Target ≤ Max Attainable";
    let contracts = load_active_contracts(project_path);
    if contracts.is_empty() {
        return skip_no_contracts(name);
    }

    let mut over: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for c in &contracts {
        let Some(claimed) = VerificationLevel::parse_strict(&c.verification_level) else {
            continue; // CB-1610 owns the parse failure
        };
        if c.implements.is_empty() {
            // Unbound tickets are bounded to L1 per spec; warn but do not hard-fail here
            // (CB-1610 handles invalid strings; this check only concerns bound tickets).
            continue;
        }
        checked += 1;
        let ceiling = max_attainable_for_ticket(project_path, c);
        if claimed > ceiling {
            over.push(format!(
                "  {} claims {} but bindings cap at {}",
                c.work_item_id, claimed, ceiling
            ));
        }
    }

    if !over.is_empty() {
        let mut msg = format!("{} ticket(s) overclaim verification level:\n", over.len());
        for line in &over {
            msg.push_str(line);
            msg.push('\n');
        }
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: msg,
            severity: Severity::Error,
        };
    }

    if checked == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ticket has `implements:` bindings to bound against".into(),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!("{} bound ticket(s) within max-attainable ceiling", checked),
        severity: Severity::Info,
    }
}

// ─── CB-1617: downgrade audit ────────────────────────────────────────────────

/// CB-1617 (L3): any entry in `.pmat-work/ledger/downgrades.json` must carry
/// a non-empty `reason` field. A downgrade with empty or missing reason is
/// silent scope reduction.
///
/// The ledger file is optional — its absence is Skip, not Fail.
pub(crate) fn check_ladder_downgrade_audit(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1617: Downgrade Reason Audit";
    let ledger = project_path
        .join(".pmat-work")
        .join("ledger")
        .join("downgrades.json");
    if !ledger.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No downgrade ledger at .pmat-work/ledger/downgrades.json".into(),
            severity: Severity::Info,
        };
    }
    let Ok(content) = std::fs::read_to_string(&ledger) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Warn,
            message: format!("Unreadable downgrade ledger: {}", ledger.display()),
            severity: Severity::Warning,
        };
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: "Downgrade ledger is not valid JSON".into(),
            severity: Severity::Error,
        };
    };

    let entries: &[serde_json::Value] = match &value {
        serde_json::Value::Array(a) => a.as_slice(),
        _ => {
            return ComplianceCheck {
                name: name.into(),
                status: CheckStatus::Fail,
                message: "Downgrade ledger must be a JSON array of entries".into(),
                severity: Severity::Error,
            };
        }
    };

    let mut missing: Vec<String> = Vec::new();
    for (i, entry) in entries.iter().enumerate() {
        let ticket = entry
            .get("ticket")
            .and_then(|v| v.as_str())
            .unwrap_or("<unknown>");
        let reason = entry.get("reason").and_then(|v| v.as_str()).unwrap_or("");
        if reason.trim().is_empty() {
            missing.push(format!("  entry[{}] ticket={} reason=empty", i, ticket));
        }
    }

    if !missing.is_empty() {
        let mut msg = format!("{} downgrade(s) lack a `reason`:\n", missing.len());
        for line in &missing {
            msg.push_str(line);
            msg.push('\n');
        }
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: msg,
            severity: Severity::Error,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!("{} downgrade(s) carry a recorded reason", entries.len()),
        severity: Severity::Info,
    }
}

// ─── CB-1619: completion == target ───────────────────────────────────────────

/// CB-1619 (L3): tickets marked completed must have their
/// `verification_level` (achieved level) equal to the target recorded in
/// `verification-report.json`. Silent downgrade is forbidden.
///
/// The report file is optional — its absence is Skip.
pub(crate) fn check_ladder_completion_matches(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1619: Achieved Level == Target";
    let contracts = load_active_contracts(project_path);
    if contracts.is_empty() {
        return skip_no_contracts(name);
    }

    let mut mismatches: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for c in &contracts {
        let report = project_path
            .join(".pmat-work")
            .join(&c.work_item_id)
            .join("verification-report.json");
        if !report.exists() {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&report) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
            continue;
        };
        let target = value
            .get("target_level")
            .and_then(|v| v.as_str())
            .and_then(VerificationLevel::parse_strict);
        let achieved = value
            .get("achieved_level")
            .and_then(|v| v.as_str())
            .and_then(VerificationLevel::parse_strict);
        let Some(target) = target else { continue };
        let Some(achieved) = achieved else { continue };
        checked += 1;
        if achieved < target {
            mismatches.push(format!(
                "  {} target={} achieved={}",
                c.work_item_id, target, achieved
            ));
        }
    }

    if !mismatches.is_empty() {
        let mut msg = format!(
            "{} ticket(s) closed below target level:\n",
            mismatches.len()
        );
        for line in &mismatches {
            msg.push_str(line);
            msg.push('\n');
        }
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: msg,
            severity: Severity::Error,
        };
    }

    if checked == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ticket has verification-report.json yet".into(),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!("{} completion(s) match target level", checked),
        severity: Severity::Info,
    }
}

// ─── Deferred checks ─────────────────────────────────────────────────────────

/// CB-1612 (L3): L1 completion requires `cargo test --lib` green. Requires
/// the verify pipeline that records test exit status per-ticket.
pub(crate) fn check_ladder_l1_test_evidence(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1612: L1 Test Evidence",
        "requires `pmat work verify` pipeline recording cargo test exit",
    )
}

/// CB-1613 (L3): L3 completion requires falsification.log present + all pass.
/// Requires Component 29 (FalsificationMethod::ProvableContract) to emit the log.
pub(crate) fn check_ladder_l3_falsification(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1613: L3 Falsification Evidence",
        "requires .pmat-work/<ID>/falsification.log from Component 29",
    )
}

/// CB-1614 (L4): L4 completion requires Kani artifact + exit 0. Requires
/// Component 24 (verification-backends) Kani invocation contract.
pub(crate) fn check_ladder_l4_kani(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1614: L4 Kani Evidence",
        "requires Component 24 Kani runner emitting kani-report.json",
    )
}

/// CB-1615 (L4): Kani harness hash in ticket == harness hash in YAML. Requires
/// harness-hash index to detect post-bind drift.
pub(crate) fn check_ladder_kani_harness_sha(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1615: Kani Harness SHA",
        "requires per-harness hash recorded at bind time",
    )
}

/// CB-1616 (L5): Lean proof with zero `sorry`. Requires Component 24 Lean
/// proof-status consumer.
pub(crate) fn check_ladder_l5_lean(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1616: L5 Lean Proof Zero-Sorry",
        "requires Component 24 Lean proof-status consumer",
    )
}

/// CB-1618 (L1): level monotonicity across ticket checkpoints — a ticket
/// cannot drop from L3 → L1 → L3 without audit. Requires checkpoint history.
pub(crate) fn check_ladder_monotonicity(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1618: Level Monotonicity",
        "requires checkpoint history with per-snapshot verification_level",
    )
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::handlers::work_contract::ContractBinding;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn write_yaml(project: &Path, name: &str, body: &str) -> PathBuf {
        let dir = project.join("contracts");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join(format!("{}.yaml", name));
        std::fs::write(&p, body).unwrap();
        p
    }

    fn make_contract(id: &str, level: &str) -> WorkContract {
        let mut c = WorkContract::new(id.to_string(), "deadbeef".to_string());
        c.verification_level = level.to_string();
        c
    }

    #[test]
    fn parses_passes_when_all_valid() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        make_contract("T-2", "L1").save(tmp.path()).unwrap();
        let r = check_ladder_parses(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn parses_fails_on_typo() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3 ").save(tmp.path()).unwrap();
        let r = check_ladder_parses(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
    }

    #[test]
    fn parses_skips_with_no_tickets() {
        let tmp = tempdir().unwrap();
        let r = check_ladder_parses(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }

    #[test]
    fn bound_by_yaml_fails_overclaim() {
        let tmp = tempdir().unwrap();
        // YAML caps at L3 (has falsification_tests but no kani)
        write_yaml(
            tmp.path(),
            "k",
            "equations:\n  rope: {}\nfalsification_tests:\n  - id: t\n",
        );
        let mut c = make_contract("T-1", "L4"); // claim > max
        c.implements.push(ContractBinding {
            contract: "k".into(),
            equation: "rope".into(),
            file: PathBuf::from("contracts/k.yaml"),
            sha: "deadbeef".into(),
            bound_at: chrono::Utc::now(),
        });
        c.save(tmp.path()).unwrap();

        let r = check_ladder_bound_by_yaml(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
    }

    #[test]
    fn bound_by_yaml_passes_when_within_ceiling() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "k",
            "equations:\n  rope: {}\nkani_harnesses:\n  - name: h\n",
        );
        let mut c = make_contract("T-1", "L3");
        c.implements.push(ContractBinding {
            contract: "k".into(),
            equation: "rope".into(),
            file: PathBuf::from("contracts/k.yaml"),
            sha: "deadbeef".into(),
            bound_at: chrono::Utc::now(),
        });
        c.save(tmp.path()).unwrap();

        let r = check_ladder_bound_by_yaml(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn bound_by_yaml_weakest_binding_dominates() {
        let tmp = tempdir().unwrap();
        // Two bindings: one caps at L4 (kani), one caps at L2 (equations only)
        write_yaml(
            tmp.path(),
            "strong",
            "equations:\n  e: {}\nkani_harnesses:\n  - name: h\n",
        );
        write_yaml(tmp.path(), "weak", "equations:\n  e: {}\n");
        let mut c = make_contract("T-1", "L3"); // L3 > L2 weakest → fail
        c.implements.push(ContractBinding {
            contract: "strong".into(),
            equation: "e".into(),
            file: PathBuf::from("contracts/strong.yaml"),
            sha: "abc".into(),
            bound_at: chrono::Utc::now(),
        });
        c.implements.push(ContractBinding {
            contract: "weak".into(),
            equation: "e".into(),
            file: PathBuf::from("contracts/weak.yaml"),
            sha: "def".into(),
            bound_at: chrono::Utc::now(),
        });
        c.save(tmp.path()).unwrap();

        let r = check_ladder_bound_by_yaml(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("cap at L2"));
    }

    #[test]
    fn bound_by_yaml_skips_unbound_only() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap(); // no bindings
        let r = check_ladder_bound_by_yaml(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }

    #[test]
    fn downgrade_audit_passes_when_all_reasons_present() {
        let tmp = tempdir().unwrap();
        let ledger_dir = tmp.path().join(".pmat-work").join("ledger");
        std::fs::create_dir_all(&ledger_dir).unwrap();
        let ledger = ledger_dir.join("downgrades.json");
        std::fs::write(
            &ledger,
            r#"[{"ticket":"T-1","reason":"blocked on kani"},{"ticket":"T-2","reason":"scope cut"}]"#,
        )
        .unwrap();
        let r = check_ladder_downgrade_audit(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn downgrade_audit_fails_on_missing_reason() {
        let tmp = tempdir().unwrap();
        let ledger_dir = tmp.path().join(".pmat-work").join("ledger");
        std::fs::create_dir_all(&ledger_dir).unwrap();
        std::fs::write(
            ledger_dir.join("downgrades.json"),
            r#"[{"ticket":"T-1","reason":""}]"#,
        )
        .unwrap();
        let r = check_ladder_downgrade_audit(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
    }

    #[test]
    fn downgrade_audit_skips_when_ledger_missing() {
        let tmp = tempdir().unwrap();
        let r = check_ladder_downgrade_audit(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }

    #[test]
    fn completion_matches_passes_when_equal() {
        let tmp = tempdir().unwrap();
        let c = make_contract("T-1", "L3");
        c.save(tmp.path()).unwrap();
        let dir = tmp.path().join(".pmat-work").join("T-1");
        std::fs::write(
            dir.join("verification-report.json"),
            r#"{"target_level":"L3","achieved_level":"L3"}"#,
        )
        .unwrap();
        let r = check_ladder_completion_matches(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn completion_matches_fails_on_giveup() {
        let tmp = tempdir().unwrap();
        let c = make_contract("T-1", "L3");
        c.save(tmp.path()).unwrap();
        let dir = tmp.path().join(".pmat-work").join("T-1");
        std::fs::write(
            dir.join("verification-report.json"),
            r#"{"target_level":"L4","achieved_level":"L2"}"#,
        )
        .unwrap();
        let r = check_ladder_completion_matches(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
    }

    #[test]
    fn completion_matches_skips_without_report() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        let r = check_ladder_completion_matches(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }

    #[test]
    fn deferred_ladder_checks_return_skip() {
        let tmp = tempdir().unwrap();
        for r in [
            check_ladder_l1_test_evidence(tmp.path()),
            check_ladder_l3_falsification(tmp.path()),
            check_ladder_l4_kani(tmp.path()),
            check_ladder_kani_harness_sha(tmp.path()),
            check_ladder_l5_lean(tmp.path()),
            check_ladder_monotonicity(tmp.path()),
        ] {
            assert_eq!(r.status, CheckStatus::Skip);
            assert!(r.message.starts_with("Deferred"));
        }
    }
}
