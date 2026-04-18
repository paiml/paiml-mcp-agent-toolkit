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

use std::path::Path;

use serde_json::Value;
use sha2::{Digest, Sha256};

use super::types::*;

// ─── Helpers ─────────────────────────────────────────────────────────────────

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

/// CB-1644 (L1): Replayability hinges on three fields per recorded agent run:
///   - `prompt_sha` — content hash of the prompt that produced the run
///   - `tool_calls` — ordered trace of tool invocations (array)
///   - `commit_sha` — git commit the run was anchored against
///
/// We scan `.pmat-work/<ID>/agent-runs/*.json`. Entries missing any required
/// field are reported. The check skips cleanly when no `agent-runs/` folder
/// exists for any ticket — Component 10's writer hasn't emitted traces yet.
pub(crate) fn check_agent_run_replayable(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1644: Agent Run Replayable";
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/` directory — agent run writer hasn't executed yet".into(),
            severity: Severity::Info,
        };
    }

    const REQUIRED_FIELDS: &[&str] = &["prompt_sha", "tool_calls", "commit_sha"];

    let mut checked_runs = 0usize;
    let mut malformed: Vec<String> = Vec::new();
    let mut incomplete: Vec<String> = Vec::new();

    let Ok(ticket_entries) = std::fs::read_dir(&work_dir) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "`.pmat-work/` unreadable — agent run writer hasn't executed yet".into(),
            severity: Severity::Info,
        };
    };

    for entry in ticket_entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let Some(ticket_id) = entry.file_name().to_str().map(|s| s.to_string()) else {
            continue;
        };
        if ticket_id.starts_with('.') || ticket_id == "ledger" {
            continue;
        }
        let runs_dir = entry.path().join("agent-runs");
        if !runs_dir.exists() {
            continue;
        }
        let Ok(run_files) = std::fs::read_dir(&runs_dir) else {
            continue;
        };
        for run in run_files.flatten() {
            let path = run.path();
            if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let run_id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("<unknown>")
                .to_string();
            checked_runs += 1;
            let Ok(bytes) = std::fs::read(&path) else {
                malformed.push(format!("{}:{}", ticket_id, run_id));
                continue;
            };
            let Ok(value) = serde_json::from_slice::<Value>(&bytes) else {
                malformed.push(format!("{}:{}", ticket_id, run_id));
                continue;
            };
            let Value::Object(map) = value else {
                malformed.push(format!("{}:{} (not an object)", ticket_id, run_id));
                continue;
            };
            let missing: Vec<&str> = REQUIRED_FIELDS
                .iter()
                .filter(|f| {
                    // Field is absent or null
                    map.get(**f).map(|v| v.is_null()).unwrap_or(true)
                })
                .copied()
                .collect();
            // `tool_calls` must specifically be an array, not any non-null shape
            let tool_calls_ok = map.get("tool_calls").map(|v| v.is_array()).unwrap_or(false);
            if !missing.is_empty() {
                incomplete.push(format!(
                    "{}:{} missing {}",
                    ticket_id,
                    run_id,
                    missing.join(", ")
                ));
            } else if !tool_calls_ok {
                incomplete.push(format!(
                    "{}:{} tool_calls is not an array",
                    ticket_id, run_id
                ));
            }
        }
    }

    if checked_runs == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/<ID>/agent-runs/*.json` files — Component 10 writer hasn't emitted runs yet".into(),
            severity: Severity::Info,
        };
    }

    if malformed.is_empty() && incomplete.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} agent run(s) carry prompt_sha/tool_calls/commit_sha",
                checked_runs
            ),
            severity: Severity::Info,
        };
    }

    let mut msg = String::new();
    if !malformed.is_empty() {
        msg.push_str(&format!(
            "{} unreadable run(s): {}\n",
            malformed.len(),
            malformed.join(", ")
        ));
    }
    if !incomplete.is_empty() {
        msg.push_str(&format!(
            "{} run(s) incomplete:\n  {}",
            incomplete.len(),
            incomplete.join("\n  ")
        ));
    }
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Fail,
        message: msg,
        severity: Severity::Error,
    }
}

/// Sanitize a work-item id for use as a filename — mirrors
/// `check_commit_enforcement_p8::generate_work_contract_yamls`.
fn sanitize_work_id(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Expected preconditions from a contract.json Value — mirrors the generator.
fn expected_preconditions(contract: &Value) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(claims) = contract
        .get("falsifiable_claims")
        .and_then(|c| c.as_array())
    {
        for c in claims {
            if let Some(s) = c.get("claim").and_then(|t| t.as_str()) {
                out.push(s.to_string());
            }
        }
    }
    if let Some(req) = contract.get("require").and_then(|r| r.as_array()) {
        for r in req {
            if let Some(s) = r.as_str() {
                out.push(s.to_string());
            }
        }
    }
    out
}

