// CB-1613: L3 falsification evidence — every line of every L3+ ticket's
// `falsification.log` must report `status: "pass"`. Included into
// `check_work_ladder.rs`.

// ─── CB-1613: L3 falsification evidence ──────────────────────────────────────

/// CB-1613 (L3): L3+ completion requires `.pmat-work/<ID>/falsification.log`
/// present, and every entry in that log must carry `status: "pass"`. Any
/// `fail`, `timeout`, or malformed status gate the ticket from claiming L3.
///
/// Skip semantics (tiered):
///   • no tickets at all                         → Skip
///   • no L3+ ticket on any active contract      → Skip
///   • L3+ tickets exist but none have a log yet → Skip (in-progress tickets
///                                                  haven't run falsification)
///   • any L3+ log has a non-pass entry OR is
///     malformed                                 → Fail
pub(crate) fn check_ladder_l3_falsification(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1613: L3 Falsification Evidence";
    let contracts = load_active_contracts(project_path);
    if contracts.is_empty() {
        return skip_no_contracts(name);
    }

    let l3_plus: Vec<&WorkContract> = contracts.iter().filter(|c| is_l3_or_higher(c)).collect();
    if l3_plus.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No L3+ ticket present".into(),
            severity: Severity::Info,
        };
    }

    let mut any_log_present = false;
    let mut checked = 0usize;
    let mut failing: Vec<String> = Vec::new();

    for c in &l3_plus {
        let log = project_path
            .join(".pmat-work")
            .join(&c.work_item_id)
            .join("falsification.log");
        if !log.exists() {
            continue; // in-progress ticket — per-ticket skip
        }
        any_log_present = true;
        checked += 1;

        let Ok(contents) = std::fs::read_to_string(&log) else {
            failing.push(format!("  {} (unreadable log)", c.work_item_id));
            continue;
        };

        for (idx, raw) in contents.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() {
                continue;
            }
            let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
                failing.push(format!(
                    "  {} line {} (malformed JSON)",
                    c.work_item_id,
                    idx + 1
                ));
                continue;
            };
            let status = v
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("<missing>");
            if status != "pass" {
                let label = v
                    .get("test_id")
                    .and_then(|s| s.as_str())
                    .or_else(|| v.get("method").and_then(|s| s.as_str()))
                    .unwrap_or("?");
                failing.push(format!(
                    "  {} entry '{}' status={}",
                    c.work_item_id, label, status
                ));
            }
        }
    }

    if !failing.is_empty() {
        let mut msg = format!("{} L3+ log entry/entries not passing:\n", failing.len());
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

    if !any_log_present {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: format!(
                "No L3+ ticket has a `falsification.log` yet ({} eligible)",
                l3_plus.len()
            ),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!("{} L3+ log(s) checked, all entries pass", checked),
        severity: Severity::Info,
    }
}

/// Ticket strings are typed like `"L3"` or `"L4 (kani_proof)"` — take the
/// first whitespace-separated token so annotated variants parse too.
fn is_l3_or_higher(contract: &WorkContract) -> bool {
    let token = contract
        .verification_level
        .split_whitespace()
        .next()
        .unwrap_or("");
    VerificationLevel::parse_lenient(token)
        .map(|lvl| lvl >= VerificationLevel::L3)
        .unwrap_or(false)
}
