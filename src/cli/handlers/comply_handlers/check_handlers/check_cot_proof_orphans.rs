// CB-1647 — No orphan CoT steps (each chains to a discharge).
// Included from `check_cot_proof.rs` — do NOT add `use` imports or `#!` attributes here.

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
