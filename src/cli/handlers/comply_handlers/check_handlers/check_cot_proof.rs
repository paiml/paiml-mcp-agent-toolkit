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
//   CB-1647 (L3) — no orphan steps: every step chains via `discharged_by`
//
// Deferred stubs (need infrastructure that hasn't landed):
//   CB-1644 (L1) — requires Component 10 agent audit log ingest
//   CB-1645 (L3) — requires `pmat work cot derive` emitting contracts/work/<ID>.yaml
//   CB-1646 (L1) — requires `cot-digest.json` SHA tracking
//   CB-1648 (L4) — requires Kani-bound axiom registry
//   CB-1649 (L5) — requires Lean theorem lemma mapping

use std::path::Path;

use serde_json::Value;

use super::types::*;

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn deferred(name: &str, reason: &str) -> ComplianceCheck {
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Skip,
        message: format!("Deferred — {}", reason),
        severity: Severity::Info,
    }
}

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

// ─── CB-1641 — Every structured step has evidence_method ─────────────────────

pub(crate) fn check_step_has_evidence_method(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1641: Step Has Evidence Method";
    let contracts = load_contract_values(project_path);
    let mut missing: Vec<String> = Vec::new();
    let mut structured_seen = 0usize;
    for (ticket, contract) in &contracts {
        for step in cot_steps(contract) {
            if !is_structured(step) {
                continue;
            }
            structured_seen += 1;
            if step.get("evidence_method").is_none() {
                missing.push(format!("{}:{}", ticket, step_id(step)));
            }
        }
    }
    if structured_seen == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No structured CoT steps found — migration pending".into(),
            severity: Severity::Info,
        };
    }
    if missing.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} structured step(s) declare evidence_method",
                structured_seen
            ),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!(
                "{} step(s) missing evidence_method: {}",
                missing.len(),
                missing.join(", ")
            ),
            severity: Severity::Error,
        }
    }
}

// ─── CB-1642 — ExistingTest evidence paths resolve on disk ───────────────────

pub(crate) fn check_existing_test_paths_resolve(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1642: Existing Test Path Resolves";
    let contracts = load_contract_values(project_path);
    let mut missing: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for (ticket, contract) in &contracts {
        for step in cot_steps(contract) {
            let Some(method) = step.get("evidence_method") else {
                continue;
            };
            let Some(existing) = method.get("ExistingTest") else {
                continue;
            };
            checked += 1;
            let path_str = existing
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("<no path>");
            let abs = project_path.join(path_str);
            if !abs.exists() {
                missing.push(format!("{}:{} -> {}", ticket, step_id(step), path_str));
            }
        }
    }
    if checked == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `ExistingTest` evidence method references found".into(),
            severity: Severity::Info,
        };
    }
    if missing.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!("{} ExistingTest reference(s) resolve on disk", checked),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!(
                "{} ExistingTest reference(s) missing on disk: {}",
                missing.len(),
                missing.join(", ")
            ),
            severity: Severity::Error,
        }
    }
}

// ─── CB-1643 — L3+ tickets: each step has assumption.expr or implication.expr ─

pub(crate) fn check_l3_structured_expr_present(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1643: L3+ Steps Have Expr";
    let contracts = load_contract_values(project_path);
    let mut missing: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for (ticket, contract) in &contracts {
        if parse_level(contract) < 3 {
            continue;
        }
        for step in cot_steps(contract) {
            if !is_structured(step) {
                continue;
            }
            checked += 1;
            let assumption_expr = step
                .get("assumption")
                .and_then(|a| a.get("expr"))
                .and_then(|e| e.as_str())
                .map(|s| !s.is_empty())
                .unwrap_or(false);
            let implication_expr = step
                .get("implication")
                .and_then(|a| a.get("expr"))
                .and_then(|e| e.as_str())
                .map(|s| !s.is_empty())
                .unwrap_or(false);
            if !assumption_expr && !implication_expr {
                missing.push(format!("{}:{}", ticket, step_id(step)));
            }
        }
    }
    if checked == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No L3+ ticket with structured CoT steps found".into(),
            severity: Severity::Info,
        };
    }
    if missing.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!("{} L3+ step(s) carry an expr field", checked),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!(
                "{} L3+ step(s) lack assumption.expr/implication.expr: {}",
                missing.len(),
                missing.join(", ")
            ),
            severity: Severity::Error,
        }
    }
}

