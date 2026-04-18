// CB-1612: L1 test evidence — `cargo test --lib` green recorded in the
// ticket's `verification-report.json`. Included into `check_work_ladder.rs`.

// ─── CB-161x check implementations (all active, skip-if-absent) ──────────────

/// CB-1612 (L3): L1 completion requires `cargo test --lib` green. Reads the
/// evidence that `pmat work verify` writes into each ticket's
/// `.pmat-work/<ID>/verification-report.json` under the `l1_test_evidence`
/// key. Accepted shapes (all case-insensitive on status strings):
///   • `"l1_test_evidence": true`                          → pass
///   • `"l1_test_evidence": {"success": true}`             → pass
///   • `"l1_test_evidence": {"exit_code": 0}`              → pass
///   • `"l1_test_evidence": {"status": "pass"|"passed"|"ok"|"success"}`
///     → pass
/// Anything else (false, non-zero exit, `status: fail` etc.) → fail.
///
/// Skip semantics (tiered):
///   • no tickets at all                             → Skip
///   • no ticket has `verification-report.json`      → Skip
///   • reports exist but none carry
///     `l1_test_evidence` (writer pending)           → Skip
///   • any report's `l1_test_evidence` shape
///     indicates failure                             → Fail
pub(crate) fn check_ladder_l1_test_evidence(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1612: L1 Test Evidence";
    let contracts = load_active_contracts(project_path);
    if contracts.is_empty() {
        return skip_no_contracts(name);
    }

    let mut any_report = false;
    let mut any_evidence = false;
    let mut checked = 0usize;
    let mut failing: Vec<String> = Vec::new();

    for c in &contracts {
        let report = project_path
            .join(".pmat-work")
            .join(&c.work_item_id)
            .join("verification-report.json");
        if !report.exists() {
            continue;
        }
        any_report = true;
        let Ok(content) = std::fs::read_to_string(&report) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
            continue;
        };
        let Some(evidence) = value.get("l1_test_evidence") else {
            continue;
        };
        any_evidence = true;
        checked += 1;
        match evaluate_l1_evidence(evidence) {
            L1Outcome::Pass => {}
            L1Outcome::Fail(reason) => {
                failing.push(format!("  {} → {}", c.work_item_id, reason));
            }
        }
    }

    if !failing.is_empty() {
        let mut msg = format!("{} ticket(s) failed L1 test evidence:\n", failing.len());
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
            message: "No ticket has `verification-report.json` yet".into(),
            severity: Severity::Info,
        };
    }
    if !any_evidence {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No verification-report.json carries `l1_test_evidence` yet".into(),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!("{} ticket(s) recorded green L1 test evidence", checked),
        severity: Severity::Info,
    }
}

enum L1Outcome {
    Pass,
    Fail(String),
}

fn evaluate_l1_evidence(v: &serde_json::Value) -> L1Outcome {
    // Boolean shorthand — `true` is pass, `false` is fail.
    if let Some(b) = v.as_bool() {
        return if b {
            L1Outcome::Pass
        } else {
            L1Outcome::Fail("evidence=false".into())
        };
    }
    if let Some(obj) = v.as_object() {
        if let Some(s) = obj.get("success").and_then(|x| x.as_bool()) {
            return if s {
                L1Outcome::Pass
            } else {
                L1Outcome::Fail("success=false".into())
            };
        }
        if let Some(code) = obj.get("exit_code").and_then(|x| x.as_i64()) {
            return if code == 0 {
                L1Outcome::Pass
            } else {
                L1Outcome::Fail(format!("exit_code={}", code))
            };
        }
        if let Some(status) = obj.get("status").and_then(|x| x.as_str()) {
            let lowered = status.to_ascii_lowercase();
            return if matches!(lowered.as_str(), "pass" | "passed" | "ok" | "success") {
                L1Outcome::Pass
            } else {
                L1Outcome::Fail(format!("status={}", status))
            };
        }
    }
    L1Outcome::Fail("unrecognized evidence shape".into())
}
