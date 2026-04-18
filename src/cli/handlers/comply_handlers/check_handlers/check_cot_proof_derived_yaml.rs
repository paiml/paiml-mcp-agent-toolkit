// CB-1645 — Derived YAML obligations reflect contract.json preconditions/postconditions.
// Included from `check_cot_proof.rs` — do NOT add `use` imports or `#!` attributes here.

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
