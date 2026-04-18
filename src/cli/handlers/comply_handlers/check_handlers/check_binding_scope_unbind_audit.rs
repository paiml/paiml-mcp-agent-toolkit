// CB-1602 Unbind Audit — included from check_binding_scope.rs.
// Do NOT add `use` imports or `#!` attributes here.

/// CB-1602 (L1): `pmat work unbind` without a DEBT follow-up ticket reference
/// indicates silent contract abandonment. Every entry in the unbind ledger
/// must cite the debt ticket that'll restore the binding.
///
/// Ledger schema (minimum): JSON array at `.pmat-work/ledger/unbinds.json`
/// where each entry has:
///   • `ticket`       — the work-item-id that unbound
///   • `contract`     — YAML path (or contract name) that was unbound from
///   • `debt_ticket`  — follow-up ticket id (e.g., "DEBT-123"), non-empty
///
/// Skip-if-absent: the ledger file is optional — until `pmat work unbind`
/// lands, it doesn't exist, and this check is Skip.
pub(crate) fn check_binding_unbind_audit(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1602: Unbind Audit";
    let ledger = project_path
        .join(".pmat-work")
        .join("ledger")
        .join("unbinds.json");
    if !ledger.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No unbind ledger at .pmat-work/ledger/unbinds.json".into(),
            severity: Severity::Info,
        };
    }
    let Ok(content) = std::fs::read_to_string(&ledger) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Warn,
            message: format!("Unreadable unbind ledger: {}", ledger.display()),
            severity: Severity::Warning,
        };
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: "Unbind ledger is not valid JSON".into(),
            severity: Severity::Error,
        };
    };

    let entries: &[serde_json::Value] = match &value {
        serde_json::Value::Array(a) => a.as_slice(),
        _ => {
            return ComplianceCheck {
                name: name.into(),
                status: CheckStatus::Fail,
                message: "Unbind ledger must be a JSON array of entries".into(),
                severity: Severity::Error,
            };
        }
    };

    if entries.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: "Unbind ledger present but empty — no audits pending".into(),
            severity: Severity::Info,
        };
    }

    let mut bad: Vec<String> = Vec::new();
    for (idx, entry) in entries.iter().enumerate() {
        let ticket = entry
            .get("ticket")
            .and_then(|t| t.as_str())
            .unwrap_or("<no-ticket>");
        let debt = entry
            .get("debt_ticket")
            .and_then(|t| t.as_str())
            .unwrap_or("");
        if debt.trim().is_empty() {
            bad.push(format!(
                "  entry {}: ticket={} missing/empty debt_ticket",
                idx, ticket
            ));
        }
    }

    if !bad.is_empty() {
        let mut msg = format!("{} unbind(s) lack DEBT ticket reference:\n", bad.len());
        for line in &bad {
            msg.push_str(line);
            msg.push('\n');
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
            "{} unbind(s) all carry DEBT ticket reference",
            entries.len()
        ),
        severity: Severity::Info,
    }
}
