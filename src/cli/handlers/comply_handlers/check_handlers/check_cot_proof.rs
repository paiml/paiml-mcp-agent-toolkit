// Work Chain-of-Thought Proof Derivation checks (CB-1640..1649) — Component 31
//
// Sub-spec: docs/specifications/components/pmat-work-cot-proof-derivation.md
//
// Today's `ChainOfThoughtStep` is prose-only — `{ step, question, answer }`.
// Component 31 restructures each step as a typed node with `assumption`,
// `implication`, `evidence_method`, and `discharged_by`, then auto-derives
// proof obligations, falsification claims, and require/ensure clauses.
//
// The schema migration is substantial (new Rust types, `pmat work cot`
// CLI surface, derivation pipeline) and is deliberately deferred: these
// checks read hypothetical structured fields via `serde_json::Value`
// introspection so they run clean against today's contracts and light up
// automatically once authors start emitting the new shape.
//
// Functional checks (work today against contract.json Value introspection):
//   CB-1640 (L3) — assumption.references resolve to prior step ids /
//                  implications, bound equation names, or axiomatic
//                  discharge (exact-string fallback per spec §Chain
//                  Integrity Rule when semantic-search vocabulary absent)
//   CB-1641 (L3) — every step with structured fields has an `evidence_method`
//   CB-1642 (L1) — `evidence_method = ExistingTest` path/name resolves on disk
//   CB-1643 (L3) — L3+ tickets: every structured step has `assumption.expr`
//                  or `implication.expr`
//   CB-1644 (L1) — `.pmat-work/<ID>/agent-runs/<run_id>.json` entries carry
//                  the replay schema (`prompt_sha`, `tool_calls`, `commit_sha`).
//                  Skip-if-absent until Component 10 writer lands.
//   CB-1645 (L3) — derived `contracts/work/<ID>.yaml` is up-to-date with
//                  contract.json preconditions/postconditions
//   CB-1646 (L1) — `.pmat-work/<ID>/cot-digest.json` SHA matches the
//                  canonical hash of `chain_of_thought` — detects manual
//                  edits that bypass `pmat work cot derive`. Skip-if-absent.
//   CB-1647 (L3) — no orphan steps: every step chains via `discharged_by`
//   CB-1648 (L4) — every `Axiomatic` discharge in an L4+ ticket is either
//                  a bound equation invariant (reason/lemma matches an
//                  `implements:` equation name) or a documented lemma
//                  (non-empty `reason` prose). Skip-if-absent.
//   CB-1649 (L5) — every structured step in an L5 ticket carries a Lean
//                  theorem/lemma mapping via `lean_theorem`, `lean_lemma`,
//                  `evidence_method.LeanTheorem`/`LeanLemma`, or
//                  `discharged_by.Lean`. Skip-if-absent.
//
// All CB-164x checks are now active (skip-if-absent).
//
// ── Layout ──────────────────────────────────────────────────────────────────
// This file is the entry point. Per-check implementations and tests live in
// semantically named sibling files, brought in via `include!()` so they share
// this module's `use` imports without re-declaring them.

use std::path::Path;

use serde_json::Value;
use sha2::{Digest, Sha256};

use super::types::*;

// ─── Helpers (shared across partitions) ──────────────────────────────────────

/// Load every `.pmat-work/<ID>/contract.json` as a raw `Value`. Schema-agnostic —
/// we look up fields by name so contracts built on the legacy `{ step, question,
/// answer }` shape round-trip without panicking on missing new fields.
fn load_contract_values(project_path: &Path) -> Vec<(String, Value)> {
    let dir = project_path.join(".pmat-work");
    if !dir.exists() {
        return Vec::new();
    }
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
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
        let path = entry.path().join("contract.json");
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        let Ok(value) = serde_json::from_slice::<Value>(&bytes) else {
            continue;
        };
        out.push((ticket_id, value));
    }
    out
}

/// Extract the `chain_of_thought` array from a contract Value, if present.
fn cot_steps(contract: &Value) -> &[Value] {
    contract
        .get("chain_of_thought")
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[])
}

/// A step is "structured" if it carries any of the Component 31 fields. Used to
/// gate checks so the legacy `{ step, question, answer }` shape doesn't trip
/// assertions meant for the new schema.
fn is_structured(step: &Value) -> bool {
    step.get("assumption").is_some()
        || step.get("implication").is_some()
        || step.get("evidence_method").is_some()
        || step.get("discharged_by").is_some()
}

fn step_id(step: &Value) -> String {
    step.get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            step.get("step")
                .and_then(|v| v.as_u64())
                .map(|n| format!("CoT-{}", n))
        })
        .unwrap_or_else(|| "CoT-?".to_string())
}

/// Parse a `verification_level` string (`"L3"`, `"L3 (kani_proof)"`, etc.) to
/// its numeric level. Unknown shapes return 0 so checks gate conservatively.
fn parse_level(value: &Value) -> u8 {
    let Some(s) = value.get("verification_level").and_then(|v| v.as_str()) else {
        return 0;
    };
    let trimmed = s.trim();
    let after_l = trimmed.strip_prefix('L').unwrap_or(trimmed);
    let digits: String = after_l.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse::<u8>().unwrap_or(0)
}

// ─── Per-check implementations ───────────────────────────────────────────────

include!("check_cot_proof_evidence.rs");
include!("check_cot_proof_structured_expr.rs");
include!("check_cot_proof_orphans.rs");
include!("check_cot_proof_references.rs");
include!("check_cot_proof_replay.rs");
include!("check_cot_proof_derived_yaml.rs");
include!("check_cot_proof_sha_fresh.rs");
include!("check_cot_proof_l4_axiomatic.rs");
include!("check_cot_proof_l5_lean.rs");

// ─── Tests ───────────────────────────────────────────────────────────────────

include!("check_cot_proof_tests_agent_run.rs");
include!("check_cot_proof_tests_schema.rs");
include!("check_cot_proof_tests_formal.rs");