/// Expected postconditions from a contract.json Value — mirrors the generator.
fn expected_postconditions(contract: &Value) -> Vec<String> {
    contract
        .get("ensure")
        .and_then(|e| e.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

/// CB-1645 (L3): for each `.pmat-work/<ID>/contract.json`, the derived
/// `contracts/work/<sanitized_id>.yaml` must exist and reflect the contract's
/// current preconditions/postconditions. Catches stale derivation when a
/// ticket's clauses are edited without rerunning `pmat comply refresh-bindings`.
/// Skips cleanly when no `.pmat-work/` tickets exist.
pub(crate) fn check_derived_yaml_obligations_present(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1645: Derived YAML Obligations";
    let contracts = load_contract_values(project_path);
    if contracts.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/` tickets to cross-check".into(),
            severity: Severity::Info,
        };
    }

    let out_dir = project_path.join("contracts/work");
    let mut missing_yaml: Vec<String> = Vec::new();
    let mut stale: Vec<String> = Vec::new();
    let mut checked = 0usize;

    for (ticket, contract) in &contracts {
        let pre = expected_preconditions(contract);
        let post = expected_postconditions(contract);
        if pre.is_empty() && post.is_empty() {
            // Nothing derivable — skip this ticket
            continue;
        }
        checked += 1;

        let safe = sanitize_work_id(ticket);
        let yaml_path = out_dir.join(format!("{}.yaml", safe));
        let Ok(yaml) = std::fs::read_to_string(&yaml_path) else {
            missing_yaml.push(ticket.clone());
            continue;
        };

        for p in &pre {
            if !yaml.contains(p) {
                stale.push(format!("  {} missing precondition: {}", ticket, p));
            }
        }
        for p in &post {
            if !yaml.contains(p) {
                stale.push(format!("  {} missing postcondition: {}", ticket, p));
            }
        }
    }

    if checked == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No tickets declare derivable preconditions/postconditions".into(),
            severity: Severity::Info,
        };
    }

    if missing_yaml.is_empty() && stale.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} ticket(s) with derived YAML match current contract.json",
                checked
            ),
            severity: Severity::Info,
        };
    }

    let mut msg = String::new();
    if !missing_yaml.is_empty() {
        msg.push_str(&format!(
            "{} ticket(s) missing derived contracts/work/<ID>.yaml: {}\n",
            missing_yaml.len(),
            missing_yaml.join(", ")
        ));
    }
    if !stale.is_empty() {
        msg.push_str(&format!(
            "{} stale entry/entries — run `pmat comply refresh-bindings`:\n",
            stale.len()
        ));
        for s in &stale {
            msg.push_str(s);
            msg.push('\n');
        }
    }
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Fail,
        message: msg,
        severity: Severity::Error,
    }
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

/// Emit `v` into `out` as canonical JSON: object keys sorted lexicographically,
/// no whitespace, arrays preserve order. Downstream deps enable
/// `serde_json/preserve_order` (IndexMap), so raw `to_vec` would hash
/// differently depending on which author typed which key first — this
/// walker erases that non-determinism. Any RFC 8785-compatible producer
/// will agree on the bytes.
fn canonicalize(v: &Value, out: &mut String) {
    use std::fmt::Write;
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => {
            let _ = write!(out, "{}", n);
        }
        Value::String(s) => {
            // serde_json handles escape rules; we just borrow the String emitter.
            if let Ok(escaped) = serde_json::to_string(s) {
                out.push_str(&escaped);
            }
        }
        Value::Array(arr) => {
            out.push('[');
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                canonicalize(item, out);
            }
            out.push(']');
        }
        Value::Object(obj) => {
            let mut keys: Vec<&String> = obj.keys().collect();
            keys.sort();
            out.push('{');
            for (i, k) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                if let Ok(escaped) = serde_json::to_string(k) {
                    out.push_str(&escaped);
                }
                out.push(':');
                canonicalize(&obj[*k], out);
            }
            out.push('}');
        }
    }
}

/// Canonical SHA-256 of a contract's `chain_of_thought` array.
fn canonical_cot_sha(contract: &Value) -> String {
    let cot = contract
        .get("chain_of_thought")
        .cloned()
        .unwrap_or(Value::Null);
    let mut buf = String::new();
    canonicalize(&cot, &mut buf);
    sha256_hex(buf.as_bytes())
}

/// Read a recorded CoT digest from `cot-digest.json`. Accepts either a
/// `{"sha": "..."}` or `{"digest": "..."}` shape — both naming conventions
/// are plausible for the forthcoming `pmat work cot derive` output.
fn read_recorded_digest(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let v: Value = serde_json::from_slice(&bytes).ok()?;
    v.get("sha")
        .or_else(|| v.get("digest"))
        .and_then(|s| s.as_str())
        .map(|s| s.to_string())
}

/// CB-1646 (L1): recomputes the canonical SHA of each ticket's
/// `chain_of_thought` and compares it to the digest recorded in
/// `.pmat-work/<ID>/cot-digest.json`. A mismatch means the CoT was edited
/// by hand after `pmat work cot derive` last ran, which defeats the
/// derivation pipeline's witness trail. Skip-if-absent: tickets without a
/// digest file are ignored, and the check skips overall when no ticket
/// has one (the digest emitter hasn't run yet).
pub(crate) fn check_cot_derivation_sha_fresh(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1646: CoT Derivation SHA";
    let contracts = load_contract_values(project_path);

    let mut mismatches: Vec<String> = Vec::new();
    let mut malformed: Vec<String> = Vec::new();
    let mut checked = 0usize;

    for (ticket, contract) in &contracts {
        let digest_path = project_path
            .join(".pmat-work")
            .join(ticket)
            .join("cot-digest.json");
        if !digest_path.exists() {
            continue;
        }
        let Some(recorded) = read_recorded_digest(&digest_path) else {
            malformed.push(ticket.clone());
            continue;
        };
        checked += 1;
        let actual = canonical_cot_sha(contract);
        if !recorded.eq_ignore_ascii_case(&actual) {
            mismatches.push(format!(
                "{}: recorded={}, actual={}",
                ticket, recorded, actual
            ));
        }
    }

    if checked == 0 && malformed.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/<ID>/cot-digest.json` files — `pmat work cot derive` hasn't emitted digests yet".into(),
            severity: Severity::Info,
        };
    }

    if mismatches.is_empty() && malformed.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} ticket(s): cot-digest.json matches canonical SHA",
                checked
            ),
            severity: Severity::Info,
        };
    }

    let mut msg = String::new();
    if !mismatches.is_empty() {
        msg.push_str(&format!(
            "{} ticket(s) with CoT drift — `pmat work cot derive` to refresh:\n  {}\n",
            mismatches.len(),
            mismatches.join("\n  ")
        ));
    }
    if !malformed.is_empty() {
        msg.push_str(&format!(
            "{} ticket(s) with unreadable cot-digest.json: {}",
            malformed.len(),
            malformed.join(", ")
        ));
    }
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Fail,
        message: msg,
        severity: Severity::Error,
    }
}

