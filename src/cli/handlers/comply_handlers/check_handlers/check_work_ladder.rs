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
//   CB-1613 (L3) — L3+ falsification.log entries must all be status=pass
//   CB-1614 (L4) — L4+ ticket completion requires `kani-report.json` with
//                  success=true (skip-if-absent until Component 24 runner)
//   CB-1616 (L5) — L5 ticket completion requires `lean-proof.json` with
//                  sorry_count=0 (skip-if-absent until Component 24 Lean)
//   CB-1617 (L3) — downgrade without `--reason` forbidden (ledger audit)
//   CB-1618 (L1) — level monotonicity across ticket checkpoints — a ticket
//                  cannot drop without an audited downgrade ledger entry
//                  (skip-if-absent until checkpoint writer records level)
//   CB-1619 (L3) — on completion, achieved level == target level
//
// Deferred (scaffolded with Skip + reason, infrastructure pending):
//
//   CB-1612 cargo-test evidence     → needs verify pipeline
//   CB-1615 Kani harness SHA        → needs harness hash index

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

// ─── CB-1613: L3 falsification evidence ──────────────────────────────────────

/// CB-1613 (L3): L3+ completion requires `.pmat-work/<ID>/falsification.log`
/// present, and every entry in that log must carry `status: "pass"`. Any
/// `fail`, `timeout`, or malformed status gate the ticket from claiming L3.
///
/// Skip semantics (tiered):
///   • no tickets at all                         → Skip
///   • no L3+ ticket on any active contract      → Skip
///   • L3+ tickets exist but none have a log yet → Skip (in-progress tickets
///                                                  haven't run falsification)
///   • any L3+ log has a non-pass entry OR is
///     malformed                                 → Fail
pub(crate) fn check_ladder_l3_falsification(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1613: L3 Falsification Evidence";
    let contracts = load_active_contracts(project_path);
    if contracts.is_empty() {
        return skip_no_contracts(name);
    }

    let l3_plus: Vec<&WorkContract> = contracts.iter().filter(|c| is_l3_or_higher(c)).collect();
    if l3_plus.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No L3+ ticket present".into(),
            severity: Severity::Info,
        };
    }

    let mut any_log_present = false;
    let mut checked = 0usize;
    let mut failing: Vec<String> = Vec::new();

    for c in &l3_plus {
        let log = project_path
            .join(".pmat-work")
            .join(&c.work_item_id)
            .join("falsification.log");
        if !log.exists() {
            continue; // in-progress ticket — per-ticket skip
        }
        any_log_present = true;
        checked += 1;

        let Ok(contents) = std::fs::read_to_string(&log) else {
            failing.push(format!("  {} (unreadable log)", c.work_item_id));
            continue;
        };

        for (idx, raw) in contents.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() {
                continue;
            }
            let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
                failing.push(format!(
                    "  {} line {} (malformed JSON)",
                    c.work_item_id,
                    idx + 1
                ));
                continue;
            };
            let status = v
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("<missing>");
            if status != "pass" {
                let label = v
                    .get("test_id")
                    .and_then(|s| s.as_str())
                    .or_else(|| v.get("method").and_then(|s| s.as_str()))
                    .unwrap_or("?");
                failing.push(format!(
                    "  {} entry '{}' status={}",
                    c.work_item_id, label, status
                ));
            }
        }
    }

    if !failing.is_empty() {
        let mut msg = format!("{} L3+ log entry/entries not passing:\n", failing.len());
        for line in &failing {
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

    if !any_log_present {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: format!(
                "No L3+ ticket has a `falsification.log` yet ({} eligible)",
                l3_plus.len()
            ),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!("{} L3+ log(s) checked, all entries pass", checked),
        severity: Severity::Info,
    }
}

/// Ticket strings are typed like `"L3"` or `"L4 (kani_proof)"` — take the
/// first whitespace-separated token so annotated variants parse too.
fn is_l3_or_higher(contract: &WorkContract) -> bool {
    let token = contract
        .verification_level
        .split_whitespace()
        .next()
        .unwrap_or("");
    VerificationLevel::parse_lenient(token)
        .map(|lvl| lvl >= VerificationLevel::L3)
        .unwrap_or(false)
}

// ─── CB-1614: L4 Kani evidence ──────────────────────────────────────────────

/// CB-1614 (L4): every L4+ ticket must have `.pmat-work/<ID>/kani-report.json`
/// present, and that report must carry `success: true`. When Component 24
/// Kani runner lands, this check enforces that L4 claims have Kani backing.
///
/// Report schema (minimum): `{ "success": bool }` — extra fields ignored.
///
/// Skip semantics (tiered):
///   • no tickets at all                          → Skip
///   • no L4+ ticket on any active contract       → Skip
///   • L4+ tickets exist but none have a report   → Skip (runner not yet
///                                                   wired; in-progress
///                                                   tickets don't falsify)
///   • any L4+ report missing `success` key,
///     reports `success: false`, or is malformed  → Fail
pub(crate) fn check_ladder_l4_kani(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1614: L4 Kani Evidence";
    let contracts = load_active_contracts(project_path);
    if contracts.is_empty() {
        return skip_no_contracts(name);
    }

    let l4_plus: Vec<&WorkContract> = contracts.iter().filter(|c| is_l4_or_higher(c)).collect();
    if l4_plus.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No L4+ ticket present".into(),
            severity: Severity::Info,
        };
    }

    let mut any_report = false;
    let mut checked = 0usize;
    let mut failing: Vec<String> = Vec::new();

    for c in &l4_plus {
        let report = project_path
            .join(".pmat-work")
            .join(&c.work_item_id)
            .join("kani-report.json");
        if !report.exists() {
            continue; // in-progress L4 ticket
        }
        any_report = true;
        checked += 1;

        let Ok(contents) = std::fs::read_to_string(&report) else {
            failing.push(format!(
                "  {} (unreadable kani-report.json)",
                c.work_item_id
            ));
            continue;
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&contents) else {
            failing.push(format!("  {} (malformed kani-report.json)", c.work_item_id));
            continue;
        };
        match v.get("success").and_then(|s| s.as_bool()) {
            Some(true) => {}
            Some(false) => failing.push(format!("  {} success=false", c.work_item_id)),
            None => failing.push(format!(
                "  {} (kani-report.json missing `success` field)",
                c.work_item_id
            )),
        }
    }

    if !failing.is_empty() {
        let mut msg = format!("{} L4+ ticket(s) failed Kani evidence:\n", failing.len());
        for line in &failing {
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

    if !any_report {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: format!(
                "No L4+ ticket has a `kani-report.json` yet ({} eligible)",
                l4_plus.len()
            ),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!("{} L4+ Kani report(s) pass", checked),
        severity: Severity::Info,
    }
}

/// Same whitespace-token shape as `is_l3_or_higher` — handles annotated
/// levels like `"L4 (kani_proof)"`.
fn is_l4_or_higher(contract: &WorkContract) -> bool {
    let token = contract
        .verification_level
        .split_whitespace()
        .next()
        .unwrap_or("");
    VerificationLevel::parse_lenient(token)
        .map(|lvl| lvl >= VerificationLevel::L4)
        .unwrap_or(false)
}

/// CB-1615 (L4): Kani harness hash in ticket == harness hash in YAML. Requires
/// harness-hash index to detect post-bind drift.
pub(crate) fn check_ladder_kani_harness_sha(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1615: Kani Harness SHA",
        "requires per-harness hash recorded at bind time",
    )
}

