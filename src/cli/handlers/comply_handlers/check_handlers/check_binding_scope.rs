// Work Contract Binding enforcement checks (CB-1600..1609) — Component 27
//
// Sub-spec: docs/specifications/components/pmat-work-contract-binding.md
//
// Each work ticket's contract.json may declare `implements: Vec<ContractBinding>`.
// These checks audit that the declared bindings remain internally consistent and
// externally anchored against the provable-contracts YAML files they cite.
//
// This first cut implements the checks that can run against today's infrastructure:
//
//   CB-1600 (L1) — orphan detection: staged files w/ bindings → active ticket
//                  must declare `implements:` covering them
//   CB-1601 (L1) — SHA drift against current YAML bytes
//   CB-1602 (L1) — unbind ledger: every `.pmat-work/ledger/unbinds.json`
//                  entry must carry a DEBT ticket reference (skip-if-absent)
//   CB-1603 (L3) — inherited clause integrity: contract.require contains each
//                  bound equation's YAML-declared preconditions
//   CB-1604 (L2) — postcondition weakening: any ticket with
//                  `inherited_postconditions` must preserve them in
//                  `ensure:` (strengthened or equal, never weakened or
//                  dropped). Delegates to `validate_subcontracting()`.
//   CB-1605 (L4) — per-binding kani_harnesses[] declared in YAML each appear
//                  with success: true in `.pmat-work/<ID>/kani-report.json`.
//                  Skip-if-absent (no YAML harness decls OR no reports yet).
//   CB-1606 (L5) — bindings whose YAML `lean_theorem.status != "proved"`
//                  require a BLOCK-ON-PROOF follow-up referenced in the
//                  ticket's contract.json. Skip-if-absent (no lean_theorem:
//                  blocks anywhere).
//   CB-1607 (L3) — equation identifier exists in referenced YAML
//   CB-1608 (L1) — cross-binding consistency: multi-bind tickets cannot
//                  mix passing and failing per-binding falsification log
//                  entries (skip-if-absent on log)
//   CB-1609 (L1) — YAML file is tracked in git
//
// All CB-16xx binding-scope checks are now functional (no deferred stubs).
//
// Per-check implementations live in semantically-named partition files that
// are `include!`'d at the bottom of this module. See:
//   - check_binding_scope_sha_drift.rs          (CB-1601)
//   - check_binding_scope_equation.rs           (CB-1607)
//   - check_binding_scope_file_tracked.rs       (CB-1609)
//   - check_binding_scope_orphan.rs             (CB-1600)
//   - check_binding_scope_unbind_audit.rs       (CB-1602)
//   - check_binding_scope_inherited.rs          (CB-1603 + CB-1604)
//   - check_binding_scope_kani.rs               (CB-1605)
//   - check_binding_scope_lean_theorem.rs       (CB-1606)
//   - check_binding_scope_cross_consistency.rs  (CB-1608)
//
// Tests for each live in matching `check_binding_scope_tests_<group>.rs`
// partitions, each wrapped in a uniquely-named `mod tests_<group>` block.

use std::path::Path;

use sha2::{Digest, Sha256};

use super::types::*;
use crate::cli::handlers::work_contract::{ContractBinding, WorkContract};
use crate::services::contract_index::ContractIndex;

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Walk `.pmat-work/<ID>/contract.json` and load each work contract.
/// Bindings from each contract feed the per-ticket checks below.
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

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let d = h.finalize();
    let mut s = String::with_capacity(d.len() * 2);
    for b in d {
        use std::fmt::Write;
        let _ = write!(&mut s, "{:02x}", b);
    }
    s
}

fn skip_no_bindings(name: &str) -> ComplianceCheck {
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Skip,
        message: "No `.pmat-work/*/contract.json` with `implements:` entries".into(),
        severity: Severity::Info,
    }
}

fn iter_bindings<'a>(
    contracts: &'a [WorkContract],
) -> impl Iterator<Item = (&'a str, &'a ContractBinding)> + 'a {
    contracts.iter().flat_map(|c| {
        c.implements
            .iter()
            .map(move |b| (c.work_item_id.as_str(), b))
    })
}

// ─── Per-check partitions ────────────────────────────────────────────────────

include!("check_binding_scope_sha_drift.rs");
include!("check_binding_scope_equation.rs");
include!("check_binding_scope_file_tracked.rs");
include!("check_binding_scope_orphan.rs");
include!("check_binding_scope_unbind_audit.rs");
include!("check_binding_scope_inherited.rs");
include!("check_binding_scope_kani.rs");
include!("check_binding_scope_lean_theorem.rs");
include!("check_binding_scope_cross_consistency.rs");

// ─── Test partitions ─────────────────────────────────────────────────────────

include!("check_binding_scope_tests_sha_equation.rs");
include!("check_binding_scope_tests_postcondition.rs");
include!("check_binding_scope_tests_unbind_audit.rs");
include!("check_binding_scope_tests_inherited.rs");
include!("check_binding_scope_tests_orphan.rs");
include!("check_binding_scope_tests_kani.rs");
include!("check_binding_scope_tests_lean_theorem.rs");
include!("check_binding_scope_tests_cross_consistency.rs");
