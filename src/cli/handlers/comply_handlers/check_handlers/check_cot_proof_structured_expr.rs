// CB-1643 — L3+ tickets: each structured step has assumption.expr or implication.expr.
// Included from `check_cot_proof.rs` — do NOT add `use` imports or `#!` attributes here.

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
