// Work Falsification Unification checks (CB-1620..1629) — Component 29
//
// Sub-spec: docs/specifications/components/pmat-work-falsification-unification.md
//
// Bridges the existing bespoke `FalsificationMethod` enum with the
// per-equation `falsification_tests[]` arrays defined in provable-contracts
// YAML. A ticket bound to a YAML equation (Component 27) automatically
// inherits `FalsificationMethod::ProvableContract{}` entries — one per YAML
// test. These checks audit that the inherited roster stays consistent with
// the YAML source and the ticket's own manual claims.
//
// Today's cut implements the checks that can run against today's infrastructure:
//
//   CB-1620 (L1) — every binding has matching `ProvableContract{}` entries
//                  per YAML `falsification_tests[]` id (WARN during migration)
//   CB-1622 (L3) — every ProvableContract roster entry has ≥1 execution line
//                  in `.pmat-work/<ID>/falsification.log` (skip-if-absent)
//   CB-1623 (L3) — no duplicate `(yaml_path, test_id)` across a ticket's roster
//   CB-1624 (L1) — manual deletion audit: any `action: "delete"` entry in
//                  `.pmat-work/ledger/roster-mutations.json` must carry
//                  `via_unbind: true` (skip-if-absent)
//   CB-1627 (L3) — post-bind YAML drift: warn when current YAML
//                  `falsification_tests[]` has IDs not seeded into the
//                  ticket's `ProvableContract{}` roster
//   CB-1625 (L3) — any inherited log line with `status != "pass"` is fatal,
//                  regardless of ticket level (skip-if-absent)
//   CB-1626 (L1) — referenced `test_id` exists in the YAML at scan time
//   CB-1628 (L3) — every inherited log line carries the required 4-field
//                  shape `{yaml, test_id, status, duration_ms}` so lines
//                  aren't silently dropped post-runner (skip-if-absent)
//   CB-1629 (L4) — L4+ tickets must not record any `status: "timeout"`
//                  line in their falsification.log — Kani-adjacent tests
//                  that time out defeat the formal-verification claim
//   CB-1621 (L1) — `ProvableContract{expected}` snapshot must match the
//                  current YAML's scalar `expected:` value for that test_id
//                  — silent drift detector with per-entry skip when the
//                  YAML's `expected:` is a complex (mapping/list) shape

use std::path::{Path, PathBuf};

use super::types::*;
use crate::cli::handlers::work_contract::{FalsificationMethod, WorkContract};
use crate::cli::handlers::work_verification_level::VerificationLevel;

// ─── Helpers ─────────────────────────────────────────────────────────────────

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
        let Some(ticket_id) = entry.file_name().to_str().map(|s| s.to_string()) else {
            continue;
        };
        if ticket_id.starts_with('.') || ticket_id == "ledger" {
            continue;
        }
        if let Ok(c) = WorkContract::load(project_path, &ticket_id) {
            out.push(c);
        }
    }
    out
}

/// Collect every `FalsificationMethod::ProvableContract` entry in a contract's
/// claims. Returns `(yaml_path, equation, test_id)` triples.
fn provable_contract_entries(c: &WorkContract) -> Vec<(PathBuf, String, String)> {
    c.claims
        .iter()
        .filter_map(|claim| match &claim.falsification_method {
            FalsificationMethod::ProvableContract {
                yaml_path,
                equation,
                test_id,
                ..
            } => Some((yaml_path.clone(), equation.clone(), test_id.clone())),
            _ => None,
        })
        .collect()
}

/// Extract the `id:` values of list items under a top-level `falsification_tests:`
/// section of a provable-contracts YAML file. Line-scan only — no full YAML
/// parser. Exits the section on the first unindented line after entry.
/// Matches common shapes:
///   falsification_tests:
///     - id: rope_periodicity_test
///     - id: "rope_linearity_test"
pub(crate) fn yaml_falsification_test_ids(yaml: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut in_section = false;
    for line in yaml.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !line.starts_with(' ') {
            in_section = trimmed.starts_with("falsification_tests:");
            continue;
        }
        if !in_section {
            continue;
        }
        // "- id: foo" or continuation "id: foo"
        if let Some(rest) = trimmed
            .strip_prefix("- id:")
            .or_else(|| trimmed.strip_prefix("id:"))
        {
            let id = rest
                .trim()
                .trim_matches(|c: char| c == '"' || c == '\'')
                .trim();
            if !id.is_empty() {
                ids.push(id.to_string());
            }
        }
    }
    ids
}

fn has_any_provable_contract_entries(contracts: &[WorkContract]) -> bool {
    contracts
        .iter()
        .any(|c| !provable_contract_entries(c).is_empty())
}

// ─── CB-1620: Inherited roster coverage (WARN during migration) ──────────────

/// CB-1620 (L1): For every `ContractBinding`, the ticket's claim roster must
/// contain a matching `ProvableContract{}` entry for each test in the bound
/// YAML's `falsification_tests[]`. Missing entries surface as **warnings**
/// during the 30-day migration window (§Migration) so users have time to run
/// `pmat work migrate --seed-inherited-falsification` before fail-mode.
pub(crate) fn check_inherited_roster_coverage(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1620: Inherited Roster Coverage";
    let contracts = load_active_contracts(project_path);
    let any_binding = contracts.iter().any(|c| !c.implements.is_empty());
    if !any_binding {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ticket has `implements:` bindings to enforce".into(),
            severity: Severity::Info,
        };
    }
    let mut gaps: Vec<String> = Vec::new();
    for c in &contracts {
        for binding in &c.implements {
            let file = if binding.file.is_absolute() {
                binding.file.clone()
            } else {
                project_path.join(&binding.file)
            };
            let Ok(yaml) = std::fs::read_to_string(&file) else {
                continue;
            };
            let expected_ids = yaml_falsification_test_ids(&yaml);
            if expected_ids.is_empty() {
                continue;
            }
            let entries = provable_contract_entries(c);
            for id in &expected_ids {
                let present = entries.iter().any(|(path, eq, test_id)| {
                    path == &binding.file && eq == &binding.equation && test_id == id
                });
                if !present {
                    gaps.push(format!(
                        "{} → {}/{}#{}",
                        c.work_item_id, binding.contract, binding.equation, id
                    ));
                }
            }
        }
    }
    if gaps.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: "All bound tickets have inherited ProvableContract entries".into(),
            severity: Severity::Info,
        }
    } else {
        let preview: Vec<String> = gaps.iter().take(5).cloned().collect();
        let more = gaps.len().saturating_sub(5);
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} inherited test(s) missing from roster (e.g. {}{}). Run `pmat work migrate --seed-inherited-falsification`.",
                gaps.len(),
                preview.join(", "),
                if more > 0 { format!(", +{} more", more) } else { String::new() }
            ),
            severity: Severity::Warning,
        }
    }
}

// ─── CB-1623: No duplicate (yaml_path, test_id) across a ticket's roster ─────

/// CB-1623 (L3): `FalsificationMethod::ProvableContract` entries with the
/// same `(yaml_path, test_id)` pair count as double-coverage and inflate
/// passing claim counts. The check walks every ticket's claim roster and
/// fails on the first duplicate found.
pub(crate) fn check_no_duplicate_provable_entries(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1623: No Duplicate ProvableContract Entries";
    let contracts = load_active_contracts(project_path);
    if !has_any_provable_contract_entries(&contracts) {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ProvableContract entries in any ticket yet".into(),
            severity: Severity::Info,
        };
    }
    let mut dupes: Vec<String> = Vec::new();
    for c in &contracts {
        let entries = provable_contract_entries(c);
        let mut seen: Vec<(PathBuf, String)> = Vec::new();
        for (path, _eq, test_id) in entries {
            let key = (path.clone(), test_id.clone());
            if seen.contains(&key) {
                dupes.push(format!(
                    "{} → {}#{}",
                    c.work_item_id,
                    path.display(),
                    test_id
                ));
            } else {
                seen.push(key);
            }
        }
    }
    if dupes.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: "No duplicate (yaml_path, test_id) pairs in any ticket roster".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!("Duplicate ProvableContract entries: {}", dupes.join("; ")),
            severity: Severity::Error,
        }
    }
}

// ─── CB-1626: Referenced test_id exists in YAML ──────────────────────────────

