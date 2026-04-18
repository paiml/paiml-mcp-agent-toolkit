// CB-1616: L5 Lean proof zero-sorry — every L5 ticket must ship a
// `lean-proof.json` with `sorry_count: 0`. Included into
// `check_work_ladder.rs`.

// ─── CB-1616: L5 Lean proof zero-sorry ──────────────────────────────────────

/// CB-1616 (L5): every L5 ticket must have `.pmat-work/<ID>/lean-proof.json`
/// present, and that report must carry `sorry_count: 0`. A Lean proof with
/// any admitted `sorry` is not a proof — it's a placeholder.
///
/// Report schema (minimum): `{ "sorry_count": non-negative-integer }`.
///
/// Skip semantics (tiered):
///   • no tickets at all                          → Skip
///   • no L5 ticket on any active contract        → Skip
///   • L5 tickets exist but none have a report    → Skip (Component 24
///                                                   Lean consumer pending)
///   • any L5 report missing `sorry_count`, has
///     non-zero count, negative count, or is
///     malformed                                  → Fail
pub(crate) fn check_ladder_l5_lean(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1616: L5 Lean Proof Zero-Sorry";
    let contracts = load_active_contracts(project_path);
    if contracts.is_empty() {
        return skip_no_contracts(name);
    }

    let l5: Vec<&WorkContract> = contracts.iter().filter(|c| is_l5(c)).collect();
    if l5.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No L5 ticket present".into(),
            severity: Severity::Info,
        };
    }

    let mut any_report = false;
    let mut checked = 0usize;
    let mut failing: Vec<String> = Vec::new();

    for c in &l5 {
        let report = project_path
            .join(".pmat-work")
            .join(&c.work_item_id)
            .join("lean-proof.json");
        if !report.exists() {
            continue;
        }
        any_report = true;
        checked += 1;

        let Ok(contents) = std::fs::read_to_string(&report) else {
            failing.push(format!("  {} (unreadable lean-proof.json)", c.work_item_id));
            continue;
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&contents) else {
            failing.push(format!("  {} (malformed lean-proof.json)", c.work_item_id));
            continue;
        };
        match v.get("sorry_count").and_then(|s| s.as_i64()) {
            Some(0) => {}
            Some(n) if n > 0 => {
                failing.push(format!("  {} sorry_count={}", c.work_item_id, n));
            }
            Some(n) => {
                failing.push(format!(
                    "  {} sorry_count={} (must be non-negative)",
                    c.work_item_id, n
                ));
            }
            None => failing.push(format!(
                "  {} (lean-proof.json missing `sorry_count` integer)",
                c.work_item_id
            )),
        }
    }

    if !failing.is_empty() {
        let mut msg = format!("{} L5 ticket(s) failed Lean evidence:\n", failing.len());
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
                "No L5 ticket has a `lean-proof.json` yet ({} eligible)",
                l5.len()
            ),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!("{} L5 Lean proof(s) discharge with zero sorry", checked),
        severity: Severity::Info,
    }
}

/// L5 is a single point on the ladder — exact match, not `>= L5`.
fn is_l5(contract: &WorkContract) -> bool {
    let token = contract
        .verification_level
        .split_whitespace()
        .next()
        .unwrap_or("");
    VerificationLevel::parse_lenient(token) == Some(VerificationLevel::L5)
}
