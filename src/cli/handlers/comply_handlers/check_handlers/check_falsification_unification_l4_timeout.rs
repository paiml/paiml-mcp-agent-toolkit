// Work Falsification Unification — CB-1629 L4 timeout gate.
//
// Included from `check_falsification_unification.rs` — do NOT add `use`
// imports or `#!` attributes here.
//
// Contains:
//   is_l4_or_higher — level parser helper.
//   CB-1629 — check_l4_timeout_gate (L4): L4+ tickets must not record any
//             `status: "timeout"` line in their falsification.log.

/// Return true if a ticket's declared `verification_level` parses to L4+.
/// Ticket strings are typed like `"L3"` or `"L4 (kani_proof)"` — we take
/// the first whitespace-separated token so annotated variants parse too.
fn is_l4_or_higher(contract: &WorkContract) -> bool {
    // Typed since MACS-004: annotated legacy variants ("L4 (kani_proof)")
    // are recovered by the migrating deserializer.
    contract.verification_level >= VerificationLevel::L4
}

/// CB-1629 (L4): an L4+ ticket's `falsification.log` must not record any
/// `status: "timeout"` line. L4 correctness depends on completed Kani
/// verification; a timed-out Kani harness is indistinguishable from an
/// unbounded counterexample and must not be claimed as passed.
///
/// Skip-if-absent: no L4+ ticket with a log → skip overall. Manual-source
/// timeouts also fail (the timeout semantics are level-gated, not
/// source-gated: an L4 ticket cannot admit timeouts of any kind).
pub(crate) fn check_l4_timeout_gate(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1629: L4 Timeout Gate";
    let contracts = load_active_contracts(project_path);

    let mut timeouts: Vec<String> = Vec::new();
    let mut checked = 0usize;

    for c in &contracts {
        if !is_l4_or_higher(c) {
            continue;
        }
        let log_path = project_path
            .join(".pmat-work")
            .join(&c.work_item_id)
            .join("falsification.log");
        let Ok(contents) = std::fs::read_to_string(&log_path) else {
            continue;
        };
        checked += 1;
        for (idx, line) in contents.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) else {
                continue;
            };
            let status = v.get("status").and_then(|s| s.as_str()).unwrap_or("");
            if status.eq_ignore_ascii_case("timeout") {
                let label = v
                    .get("test_id")
                    .or_else(|| v.get("method"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("?");
                timeouts.push(format!("{}:{} ({})", c.work_item_id, idx + 1, label));
            }
        }
    }

    if checked == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No L4+ ticket has a `falsification.log` to check".into(),
            severity: Severity::Info,
        };
    }

    if timeouts.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!("{} L4+ ticket(s): no timeouts recorded", checked),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Fail,
        message: format!(
            "{} timeout(s) in L4+ ticket log(s) — Kani-adjacent flakes defeat the level: {}",
            timeouts.len(),
            timeouts.join(", ")
        ),
        severity: Severity::Error,
    }
}