/// CB-1626 (L1): Each `ProvableContract.test_id` must exist in the bound
/// YAML's `falsification_tests[]` at scan time. A stale reference means the
/// YAML removed the test post-bind and the ticket's claim is unanchored.
pub(crate) fn check_test_id_exists_in_yaml(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1626: Referenced test_id Exists in YAML";
    let contracts = load_active_contracts(project_path);
    if !has_any_provable_contract_entries(&contracts) {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ProvableContract entries in any ticket yet".into(),
            severity: Severity::Info,
        };
    }
    let mut stale: Vec<String> = Vec::new();
    for c in &contracts {
        for (yaml_path, _eq, test_id) in provable_contract_entries(c) {
            let file = if yaml_path.is_absolute() {
                yaml_path.clone()
            } else {
                project_path.join(&yaml_path)
            };
            let Ok(yaml) = std::fs::read_to_string(&file) else {
                stale.push(format!(
                    "{} → {} (YAML missing)",
                    c.work_item_id,
                    yaml_path.display()
                ));
                continue;
            };
            let ids = yaml_falsification_test_ids(&yaml);
            if !ids.iter().any(|i| i == &test_id) {
                stale.push(format!(
                    "{} → {}#{}",
                    c.work_item_id,
                    yaml_path.display(),
                    test_id
                ));
            }
        }
    }
    if stale.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: "All referenced test_ids exist in their YAML sources".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!("Stale test_id references: {}", stale.join("; ")),
            severity: Severity::Error,
        }
    }
}

// ─── CB-1621: Expected-snapshot drift ───────────────────────────────────────