/// CB-1648 (L4): every `Axiomatic` discharge in an L4+ ticket's chain-of-
/// thought must be backed by one of:
///
/// * a bound equation invariant — the `Axiomatic` reason or lemma name
///   matches an `equation` declared in the ticket's `implements:` array
/// * a documented lemma — the `Axiomatic` object carries a non-empty
///   `reason` string (prose-level documentation is acceptable at L4; L5
///   adds the Lean mapping requirement via CB-1649).
///
/// An Axiomatic discharge with neither is an "unchecked axiom" — the step
/// asserts something without evidence, which is exactly what formal
/// verification claims are supposed to prevent.
///
/// # Skip semantics (tiered)
///
/// * no tickets                                 → Skip
/// * no L4+ ticket                              → Skip
/// * no L4+ step uses `Axiomatic` discharge     → Skip
///
/// # Fail
///
/// Any Axiomatic discharge lacks both a `reason` and a match against a
/// bound equation name.
pub(crate) fn check_l4_axiomatic_discharge_bounded(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1648: L4 Axiomatic Discharge Bounded";
    let contracts = load_contract_values(project_path);
    if contracts.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/<ID>/contract.json` tickets present".into(),
            severity: Severity::Info,
        };
    }

    let l4_plus: Vec<&(String, Value)> = contracts
        .iter()
        .filter(|(_, c)| parse_level(c) >= 4)
        .collect();
    if l4_plus.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No L4+ ticket present".into(),
            severity: Severity::Info,
        };
    }

    let mut saw_axiomatic = false;
    let mut checked = 0usize;
    let mut violations: Vec<String> = Vec::new();

    for (ticket, contract) in &l4_plus {
        let equation_names: Vec<String> = contract
            .get("implements")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|b| b.get("equation").and_then(|e| e.as_str()).map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        for step in cot_steps(contract) {
            let Some(axiomatic) = step.get("discharged_by").and_then(|d| d.get("Axiomatic")) else {
                continue;
            };
            saw_axiomatic = true;
            checked += 1;

            let reason = axiomatic
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim();
            let lemma = axiomatic
                .get("lemma")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim();

            let matches_equation = equation_names
                .iter()
                .any(|eq| reason.contains(eq) || lemma.contains(eq));

            if !matches_equation && reason.is_empty() && lemma.is_empty() {
                violations.push(format!(
                    "  {}:{} Axiomatic discharge lacks `reason`/`lemma` and no bound equation match",
                    ticket,
                    step_id(step)
                ));
            }
        }
    }

    if !saw_axiomatic {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: format!(
                "No L4+ step uses `Axiomatic` discharge ({} eligible)",
                l4_plus.len()
            ),
            severity: Severity::Info,
        };
    }

    if !violations.is_empty() {
        let mut msg = format!(
            "{} unchecked Axiomatic discharge(s) in L4+ ticket(s):\n",
            violations.len()
        );
        let preview: Vec<&String> = violations.iter().take(5).collect();
        for line in preview {
            msg.push_str(line);
            msg.push('\n');
        }
        if violations.len() > 5 {
            msg.push_str(&format!("  …and {} more\n", violations.len() - 5));
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
            "{} Axiomatic discharge(s) in L4+ ticket(s) are bounded",
            checked
        ),
        severity: Severity::Info,
    }
}

/// CB-1649 (L5): every structured step in an L5 ticket must declare a
/// mapping to a Lean theorem/lemma. At L5 the ticket's truth is witnessed
/// by a machine-checked Lean proof; each reasoning step must point at the
/// specific theorem/lemma that discharges it.
///
/// # Accepted mapping shapes
///
/// Any of the following is sufficient evidence:
///
/// * top-level `lean_theorem: "..."` key on the step
/// * top-level `lean_lemma: "..."` key on the step
/// * `evidence_method.LeanTheorem: { name: "..." }`
/// * `evidence_method.LeanLemma: { name: "..." }`
/// * `discharged_by.Lean: { lemma: "..." }` (axiom-like discharge via Lean)
///
/// # Skip semantics (tiered)
///
/// * no tickets                                 → Skip
/// * no L5 ticket                               → Skip
/// * no structured step in any L5 ticket        → Skip (migration pending)
///
/// # Fail
///
/// Any structured L5 step lacks a Lean mapping.
pub(crate) fn check_l5_lean_theorem_mapping(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1649: L5 Lean Theorem Mapping";
    let contracts = load_contract_values(project_path);
    if contracts.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/<ID>/contract.json` tickets present".into(),
            severity: Severity::Info,
        };
    }

    let l5: Vec<&(String, Value)> = contracts
        .iter()
        .filter(|(_, c)| parse_level(c) >= 5)
        .collect();
    if l5.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No L5 ticket present".into(),
            severity: Severity::Info,
        };
    }

    let mut structured_seen = 0usize;
    let mut missing: Vec<String> = Vec::new();

    for (ticket, contract) in &l5 {
        for step in cot_steps(contract) {
            if !is_structured(step) {
                continue;
            }
            structured_seen += 1;
            if !step_has_lean_mapping(step) {
                missing.push(format!("{}:{}", ticket, step_id(step)));
            }
        }
    }

    if structured_seen == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: format!(
                "No structured CoT step in any L5 ticket ({} eligible) — migration pending",
                l5.len()
            ),
            severity: Severity::Info,
        };
    }

    if !missing.is_empty() {
        let mut msg = format!(
            "{} L5 step(s) lack a Lean theorem/lemma mapping:\n",
            missing.len()
        );
        let preview: Vec<&String> = missing.iter().take(5).collect();
        for line in preview {
            msg.push_str("  ");
            msg.push_str(line);
            msg.push('\n');
        }
        if missing.len() > 5 {
            msg.push_str(&format!("  …and {} more\n", missing.len() - 5));
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
            "{} L5 structured step(s) map to Lean theorems/lemmas",
            structured_seen
        ),
        severity: Severity::Info,
    }
}

