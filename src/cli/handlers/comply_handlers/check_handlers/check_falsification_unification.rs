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
//
// Implementation is split across sibling files via `include!()`:
//   * _roster.rs            CB-1620, CB-1623, CB-1626
//   * _snapshot.rs          CB-1621 + yaml-scalar helpers
//   * _execution.rs         CB-1622, CB-1625 + parse_falsification_log
//   * _deletion.rs          CB-1624
//   * _post_bind.rs         CB-1627 + is_inherited_receipt helper
//   * _log_line.rs          CB-1628
//   * _l4_timeout.rs        CB-1629 + is_l4_or_higher helper
//   * _tests_shared.rs      Cross-CB test fixtures (module-scope, #[cfg(test)])
//   * _tests_<group>.rs     One test module per CB group.

use std::path::{Path, PathBuf};

use chrono::{NaiveDate, Utc};

use super::types::*;
use crate::cli::handlers::work_contract::{FalsificationMethod, WorkContract};
use crate::cli::handlers::work_verification_level::VerificationLevel;

// ─── Shared production helpers ───────────────────────────────────────────────

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

// ─── Per-CB check implementations ────────────────────────────────────────────

include!("check_falsification_unification_roster.rs");
include!("check_falsification_unification_snapshot.rs");
include!("check_falsification_unification_execution.rs");
include!("check_falsification_unification_deletion.rs");
include!("check_falsification_unification_post_bind.rs");
include!("check_falsification_unification_log_line.rs");
include!("check_falsification_unification_l4_timeout.rs");

// ─── Tests ───────────────────────────────────────────────────────────────────

include!("check_falsification_unification_tests_shared.rs");
include!("check_falsification_unification_tests_roster.rs");
include!("check_falsification_unification_tests_snapshot.rs");
include!("check_falsification_unification_tests_execution.rs");
include!("check_falsification_unification_tests_deletion.rs");
include!("check_falsification_unification_tests_post_bind.rs");
include!("check_falsification_unification_tests_log_line.rs");
include!("check_falsification_unification_tests_l4_timeout.rs");