/// CB-1621 (L1): `ProvableContract{expected}` snapshots taken at bind time
/// must match the current YAML's `expected:` field for that test. A
/// divergence is *silent* test-expectation drift — the test_id is stable,
/// the method executes the same code, but the asserted value changed.
///
/// # Schema
///
/// Snapshot: `ProvableContract.expected` — canonical JSON emitted by the
/// bind step. Today's writer may leave this empty; empty snapshots skip
/// per-entry (nothing to diff against).
///
/// Current value: line-scanned from the bound YAML's `falsification_tests:`
/// block. Only scalar shapes (bool, number, quoted/bare string, null) are
/// compared — inline mappings (`expected: {a: 1}`) and block scalars
/// (`expected: |\n  ...`) need a structural YAML parser and are silently
/// skipped until the pv-yaml-loader (Component 29) is in place.
///
/// # Skip semantics (tiered)
///
/// * no `.pmat-work/*/contract.json` tickets        → Skip
/// * no ticket has a ProvableContract entry with a
///   non-empty `expected` snapshot                  → Skip
/// * YAMLs exist but none declare scalar `expected:`
///   for any bound test_id                          → Skip
///
/// # Fail
///
/// * snapshot and current scalar decode to different JSON values
pub(crate) fn check_expected_snapshot_drift(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1621: Expected Snapshot Drift";
    let contracts = load_active_contracts(project_path);
    if contracts.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/*/contract.json` tickets present".into(),
            severity: Severity::Info,
        };
    }

    let mut any_snapshot = false;
    let mut compared = 0usize;
    let mut drift: Vec<String> = Vec::new();

    for c in &contracts {
        for (yaml_rel, equation, test_id, snapshot) in provable_contract_entries_with_expected(c) {
            if snapshot.trim().is_empty() {
                continue;
            }
            any_snapshot = true;
            let yaml_abs = if yaml_rel.is_absolute() {
                yaml_rel.clone()
            } else {
                project_path.join(&yaml_rel)
            };
            let Ok(yaml_body) = std::fs::read_to_string(&yaml_abs) else {
                continue;
            };
            let map = yaml_expected_by_test_id(&yaml_body);
            let Some(current_json) = map.get(&test_id) else {
                continue;
            };
            compared += 1;

            // Structural compare: both sides canonicalize to JSON; an
            // unparseable snapshot or current value falls back to string
            // equality so we still catch trivial differences.
            let match_ok = match (
                serde_json::from_str::<serde_json::Value>(&snapshot),
                serde_json::from_str::<serde_json::Value>(current_json),
            ) {
                (Ok(a), Ok(b)) => a == b,
                _ => current_json == &snapshot,
            };
            if !match_ok {
                drift.push(format!(
                    "  {} [{}/{}#{}] expected drifted: {} → {}",
                    c.work_item_id,
                    yaml_rel.display(),
                    equation,
                    test_id,
                    snapshot,
                    current_json
                ));
            }
        }
    }

    if !any_snapshot {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ProvableContract entry has a non-empty `expected` snapshot".into(),
            severity: Severity::Info,
        };
    }
    if compared == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No bound YAML declares scalar `expected:` for any seeded test_id".into(),
            severity: Severity::Info,
        };
    }

    if !drift.is_empty() {
        let mut msg = format!("{} expected snapshot drift(s):\n", drift.len());
        for line in &drift {
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
        message: format!(
            "{} ProvableContract snapshot(s) match current YAML `expected:`",
            compared
        ),
        severity: Severity::Info,
    }
}

/// Collect ProvableContract roster entries with their bind-time `expected`
/// snapshot. Sibling of `provable_contract_entries` — that helper drops
/// the snapshot for checks that only care about `(yaml, eq, test_id)`.
fn provable_contract_entries_with_expected(
    c: &WorkContract,
) -> Vec<(PathBuf, String, String, String)> {
    c.claims
        .iter()
        .filter_map(|claim| match &claim.falsification_method {
            FalsificationMethod::ProvableContract {
                yaml_path,
                equation,
                test_id,
                expected,
            } => Some((
                yaml_path.clone(),
                equation.clone(),
                test_id.clone(),
                expected.clone(),
            )),
            _ => None,
        })
        .collect()
}

/// Parse a YAML's top-level `falsification_tests:` block and return a
/// `{test_id → canonical-JSON expected}` map. Only scalar `expected:`
/// values are converted; inline mappings/sequences and block scalars are
/// silently skipped — they'd need a structural parser (Component 29
/// pv-yaml-loader) to round-trip safely.
fn yaml_expected_by_test_id(yaml: &str) -> std::collections::HashMap<String, String> {
    let mut out: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut in_section = false;
    let mut current_id: Option<String> = None;
    let mut current_exp: Option<String> = None;

    fn flush(
        out: &mut std::collections::HashMap<String, String>,
        id: &mut Option<String>,
        exp: &mut Option<String>,
    ) {
        if let (Some(i), Some(e)) = (id.take(), exp.take()) {
            out.insert(i, e);
        } else {
            *id = None;
            *exp = None;
        }
    }

    for line in yaml.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !line.starts_with(' ') && !line.starts_with('-') {
            flush(&mut out, &mut current_id, &mut current_exp);
            in_section = trimmed.starts_with("falsification_tests:");
            continue;
        }
        if !in_section {
            continue;
        }
        if trimmed.starts_with('-') {
            flush(&mut out, &mut current_id, &mut current_exp);
            let item = trimmed[1..].trim();
            if let Some(rest) = item.strip_prefix("id:") {
                let id = rest
                    .trim()
                    .trim_matches(|c: char| c == '"' || c == '\'')
                    .to_string();
                if !id.is_empty() {
                    current_id = Some(id);
                }
            } else if let Some(rest) = item.strip_prefix("expected:") {
                if let Some(j) = yaml_scalar_to_canonical_json(rest) {
                    current_exp = Some(j);
                }
            }
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("id:") {
            let id = rest
                .trim()
                .trim_matches(|c: char| c == '"' || c == '\'')
                .to_string();
            if !id.is_empty() {
                current_id = Some(id);
            }
        } else if let Some(rest) = trimmed.strip_prefix("expected:") {
            if let Some(j) = yaml_scalar_to_canonical_json(rest) {
                current_exp = Some(j);
            }
        }
    }
    flush(&mut out, &mut current_id, &mut current_exp);
    out
}

/// Convert a YAML scalar to canonical JSON. Handles booleans, nulls,
/// integers/floats, quoted strings, and bare strings. Returns `None` for
/// complex shapes (inline mappings, sequences, block scalars) so callers
/// skip the entry instead of comparing wrong shapes.
fn yaml_scalar_to_canonical_json(raw: &str) -> Option<String> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }
    // Inline mapping / sequence / block scalar — bail.
    if s.starts_with('{') || s.starts_with('[') || s.starts_with('|') || s.starts_with('>') {
        return None;
    }
    match s {
        "true" | "false" => return Some(s.to_string()),
        "null" | "~" => return Some("null".to_string()),
        _ => {}
    }
    if let Ok(n) = s.parse::<i64>() {
        return Some(n.to_string());
    }
    if let Ok(n) = s.parse::<f64>() {
        if let Ok(j) = serde_json::to_string(&n) {
            return Some(j);
        }
    }
    if s.len() >= 2 {
        if s.starts_with('"') && s.ends_with('"') {
            let inner = &s[1..s.len() - 1];
            return serde_json::to_string(inner).ok();
        }
        if s.starts_with('\'') && s.ends_with('\'') {
            let inner = &s[1..s.len() - 1];
            return serde_json::to_string(inner).ok();
        }
    }
    serde_json::to_string(s).ok()
}

/// Parse a `falsification.log` JSONL file into the set of
/// `(yaml_path, test_id)` pairs it covers. The format (per Component 29
/// §falsification.log) is one JSON object per line:
/// `{"yaml":"...","equation":"...","test_id":"...","status":"pass",...}`.
/// Malformed or non-inherited lines (manual-source entries have no `yaml`)
/// are silently ignored — we only care which roster entries executed.
fn parse_falsification_log(contents: &str) -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let Some(yaml) = v.get("yaml").and_then(|y| y.as_str()) else {
            continue;
        };
        let Some(test_id) = v.get("test_id").and_then(|t| t.as_str()) else {
            continue;
        };
        out.push((PathBuf::from(yaml), test_id.to_string()));
    }
    out
}

/// CB-1622 (L3): every `ProvableContract` roster entry must have at least
/// one execution line in `.pmat-work/<ID>/falsification.log`. A roster
/// entry without a receipt is an "untested claim" — it declares coverage
/// the runner never verified.
///
/// Skip-if-absent: the unified falsification runner (Component 29) that
/// emits `falsification.log` hasn't landed, so today's check skips
/// cleanly when no ticket has the log. Once any ticket gains a log, this
/// check lights up for that ticket while others still skip.
pub(crate) fn check_roster_execution_coverage(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1622: Roster Execution Coverage";
    let contracts = load_active_contracts(project_path);

    let mut uncovered: Vec<String> = Vec::new();
    let mut checked = 0usize;

    for c in &contracts {
        let entries = provable_contract_entries(c);
        if entries.is_empty() {
            continue;
        }
        let log_path = project_path
            .join(".pmat-work")
            .join(&c.work_item_id)
            .join("falsification.log");
        let Ok(contents) = std::fs::read_to_string(&log_path) else {
            // No log for this ticket → skip per-ticket, don't fail
            continue;
        };
        let covered = parse_falsification_log(&contents);
        checked += 1;
        for (yaml_path, _eq, test_id) in entries {
            let present = covered
                .iter()
                .any(|(y, t)| y == &yaml_path && t == &test_id);
            if !present {
                uncovered.push(format!(
                    "{} → {}#{}",
                    c.work_item_id,
                    yaml_path.display(),
                    test_id
                ));
            }
        }
    }

    if checked == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/<ID>/falsification.log` files — unified runner hasn't executed rosters yet".into(),
            severity: Severity::Info,
        };
    }

    if uncovered.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} ticket(s): every roster entry has an execution receipt",
                checked
            ),
            severity: Severity::Info,
        };
    }

    let preview: Vec<String> = uncovered.iter().take(5).cloned().collect();
    let more = uncovered.len().saturating_sub(5);
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Fail,
        message: format!(
            "{} roster entry/entries never executed — run `pmat work falsify`: {}{}",
            uncovered.len(),
            preview.join(", "),
            if more > 0 {
                format!(", +{} more", more)
            } else {
                String::new()
            }
        ),
        severity: Severity::Error,
    }
}

/// CB-1624 (L1): `ProvableContract{}` entries cannot be deleted outside the
/// `pmat work unbind` path. The roster-mutations ledger at
/// `.pmat-work/ledger/roster-mutations.json` records every add/remove event
/// on a ticket's inherited roster. Deletion entries must carry `via_unbind:
/// true` (the `unbind` command sets this when writing the mutation) — any
/// other deletion represents a manual `contract.json` edit that bypassed
/// the unbind audit and erodes scope silently.
///
/// Ledger schema (minimum): JSON array at
/// `.pmat-work/ledger/roster-mutations.json` where each entry has:
///   • `ticket`        — the work-item-id whose roster changed
///   • `action`        — one of `"delete"`, `"add"`, `"update"`; deletion
///                        family matched case-insensitively on the `"delet"`
///                        prefix (so `delete`, `Delete`, `DELETED`,
///                        `deletion` all trigger the via_unbind check)
///   • `target`        — `{ yaml, equation, test_id }` identifying the
///                        inherited entry that was mutated (informational;
///                        this check does not validate target shape)
///   • `via_unbind`    — boolean flag that `pmat work unbind` sets to true;
///                        deletion without this flag is a violation
///
/// Skip semantics (tiered):
///   • no `.pmat-work/` directory                → Skip
///   • no `ledger/roster-mutations.json` file    → Skip (mutation writer
///                                                  hasn't landed)
///   • ledger exists but contains no entries     → Pass (nothing to audit)
///   • any entry has unrecognized shape          → Warn (per-entry)
///   • any deletion entry lacks `via_unbind`     → Fail
pub(crate) fn check_no_manual_deletion(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1624: No Manual Deletion of Inherited Entries";
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/` directory".into(),
            severity: Severity::Info,
        };
    }

    let ledger = work_dir.join("ledger").join("roster-mutations.json");
    if !ledger.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No roster-mutations ledger at .pmat-work/ledger/roster-mutations.json".into(),
            severity: Severity::Info,
        };
    }

    let Ok(content) = std::fs::read_to_string(&ledger) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Warn,
            message: format!("Unreadable roster-mutations ledger: {}", ledger.display()),
            severity: Severity::Warning,
        };
    };

    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: "Roster-mutations ledger is not valid JSON".into(),
            severity: Severity::Error,
        };
    };

    let Some(entries) = value.as_array() else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: "Roster-mutations ledger must be a JSON array of entries".into(),
            severity: Severity::Error,
        };
    };

    let mut manual_deletions: Vec<String> = Vec::new();
    let mut deletions_checked = 0usize;
    let total = entries.len();

    for (i, entry) in entries.iter().enumerate() {
        let action = entry
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        // `delet` catches delete, Delete, DELETED, deletion — any deletion
        // verb family — while ignoring unrelated actions like `add`/`update`.
        if !action.starts_with("delet") {
            continue;
        }
        deletions_checked += 1;
        let via_unbind = entry
            .get("via_unbind")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if via_unbind {
            continue;
        }
        let ticket = entry
            .get("ticket")
            .and_then(|v| v.as_str())
            .unwrap_or("<unknown>");
        let target_label = entry
            .get("target")
            .and_then(|t| {
                let y = t.get("yaml").and_then(|v| v.as_str()).unwrap_or("?");
                let tid = t.get("test_id").and_then(|v| v.as_str()).unwrap_or("?");
                Some(format!("{}#{}", y, tid))
            })
            .unwrap_or_else(|| "?".to_string());
        manual_deletions.push(format!(
            "  entry[{}] ticket={} target={} action={}",
            i, ticket, target_label, action
        ));
    }

    if !manual_deletions.is_empty() {
        let mut msg = format!(
            "{} manual deletion(s) bypassed `pmat work unbind`:\n",
            manual_deletions.len()
        );
        for line in &manual_deletions {
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

    if total == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: "Roster-mutations ledger is empty — no roster mutations to audit".into(),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!(
            "{} mutation(s) audited, {} deletion(s) all via `pmat work unbind`",
            total, deletions_checked
        ),
        severity: Severity::Info,
    }
}

/// CB-1625 (L3): any *inherited* falsification log line with `status` other
/// than `"pass"` is fatal, regardless of the ticket's verification level.
/// An inherited entry that failed once stays failed until a subsequent pass
/// supersedes it — the runner appends, never mutates, so a trailing `fail`
/// means the bound equation is still unproven.
///
/// Scope:
///   • Only *inherited* lines (those carrying `yaml`+`test_id`) count —
///     manual `method`-keyed lines are governed by CB-1629's L4 timeout gate
///     and caller-specific logic, not this blanket rule
///   • Scans every ticket regardless of `verification_level` — if you ran
///     a roster test, its result is binding
///
/// Skip-if-absent: no `.pmat-work/*/falsification.log` files → overall skip.
pub(crate) fn check_inherited_failure_fatal(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1625: Inherited Failure Fatal";
    let contracts = load_active_contracts(project_path);

    let mut checked_logs = 0usize;
    let mut checked_lines = 0usize;
    let mut failing: Vec<String> = Vec::new();

    for c in &contracts {
        let log_path = project_path
            .join(".pmat-work")
            .join(&c.work_item_id)
            .join("falsification.log");
        let Ok(contents) = std::fs::read_to_string(&log_path) else {
            continue;
        };
        checked_logs += 1;
        for (idx, line) in contents.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) else {
                continue; // CB-1628 owns malformed JSON detection
            };
            if !is_inherited_receipt(&v) {
                continue;
            }
            let Some(status) = v.get("status").and_then(|s| s.as_str()) else {
                continue; // CB-1628 owns missing-field detection
            };
            checked_lines += 1;
            if status != "pass" {
                let label = v
                    .get("test_id")
                    .and_then(|s| s.as_str())
                    .unwrap_or("<no-test-id>");
                let yaml = v
                    .get("yaml")
                    .and_then(|s| s.as_str())
                    .unwrap_or("<no-yaml>");
                failing.push(format!(
                    "  {}:{} {}::{} status={}",
                    c.work_item_id,
                    idx + 1,
                    yaml,
                    label,
                    status
                ));
            }
        }
    }

    if checked_logs == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/<ID>/falsification.log` files to inspect".into(),
            severity: Severity::Info,
        };
    }

    if !failing.is_empty() {
        let mut msg = format!(
            "{} inherited falsification line(s) did not pass — contract is falsified:\n",
            failing.len()
        );
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

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!(
            "All {} inherited line(s) across {} log(s) passed",
            checked_lines, checked_logs
        ),
        severity: Severity::Info,
    }
}

