// CB-1614: L4 Kani evidence — every L4+ ticket must record a passing
// `kani-report.json`. Included into `check_work_ladder.rs`.

// ─── CB-1614: L4 Kani evidence ──────────────────────────────────────────────

/// CB-1614 (L4): every L4+ ticket must have `.pmat-work/<ID>/kani-report.json`
/// present, and that report must carry `success: true`. When Component 24
/// Kani runner lands, this check enforces that L4 claims have Kani backing.
///
/// Report schema (minimum): `{ "success": bool }` — extra fields ignored.
///
/// Skip semantics (tiered):
///   • no tickets at all                          → Skip
///   • no L4+ ticket on any active contract       → Skip
///   • L4+ tickets exist but none have a report   → Skip (runner not yet
///                                                   wired; in-progress
///                                                   tickets don't falsify)
///   • any L4+ report missing `success` key,
///     reports `success: false`, or is malformed  → Fail
pub(crate) fn check_ladder_l4_kani(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1614: L4 Kani Evidence";
    let contracts = load_active_contracts(project_path);
    if contracts.is_empty() {
        return skip_no_contracts(name);
    }

    let l4_plus: Vec<&WorkContract> = contracts.iter().filter(|c| is_l4_or_higher(c)).collect();
    if l4_plus.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No L4+ ticket present".into(),
            severity: Severity::Info,
        };
    }

    let mut any_report = false;
    let mut checked = 0usize;
    let mut failing: Vec<String> = Vec::new();

    for c in &l4_plus {
        let report = project_path
            .join(".pmat-work")
            .join(&c.work_item_id)
            .join("kani-report.json");
        if !report.exists() {
            continue; // in-progress L4 ticket
        }
        any_report = true;
        checked += 1;

        let Ok(contents) = std::fs::read_to_string(&report) else {
            failing.push(format!(
                "  {} (unreadable kani-report.json)",
                c.work_item_id
            ));
            continue;
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&contents) else {
            failing.push(format!("  {} (malformed kani-report.json)", c.work_item_id));
            continue;
        };
        match v.get("success").and_then(|s| s.as_bool()) {
            Some(true) => {}
            Some(false) => failing.push(format!("  {} success=false", c.work_item_id)),
            None => failing.push(format!(
                "  {} (kani-report.json missing `success` field)",
                c.work_item_id
            )),
        }
    }

    if !failing.is_empty() {
        let mut msg = format!("{} L4+ ticket(s) failed Kani evidence:\n", failing.len());
        for line in &failing {
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

    if !any_report {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: format!(
                "No L4+ ticket has a `kani-report.json` yet ({} eligible)",
                l4_plus.len()
            ),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!("{} L4+ Kani report(s) pass", checked),
        severity: Severity::Info,
    }
}

/// Same whitespace-token shape as `is_l3_or_higher` — handles annotated
/// levels like `"L4 (kani_proof)"`.
fn is_l4_or_higher(contract: &WorkContract) -> bool {
    // Typed since MACS-004: annotated legacy variants ("L4 (kani_proof)")
    // are recovered by the migrating deserializer.
    contract.verification_level >= VerificationLevel::L4
}