// ─── CB-1616: L5 Lean proof zero-sorry ──────────────────────────────────────

/// CB-1616 (L5): every L5 ticket must have `.pmat-work/<ID>/lean-proof.json`
/// present, and that report must carry `sorry_count: 0`. A Lean proof with
/// any admitted `sorry` is not a proof — it's a placeholder.
///
/// Report schema (minimum): `{ "sorry_count": non-negative-integer }`.
///
/// Skip semantics (tiered):
///   • no tickets at all                          → Skip
///   • no L5 ticket on any active contract        → Skip
///   • L5 tickets exist but none have a report    → Skip (Component 24
///                                                   Lean consumer pending)
///   • any L5 report missing `sorry_count`, has
///     non-zero count, negative count, or is
///     malformed                                  → Fail
pub(crate) fn check_ladder_l5_lean(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1616: L5 Lean Proof Zero-Sorry";
    let contracts = load_active_contracts(project_path);
    if contracts.is_empty() {
        return skip_no_contracts(name);
    }

    let l5: Vec<&WorkContract> = contracts.iter().filter(|c| is_l5(c)).collect();
    if l5.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No L5 ticket present".into(),
            severity: Severity::Info,
        };
    }

    let mut any_report = false;
    let mut checked = 0usize;
    let mut failing: Vec<String> = Vec::new();

    for c in &l5 {
        let report = project_path
            .join(".pmat-work")
            .join(&c.work_item_id)
            .join("lean-proof.json");
        if !report.exists() {
            continue;
        }
        any_report = true;
        checked += 1;

        let Ok(contents) = std::fs::read_to_string(&report) else {
            failing.push(format!("  {} (unreadable lean-proof.json)", c.work_item_id));
            continue;
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&contents) else {
            failing.push(format!("  {} (malformed lean-proof.json)", c.work_item_id));
            continue;
        };
        match v.get("sorry_count").and_then(|s| s.as_i64()) {
            Some(0) => {}
            Some(n) if n > 0 => {
                failing.push(format!("  {} sorry_count={}", c.work_item_id, n));
            }
            Some(n) => {
                failing.push(format!(
                    "  {} sorry_count={} (must be non-negative)",
                    c.work_item_id, n
                ));
            }
            None => failing.push(format!(
                "  {} (lean-proof.json missing `sorry_count` integer)",
                c.work_item_id
            )),
        }
    }

    if !failing.is_empty() {
        let mut msg = format!("{} L5 ticket(s) failed Lean evidence:\n", failing.len());
        for line in &failing {
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

    if !any_report {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: format!(
                "No L5 ticket has a `lean-proof.json` yet ({} eligible)",
                l5.len()
            ),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!("{} L5 Lean proof(s) discharge with zero sorry", checked),
        severity: Severity::Info,
    }
}

/// L5 is a single point on the ladder — exact match, not `>= L5`.
fn is_l5(contract: &WorkContract) -> bool {
    let token = contract
        .verification_level
        .split_whitespace()
        .next()
        .unwrap_or("");
    VerificationLevel::parse_lenient(token) == Some(VerificationLevel::L5)
}

/// CB-1618 (L1): level monotonicity across ticket checkpoints — a ticket
/// cannot drop from L3 → L1 → L3 without an audit. Scans
/// `.pmat-work/<ID>/checkpoints/*.json` for the optional `verification_level`
/// field. Any regression in the per-ticket checkpoint timeline must be
/// matched by an entry in `.pmat-work/ledger/downgrades.json` (the CB-1617
/// audit surface — presence alone is enough here; CB-1617 validates the
/// `reason` content).
///
/// Skip semantics (tiered):
///   • no `.pmat-work/` directory                      → Skip
///   • no ticket has a non-empty `checkpoints/` dir    → Skip
///   • checkpoints exist but none carry
///     `verification_level` yet (writer pending)       → Skip
///   • per-ticket: fewer than 2 leveled checkpoints    → ignored (not enough
///                                                        history to judge)
///   • regression found AND ticket missing from
///     downgrade ledger                                → Fail
pub(crate) fn check_ladder_monotonicity(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1618: Level Monotonicity";
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/` directory".into(),
            severity: Severity::Info,
        };
    }

    let tickets = collect_ticket_checkpoints(&work_dir);
    if tickets.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ticket has a non-empty `checkpoints/` directory".into(),
            severity: Severity::Info,
        };
    }

    let audited = load_audited_tickets(project_path);
    let mut checked_tickets = 0usize;
    let mut unaudited_regressions: Vec<String> = Vec::new();

    for (ticket_id, cp_files) in &tickets {
        let leveled = load_leveled_timeline(cp_files);
        if leveled.len() < 2 {
            continue; // per-ticket skip: insufficient history
        }
        checked_tickets += 1;

        if has_regression(&leveled) && !audited.contains(ticket_id) {
            unaudited_regressions.push(format!(
                "  {}: checkpoint level regresses without a downgrade ledger entry ({})",
                ticket_id,
                summarize_timeline(&leveled),
            ));
        }
    }

    if !unaudited_regressions.is_empty() {
        let mut msg = format!(
            "{} ticket(s) regressed without audit:\n",
            unaudited_regressions.len()
        );
        for line in &unaudited_regressions {
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

    if checked_tickets == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No checkpoint records the `verification_level` field yet".into(),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!("{} ticket(s) monotonic across checkpoints", checked_tickets),
        severity: Severity::Info,
    }
}

/// Scan `.pmat-work/<ID>/checkpoints/*.json`, ignoring the `ledger` and
/// hidden directories. Returns `(ticket_id, sorted_checkpoint_paths)` pairs
/// for tickets with at least one JSON file. Path sort is by filename, which
/// — given the `checkpoint-<uuid>.json` shape — does not encode time. The
/// caller sorts further by the `timestamp` field when loading.
fn collect_ticket_checkpoints(work_dir: &Path) -> Vec<(String, Vec<std::path::PathBuf>)> {
    let Ok(entries) = std::fs::read_dir(work_dir) else {
        return Vec::new();
    };
    let mut out: Vec<(String, Vec<std::path::PathBuf>)> = Vec::new();
    for e in entries.flatten() {
        if !e.path().is_dir() {
            continue;
        }
        let Some(id) = e.file_name().to_str().map(String::from) else {
            continue;
        };
        if id.starts_with('.') || id == "ledger" {
            continue;
        }
        let cp_dir = e.path().join("checkpoints");
        if !cp_dir.exists() {
            continue;
        }
        let Ok(files) = std::fs::read_dir(&cp_dir) else {
            continue;
        };
        let mut cp_files: Vec<std::path::PathBuf> = files
            .flatten()
            .filter(|f| f.path().extension().and_then(|s| s.to_str()) == Some("json"))
            .map(|f| f.path())
            .collect();
        if cp_files.is_empty() {
            continue;
        }
        cp_files.sort();
        out.push((id, cp_files));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Load each checkpoint, keep only those carrying `verification_level`,
/// sort by `timestamp` (stable). Unparseable files are silently dropped.
fn load_leveled_timeline(cp_files: &[std::path::PathBuf]) -> Vec<(String, VerificationLevel)> {
    let mut rows: Vec<(String, VerificationLevel)> = Vec::new();
    for path in cp_files {
        let Ok(body) = std::fs::read_to_string(path) else {
            continue;
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) else {
            continue;
        };
        let ts = v
            .get("timestamp")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string();
        let Some(lvl) = v
            .get("verification_level")
            .and_then(|s| s.as_str())
            .and_then(VerificationLevel::parse_lenient)
        else {
            continue;
        };
        rows.push((ts, lvl));
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows
}

fn has_regression(timeline: &[(String, VerificationLevel)]) -> bool {
    timeline.windows(2).any(|pair| pair[1].1 < pair[0].1)
}

fn summarize_timeline(timeline: &[(String, VerificationLevel)]) -> String {
    let levels: Vec<&str> = timeline.iter().map(|(_, l)| l.as_str()).collect();
    levels.join(" → ")
}

/// Set of `ticket` ids appearing at least once in the downgrade ledger.
/// Absent/malformed ledger returns an empty set (CB-1617 reports that).
fn load_audited_tickets(project_path: &Path) -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    let ledger = project_path
        .join(".pmat-work")
        .join("ledger")
        .join("downgrades.json");
    let Ok(content) = std::fs::read_to_string(&ledger) else {
        return out;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return out;
    };
    let Some(array) = value.as_array() else {
        return out;
    };
    for e in array {
        if let Some(t) = e.get("ticket").and_then(|v| v.as_str()) {
            out.insert(t.to_string());
        }
    }
    out
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
            check_ladder_kani_harness_sha(tmp.path()),
        ] {
            assert_eq!(r.status, CheckStatus::Skip);
            assert!(r.message.starts_with("Deferred"));
        }
    }

    // ─── CB-1618: level monotonicity across checkpoints ──────────────────────

    fn write_checkpoint(project: &Path, id: &str, filename: &str, body: &str) {
        let dir = project.join(".pmat-work").join(id).join("checkpoints");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(filename), body).unwrap();
    }

    fn write_downgrade_ledger(project: &Path, body: &str) {
        let dir = project.join(".pmat-work").join("ledger");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("downgrades.json"), body).unwrap();
    }

    fn checkpoint_body(ts: &str, level: Option<&str>) -> String {
        match level {
            Some(l) => format!(
                "{{\"timestamp\": \"{}\", \"verification_level\": \"{}\"}}",
                ts, l
            ),
            None => format!("{{\"timestamp\": \"{}\"}}", ts),
        }
    }

    #[test]
    fn monotonicity_skips_without_pmat_work() {
        let tmp = tempdir().unwrap();
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/`"));
    }

    #[test]
    fn monotonicity_skips_with_no_checkpoint_dirs() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r
            .message
            .contains("No ticket has a non-empty `checkpoints/`"));
    }

    #[test]
    fn monotonicity_skips_when_checkpoints_lack_level_field() {
        let tmp = tempdir().unwrap();
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-a.json",
            &checkpoint_body("2026-04-01T10:00:00Z", None),
        );
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-b.json",
            &checkpoint_body("2026-04-01T11:00:00Z", None),
        );
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("verification_level"));
    }

    #[test]
    fn monotonicity_ignores_ticket_with_one_leveled_checkpoint() {
        // A single leveled checkpoint can't demonstrate regression — ignored.
        let tmp = tempdir().unwrap();
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-a.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L3")),
        );
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }

    #[test]
    fn monotonicity_passes_on_ascending_levels() {
        let tmp = tempdir().unwrap();
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-a.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L1")),
        );
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-b.json",
            &checkpoint_body("2026-04-01T11:00:00Z", Some("L3")),
        );
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-c.json",
            &checkpoint_body("2026-04-01T12:00:00Z", Some("L4")),
        );
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("1 ticket(s) monotonic"));
    }

    #[test]
    fn monotonicity_passes_on_flat_levels() {
        let tmp = tempdir().unwrap();
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-a.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L3")),
        );
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-b.json",
            &checkpoint_body("2026-04-01T11:00:00Z", Some("L3")),
        );
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass);
    }

    #[test]
    fn monotonicity_fails_on_regression_without_ledger() {
        let tmp = tempdir().unwrap();
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-a.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L3")),
        );
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-b.json",
            &checkpoint_body("2026-04-01T11:00:00Z", Some("L1")),
        );
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("T-1"));
        assert!(r.message.contains("L3 → L1"));
    }

    #[test]
    fn monotonicity_passes_on_regression_with_ledger() {
        let tmp = tempdir().unwrap();
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-a.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L4")),
        );
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-b.json",
            &checkpoint_body("2026-04-01T11:00:00Z", Some("L3")),
        );
        write_downgrade_ledger(
            tmp.path(),
            r#"[{"ticket":"T-1","reason":"kani runner offline for review cycle"}]"#,
        );
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn monotonicity_aggregates_across_tickets() {
        // T-1 monotonic, T-2 regresses without ledger → Fail names only T-2.
        let tmp = tempdir().unwrap();
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-a.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L2")),
        );
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-b.json",
            &checkpoint_body("2026-04-01T11:00:00Z", Some("L3")),
        );
        write_checkpoint(
            tmp.path(),
            "T-2",
            "checkpoint-a.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L4")),
        );
        write_checkpoint(
            tmp.path(),
            "T-2",
            "checkpoint-b.json",
            &checkpoint_body("2026-04-01T11:00:00Z", Some("L2")),
        );
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-2"));
        assert!(!r.message.contains("T-1:"));
    }

    #[test]
    fn monotonicity_sorts_by_timestamp_not_filename() {
        // Filename `z.json` is created first chronologically; filename order
        // would read it second and flag a bogus regression. Verify the check
        // sorts by the `timestamp` field.
        let tmp = tempdir().unwrap();
        write_checkpoint(
            tmp.path(),
            "T-1",
            "z-first.json",
            &checkpoint_body("2026-04-01T09:00:00Z", Some("L1")),
        );
        write_checkpoint(
            tmp.path(),
            "T-1",
            "a-second.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L3")),
        );
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn monotonicity_skips_hidden_and_ledger_dirs() {
        let tmp = tempdir().unwrap();
        // Hidden dir and `ledger` dir must be ignored, even if they happen
        // to contain `checkpoints/`.
        write_checkpoint(
            tmp.path(),
            ".hidden",
            "checkpoint-a.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L5")),
        );
        write_checkpoint(
            tmp.path(),
            ".hidden",
            "checkpoint-b.json",
            &checkpoint_body("2026-04-01T11:00:00Z", Some("L1")),
        );
        write_checkpoint(
            tmp.path(),
            "ledger",
            "checkpoint-a.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L5")),
        );
        write_checkpoint(
            tmp.path(),
            "ledger",
            "checkpoint-b.json",
            &checkpoint_body("2026-04-01T11:00:00Z", Some("L1")),
        );
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No ticket"));
    }

    #[test]
    fn monotonicity_ignores_malformed_checkpoint_json() {
        let tmp = tempdir().unwrap();
        write_checkpoint(tmp.path(), "T-1", "checkpoint-a.json", "not-a-json-file");
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-b.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L3")),
        );
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-c.json",
            &checkpoint_body("2026-04-01T11:00:00Z", Some("L4")),
        );
        // Malformed row is dropped; the remaining two rows ascend → Pass.
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn monotonicity_empty_ledger_does_not_audit() {
        let tmp = tempdir().unwrap();
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-a.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L4")),
        );
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-b.json",
            &checkpoint_body("2026-04-01T11:00:00Z", Some("L2")),
        );
        write_downgrade_ledger(tmp.path(), "[]");
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
    }

    // ─── CB-1613: L3 falsification evidence ──────────────────────────────────

    fn write_falsification_log(project: &Path, id: &str, body: &str) {
        let dir = project.join(".pmat-work").join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("falsification.log"), body).unwrap();
    }

    #[test]
    fn l3_falsification_skips_with_no_tickets() {
        let tmp = tempdir().unwrap();
        let r = check_ladder_l3_falsification(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/*/contract.json`"));
    }

    #[test]
    fn l3_falsification_skips_with_only_l1_tickets() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L1").save(tmp.path()).unwrap();
        make_contract("T-2", "L2").save(tmp.path()).unwrap();
        let r = check_ladder_l3_falsification(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No L3+"));
    }

    #[test]
    fn l3_falsification_skips_when_no_log_yet() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        make_contract("T-2", "L4 (kani_proof)")
            .save(tmp.path())
            .unwrap();
        let r = check_ladder_l3_falsification(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No L3+ ticket has a"));
        assert!(r.message.contains("2 eligible"));
    }

    #[test]
    fn l3_falsification_passes_when_all_entries_pass() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        write_falsification_log(
            tmp.path(),
            "T-1",
            concat!(
                r#"{"yaml":"k.yaml","test_id":"t1","status":"pass","duration_ms":5}"#,
                "\n",
                r#"{"method":"rope","status":"pass","duration_ms":2}"#,
                "\n",
            ),
        );
        let r = check_ladder_l3_falsification(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn l3_falsification_fails_on_failing_entry() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        write_falsification_log(
            tmp.path(),
            "T-1",
            r#"{"yaml":"k.yaml","test_id":"t1","status":"fail","duration_ms":5}"#,
        );
        let r = check_ladder_l3_falsification(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
        assert!(r.message.contains("status=fail"));
    }

    #[test]
    fn l3_falsification_fails_on_timeout() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L4").save(tmp.path()).unwrap();
        write_falsification_log(
            tmp.path(),
            "T-1",
            r#"{"yaml":"k.yaml","test_id":"t1","status":"timeout","duration_ms":30000}"#,
        );
        let r = check_ladder_l3_falsification(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("status=timeout"));
    }

    #[test]
    fn l3_falsification_fails_on_malformed_json() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        write_falsification_log(tmp.path(), "T-1", "not-json\n");
        let r = check_ladder_l3_falsification(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("malformed JSON"));
    }

    #[test]
    fn l3_falsification_ignores_below_l3() {
        let tmp = tempdir().unwrap();
        // L2 ticket with a failing log — must NOT fail the check
        make_contract("T-1", "L2").save(tmp.path()).unwrap();
        write_falsification_log(
            tmp.path(),
            "T-1",
            r#"{"yaml":"k.yaml","test_id":"t1","status":"fail"}"#,
        );
        let r = check_ladder_l3_falsification(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No L3+"));
    }

    #[test]
    fn l3_falsification_per_ticket_skip_when_some_have_no_log() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        make_contract("T-2", "L3").save(tmp.path()).unwrap();
        // Only T-1 has a log; T-2 is in-progress
        write_falsification_log(
            tmp.path(),
            "T-1",
            r#"{"yaml":"k.yaml","test_id":"t1","status":"pass","duration_ms":1}"#,
        );
        let r = check_ladder_l3_falsification(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("1 L3+ log"));
    }

    #[test]
    fn is_l3_or_higher_accepts_ladder() {
        for (s, want) in [
            ("L0", false),
            ("L1", false),
            ("L2", false),
            ("L3", true),
            ("L4", true),
            ("L4 (kani_proof)", true),
            ("L5", true),
            ("strong", false),
        ] {
            let mut c = WorkContract::new("T".into(), "deadbeef".into());
            c.verification_level = s.to_string();
            assert_eq!(is_l3_or_higher(&c), want, "for '{}'", s);
        }
    }

    // ─── CB-1614: L4 Kani evidence ───────────────────────────────────────────

    fn write_kani_report(project: &Path, id: &str, body: &str) {
        let dir = project.join(".pmat-work").join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("kani-report.json"), body).unwrap();
    }

    #[test]
    fn l4_kani_skips_with_no_tickets() {
        let tmp = tempdir().unwrap();
        let r = check_ladder_l4_kani(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/*/contract.json`"));
    }

    #[test]
    fn l4_kani_skips_without_l4_ticket() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        let r = check_ladder_l4_kani(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No L4+"));
    }

    #[test]
    fn l4_kani_skips_when_no_report_yet() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L4").save(tmp.path()).unwrap();
        make_contract("T-2", "L5").save(tmp.path()).unwrap();
        let r = check_ladder_l4_kani(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("2 eligible"));
    }

    #[test]
    fn l4_kani_passes_on_success_report() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L4").save(tmp.path()).unwrap();
        write_kani_report(
            tmp.path(),
            "T-1",
            r#"{"success":true,"harnesses":[{"name":"h","status":"pass"}]}"#,
        );
        let r = check_ladder_l4_kani(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn l4_kani_fails_on_failure_report() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L4").save(tmp.path()).unwrap();
        write_kani_report(tmp.path(), "T-1", r#"{"success":false}"#);
        let r = check_ladder_l4_kani(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
        assert!(r.message.contains("success=false"));
    }

    #[test]
    fn l4_kani_fails_on_malformed_report() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L4").save(tmp.path()).unwrap();
        write_kani_report(tmp.path(), "T-1", "not-json");
        let r = check_ladder_l4_kani(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("malformed"));
    }

    #[test]
    fn l4_kani_fails_when_success_missing() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L4").save(tmp.path()).unwrap();
        write_kani_report(tmp.path(), "T-1", r#"{"harnesses":[]}"#);
        let r = check_ladder_l4_kani(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("missing `success`"));
    }

    #[test]
    fn l4_kani_ignores_below_l4() {
        let tmp = tempdir().unwrap();
        // L3 ticket with a failing report — must NOT fail this check
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        write_kani_report(tmp.path(), "T-1", r#"{"success":false}"#);
        let r = check_ladder_l4_kani(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No L4+"));
    }

    #[test]
    fn l4_kani_accepts_annotated_level() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L4 (kani_proof)")
            .save(tmp.path())
            .unwrap();
        write_kani_report(tmp.path(), "T-1", r#"{"success":true}"#);
        let r = check_ladder_l4_kani(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn is_l4_or_higher_accepts_ladder() {
        for (s, want) in [
            ("L3", false),
            ("L4", true),
            ("L4 (kani_proof)", true),
            ("L5", true),
            ("bogus", false),
        ] {
            let mut c = WorkContract::new("T".into(), "deadbeef".into());
            c.verification_level = s.to_string();
            assert_eq!(is_l4_or_higher(&c), want, "for '{}'", s);
        }
    }

    // ─── CB-1616: L5 Lean proof zero-sorry ───────────────────────────────────

    fn write_lean_proof(project: &Path, id: &str, body: &str) {
        let dir = project.join(".pmat-work").join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("lean-proof.json"), body).unwrap();
    }

    #[test]
    fn l5_lean_skips_with_no_tickets() {
        let tmp = tempdir().unwrap();
        let r = check_ladder_l5_lean(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/*/contract.json`"));
    }

    #[test]
    fn l5_lean_skips_without_l5_ticket() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L4").save(tmp.path()).unwrap();
        let r = check_ladder_l5_lean(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No L5"));
    }

    #[test]
    fn l5_lean_skips_when_no_report_yet() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L5").save(tmp.path()).unwrap();
        let r = check_ladder_l5_lean(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("1 eligible"));
    }

    #[test]
    fn l5_lean_passes_on_zero_sorry() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L5").save(tmp.path()).unwrap();
        write_lean_proof(
            tmp.path(),
            "T-1",
            r#"{"sorry_count":0,"theorems":[{"name":"rope_correct","status":"proved"}]}"#,
        );
        let r = check_ladder_l5_lean(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn l5_lean_fails_on_nonzero_sorry() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L5").save(tmp.path()).unwrap();
        write_lean_proof(tmp.path(), "T-1", r#"{"sorry_count":3}"#);
        let r = check_ladder_l5_lean(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
        assert!(r.message.contains("sorry_count=3"));
    }

    #[test]
    fn l5_lean_fails_on_negative_sorry() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L5").save(tmp.path()).unwrap();
        write_lean_proof(tmp.path(), "T-1", r#"{"sorry_count":-1}"#);
        let r = check_ladder_l5_lean(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("non-negative"));
    }

    #[test]
    fn l5_lean_fails_on_malformed_report() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L5").save(tmp.path()).unwrap();
        write_lean_proof(tmp.path(), "T-1", "not-json");
        let r = check_ladder_l5_lean(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("malformed"));
    }

    #[test]
    fn l5_lean_fails_when_sorry_count_missing() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L5").save(tmp.path()).unwrap();
        write_lean_proof(tmp.path(), "T-1", r#"{"theorems":[]}"#);
        let r = check_ladder_l5_lean(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("missing `sorry_count`"));
    }

    #[test]
    fn l5_lean_ignores_below_l5() {
        let tmp = tempdir().unwrap();
        // L4 with failing lean proof — must NOT fail this check
        make_contract("T-1", "L4").save(tmp.path()).unwrap();
        write_lean_proof(tmp.path(), "T-1", r#"{"sorry_count":5}"#);
        let r = check_ladder_l5_lean(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No L5"));
    }

    #[test]
    fn is_l5_is_exact_match() {
        for (s, want) in [
            ("L3", false),
            ("L4", false),
            ("L4 (kani_proof)", false),
            ("L5", true),
            ("L5 (lean)", true),
            ("bogus", false),
        ] {
            let mut c = WorkContract::new("T".into(), "deadbeef".into());
            c.verification_level = s.to_string();
            assert_eq!(is_l5(&c), want, "for '{}'", s);
        }
    }
}