/// CB-1627 (L3): warn when the bound YAML's `falsification_tests[]` has
/// gained entries since bind time. The bind-time snapshot is implicit:
/// every `ProvableContract{ yaml_path, equation, test_id }` entry in the
/// ticket's claims roster was seeded *at bind time* from the YAML that
/// existed then. Any test_id currently in the YAML that is NOT in that
/// seeded roster is a post-bind addition — the contract owner should
/// re-bind (or explicitly opt out) to pick up the new coverage.
///
/// Result is always WARN — per spec §CB-1627 this is a drift signal, not
/// a hard failure. The ticket still completes; the warning surfaces the
/// missed coverage for the next planning cycle.
///
/// Skip semantics (tiered):
///   • no `.pmat-work/` directory                → Skip
///   • no ticket has `implements:` bindings      → Skip
///   • every binding's YAML is missing or has
///     no `falsification_tests[]`                → Skip
pub(crate) fn check_post_bind_yaml_drift(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1627: Post-bind YAML Drift";
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/` directory".into(),
            severity: Severity::Info,
        };
    }

    let contracts = load_active_contracts(project_path);
    if contracts.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/*/contract.json` tickets present".into(),
            severity: Severity::Info,
        };
    }

    let any_binding = contracts.iter().any(|c| !c.implements.is_empty());
    if !any_binding {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ticket has `implements:` bindings".into(),
            severity: Severity::Info,
        };
    }

    let mut checked_bindings = 0usize;
    let mut drift: Vec<String> = Vec::new();

    for c in &contracts {
        let seeded = provable_contract_entries(c);
        for binding in &c.implements {
            let yaml_path = if binding.file.is_absolute() {
                binding.file.clone()
            } else {
                project_path.join(&binding.file)
            };
            let Ok(yaml_body) = std::fs::read_to_string(&yaml_path) else {
                continue;
            };
            let current_ids = yaml_falsification_test_ids(&yaml_body);
            if current_ids.is_empty() {
                continue;
            }
            checked_bindings += 1;

            let bound_ids: std::collections::HashSet<&str> = seeded
                .iter()
                .filter(|(path, eq, _)| path == &binding.file && eq == &binding.equation)
                .map(|(_, _, tid)| tid.as_str())
                .collect();

            for id in &current_ids {
                if !bound_ids.contains(id.as_str()) {
                    drift.push(format!(
                        "{} → {}/{}#{}",
                        c.work_item_id, binding.contract, binding.equation, id
                    ));
                }
            }
        }
    }

    if checked_bindings == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No binding's YAML declares `falsification_tests[]`".into(),
            severity: Severity::Info,
        };
    }

    if drift.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} binding(s) checked, no post-bind YAML additions",
                checked_bindings
            ),
            severity: Severity::Info,
        };
    }

    let preview: Vec<String> = drift.iter().take(5).cloned().collect();
    let more = drift.len().saturating_sub(5);
    let suffix = if more > 0 {
        format!(", +{} more", more)
    } else {
        String::new()
    };
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Warn,
        message: format!(
            "{} new `falsification_tests[]` entry/entries post-bind — rebind or opt-out: {}{}",
            drift.len(),
            preview.join(", "),
            suffix
        ),
        severity: Severity::Warning,
    }
}

/// Decide whether a log line is an inherited (ProvableContract) receipt.
/// Per spec §falsification.log, inherited lines carry a `yaml` key;
/// manual lines carry a `method` key instead. Only inherited lines are
/// subject to the 4-field shape requirement in CB-1628.
fn is_inherited_receipt(v: &serde_json::Value) -> bool {
    v.get("yaml").and_then(|y| y.as_str()).is_some()
        || v.get("test_id").and_then(|t| t.as_str()).is_some()
}

