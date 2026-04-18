// CB-1640 — Assumption references resolve to prior steps, bound equation
// names, or axiomatic discharge (exact-string fallback per spec §Chain
// Integrity Rule when semantic-search vocabulary is absent).
// Included from `check_cot_proof.rs` — do NOT add `use` imports or `#!` attributes here.

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
