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
//   CB-1612 (L3) — L1 test evidence: verification-report.json must carry
//                  `l1_test_evidence` with success/exit-code/status shape;
//                  skip-if-absent until `pmat work verify` records it
//   CB-1617 (L3) — downgrade without `--reason` forbidden (ledger audit)
//   CB-1618 (L1) — level monotonicity across ticket checkpoints — a ticket
//                  cannot drop without an audited downgrade ledger entry
//                  (skip-if-absent until checkpoint writer records level)
//   CB-1619 (L3) — on completion, achieved level == target level
//   CB-1615 (L4) — Kani harness SHA matches bind-time snapshot: bind-time
//                  `.pmat-work/<ID>/kani-harness-shas.json` captures each
//                  `kani_harnesses[]` body hash; current YAML per-harness
//                  `sha:` field must match. Skip-if-absent until Component
//                  27 bind step writes the snapshot file.
//
// Deferred (scaffolded with Skip + reason, infrastructure pending):
//
//   (none — all CB-16xx ladder checks are functional as of this commit)
//
// File layout (split via `include!()` to keep each partition under 600 lines):
//
//   check_work_ladder_declaration.rs       — CB-1610, CB-1611
//   check_work_ladder_audit.rs             — CB-1617, CB-1619
//   check_work_ladder_l1_evidence.rs       — CB-1612
//   check_work_ladder_l3_falsification.rs  — CB-1613
//   check_work_ladder_l4_kani.rs           — CB-1614
//   check_work_ladder_kani_sha.rs          — CB-1615
//   check_work_ladder_l5_lean.rs           — CB-1616
//   check_work_ladder_monotonicity.rs      — CB-1618
//   check_work_ladder_tests_<group>.rs     — grouped test partitions

use std::path::Path;

use super::types::*;
use crate::cli::handlers::work_contract::WorkContract;
use crate::cli::handlers::work_verification_level::VerificationLevel;

// ─── Shared helpers ──────────────────────────────────────────────────────────

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

// ─── Shared test helpers ─────────────────────────────────────────────────────
//
// Defined at module scope (not inside a `mod tests`) so every included test
// partition — each of which nests a `mod tests_<group> { use super::*; … }` —
// can reach them via the standard `super` chain without duplicating the
// constructors.

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
fn write_yaml(project: &Path, name: &str, body: &str) -> std::path::PathBuf {
    let dir = project.join("contracts");
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join(format!("{}.yaml", name));
    std::fs::write(&p, body).unwrap();
    p
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
fn make_contract(id: &str, level: &str) -> WorkContract {
    let mut c = WorkContract::new(id.to_string(), "deadbeef".to_string());
    c.verification_level = level.to_string();
    c
}

// ─── Production partitions ───────────────────────────────────────────────────

include!("check_work_ladder_declaration.rs");
include!("check_work_ladder_audit.rs");
include!("check_work_ladder_l1_evidence.rs");
include!("check_work_ladder_l3_falsification.rs");
include!("check_work_ladder_l4_kani.rs");
include!("check_work_ladder_kani_sha.rs");
include!("check_work_ladder_l5_lean.rs");
include!("check_work_ladder_monotonicity.rs");

// ─── Test partitions ─────────────────────────────────────────────────────────

include!("check_work_ladder_tests_declaration.rs");
include!("check_work_ladder_tests_kani_sha.rs");
include!("check_work_ladder_tests_monotonicity.rs");
include!("check_work_ladder_tests_levels.rs");
include!("check_work_ladder_tests_l1_evidence.rs");