// ─── CB-1647 — No orphan CoT steps (each chains to a discharge) ─────────────

pub(crate) fn check_no_orphan_steps(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1647: No Orphan CoT Steps";
    let contracts = load_contract_values(project_path);
    let mut orphans: Vec<String> = Vec::new();
    let mut structured_seen = 0usize;
    for (ticket, contract) in &contracts {
        for step in cot_steps(contract) {
            if !is_structured(step) {
                continue;
            }
            structured_seen += 1;
            let discharged = step
                .get("discharged_by")
                .map(|v| !v.is_null())
                .unwrap_or(false);
            if !discharged {
                orphans.push(format!("{}:{}", ticket, step_id(step)));
            }
        }
    }
    if structured_seen == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No structured CoT steps to check for orphans".into(),
            severity: Severity::Info,
        };
    }
    if orphans.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!("{} step(s) chain via discharged_by", structured_seen),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!(
                "{} orphan step(s) without discharged_by: {}",
                orphans.len(),
                orphans.join(", ")
            ),
            severity: Severity::Error,
        }
    }
}

// ─── CB-1640 — Assumption references resolve ────────────────────────────────

/// Collect every identifier an assumption can legitimately reference in a
/// single contract: prior step ids, prior implication predicates/exprs, and
/// the equation names of any declared `implements:` bindings. The check
/// consults this set per-step; references not in it become violations.
fn resolvable_references(contract: &Value, up_to_step_index: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let steps = cot_steps(contract);
    for prior in steps.iter().take(up_to_step_index) {
        if let Some(id) = prior.get("id").and_then(|v| v.as_str()) {
            out.push(id.to_string());
        }
        if let Some(pred) = prior
            .get("implication")
            .and_then(|i| i.get("predicate"))
            .and_then(|v| v.as_str())
        {
            out.push(pred.to_string());
        }
        if let Some(expr) = prior
            .get("implication")
            .and_then(|i| i.get("expr"))
            .and_then(|v| v.as_str())
        {
            out.push(expr.to_string());
        }
    }
    if let Some(implements) = contract.get("implements").and_then(|v| v.as_array()) {
        for binding in implements {
            if let Some(eq) = binding.get("equation").and_then(|v| v.as_str()) {
                out.push(eq.to_string());
            }
        }
    }
    out
}

/// A reference resolves if (a) a step with `discharged_by.Axiomatic` is
/// self-discharging regardless of its references (spec §Chain Integrity
/// Rule — "axiomatic discharge with explicit reason"), or (b) the string
/// appears in the resolvable set via exact match — the spec's mandated
/// fallback when the TF-IDF semantic-search vocabulary is unavailable.
fn is_axiomatic(step: &Value) -> bool {
    step.get("discharged_by")
        .and_then(|d| d.get("Axiomatic"))
        .is_some()
}

pub(crate) fn check_assumption_references_resolve(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1640: Assumption References Resolve";
    let contracts = load_contract_values(project_path);
    let mut unmatched: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for (ticket, contract) in &contracts {
        for (idx, step) in cot_steps(contract).iter().enumerate() {
            if !is_structured(step) {
                continue;
            }
            if is_axiomatic(step) {
                continue;
            }
            let Some(refs) = step
                .get("assumption")
                .and_then(|a| a.get("references"))
                .and_then(|r| r.as_array())
            else {
                continue;
            };
            if refs.is_empty() {
                continue;
            }
            let resolvable = resolvable_references(contract, idx);
            for reference in refs {
                let Some(reference_str) = reference.as_str() else {
                    continue;
                };
                checked += 1;
                if !resolvable.iter().any(|r| r == reference_str) {
                    unmatched.push(format!(
                        "{}:{} -> \"{}\"",
                        ticket,
                        step_id(step),
                        reference_str
                    ));
                }
            }
        }
    }
    if checked == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No structured CoT assumption references to resolve".into(),
            severity: Severity::Info,
        };
    }
    if unmatched.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} assumption reference(s) resolve via exact-match fallback",
                checked
            ),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!(
                "{} unmatched assumption reference(s): {}",
                unmatched.len(),
                unmatched.join(", ")
            ),
            severity: Severity::Error,
        }
    }
}

