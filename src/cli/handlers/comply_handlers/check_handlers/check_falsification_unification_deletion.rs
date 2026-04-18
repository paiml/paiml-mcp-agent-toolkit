// Work Falsification Unification — CB-1624 manual-deletion audit.
//
// Included from `check_falsification_unification.rs` — do NOT add `use`
// imports or `#!` attributes here.
//
// Contains:
//   CB-1624 — check_no_manual_deletion (L1): deletions in
//             `.pmat-work/ledger/roster-mutations.json` must carry
//             `via_unbind: true`.

/// CB-1624 (L1): `ProvableContract{}` entries cannot be deleted outside the
/// `pmat work unbind` path. The roster-mutations ledger at
/// `.pmat-work/ledger/roster-mutations.json` records every add/remove event
/// on a ticket's inherited roster. Deletion entries must carry `via_unbind:
/// true` (the `unbind` command sets this when writing the mutation) — any
/// other deletion represents a manual `contract.json` edit that bypassed
/// the unbind audit and erodes scope silently.
///
/// Ledger schema (minimum): JSON array at
/// `.pmat-work/ledger/roster-mutations.json` where each entry has:
///   • `ticket`        — the work-item-id whose roster changed
///   • `action`        — one of `"delete"`, `"add"`, `"update"`; deletion
///                        family matched case-insensitively on the `"delet"`
///                        prefix (so `delete`, `Delete`, `DELETED`,
///                        `deletion` all trigger the via_unbind check)
///   • `target`        — `{ yaml, equation, test_id }` identifying the
///                        inherited entry that was mutated (informational;
///                        this check does not validate target shape)
///   • `via_unbind`    — boolean flag that `pmat work unbind` sets to true;
///                        deletion without this flag is a violation
///
/// Skip semantics (tiered):
///   • no `.pmat-work/` directory                → Skip
///   • no `ledger/roster-mutations.json` file    → Skip (mutation writer
///                                                  hasn't landed)
///   • ledger exists but contains no entries     → Pass (nothing to audit)
///   • any entry has unrecognized shape          → Warn (per-entry)
///   • any deletion entry lacks `via_unbind`     → Fail
pub(crate) fn check_no_manual_deletion(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1624: No Manual Deletion of Inherited Entries";
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/` directory".into(),
            severity: Severity::Info,
        };
    }

    let ledger = work_dir.join("ledger").join("roster-mutations.json");
    if !ledger.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No roster-mutations ledger at .pmat-work/ledger/roster-mutations.json".into(),
            severity: Severity::Info,
        };
    }

    let Ok(content) = std::fs::read_to_string(&ledger) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Warn,
            message: format!("Unreadable roster-mutations ledger: {}", ledger.display()),
            severity: Severity::Warning,
        };
    };

    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: "Roster-mutations ledger is not valid JSON".into(),
            severity: Severity::Error,
        };
    };

    let Some(entries) = value.as_array() else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: "Roster-mutations ledger must be a JSON array of entries".into(),
            severity: Severity::Error,
        };
    };

    let mut manual_deletions: Vec<String> = Vec::new();
    let mut deletions_checked = 0usize;
    let total = entries.len();

    for (i, entry) in entries.iter().enumerate() {
        let action = entry
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        // `delet` catches delete, Delete, DELETED, deletion — any deletion
        // verb family — while ignoring unrelated actions like `add`/`update`.
        if !action.starts_with("delet") {
            continue;
        }
        deletions_checked += 1;
        let via_unbind = entry
            .get("via_unbind")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if via_unbind {
            continue;
        }
        let ticket = entry
            .get("ticket")
            .and_then(|v| v.as_str())
            .unwrap_or("<unknown>");
        let target_label = entry
            .get("target")
            .and_then(|t| {
                let y = t.get("yaml").and_then(|v| v.as_str()).unwrap_or("?");
                let tid = t.get("test_id").and_then(|v| v.as_str()).unwrap_or("?");
                Some(format!("{}#{}", y, tid))
            })
            .unwrap_or_else(|| "?".to_string());
        manual_deletions.push(format!(
            "  entry[{}] ticket={} target={} action={}",
            i, ticket, target_label, action
        ));
    }

    if !manual_deletions.is_empty() {
        let mut msg = format!(
            "{} manual deletion(s) bypassed `pmat work unbind`:\n",
            manual_deletions.len()
        );
        for line in &manual_deletions {
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

    if total == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: "Roster-mutations ledger is empty — no roster mutations to audit".into(),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!(
            "{} mutation(s) audited, {} deletion(s) all via `pmat work unbind`",
            total, deletions_checked
        ),
        severity: Severity::Info,
    }
}