/// CB-1628 (L3): each line in `.pmat-work/<ID>/falsification.log` that
/// represents an inherited run must carry the 4-field shape
/// `{yaml, test_id, status, duration_ms}`. Missing fields mean the
/// emitter dropped data — silent skips are indistinguishable from real
/// passes post-hoc. Manual-source lines (no `yaml`) are ignored per spec.
///
/// Skip-if-absent: no falsification.log files → skip overall.
pub(crate) fn check_per_run_log_line(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1628: Per-run Log Line Emitted";
    let contracts = load_active_contracts(project_path);

    let mut malformed: Vec<String> = Vec::new();
    let mut missing_fields: Vec<String> = Vec::new();
    let mut checked_logs = 0usize;
    let mut checked_lines = 0usize;

    for c in &contracts {
        let log_path = project_path
            .join(".pmat-work")
            .join(&c.work_item_id)
            .join("falsification.log");
        let Ok(contents) = std::fs::read_to_string(&log_path) else {
            continue;
        };
        checked_logs += 1;
        for (idx, line) in contents.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let lineno = idx + 1;
            let v: serde_json::Value = match serde_json::from_str(trimmed) {
                Ok(v) => v,
                Err(_) => {
                    malformed.push(format!("{}:{}", c.work_item_id, lineno));
                    continue;
                }
            };
            if !is_inherited_receipt(&v) {
                continue;
            }
            checked_lines += 1;
            let mut missing: Vec<&'static str> = Vec::new();
            for field in ["yaml", "test_id", "status", "duration_ms"] {
                if v.get(field).is_none() {
                    missing.push(field);
                }
            }
            if !missing.is_empty() {
                missing_fields.push(format!(
                    "{}:{} missing [{}]",
                    c.work_item_id,
                    lineno,
                    missing.join(", ")
                ));
            }
        }
    }

    if checked_logs == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/<ID>/falsification.log` files to validate".into(),
            severity: Severity::Info,
        };
    }

    if malformed.is_empty() && missing_fields.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} log(s), {} inherited line(s) carry the 4-field shape",
                checked_logs, checked_lines
            ),
            severity: Severity::Info,
        };
    }

    let mut msg = String::new();
    if !malformed.is_empty() {
        msg.push_str(&format!(
            "{} malformed JSONL line(s): {}\n",
            malformed.len(),
            malformed.join(", ")
        ));
    }
    if !missing_fields.is_empty() {
        msg.push_str(&format!(
            "{} line(s) missing required fields: {}",
            missing_fields.len(),
            missing_fields.join("; ")
        ));
    }
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Fail,
        message: msg,
        severity: Severity::Error,
    }
}

/// Return true if a ticket's declared `verification_level` parses to L4+.
/// Ticket strings are typed like `"L3"` or `"L4 (kani_proof)"` — we take
/// the first whitespace-separated token so annotated variants parse too.
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

/// CB-1629 (L4): an L4+ ticket's `falsification.log` must not record any
/// `status: "timeout"` line. L4 correctness depends on completed Kani
/// verification; a timed-out Kani harness is indistinguishable from an
/// unbounded counterexample and must not be claimed as passed.
///
/// Skip-if-absent: no L4+ ticket with a log → skip overall. Manual-source
/// timeouts also fail (the timeout semantics are level-gated, not
/// source-gated: an L4 ticket cannot admit timeouts of any kind).
pub(crate) fn check_l4_timeout_gate(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1629: L4 Timeout Gate";
    let contracts = load_active_contracts(project_path);

    let mut timeouts: Vec<String> = Vec::new();
    let mut checked = 0usize;

    for c in &contracts {
        if !is_l4_or_higher(c) {
            continue;
        }
        let log_path = project_path
            .join(".pmat-work")
            .join(&c.work_item_id)
            .join("falsification.log");
        let Ok(contents) = std::fs::read_to_string(&log_path) else {
            continue;
        };
        checked += 1;
        for (idx, line) in contents.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) else {
                continue;
            };
            let status = v.get("status").and_then(|s| s.as_str()).unwrap_or("");
            if status.eq_ignore_ascii_case("timeout") {
                let label = v
                    .get("test_id")
                    .or_else(|| v.get("method"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("?");
                timeouts.push(format!("{}:{} ({})", c.work_item_id, idx + 1, label));
            }
        }
    }

    if checked == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No L4+ ticket has a `falsification.log` to check".into(),
            severity: Severity::Info,
        };
    }

    if timeouts.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!("{} L4+ ticket(s): no timeouts recorded", checked),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Fail,
        message: format!(
            "{} timeout(s) in L4+ ticket log(s) — Kani-adjacent flakes defeat the level: {}",
            timeouts.len(),
            timeouts.join(", ")
        ),
        severity: Severity::Error,
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn yaml_ids_extracts_list_form() {
        let y = "falsification_tests:\n  - id: a\n  - id: \"b\"\n  - id: 'c'\n";
        assert_eq!(yaml_falsification_test_ids(y), vec!["a", "b", "c"]);
    }

    #[test]
    fn yaml_ids_ignores_other_sections() {
        let y = "equations:\n  - id: not_here\nfalsification_tests:\n  - id: actual\nkani_harnesses:\n  - id: neither\n";
        assert_eq!(yaml_falsification_test_ids(y), vec!["actual"]);
    }

    #[test]
    fn yaml_ids_empty_when_section_absent() {
        assert!(yaml_falsification_test_ids("equations:\n  rope: {}\n").is_empty());
    }

    #[test]
    fn yaml_ids_empty_flow_list() {
        let y = "falsification_tests: []\n";
        assert!(yaml_falsification_test_ids(y).is_empty());
    }

    // ── CB-1621: Expected Snapshot Drift tests ──────────────────────────

    fn write_contract_with_expected_snapshot(
        project: &Path,
        ticket: &str,
        yaml_path: &str,
        equation: &str,
        test_id: &str,
        expected_canonical: &str,
    ) {
        use crate::cli::handlers::work_contract::{EvidenceType, FalsifiableClaim};
        let mut c = WorkContract::new(ticket.into(), "deadbeef".into());
        c.claims.push(FalsifiableClaim {
            hypothesis: "inherited claim".into(),
            falsification_method: FalsificationMethod::ProvableContract {
                yaml_path: PathBuf::from(yaml_path),
                equation: equation.into(),
                test_id: test_id.into(),
                expected: expected_canonical.into(),
            },
            evidence_required: EvidenceType::BooleanCheck(true),
            result: None,
            override_info: None,
        });
        let dir = project.join(".pmat-work").join(ticket);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("contract.json"),
            serde_json::to_string_pretty(&c).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn expected_drift_skips_with_no_contracts() {
        let tmp = tempdir().unwrap();
        let r = check_expected_snapshot_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/*/contract.json`"));
    }

    #[test]
    fn expected_drift_skips_when_no_snapshot_set() {
        // ProvableContract entry with empty `expected` → no snapshot to compare.
        let tmp = tempdir().unwrap();
        write_contract_with_expected_snapshot(
            tmp.path(),
            "T1",
            "contracts/k.yaml",
            "rope",
            "t1",
            "",
        );
        let r = check_expected_snapshot_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No ProvableContract entry"));
    }

    #[test]
    fn expected_drift_skips_when_no_yaml_declares_expected() {
        let tmp = tempdir().unwrap();
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            "falsification_tests:\n  - id: t1\n",
        );
        write_contract_with_expected_snapshot(
            tmp.path(),
            "T1",
            "contracts/k.yaml",
            "rope",
            "t1",
            "true",
        );
        let r = check_expected_snapshot_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No bound YAML declares scalar"));
    }

    #[test]
    fn expected_drift_passes_on_matching_bool() {
        let tmp = tempdir().unwrap();
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            "falsification_tests:\n  - id: t1\n    expected: true\n",
        );
        write_contract_with_expected_snapshot(
            tmp.path(),
            "T1",
            "contracts/k.yaml",
            "rope",
            "t1",
            "true",
        );
        let r = check_expected_snapshot_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn expected_drift_passes_on_matching_number() {
        let tmp = tempdir().unwrap();
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            "falsification_tests:\n  - id: t1\n    expected: 42\n",
        );
        write_contract_with_expected_snapshot(
            tmp.path(),
            "T1",
            "contracts/k.yaml",
            "rope",
            "t1",
            "42",
        );
        let r = check_expected_snapshot_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn expected_drift_passes_on_matching_quoted_string() {
        // YAML `expected: "abc"` and snapshot `"\"abc\""` should both
        // canonicalize to JSON `"abc"`.
        let tmp = tempdir().unwrap();
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            "falsification_tests:\n  - id: t1\n    expected: \"abc\"\n",
        );
        write_contract_with_expected_snapshot(
            tmp.path(),
            "T1",
            "contracts/k.yaml",
            "rope",
            "t1",
            "\"abc\"",
        );
        let r = check_expected_snapshot_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn expected_drift_fails_on_bool_flip() {
        let tmp = tempdir().unwrap();
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            "falsification_tests:\n  - id: t1\n    expected: false\n",
        );
        write_contract_with_expected_snapshot(
            tmp.path(),
            "T1",
            "contracts/k.yaml",
            "rope",
            "t1",
            "true",
        );
        let r = check_expected_snapshot_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("t1"));
        assert!(r.message.contains("drifted"));
    }

    #[test]
    fn expected_drift_fails_on_number_change() {
        let tmp = tempdir().unwrap();
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            "falsification_tests:\n  - id: t1\n    expected: 100\n",
        );
        write_contract_with_expected_snapshot(
            tmp.path(),
            "T1",
            "contracts/k.yaml",
            "rope",
            "t1",
            "42",
        );
        let r = check_expected_snapshot_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
    }

    #[test]
    fn expected_drift_skips_complex_inline_mapping() {
        // Inline mapping (`expected: {a: 1}`) bails out of scalar extraction
        // → per-entry skip → overall Skip because nothing was compared.
        let tmp = tempdir().unwrap();
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            "falsification_tests:\n  - id: t1\n    expected: {a: 1}\n",
        );
        write_contract_with_expected_snapshot(
            tmp.path(),
            "T1",
            "contracts/k.yaml",
            "rope",
            "t1",
            "{\"a\": 1}",
        );
        let r = check_expected_snapshot_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("scalar"));
    }

    #[test]
    fn expected_drift_tolerates_missing_yaml() {
        // Bound YAML file doesn't exist — per-entry skip, overall Skip.
        let tmp = tempdir().unwrap();
        write_contract_with_expected_snapshot(
            tmp.path(),
            "T1",
            "contracts/missing.yaml",
            "rope",
            "t1",
            "true",
        );
        let r = check_expected_snapshot_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }

    #[test]
    fn expected_drift_reports_multiple_entries() {
        // Two tickets each with a drifted test_id — both show in the message.
        let tmp = tempdir().unwrap();
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            "falsification_tests:\n  - id: t1\n    expected: false\n  - id: t2\n    expected: 7\n",
        );
        write_contract_with_expected_snapshot(
            tmp.path(),
            "T1",
            "contracts/k.yaml",
            "rope",
            "t1",
            "true",
        );
        write_contract_with_expected_snapshot(
            tmp.path(),
            "T2",
            "contracts/k.yaml",
            "rope",
            "t2",
            "42",
        );
        let r = check_expected_snapshot_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T1"));
        assert!(r.message.contains("T2"));
    }

    #[test]
    fn yaml_expected_map_handles_multiple_tests() {
        let yaml = "falsification_tests:\n  - id: a\n    expected: true\n  - id: b\n    expected: 2\n  - id: c\n    expected: \"x\"\n";
        let m = yaml_expected_by_test_id(yaml);
        assert_eq!(m.get("a").unwrap(), "true");
        assert_eq!(m.get("b").unwrap(), "2");
        assert_eq!(m.get("c").unwrap(), "\"x\"");
    }

    #[test]
    fn yaml_scalar_null_tilde_becomes_json_null() {
        assert_eq!(yaml_scalar_to_canonical_json("~").unwrap(), "null");
        assert_eq!(yaml_scalar_to_canonical_json("null").unwrap(), "null");
    }

    #[test]
    fn yaml_scalar_bare_string_gets_quoted() {
        assert_eq!(yaml_scalar_to_canonical_json("hello").unwrap(), "\"hello\"");
    }

    #[test]
    fn yaml_scalar_rejects_complex_shapes() {
        assert!(yaml_scalar_to_canonical_json("{a: 1}").is_none());
        assert!(yaml_scalar_to_canonical_json("[1, 2]").is_none());
        assert!(yaml_scalar_to_canonical_json("|").is_none());
    }

    // ── CB-1629 L4 timeout gate tests ────────────────────────────────────

    fn contract_at_level(ticket: &str, level: &str) -> WorkContract {
        let mut c = WorkContract::new(ticket.into(), "deadbeef".into());
        c.verification_level = level.into();
        c
    }

    #[test]
    fn l4_timeout_skips_when_no_l4_ticket() {
        let tmp = tempdir().unwrap();
        let c = contract_at_level("T1", "L3");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"yaml":"a.yaml","test_id":"t1","status":"timeout","duration_ms":60000}"#,
                "\n",
            ),
        );
        let check = check_l4_timeout_gate(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn l4_timeout_skips_when_l4_ticket_has_no_log() {
        let tmp = tempdir().unwrap();
        let c = contract_at_level("T1", "L4");
        write_contract_json(tmp.path(), "T1", &c);
        let check = check_l4_timeout_gate(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn l4_timeout_passes_when_no_timeout_recorded() {
        let tmp = tempdir().unwrap();
        let c = contract_at_level("T1", "L4");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"yaml":"a.yaml","test_id":"t1","status":"pass","duration_ms":500}"#,
                "\n",
                r#"{"yaml":"a.yaml","test_id":"t2","status":"fail","duration_ms":200}"#,
                "\n",
            ),
        );
        let check = check_l4_timeout_gate(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn l4_timeout_fails_on_inherited_timeout() {
        let tmp = tempdir().unwrap();
        let c = contract_at_level("T1", "L4 (kani_proof)");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"yaml":"a.yaml","test_id":"rope_big","status":"timeout","duration_ms":60000}"#,
                "\n",
            ),
        );
        let check = check_l4_timeout_gate(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("T1"));
        assert!(check.message.contains("rope_big"));
    }

    #[test]
    fn l4_timeout_fails_on_manual_timeout_too() {
        // L4 is source-agnostic — manual-source timeouts also fail.
        let tmp = tempdir().unwrap();
        let c = contract_at_level("T1", "L5");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"method":"TdgRegression","status":"timeout","duration_ms":300000}"#,
                "\n",
            ),
        );
        let check = check_l4_timeout_gate(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("TdgRegression"));
    }

    #[test]
    fn is_l4_or_higher_accepts_ladder() {
        let mut c = WorkContract::new("T".into(), "deadbeef".into());
        for (s, want) in [
            ("L0", false),
            ("L3", false),
            ("L4", true),
            ("L4 (kani_proof)", true),
            ("L5", true),
        ] {
            c.verification_level = s.into();
            assert_eq!(is_l4_or_higher(&c), want, "{}", s);
        }
    }

    // ── CB-1628 per-run log line emitted tests ───────────────────────────

    #[test]
    fn per_run_log_skips_when_no_logs() {
        let tmp = tempdir().unwrap();
        let check = check_per_run_log_line(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn per_run_log_passes_when_all_inherited_lines_complete() {
        let tmp = tempdir().unwrap();
        let c = contract_with_provable("T1", "a.yaml", "e", "t1");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"yaml":"a.yaml","test_id":"t1","status":"pass","duration_ms":12}"#,
                "\n",
                r#"{"yaml":"a.yaml","test_id":"t2","status":"fail","duration_ms":99}"#,
                "\n",
            ),
        );
        let check = check_per_run_log_line(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn per_run_log_ignores_manual_lines() {
        // Manual lines carry `method`, not `yaml` — not subject to field check.
        let tmp = tempdir().unwrap();
        let c = WorkContract::new("T1".into(), "deadbeef".into());
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"method":"TdgRegression","status":"pass","duration_ms":100}"#,
                "\n",
            ),
        );
        let check = check_per_run_log_line(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn per_run_log_fails_on_missing_duration_ms() {
        let tmp = tempdir().unwrap();
        let c = contract_with_provable("T1", "a.yaml", "e", "t1");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(r#"{"yaml":"a.yaml","test_id":"t1","status":"pass"}"#, "\n",),
        );
        let check = check_per_run_log_line(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("duration_ms"));
        assert!(check.message.contains("T1"));
    }

    #[test]
    fn per_run_log_fails_on_malformed_json() {
        let tmp = tempdir().unwrap();
        let c = WorkContract::new("T1".into(), "deadbeef".into());
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"yaml":"a.yaml","test_id":"t1","status":"pass","duration_ms":1}"#,
                "\n",
                "not json{\n",
            ),
        );
        let check = check_per_run_log_line(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("malformed"));
    }

    #[test]
    fn per_run_log_empty_lines_ignored() {
        let tmp = tempdir().unwrap();
        let c = WorkContract::new("T1".into(), "deadbeef".into());
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"yaml":"a.yaml","test_id":"t1","status":"pass","duration_ms":1}"#,
                "\n",
                "\n",
                "   \n",
                r#"{"yaml":"b.yaml","test_id":"t2","status":"pass","duration_ms":2}"#,
                "\n",
            ),
        );
        let check = check_per_run_log_line(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    // ── CB-1622 roster execution coverage tests ──────────────────────────

    fn write_contract_json(project: &Path, ticket: &str, contract: &WorkContract) {
        let dir = project.join(".pmat-work").join(ticket);
        std::fs::create_dir_all(&dir).unwrap();
        let json = serde_json::to_string_pretty(contract).unwrap();
        std::fs::write(dir.join("contract.json"), json).unwrap();
    }

    fn write_log(project: &Path, ticket: &str, jsonl: &str) {
        let dir = project.join(".pmat-work").join(ticket);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("falsification.log"), jsonl).unwrap();
    }

    fn contract_with_provable(
        ticket: &str,
        yaml_path: &str,
        equation: &str,
        test_id: &str,
    ) -> WorkContract {
        use crate::cli::handlers::work_contract::{EvidenceType, FalsifiableClaim};
        let mut c = WorkContract::new(ticket.into(), "deadbeef".into());
        c.claims.push(FalsifiableClaim {
            hypothesis: "inherited claim".into(),
            falsification_method: FalsificationMethod::ProvableContract {
                yaml_path: PathBuf::from(yaml_path),
                equation: equation.into(),
                test_id: test_id.into(),
                expected: "\"canonical\"".into(),
            },
            evidence_required: EvidenceType::BooleanCheck(true),
            result: None,
            override_info: None,
        });
        c
    }

    #[test]
    fn roster_coverage_skips_when_no_log_files() {
        let tmp = tempdir().unwrap();
        let c = contract_with_provable("T1", "contracts/k.yaml", "rope", "rope_test");
        write_contract_json(tmp.path(), "T1", &c);
        let check = check_roster_execution_coverage(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("falsification.log"));
    }

    #[test]
    fn roster_coverage_skips_when_no_provable_entries() {
        // Contract with no ProvableContract entries — irrelevant, skip overall.
        let tmp = tempdir().unwrap();
        let c = WorkContract::new("T1".into(), "deadbeef".into());
        write_contract_json(tmp.path(), "T1", &c);
        let check = check_roster_execution_coverage(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn roster_coverage_passes_when_every_entry_has_receipt() {
        let tmp = tempdir().unwrap();
        let c = contract_with_provable("T1", "contracts/k.yaml", "rope", "rope_test");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"ts":"2026-04-18T00:00:00Z","source":"inherited","yaml":"contracts/k.yaml","equation":"rope","test_id":"rope_test","status":"pass","duration_ms":10}"#,
                "\n"
            ),
        );
        let check = check_roster_execution_coverage(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn roster_coverage_fails_when_entry_unexecuted() {
        let tmp = tempdir().unwrap();
        let c = contract_with_provable("T1", "contracts/k.yaml", "rope", "rope_test");
        write_contract_json(tmp.path(), "T1", &c);
        // Log covers a DIFFERENT test_id
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"ts":"2026-04-18T00:00:00Z","yaml":"contracts/k.yaml","equation":"rope","test_id":"other_test","status":"pass"}"#,
                "\n"
            ),
        );
        let check = check_roster_execution_coverage(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("T1"));
        assert!(check.message.contains("rope_test"));
        assert!(check.message.contains("pmat work falsify"));
    }

    #[test]
    fn roster_coverage_ignores_malformed_log_lines() {
        let tmp = tempdir().unwrap();
        let c = contract_with_provable("T1", "contracts/k.yaml", "rope", "rope_test");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                "not json at all\n",
                r#"{"missing_fields": true}"#,
                "\n",
                r#"{"yaml":"contracts/k.yaml","test_id":"rope_test","status":"pass"}"#,
                "\n",
            ),
        );
        let check = check_roster_execution_coverage(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn roster_coverage_per_ticket_skip_when_only_some_have_log() {
        // T1 has a log; T2 doesn't. T2 is silently skipped.
        let tmp = tempdir().unwrap();
        let c1 = contract_with_provable("T1", "contracts/k.yaml", "rope", "rope_test");
        let c2 = contract_with_provable("T2", "contracts/k.yaml", "rope", "another_test");
        write_contract_json(tmp.path(), "T1", &c1);
        write_contract_json(tmp.path(), "T2", &c2);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"yaml":"contracts/k.yaml","test_id":"rope_test","status":"pass"}"#,
                "\n"
            ),
        );
        let check = check_roster_execution_coverage(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
        assert!(check.message.contains("1 ticket"));
    }

    #[test]
    fn parse_falsification_log_extracts_valid_lines() {
        let input = concat!(
            r#"{"yaml":"a.yaml","test_id":"t1","status":"pass"}"#,
            "\n",
            "\n",
            r#"{"source":"manual","method":"TdgRegression","status":"pass"}"#,
            "\n",
            r#"{"yaml":"b.yaml","test_id":"t2","status":"fail"}"#,
            "\n",
        );
        let parsed = parse_falsification_log(input);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0], (PathBuf::from("a.yaml"), "t1".into()));
        assert_eq!(parsed[1], (PathBuf::from("b.yaml"), "t2".into()));
    }

    #[test]
    fn roster_coverage_skips_without_bindings() {
        let tmp = tempdir().unwrap();
        let check = check_inherited_roster_coverage(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn duplicate_entries_skips_without_provable_contract() {
        let tmp = tempdir().unwrap();
        let check = check_no_duplicate_provable_entries(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_id_exists_skips_without_provable_contract() {
        let tmp = tempdir().unwrap();
        let check = check_test_id_exists_in_yaml(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn provable_contract_variant_round_trips_through_serde() {
        let m = FalsificationMethod::ProvableContract {
            yaml_path: PathBuf::from("contracts/rope-kernel-v1.yaml"),
            equation: "rope".into(),
            test_id: "rope_periodicity_test".into(),
            expected: "1.0".into(),
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: FalsificationMethod = serde_json::from_str(&json).unwrap();
        match back {
            FalsificationMethod::ProvableContract {
                yaml_path,
                equation,
                test_id,
                expected,
            } => {
                assert_eq!(yaml_path, PathBuf::from("contracts/rope-kernel-v1.yaml"));
                assert_eq!(equation, "rope");
                assert_eq!(test_id, "rope_periodicity_test");
                assert_eq!(expected, "1.0");
            }
            other => panic!("round-trip landed on wrong variant: {:?}", other),
        }
    }

    // ── CB-1625 inherited failure fatal tests ────────────────────────────

    #[test]
    fn inherited_failure_skips_when_no_logs() {
        let tmp = tempdir().unwrap();
        let c = contract_at_level("T1", "L3");
        write_contract_json(tmp.path(), "T1", &c);
        let check = check_inherited_failure_fatal(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("falsification.log"));
    }

    #[test]
    fn inherited_failure_skips_when_no_contracts_at_all() {
        let tmp = tempdir().unwrap();
        let check = check_inherited_failure_fatal(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn inherited_failure_passes_when_all_inherited_pass() {
        let tmp = tempdir().unwrap();
        let c = contract_at_level("T1", "L1"); // even L1 counts
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"yaml":"a.yaml","test_id":"t1","status":"pass","duration_ms":2}"#,
                "\n",
                r#"{"yaml":"a.yaml","test_id":"t2","status":"pass","duration_ms":4}"#,
                "\n",
            ),
        );
        let check = check_inherited_failure_fatal(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
        assert!(check.message.contains("2 inherited"));
    }

    #[test]
    fn inherited_failure_fails_on_inherited_fail() {
        let tmp = tempdir().unwrap();
        let c = contract_at_level("T1", "L1");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                r#"{"yaml":"rope.yaml","test_id":"t1","status":"pass","duration_ms":2}"#,
                "\n",
                r#"{"yaml":"rope.yaml","test_id":"t2","status":"fail","duration_ms":9}"#,
                "\n",
            ),
        );
        let check = check_inherited_failure_fatal(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("T1:2"));
        assert!(check.message.contains("rope.yaml::t2"));
        assert!(check.message.contains("status=fail"));
    }

    #[test]
    fn inherited_failure_fails_on_timeout_too() {
        let tmp = tempdir().unwrap();
        let c = contract_at_level("T1", "L3");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            r#"{"yaml":"k.yaml","test_id":"t","status":"timeout","duration_ms":60000}"#,
        );
        let check = check_inherited_failure_fatal(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("status=timeout"));
    }

    #[test]
    fn inherited_failure_ignores_manual_lines() {
        let tmp = tempdir().unwrap();
        let c = contract_at_level("T1", "L1");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                // Manual line (method, not yaml/test_id) — ignored even though it "fails"
                r#"{"method":"UnitTest","status":"fail","duration_ms":2}"#,
                "\n",
                // Inherited line — all pass
                r#"{"yaml":"a.yaml","test_id":"t","status":"pass","duration_ms":1}"#,
                "\n",
            ),
        );
        let check = check_inherited_failure_fatal(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn inherited_failure_ignores_malformed_lines() {
        let tmp = tempdir().unwrap();
        let c = contract_at_level("T1", "L1");
        write_contract_json(tmp.path(), "T1", &c);
        write_log(
            tmp.path(),
            "T1",
            concat!(
                "not-json\n",
                r#"{"yaml":"a.yaml","test_id":"t","status":"pass","duration_ms":1}"#,
                "\n",
            ),
        );
        // Malformed lines belong to CB-1628 — 1625 stays Pass here
        let check = check_inherited_failure_fatal(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn inherited_failure_counts_failures_across_tickets() {
        let tmp = tempdir().unwrap();
        let c1 = contract_at_level("T1", "L1");
        let c2 = contract_at_level("T2", "L1");
        write_contract_json(tmp.path(), "T1", &c1);
        write_contract_json(tmp.path(), "T2", &c2);
        write_log(
            tmp.path(),
            "T1",
            r#"{"yaml":"a.yaml","test_id":"t1","status":"fail","duration_ms":1}"#,
        );
        write_log(
            tmp.path(),
            "T2",
            r#"{"yaml":"b.yaml","test_id":"t2","status":"fail","duration_ms":1}"#,
        );
        let check = check_inherited_failure_fatal(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("2 inherited"));
        assert!(check.message.contains("T1"));
        assert!(check.message.contains("T2"));
    }

    // ─── CB-1624: no manual deletion of inherited entries ────────────────────

    fn write_roster_mutations(project: &Path, body: &str) {
        let dir = project.join(".pmat-work").join("ledger");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("roster-mutations.json"), body).unwrap();
    }

    #[test]
    fn manual_deletion_skips_without_pmat_work_dir() {
        let tmp = tempdir().unwrap();
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/`"));
    }

    #[test]
    fn manual_deletion_skips_without_ledger() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat-work")).unwrap();
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("roster-mutations.json"));
    }

    #[test]
    fn manual_deletion_passes_on_empty_ledger() {
        let tmp = tempdir().unwrap();
        write_roster_mutations(tmp.path(), "[]");
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass);
        assert!(r.message.contains("empty"));
    }

    #[test]
    fn manual_deletion_passes_when_no_delete_entries() {
        let tmp = tempdir().unwrap();
        write_roster_mutations(
            tmp.path(),
            r#"[
                {"ticket":"T-1","action":"add","target":{"yaml":"a.yaml","test_id":"t1"}},
                {"ticket":"T-1","action":"update","target":{"yaml":"a.yaml","test_id":"t2"}}
            ]"#,
        );
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass);
        assert!(r.message.contains("0 deletion"));
    }

    #[test]
    fn manual_deletion_passes_when_delete_is_via_unbind() {
        let tmp = tempdir().unwrap();
        write_roster_mutations(
            tmp.path(),
            r#"[
                {"ticket":"T-1","action":"delete","target":{"yaml":"a.yaml","test_id":"t1"},"via_unbind":true},
                {"ticket":"T-2","action":"delete","target":{"yaml":"b.yaml","test_id":"t2"},"via_unbind":true}
            ]"#,
        );
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass);
        assert!(r.message.contains("2 deletion"));
        assert!(r.message.contains("via `pmat work unbind`"));
    }

    #[test]
    fn manual_deletion_fails_on_delete_without_via_unbind_flag() {
        let tmp = tempdir().unwrap();
        write_roster_mutations(
            tmp.path(),
            r#"[
                {"ticket":"T-1","action":"delete","target":{"yaml":"a.yaml","test_id":"t1"}}
            ]"#,
        );
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
        assert!(r.message.contains("bypassed"));
    }

    #[test]
    fn manual_deletion_fails_when_via_unbind_is_false() {
        let tmp = tempdir().unwrap();
        write_roster_mutations(
            tmp.path(),
            r#"[
                {"ticket":"T-1","action":"delete","target":{"yaml":"a.yaml","test_id":"t1"},"via_unbind":false}
            ]"#,
        );
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
    }

    #[test]
    fn manual_deletion_detects_capitalized_action_variants() {
        let tmp = tempdir().unwrap();
        write_roster_mutations(
            tmp.path(),
            r#"[
                {"ticket":"T-1","action":"Delete","target":{"yaml":"a.yaml","test_id":"t1"}},
                {"ticket":"T-2","action":"DELETED","target":{"yaml":"b.yaml","test_id":"t2"}},
                {"ticket":"T-3","action":"deletion","target":{"yaml":"c.yaml","test_id":"t3"}}
            ]"#,
        );
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("3 manual deletion"));
    }

    #[test]
    fn manual_deletion_mixed_ledger_reports_only_offenders() {
        let tmp = tempdir().unwrap();
        write_roster_mutations(
            tmp.path(),
            r#"[
                {"ticket":"T-1","action":"add","target":{"yaml":"a.yaml","test_id":"t1"}},
                {"ticket":"T-2","action":"delete","target":{"yaml":"b.yaml","test_id":"t2"},"via_unbind":true},
                {"ticket":"T-3","action":"delete","target":{"yaml":"c.yaml","test_id":"t3"}}
            ]"#,
        );
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("1 manual deletion"));
        assert!(r.message.contains("T-3"));
        assert!(!r.message.contains("T-2"));
    }

    #[test]
    fn manual_deletion_fails_on_malformed_json() {
        let tmp = tempdir().unwrap();
        write_roster_mutations(tmp.path(), "{not-json");
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("not valid JSON"));
    }

    #[test]
    fn manual_deletion_fails_on_non_array_ledger() {
        let tmp = tempdir().unwrap();
        write_roster_mutations(tmp.path(), r#"{"ticket":"T-1"}"#);
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("JSON array"));
    }

    #[test]
    fn manual_deletion_handles_missing_target_field() {
        let tmp = tempdir().unwrap();
        write_roster_mutations(
            tmp.path(),
            r#"[
                {"ticket":"T-1","action":"delete"}
            ]"#,
        );
        let r = check_no_manual_deletion(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
    }

    // ─── CB-1627: post-bind YAML drift ───────────────────────────────────────

    fn write_bound_contract(
        project: &Path,
        ticket: &str,
        yaml_rel: &str,
        equation: &str,
        seeded_test_ids: &[&str],
    ) {
        use crate::cli::handlers::work_contract::{
            ContractBinding, EvidenceType, FalsifiableClaim,
        };
        let mut c = WorkContract::new(ticket.into(), "deadbeef".into());
        c.implements.push(ContractBinding {
            contract: "k".into(),
            equation: equation.into(),
            file: PathBuf::from(yaml_rel),
            sha: "abc".into(),
            bound_at: chrono::Utc::now(),
        });
        for id in seeded_test_ids {
            c.claims.push(FalsifiableClaim {
                hypothesis: "inherited".into(),
                falsification_method: FalsificationMethod::ProvableContract {
                    yaml_path: PathBuf::from(yaml_rel),
                    equation: equation.into(),
                    test_id: (*id).into(),
                    expected: "\"canonical\"".into(),
                },
                evidence_required: EvidenceType::BooleanCheck(true),
                result: None,
                override_info: None,
            });
        }
        write_contract_json(project, ticket, &c);
    }

    fn write_yaml_at(project: &Path, rel: &str, body: &str) {
        let p = project.join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&p, body).unwrap();
    }

    fn yaml_with_tests(ids: &[&str]) -> String {
        let mut s = String::from("falsification_tests:\n");
        for id in ids {
            s.push_str(&format!("  - id: {}\n", id));
        }
        s
    }

    #[test]
    fn post_bind_drift_skips_without_pmat_work() {
        let tmp = tempdir().unwrap();
        let r = check_post_bind_yaml_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/`"));
    }

    #[test]
    fn post_bind_drift_skips_with_no_contracts() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat-work")).unwrap();
        let r = check_post_bind_yaml_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("contract.json"));
    }

    #[test]
    fn post_bind_drift_skips_without_bindings() {
        let tmp = tempdir().unwrap();
        let c = WorkContract::new("T-1".into(), "deadbeef".into());
        write_contract_json(tmp.path(), "T-1", &c);
        let r = check_post_bind_yaml_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("`implements:` bindings"));
    }

    #[test]
    fn post_bind_drift_skips_when_yaml_missing() {
        let tmp = tempdir().unwrap();
        write_bound_contract(tmp.path(), "T-1", "contracts/k.yaml", "rope", &["t1"]);
        // YAML file does not exist on disk
        let r = check_post_bind_yaml_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("falsification_tests"));
    }

    #[test]
    fn post_bind_drift_skips_when_yaml_has_no_falsification_tests() {
        let tmp = tempdir().unwrap();
        write_bound_contract(tmp.path(), "T-1", "contracts/k.yaml", "rope", &["t1"]);
        write_yaml_at(tmp.path(), "contracts/k.yaml", "equations:\n  rope: {}\n");
        let r = check_post_bind_yaml_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }

    #[test]
    fn post_bind_drift_passes_when_roster_matches_yaml() {
        let tmp = tempdir().unwrap();
        write_bound_contract(tmp.path(), "T-1", "contracts/k.yaml", "rope", &["t1", "t2"]);
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            &yaml_with_tests(&["t1", "t2"]),
        );
        let r = check_post_bind_yaml_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn post_bind_drift_passes_when_roster_superset_of_yaml() {
        // Deletion from YAML is CB-1626's concern (stale roster references),
        // not CB-1627's — this check only flags ADDITIONS post-bind.
        let tmp = tempdir().unwrap();
        write_bound_contract(
            tmp.path(),
            "T-1",
            "contracts/k.yaml",
            "rope",
            &["t1", "t2", "t3"],
        );
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            &yaml_with_tests(&["t1", "t2"]),
        );
        let r = check_post_bind_yaml_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass);
    }

    #[test]
    fn post_bind_drift_warns_on_new_yaml_entry() {
        let tmp = tempdir().unwrap();
        write_bound_contract(tmp.path(), "T-1", "contracts/k.yaml", "rope", &["t1"]);
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            &yaml_with_tests(&["t1", "t2"]),
        );
        let r = check_post_bind_yaml_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Warn);
        assert!(r.message.contains("T-1"));
        assert!(r.message.contains("t2"));
    }

    #[test]
    fn post_bind_drift_warns_listing_multiple_additions() {
        let tmp = tempdir().unwrap();
        write_bound_contract(tmp.path(), "T-1", "contracts/k.yaml", "rope", &["t1"]);
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            &yaml_with_tests(&["t1", "t2", "t3"]),
        );
        let r = check_post_bind_yaml_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Warn);
        assert!(r.message.contains("t2"));
        assert!(r.message.contains("t3"));
        assert!(r.message.contains("2 new"));
    }

    #[test]
    fn post_bind_drift_is_per_binding_equation_scoped() {
        // A ProvableContract for equation `rope` should not mask drift
        // for equation `linear` in the same YAML.
        let tmp = tempdir().unwrap();
        use crate::cli::handlers::work_contract::{
            ContractBinding, EvidenceType, FalsifiableClaim,
        };
        let mut c = WorkContract::new("T-1".into(), "deadbeef".into());
        c.implements.push(ContractBinding {
            contract: "k".into(),
            equation: "linear".into(),
            file: PathBuf::from("contracts/k.yaml"),
            sha: "abc".into(),
            bound_at: chrono::Utc::now(),
        });
        // Claim is for equation `rope`, NOT `linear`.
        c.claims.push(FalsifiableClaim {
            hypothesis: "inherited".into(),
            falsification_method: FalsificationMethod::ProvableContract {
                yaml_path: PathBuf::from("contracts/k.yaml"),
                equation: "rope".into(),
                test_id: "t1".into(),
                expected: "\"\"".into(),
            },
            evidence_required: EvidenceType::BooleanCheck(true),
            result: None,
            override_info: None,
        });
        write_contract_json(tmp.path(), "T-1", &c);
        write_yaml_at(tmp.path(), "contracts/k.yaml", &yaml_with_tests(&["t1"]));
        // From the `linear` binding's perspective, seeded set is EMPTY — so
        // YAML entry `t1` shows up as drift.
        let r = check_post_bind_yaml_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Warn);
        assert!(r.message.contains("linear"));
    }

    #[test]
    fn post_bind_drift_aggregates_across_tickets() {
        let tmp = tempdir().unwrap();
        write_bound_contract(tmp.path(), "T-1", "contracts/a.yaml", "eq", &["t1"]);
        write_bound_contract(tmp.path(), "T-2", "contracts/b.yaml", "eq", &["s1"]);
        write_yaml_at(
            tmp.path(),
            "contracts/a.yaml",
            &yaml_with_tests(&["t1", "t2"]),
        );
        write_yaml_at(tmp.path(), "contracts/b.yaml", &yaml_with_tests(&["s1"]));
        let r = check_post_bind_yaml_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Warn);
        assert!(r.message.contains("T-1"));
        assert!(!r.message.contains("T-2"));
    }
}
