// Included from check_macs.rs — do NOT add `use` imports or `#!` attributes here.

/// CB-1653: claimed-vs-achieved ladder drift (MACS F2).
///
/// Two evidence sources, two severities:
/// - **Fail**: a falsification receipt records `claimed_level > achieved_level`
///   — a ticket was closed above its evidenced level (the P0 condition the
///   MACS-005 gate exists to prevent; a red here means the gate was bypassed
///   or the receipt was produced by a pre-gate binary).
/// - **Warn**: a live contract currently claims more than its evidence
///   supports — visible drift on open work; the completion gate will block
///   it, this check just surfaces it early.
pub(crate) fn check_ladder_claim_drift(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1653: Ladder Claim Drift";
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return skip_check(name, "No .pmat-work directory");
    }

    // Receipt-recorded drift (closed above evidence) — hard fail.
    let mut receipt_drift: Vec<String> = Vec::new();
    let mut receipts_with_levels = 0usize;
    for receipt_path in macs_receipt_files(&work_dir) {
        let Ok(text) = std::fs::read_to_string(&receipt_path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        let claimed = value
            .get("claimed_level")
            .and_then(serde_json::Value::as_str)
            .and_then(crate::cli::handlers::work_verification_level::VerificationLevel::parse_strict);
        let achieved = value
            .get("achieved_level")
            .and_then(serde_json::Value::as_str)
            .and_then(crate::cli::handlers::work_verification_level::VerificationLevel::parse_strict);
        let (Some(claimed), Some(achieved)) = (claimed, achieved) else {
            continue;
        };
        receipts_with_levels += 1;
        let allows = value
            .get("summary")
            .and_then(|s| s.get("allows_completion"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        if allows && claimed > achieved {
            receipt_drift.push(format!(
                "{}: receipt closed claimed={} > achieved={}",
                receipt_path.display(),
                claimed,
                achieved
            ));
        }
    }

    // Live contract drift (open work over-claiming) — advisory.
    let mut contract_drift: Vec<String> = Vec::new();
    let mut contracts_checked = 0usize;
    if let Ok(entries) = std::fs::read_dir(&work_dir) {
        for entry in entries.flatten() {
            let ticket = entry.file_name().to_string_lossy().to_string();
            if ticket == "ledger" || ticket == "ledger.jsonl" || ticket == "config.toml" {
                continue;
            }
            if !entry.path().join("contract.json").exists() {
                continue;
            }
            let Ok(contract) = crate::cli::handlers::work_contract::WorkContract::load(
                project_path,
                &ticket,
            ) else {
                continue;
            };
            contracts_checked += 1;
            let claimed = contract.verification_level;
            let achieved =
                crate::quality::ladder_evidence::achieved_level(project_path, &contract);
            if claimed > achieved {
                contract_drift.push(format!(
                    "{ticket}: claims {claimed}, evidence supports {achieved}"
                ));
            }
        }
    }

    if !receipt_drift.is_empty() {
        return ComplianceCheck {
            name: name.to_string(),
            status: CheckStatus::Fail,
            message: format!(
                "{} receipt(s) closed above evidenced level:\n{}",
                receipt_drift.len(),
                format_violation_list(&receipt_drift)
            ),
            severity: Severity::Error,
        };
    }
    if !contract_drift.is_empty() {
        return ComplianceCheck {
            name: name.to_string(),
            status: CheckStatus::Warn,
            message: format!(
                "{} open ticket(s) claim above evidenced level (gate will block completion):\n{}",
                contract_drift.len(),
                format_violation_list(&contract_drift)
            ),
            severity: Severity::Warning,
        };
    }
    if receipts_with_levels == 0 && contracts_checked == 0 {
        return skip_check(name, "No contracts or level-carrying receipts yet");
    }
    ComplianceCheck {
        name: name.to_string(),
        status: CheckStatus::Pass,
        message: format!(
            "{contracts_checked} contract(s), {receipts_with_levels} level-carrying receipt(s): no drift"
        ),
        severity: Severity::Info,
    }
}
