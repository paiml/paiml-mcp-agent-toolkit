// CB-1648 — L4 Axiomatic discharge bounded: every `Axiomatic` discharge in
// an L4+ ticket is backed by either a bound equation invariant (name match
// against `implements:` bindings) or a documented lemma (non-empty `reason`).
// Included from `check_cot_proof.rs` — do NOT add `use` imports or `#!` attributes here.

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
