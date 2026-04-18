// Work ladder audit checks: downgrade ledger reasons (CB-1617) and
// completion-matches-target (CB-1619). Included into `check_work_ladder.rs`.

// ─── CB-1617: downgrade audit ────────────────────────────────────────────────

/// CB-1617 (L3): any entry in `.pmat-work/ledger/downgrades.json` must carry
/// a non-empty `reason` field. A downgrade with empty or missing reason is
/// silent scope reduction.
///
/// The ledger file is optional — its absence is Skip, not Fail.
pub(crate) fn check_ladder_downgrade_audit(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1617: Downgrade Reason Audit";
    let ledger = project_path
        .join(".pmat-work")
        .join("ledger")
        .join("downgrades.json");
    if !ledger.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No downgrade ledger at .pmat-work/ledger/downgrades.json".into(),
            severity: Severity::Info,
        };
    }
    let Ok(content) = std::fs::read_to_string(&ledger) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Warn,
            message: format!("Unreadable downgrade ledger: {}", ledger.display()),
            severity: Severity::Warning,
        };
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: "Downgrade ledger is not valid JSON".into(),
            severity: Severity::Error,
        };
    };

    let entries: &[serde_json::Value] = match &value {
        serde_json::Value::Array(a) => a.as_slice(),
        _ => {
            return ComplianceCheck {
                name: name.into(),
                status: CheckStatus::Fail,
                message: "Downgrade ledger must be a JSON array of entries".into(),
                severity: Severity::Error,
            };
        }
    };

    let mut missing: Vec<String> = Vec::new();
    for (i, entry) in entries.iter().enumerate() {
        let ticket = entry
            .get("ticket")
            .and_then(|v| v.as_str())
            .unwrap_or("<unknown>");
        let reason = entry.get("reason").and_then(|v| v.as_str()).unwrap_or("");
        if reason.trim().is_empty() {
            missing.push(format!("  entry[{}] ticket={} reason=empty", i, ticket));
        }
    }

    if !missing.is_empty() {
        let mut msg = format!("{} downgrade(s) lack a `reason`:\n", missing.len());
        for line in &missing {
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
        message: format!("{} downgrade(s) carry a recorded reason", entries.len()),
        severity: Severity::Info,
    }
}

// ─── CB-1619: completion == target ───────────────────────────────────────────

/// CB-1619 (L3): tickets marked completed must have their
/// `verification_level` (achieved level) equal to the target recorded in
/// `verification-report.json`. Silent downgrade is forbidden.
///
/// The report file is optional — its absence is Skip.
pub(crate) fn check_ladder_completion_matches(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1619: Achieved Level == Target";
    let contracts = load_active_contracts(project_path);
    if contracts.is_empty() {
        return skip_no_contracts(name);
    }

    let mut mismatches: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for c in &contracts {
        let report = project_path
            .join(".pmat-work")
            .join(&c.work_item_id)
            .join("verification-report.json");
        if !report.exists() {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&report) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
            continue;
        };
        let target = value
            .get("target_level")
            .and_then(|v| v.as_str())
            .and_then(VerificationLevel::parse_strict);
        let achieved = value
            .get("achieved_level")
            .and_then(|v| v.as_str())
            .and_then(VerificationLevel::parse_strict);
        let Some(target) = target else { continue };
        let Some(achieved) = achieved else { continue };
        checked += 1;
        if achieved < target {
            mismatches.push(format!(
                "  {} target={} achieved={}",
                c.work_item_id, target, achieved
            ));
        }
    }

    if !mismatches.is_empty() {
        let mut msg = format!(
            "{} ticket(s) closed below target level:\n",
            mismatches.len()
        );
        for line in &mismatches {
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

    if checked == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ticket has verification-report.json yet".into(),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!("{} completion(s) match target level", checked),
        severity: Severity::Info,
    }
}