// ─── Deferred stubs ─────────────────────────────────────────────────────────

pub(crate) fn check_agent_run_replayable(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1644: Agent Run Replayable",
        "requires Component 10 agent audit log ingest (`.pmat-work/<ID>/agent-runs/<run_id>.json`)",
    )
}

pub(crate) fn check_derived_yaml_obligations_present(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1645: Derived YAML Obligations",
        "requires `pmat work cot derive` emitting contracts/work/<ID>.yaml with proof_obligations",
    )
}

pub(crate) fn check_cot_derivation_sha_fresh(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1646: CoT Derivation SHA",
        "requires `.pmat-work/<ID>/cot-digest.json` emitted by `pmat work cot derive`",
    )
}

pub(crate) fn check_l4_axiomatic_discharge_bounded(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1648: L4 Axiomatic Discharge Bounded",
        "requires Kani-bound axiom registry (lemma references per L4 ticket)",
    )
}

pub(crate) fn check_l5_lean_theorem_mapping(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1649: L5 Lean Theorem Mapping",
        "requires Component 24 Lean theorem lemma mapping per step",
    )
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(all(test, not(coverage_nightly)))]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    fn write_contract(project: &Path, ticket: &str, contract: Value) {
        let dir = project.join(".pmat-work").join(ticket);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("contract.json"),
            serde_json::to_string_pretty(&contract).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn deferred_stubs_return_skip_with_reason() {
        let project = tempdir().unwrap();
        for check in [
            check_agent_run_replayable(project.path()),
            check_derived_yaml_obligations_present(project.path()),
            check_cot_derivation_sha_fresh(project.path()),
            check_l4_axiomatic_discharge_bounded(project.path()),
            check_l5_lean_theorem_mapping(project.path()),
        ] {
            assert_eq!(check.status, CheckStatus::Skip);
            assert!(check.message.starts_with("Deferred — "));
        }
    }

    #[test]
    fn assumption_refs_skip_when_no_references_exist() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" }
                    }
                ]
            }),
        );
        let check = check_assumption_references_resolve(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn assumption_refs_pass_matching_prior_step_id() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "a" },
                        "implication": { "predicate": "b" }
                    },
                    {
                        "id": "CoT-2",
                        "assumption": { "predicate": "c", "references": ["CoT-1"] },
                        "implication": { "predicate": "d" }
                    }
                ]
            }),
        );
        let check = check_assumption_references_resolve(project.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn assumption_refs_pass_matching_prior_implication_predicate() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "implication": { "predicate": "array is sorted" }
                    },
                    {
                        "id": "CoT-2",
                        "assumption": {
                            "predicate": "search terminates",
                            "references": ["array is sorted"]
                        }
                    }
                ]
            }),
        );
        let check = check_assumption_references_resolve(project.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn assumption_refs_pass_matching_bound_equation() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "implements": [
                    { "equation": "rope_periodicity", "yaml_path": "rope.yaml" }
                ],
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": {
                            "predicate": "rope norm preserved",
                            "references": ["rope_periodicity"]
                        }
                    }
                ]
            }),
        );
        let check = check_assumption_references_resolve(project.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn assumption_refs_fail_on_unmatched_reference() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": {
                            "predicate": "p",
                            "references": ["nonexistent"]
                        }
                    }
                ]
            }),
        );
        let check = check_assumption_references_resolve(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("T1:CoT-1"));
        assert!(check.message.contains("nonexistent"));
    }

    #[test]
    fn assumption_refs_axiomatic_skipped_even_with_refs() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": {
                            "predicate": "p",
                            "references": ["nonexistent"]
                        },
                        "discharged_by": { "Axiomatic": { "reason": "base case" } }
                    }
                ]
            }),
        );
        let check = check_assumption_references_resolve(project.path());
        // Axiomatic discharge bypasses reference resolution (spec §Chain
        // Integrity Rule), so this should Skip not Fail.
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn assumption_refs_forward_reference_fails() {
        // CoT-1 references CoT-2 (not yet defined) → violation (only prior
        // steps count as resolvable).
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": {
                            "predicate": "p",
                            "references": ["CoT-2"]
                        }
                    },
                    {
                        "id": "CoT-2",
                        "implication": { "predicate": "q" }
                    }
                ]
            }),
        );
        let check = check_assumption_references_resolve(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
    }

    #[test]
    fn evidence_method_skip_without_structured_steps() {
        let project = tempdir().unwrap();
        // Legacy schema — no structured fields
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    { "step": 1, "question": "q", "answer": "a" },
                ]
            }),
        );
        let check = check_step_has_evidence_method(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn evidence_method_passes_when_all_present() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" },
                        "implication": { "predicate": "q" },
                        "evidence_method": { "ReviewOnly": { "reviewer_sha": "abc" } }
                    }
                ]
            }),
        );
        let check = check_step_has_evidence_method(project.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn evidence_method_fails_when_missing() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" },
                        "implication": { "predicate": "q" }
                    }
                ]
            }),
        );
        let check = check_step_has_evidence_method(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("T1:CoT-1"));
    }

    #[test]
    fn existing_test_fails_on_missing_path() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "evidence_method": {
                            "ExistingTest": {
                                "path": "nonexistent/path.rs",
                                "name": "test_foo"
                            }
                        }
                    }
                ]
            }),
        );
        let check = check_existing_test_paths_resolve(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
    }

    #[test]
    fn existing_test_passes_when_path_exists() {
        let project = tempdir().unwrap();
        let file = project.path().join("tests/real.rs");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "").unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "evidence_method": {
                            "ExistingTest": {
                                "path": "tests/real.rs",
                                "name": "test_foo"
                            }
                        }
                    }
                ]
            }),
        );
        let check = check_existing_test_paths_resolve(project.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn existing_test_skips_when_no_entries() {
        let project = tempdir().unwrap();
        write_contract(project.path(), "T1", json!({ "chain_of_thought": [] }));
        let check = check_existing_test_paths_resolve(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn l3_expr_passes_when_assumption_expr_present() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L3",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p", "expr": "x.is_finite()" },
                        "implication": { "predicate": "q" }
                    }
                ]
            }),
        );
        let check = check_l3_structured_expr_present(project.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn l3_expr_fails_when_neither_expr_present() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L3",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" },
                        "implication": { "predicate": "q" }
                    }
                ]
            }),
        );
        let check = check_l3_structured_expr_present(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
    }

    #[test]
    fn l3_expr_skips_for_sub_l3_tickets() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L1",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" }
                    }
                ]
            }),
        );
        let check = check_l3_structured_expr_present(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn orphan_steps_detected() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" }
                        // No discharged_by
                    }
                ]
            }),
        );
        let check = check_no_orphan_steps(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("T1:CoT-1"));
    }

    #[test]
    fn discharged_steps_pass() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" },
                        "discharged_by": { "Axiomatic": { "reason": "base case" } }
                    }
                ]
            }),
        );
        let check = check_no_orphan_steps(project.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn parse_level_handles_l_prefix() {
        assert_eq!(parse_level(&json!({ "verification_level": "L3" })), 3);
        assert_eq!(
            parse_level(&json!({ "verification_level": "L3 (kani_proof)" })),
            3
        );
        assert_eq!(parse_level(&json!({ "verification_level": "L5" })), 5);
        assert_eq!(parse_level(&json!({})), 0);
    }

    #[test]
    fn step_id_falls_back_to_numeric_step() {
        assert_eq!(step_id(&json!({ "id": "CoT-5" })), "CoT-5");
        assert_eq!(step_id(&json!({ "step": 2 })), "CoT-2");
        assert_eq!(step_id(&json!({})), "CoT-?");
    }

    #[test]
    fn load_contracts_skips_ledger_and_hidden() {
        let project = tempdir().unwrap();
        std::fs::create_dir_all(project.path().join(".pmat-work/ledger")).unwrap();
        std::fs::create_dir_all(project.path().join(".pmat-work/.hidden")).unwrap();
        write_contract(project.path(), "T1", json!({ "chain_of_thought": [] }));
        let loaded = load_contract_values(project.path());
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].0, "T1");
    }
}
