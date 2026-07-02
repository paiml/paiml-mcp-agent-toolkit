// Included from check_macs.rs — do NOT add `use` imports or `#!` attributes here.

/// CB-1658: derivation completeness (MACS F3). For every ticket that has run
/// `pmat work cot derive` (a cot-digest.json exists), the derivation artifact
/// `contracts/work/<ID>.cot.yaml` must carry exactly one proof obligation and
/// one falsifiable claim per CoT step, with hypothesis/method copied verbatim
/// (contracts/macs-cot-v1.yaml#derivation_complete).
pub(crate) fn check_derivation_completeness(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1658: CoT Derivation Completeness";
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return skip_check(name, "No .pmat-work directory");
    }

    let mut checked = 0usize;
    let mut violations: Vec<String> = Vec::new();

    let Ok(entries) = std::fs::read_dir(&work_dir) else {
        return skip_check(name, "work store unreadable");
    };
    for entry in entries.flatten() {
        let ticket = entry.file_name().to_string_lossy().to_string();
        if !entry.path().join("cot-digest.json").exists() {
            continue; // derive has not run for this ticket
        }
        let Ok(text) = std::fs::read_to_string(entry.path().join("contract.json")) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        let steps = crate::models::work_cot::parse_steps(&value);
        checked += 1;

        let safe_id: String = ticket
            .chars()
            .map(|ch| if ch.is_alphanumeric() || ch == '-' { ch } else { '_' })
            .collect();
        let artifact = project_path
            .join("contracts")
            .join("work")
            .join(format!("{safe_id}.cot.yaml"));
        let Ok(yaml_text) = std::fs::read_to_string(&artifact) else {
            violations.push(format!(
                "{ticket}: cot-digest.json exists but {} is missing",
                artifact.display()
            ));
            continue;
        };
        let Ok(doc) = serde_yaml_ng::from_str::<serde_json::Value>(&yaml_text) else {
            violations.push(format!("{ticket}: derivation artifact is not valid YAML"));
            continue;
        };
        let obligations = doc
            .get("proof_obligations")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len)
            .unwrap_or(0);
        let claims = doc
            .get("falsifiable_claims")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len)
            .unwrap_or(0);
        if obligations != steps.len() || claims != steps.len() {
            violations.push(format!(
                "{ticket}: {} step(s) but {obligations} obligation(s) / {claims} claim(s)",
                steps.len()
            ));
            continue;
        }
        // Verbatim fields: every claim hypothesis/method must equal the
        // corresponding step's implication/evidence_method.
        if let Some(claim_list) = doc
            .get("falsifiable_claims")
            .and_then(serde_json::Value::as_array)
        {
            for (step, claim) in steps.iter().zip(claim_list) {
                let hypothesis = claim
                    .get("hypothesis")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default();
                let method = claim
                    .get("method")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default();
                if hypothesis != step.implication || method != step.evidence_method {
                    violations.push(format!(
                        "{ticket}:{}: claim fields drifted from step (paraphrase drift)",
                        step.id
                    ));
                }
            }
        }
    }

    if checked == 0 {
        return skip_check(name, "No ticket has run `pmat work cot derive` yet");
    }
    if violations.is_empty() {
        return ComplianceCheck {
            name: name.to_string(),
            status: CheckStatus::Pass,
            message: format!("{checked} derived ticket(s): one obligation + one claim per step, verbatim"),
            severity: Severity::Info,
        };
    }
    ComplianceCheck {
        name: name.to_string(),
        status: CheckStatus::Fail,
        message: format!(
            "{} derivation completeness violation(s):\n{}",
            violations.len(),
            format_violation_list(&violations)
        ),
        severity: Severity::Error,
    }
}