/// Return true iff the step declares a Lean theorem/lemma mapping in any
/// of the accepted shapes. Schema-pragmatic: the exact field name has not
/// been finalised, so accept the obvious variants.
fn step_has_lean_mapping(step: &Value) -> bool {
    if step
        .get("lean_theorem")
        .and_then(|v| v.as_str())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
    {
        return true;
    }
    if step
        .get("lean_lemma")
        .and_then(|v| v.as_str())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
    {
        return true;
    }
    if let Some(method) = step.get("evidence_method") {
        if method.get("LeanTheorem").is_some() || method.get("LeanLemma").is_some() {
            return true;
        }
    }
    if let Some(discharged) = step.get("discharged_by") {
        if discharged
            .get("Lean")
            .and_then(|v| v.get("lemma"))
            .and_then(|v| v.as_str())
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false)
        {
            return true;
        }
    }
    false
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

    // ── CB-1644 agent run replayable tests ───────────────────────────────

    fn write_agent_run(project: &Path, ticket: &str, run_id: &str, body: &str) {
        let dir = project.join(".pmat-work").join(ticket).join("agent-runs");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{}.json", run_id)), body).unwrap();
    }

    #[test]
    fn agent_run_skip_when_no_work_dir() {
        let project = tempdir().unwrap();
        let check = check_agent_run_replayable(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("`.pmat-work/`"));
    }

    #[test]
    fn agent_run_skip_when_no_agent_runs_dirs() {
        let project = tempdir().unwrap();
        write_contract(project.path(), "T1", json!({ "work_item_id": "T1" }));
        let check = check_agent_run_replayable(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("agent-runs"));
    }

    #[test]
    fn agent_run_pass_with_full_schema() {
        let project = tempdir().unwrap();
        write_agent_run(
            project.path(),
            "T1",
            "run-001",
            r#"{
                "prompt_sha": "abc123",
                "tool_calls": [{"name": "Read"}, {"name": "Edit"}],
                "commit_sha": "deadbeef"
            }"#,
        );
        let check = check_agent_run_replayable(project.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
        assert!(check.message.contains("1 agent run"));
    }

    #[test]
    fn agent_run_fail_when_field_missing() {
        let project = tempdir().unwrap();
        write_agent_run(
            project.path(),
            "T1",
            "run-001",
            r#"{
                "prompt_sha": "abc123",
                "tool_calls": []
            }"#,
        );
        let check = check_agent_run_replayable(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("commit_sha"));
        assert!(check.message.contains("T1:run-001"));
    }

    #[test]
    fn agent_run_fail_when_null_field() {
        let project = tempdir().unwrap();
        write_agent_run(
            project.path(),
            "T1",
            "run-001",
            r#"{
                "prompt_sha": null,
                "tool_calls": [],
                "commit_sha": "dead"
            }"#,
        );
        let check = check_agent_run_replayable(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("prompt_sha"));
    }

    #[test]
    fn agent_run_fail_when_tool_calls_not_array() {
        let project = tempdir().unwrap();
        write_agent_run(
            project.path(),
            "T1",
            "run-001",
            r#"{
                "prompt_sha": "a",
                "tool_calls": "should-be-array",
                "commit_sha": "b"
            }"#,
        );
        let check = check_agent_run_replayable(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("tool_calls is not an array"));
    }

    #[test]
    fn agent_run_fail_when_malformed_json() {
        let project = tempdir().unwrap();
        write_agent_run(project.path(), "T1", "run-001", "{ not json");
        let check = check_agent_run_replayable(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("unreadable"));
        assert!(check.message.contains("T1:run-001"));
    }

    #[test]
    fn agent_run_fail_when_top_level_not_object() {
        let project = tempdir().unwrap();
        write_agent_run(project.path(), "T1", "run-001", "[1, 2, 3]");
        let check = check_agent_run_replayable(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("not an object"));
    }

    #[test]
    fn agent_run_ignores_non_json_files() {
        let project = tempdir().unwrap();
        // A stray README in agent-runs/ doesn't count toward checked_runs
        let runs_dir = project.path().join(".pmat-work/T1/agent-runs");
        std::fs::create_dir_all(&runs_dir).unwrap();
        std::fs::write(runs_dir.join("README.md"), "notes").unwrap();
        let check = check_agent_run_replayable(project.path());
        // No *.json files present → treat as Skip (no runs emitted)
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn agent_run_ignores_ledger_and_hidden_dirs() {
        let project = tempdir().unwrap();
        std::fs::create_dir_all(project.path().join(".pmat-work/ledger/agent-runs")).unwrap();
        std::fs::create_dir_all(project.path().join(".pmat-work/.hidden/agent-runs")).unwrap();
        // Write valid runs under both — they should be skipped by the check
        std::fs::write(
            project.path().join(".pmat-work/ledger/agent-runs/x.json"),
            r#"{"prompt_sha":"a","tool_calls":[],"commit_sha":"b"}"#,
        )
        .unwrap();
        let check = check_agent_run_replayable(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn agent_run_pass_across_multiple_tickets() {
        let project = tempdir().unwrap();
        write_agent_run(
            project.path(),
            "T1",
            "run-001",
            r#"{"prompt_sha":"a","tool_calls":[],"commit_sha":"b"}"#,
        );
        write_agent_run(
            project.path(),
            "T2",
            "run-001",
            r#"{"prompt_sha":"c","tool_calls":[],"commit_sha":"d"}"#,
        );
        let check = check_agent_run_replayable(project.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("2 agent run"));
    }

    // ── CB-1646 CoT derivation SHA tests ─────────────────────────────────

    fn write_digest(project: &Path, ticket: &str, body: &str) {
        let dir = project.join(".pmat-work").join(ticket);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("cot-digest.json"), body).unwrap();
    }

    #[test]
    fn cot_digest_skips_when_no_digest_files() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({ "chain_of_thought": [{ "step": 1, "question": "q", "answer": "a" }] }),
        );
        let check = check_cot_derivation_sha_fresh(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("cot-digest.json"));
    }

    #[test]
    fn cot_digest_passes_when_sha_matches() {
        let project = tempdir().unwrap();
        let contract = json!({
            "chain_of_thought": [
                { "id": "CoT-1", "assumption": { "predicate": "p" } }
            ]
        });
        write_contract(project.path(), "T1", contract.clone());
        let expected = canonical_cot_sha(&contract);
        write_digest(
            project.path(),
            "T1",
            &format!("{{\"sha\": \"{}\"}}", expected),
        );
        let check = check_cot_derivation_sha_fresh(project.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn cot_digest_accepts_digest_key_alias() {
        let project = tempdir().unwrap();
        let contract = json!({ "chain_of_thought": [{ "id": "CoT-1" }] });
        write_contract(project.path(), "T1", contract.clone());
        let expected = canonical_cot_sha(&contract);
        // Alternate naming — `digest` instead of `sha`
        write_digest(
            project.path(),
            "T1",
            &format!("{{\"digest\": \"{}\"}}", expected),
        );
        let check = check_cot_derivation_sha_fresh(project.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn cot_digest_fails_when_sha_mismatches() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({ "chain_of_thought": [{ "id": "CoT-1", "assumption": { "predicate": "p" } }] }),
        );
        // Simulate someone editing the CoT after derivation: record an old SHA
        write_digest(project.path(), "T1", "{\"sha\": \"deadbeef\"}");
        let check = check_cot_derivation_sha_fresh(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("T1"));
        assert!(check.message.contains("deadbeef"));
        assert!(check.message.contains("pmat work cot derive"));
    }

    #[test]
    fn cot_digest_fails_on_malformed_digest_file() {
        let project = tempdir().unwrap();
        write_contract(project.path(), "T1", json!({ "chain_of_thought": [] }));
        write_digest(project.path(), "T1", "{ not valid json");
        let check = check_cot_derivation_sha_fresh(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("unreadable"));
        assert!(check.message.contains("T1"));
    }

    #[test]
    fn cot_digest_case_insensitive_hex_match() {
        // Recorded digest is uppercase; canonical is lowercase — must still match.
        let project = tempdir().unwrap();
        let contract = json!({ "chain_of_thought": [{ "id": "CoT-1" }] });
        write_contract(project.path(), "T1", contract.clone());
        let expected = canonical_cot_sha(&contract).to_uppercase();
        write_digest(
            project.path(),
            "T1",
            &format!("{{\"sha\": \"{}\"}}", expected),
        );
        let check = check_cot_derivation_sha_fresh(project.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn cot_digest_mixed_coverage_checks_only_tickets_with_digest() {
        let project = tempdir().unwrap();
        // T1 has a digest; T2 does not — T2 is silently skipped.
        let contract1 = json!({ "chain_of_thought": [{ "id": "CoT-1" }] });
        write_contract(project.path(), "T1", contract1.clone());
        write_digest(
            project.path(),
            "T1",
            &format!("{{\"sha\": \"{}\"}}", canonical_cot_sha(&contract1)),
        );
        write_contract(project.path(), "T2", json!({ "chain_of_thought": [] }));
        let check = check_cot_derivation_sha_fresh(project.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
        // Only one ticket was checked
        assert!(check.message.contains("1 ticket"));
    }

    #[test]
    fn canonical_cot_sha_is_deterministic() {
        let a = json!({
            "chain_of_thought": [
                { "id": "CoT-1", "assumption": { "predicate": "p" } }
            ]
        });
        let b = json!({
            "chain_of_thought": [
                { "assumption": { "predicate": "p" }, "id": "CoT-1" }
            ]
        });
        // BTreeMap key ordering means differently-authored JSON with the
        // same semantic content hashes identically.
        assert_eq!(canonical_cot_sha(&a), canonical_cot_sha(&b));
    }

    // ── CB-1645 derived YAML obligations tests ───────────────────────────

    fn write_derived_yaml(project: &Path, safe_id: &str, body: &str) {
        let dir = project.join("contracts/work");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{}.yaml", safe_id)), body).unwrap();
    }

    #[test]
    fn derived_yaml_skips_when_no_tickets() {
        let project = tempdir().unwrap();
        let check = check_derived_yaml_obligations_present(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("No `.pmat-work/` tickets"));
    }

    #[test]
    fn derived_yaml_skips_when_no_derivable_clauses() {
        let project = tempdir().unwrap();
        write_contract(project.path(), "T1", json!({ "work_item_id": "T1" }));
        let check = check_derived_yaml_obligations_present(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("derivable"));
    }

    #[test]
    fn derived_yaml_fails_when_file_missing() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "work_item_id": "T1",
                "ensure": ["returns sorted array"]
            }),
        );
        let check = check_derived_yaml_obligations_present(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("missing derived"));
        assert!(check.message.contains("T1"));
    }

    #[test]
    fn derived_yaml_fails_when_stale() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "work_item_id": "T1",
                "ensure": ["returns sorted array"]
            }),
        );
        // Derived YAML doesn't mention the postcondition → stale
        write_derived_yaml(
            project.path(),
            "T1",
            "name: \"T1\"\npostconditions:\n  - \"some other thing\"\n",
        );
        let check = check_derived_yaml_obligations_present(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("stale"));
        assert!(check.message.contains("returns sorted array"));
    }

    #[test]
    fn derived_yaml_passes_when_up_to_date() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "work_item_id": "T1",
                "ensure": ["returns sorted array"],
                "require": ["input is a slice of integers"]
            }),
        );
        write_derived_yaml(
            project.path(),
            "T1",
            "name: \"T1\"\n\
             preconditions:\n  - \"input is a slice of integers\"\n\
             postconditions:\n  - \"returns sorted array\"\n",
        );
        let check = check_derived_yaml_obligations_present(project.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn derived_yaml_sanitizes_special_chars_in_id() {
        let project = tempdir().unwrap();
        // A ticket id with a colon — generator replaces ':' with '_'
        write_contract(
            project.path(),
            "GH-168: fix leak",
            json!({
                "work_item_id": "GH-168: fix leak",
                "ensure": ["no memory leak"]
            }),
        );
        write_derived_yaml(
            project.path(),
            "GH-168__fix_leak",
            "postconditions:\n  - \"no memory leak\"\n",
        );
        let check = check_derived_yaml_obligations_present(project.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
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

    // ── CB-1648 L4 Axiomatic discharge bounded tests ─────────────────────

    #[test]
    fn cb1648_skips_when_no_tickets() {
        let project = tempdir().unwrap();
        let r = check_l4_axiomatic_discharge_bounded(project.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/<ID>/contract.json`"));
    }

    #[test]
    fn cb1648_skips_when_no_l4_ticket() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({ "verification_level": "L3", "chain_of_thought": [] }),
        );
        let r = check_l4_axiomatic_discharge_bounded(project.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No L4+ ticket"));
    }

    #[test]
    fn cb1648_skips_when_no_axiomatic_discharge() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L4",
                "chain_of_thought": [
                    { "id": "CoT-1", "assumption": { "predicate": "p" } }
                ]
            }),
        );
        let r = check_l4_axiomatic_discharge_bounded(project.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("Axiomatic"));
    }

    #[test]
    fn cb1648_passes_when_axiomatic_has_reason() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L4",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "discharged_by": {
                            "Axiomatic": { "reason": "IEEE-754 finite arithmetic" }
                        }
                    }
                ]
            }),
        );
        let r = check_l4_axiomatic_discharge_bounded(project.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("1 Axiomatic"));
    }

    #[test]
    fn cb1648_passes_when_axiomatic_matches_bound_equation() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L4",
                "implements": [
                    {
                        "contract": "pmat-core",
                        "equation": "rope",
                        "file": "contracts/pmat-core.yaml",
                        "sha": "abc",
                        "bound_at": "2026-04-18T00:00:00Z"
                    }
                ],
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "discharged_by": {
                            "Axiomatic": { "lemma": "rope invariant" }
                        }
                    }
                ]
            }),
        );
        let r = check_l4_axiomatic_discharge_bounded(project.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1648_fails_when_axiomatic_has_no_reason_or_equation_match() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L4",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "discharged_by": { "Axiomatic": {} }
                    }
                ]
            }),
        );
        let r = check_l4_axiomatic_discharge_bounded(project.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("CoT-1"));
        assert!(r.message.contains("T1"));
    }

    #[test]
    fn cb1648_fails_when_reason_empty_and_no_equation() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L5",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "discharged_by": {
                            "Axiomatic": { "reason": "   " }
                        }
                    }
                ]
            }),
        );
        let r = check_l4_axiomatic_discharge_bounded(project.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
    }

    #[test]
    fn cb1648_aggregates_multiple_violations() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L4",
                "chain_of_thought": [
                    { "id": "CoT-1", "discharged_by": { "Axiomatic": {} } },
                    { "id": "CoT-2", "discharged_by": { "Axiomatic": {} } },
                ]
            }),
        );
        let r = check_l4_axiomatic_discharge_bounded(project.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("2 unchecked"));
    }

    #[test]
    fn cb1648_ignores_non_l4_axiomatic_violations() {
        let project = tempdir().unwrap();
        // L3 ticket with unchecked axiom — not our concern at L4 gate.
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L3",
                "chain_of_thought": [
                    { "id": "CoT-1", "discharged_by": { "Axiomatic": {} } }
                ]
            }),
        );
        let r = check_l4_axiomatic_discharge_bounded(project.path());
        assert_eq!(r.status, CheckStatus::Skip, "{}", r.message);
        assert!(r.message.contains("L4+"));
    }

    // ── CB-1649 L5 Lean theorem mapping tests ────────────────────────────

    #[test]
    fn cb1649_skips_when_no_tickets() {
        let project = tempdir().unwrap();
        let r = check_l5_lean_theorem_mapping(project.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }

    #[test]
    fn cb1649_skips_when_no_l5_ticket() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({ "verification_level": "L4", "chain_of_thought": [] }),
        );
        let r = check_l5_lean_theorem_mapping(project.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No L5 ticket"));
    }

    #[test]
    fn cb1649_skips_when_no_structured_steps_in_l5() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L5",
                "chain_of_thought": [
                    { "step": 1, "question": "q", "answer": "a" }
                ]
            }),
        );
        let r = check_l5_lean_theorem_mapping(project.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("migration pending"));
    }

    #[test]
    fn cb1649_passes_when_step_has_lean_theorem_key() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L5",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" },
                        "lean_theorem": "Rope.preserves_norm"
                    }
                ]
            }),
        );
        let r = check_l5_lean_theorem_mapping(project.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1649_passes_when_step_has_lean_lemma_key() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L5",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "implication": { "predicate": "q" },
                        "lean_lemma": "Arith.add_comm"
                    }
                ]
            }),
        );
        let r = check_l5_lean_theorem_mapping(project.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1649_passes_when_evidence_method_is_lean_theorem() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L5",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" },
                        "evidence_method": { "LeanTheorem": { "name": "Rope.norm" } }
                    }
                ]
            }),
        );
        let r = check_l5_lean_theorem_mapping(project.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1649_passes_when_discharged_by_lean() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L5",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" },
                        "discharged_by": { "Lean": { "lemma": "FinSet.nonempty" } }
                    }
                ]
            }),
        );
        let r = check_l5_lean_theorem_mapping(project.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1649_fails_when_structured_step_has_no_lean_mapping() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L5",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" }
                    }
                ]
            }),
        );
        let r = check_l5_lean_theorem_mapping(project.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("CoT-1"));
        assert!(r.message.contains("T1"));
    }

    #[test]
    fn cb1649_fails_when_lean_theorem_is_empty_string() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L5",
                "chain_of_thought": [
                    {
                        "id": "CoT-1",
                        "assumption": { "predicate": "p" },
                        "lean_theorem": "   "
                    }
                ]
            }),
        );
        let r = check_l5_lean_theorem_mapping(project.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
    }

    #[test]
    fn cb1649_step_has_lean_mapping_recognises_all_forms() {
        assert!(step_has_lean_mapping(&json!({ "lean_theorem": "Foo.bar" })));
        assert!(step_has_lean_mapping(&json!({ "lean_lemma": "Foo.baz" })));
        assert!(step_has_lean_mapping(&json!({
            "evidence_method": { "LeanTheorem": { "name": "Foo" } }
        })));
        assert!(step_has_lean_mapping(&json!({
            "evidence_method": { "LeanLemma": { "name": "Foo" } }
        })));
        assert!(step_has_lean_mapping(&json!({
            "discharged_by": { "Lean": { "lemma": "Foo" } }
        })));
        assert!(!step_has_lean_mapping(&json!({})));
        assert!(!step_has_lean_mapping(&json!({ "lean_theorem": "" })));
        assert!(!step_has_lean_mapping(&json!({
            "evidence_method": { "ExistingTest": { "path": "t.rs" } }
        })));
    }

    #[test]
    fn cb1649_ignores_non_l5_tickets() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({
                "verification_level": "L4",
                "chain_of_thought": [
                    { "id": "CoT-1", "assumption": { "predicate": "p" } }
                ]
            }),
        );
        let r = check_l5_lean_theorem_mapping(project.path());
        assert_eq!(r.status, CheckStatus::Skip, "{}", r.message);
    }
}
