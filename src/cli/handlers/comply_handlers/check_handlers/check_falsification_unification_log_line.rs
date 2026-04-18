// Work Falsification Unification — CB-1628 per-run log line shape.
//
// Included from `check_falsification_unification.rs` — do NOT add `use`
// imports or `#!` attributes here.
//
// Contains:
//   CB-1628 — check_per_run_log_line (L3): every inherited log line must
//             carry `{yaml, test_id, status, duration_ms}`.

/// CB-1628 (L3): each line in `.pmat-work/<ID>/falsification.log` that
/// represents an inherited run must carry the 4-field shape
/// `{yaml, test_id, status, duration_ms}`. Missing fields mean the
/// emitter dropped data — silent skips are indistinguishable from real
/// passes post-hoc. Manual-source lines (no `yaml`) are ignored per spec.
///
/// Skip-if-absent: no falsification.log files → skip overall.
pub(crate) fn check_per_run_log_line(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1628: Per-run Log Line Emitted";
    let contracts = load_active_contracts(project_path);

    let mut malformed: Vec<String> = Vec::new();
    let mut missing_fields: Vec<String> = Vec::new();
    let mut checked_logs = 0usize;
    let mut checked_lines = 0usize;

    for c in &contracts {
        let log_path = project_path
            .join(".pmat-work")
            .join(&c.work_item_id)
            .join("falsification.log");
        let Ok(contents) = std::fs::read_to_string(&log_path) else {
            continue;
        };
        checked_logs += 1;
        for (idx, line) in contents.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let lineno = idx + 1;
            let v: serde_json::Value = match serde_json::from_str(trimmed) {
                Ok(v) => v,
                Err(_) => {
                    malformed.push(format!("{}:{}", c.work_item_id, lineno));
                    continue;
                }
            };
            if !is_inherited_receipt(&v) {
                continue;
            }
            checked_lines += 1;
            let mut missing: Vec<&'static str> = Vec::new();
            for field in ["yaml", "test_id", "status", "duration_ms"] {
                if v.get(field).is_none() {
                    missing.push(field);
                }
            }
            if !missing.is_empty() {
                missing_fields.push(format!(
                    "{}:{} missing [{}]",
                    c.work_item_id,
                    lineno,
                    missing.join(", ")
                ));
            }
        }
    }

    if checked_logs == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/<ID>/falsification.log` files to validate".into(),
            severity: Severity::Info,
        };
    }

    if malformed.is_empty() && missing_fields.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} log(s), {} inherited line(s) carry the 4-field shape",
                checked_logs, checked_lines
            ),
            severity: Severity::Info,
        };
    }

    let mut msg = String::new();
    if !malformed.is_empty() {
        msg.push_str(&format!(
            "{} malformed JSONL line(s): {}\n",
            malformed.len(),
            malformed.join(", ")
        ));
    }
    if !missing_fields.is_empty() {
        msg.push_str(&format!(
            "{} line(s) missing required fields: {}",
            missing_fields.len(),
            missing_fields.join("; ")
        ));
    }
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Fail,
        message: msg,
        severity: Severity::Error,
    }
}
