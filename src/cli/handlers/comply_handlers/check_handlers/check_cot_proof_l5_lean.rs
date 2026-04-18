// CB-1649 — L5 Lean theorem mapping: every structured step in an L5 ticket
// carries a Lean theorem/lemma mapping via `lean_theorem`, `lean_lemma`,
// `evidence_method.LeanTheorem/LeanLemma`, or `discharged_by.Lean`.
// Included from `check_cot_proof.rs` — do NOT add `use` imports or `#!` attributes here.

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
