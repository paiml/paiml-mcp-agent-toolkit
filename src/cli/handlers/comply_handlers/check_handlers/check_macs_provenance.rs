// Included from check_macs.rs — do NOT add `use` imports or `#!` attributes here.

/// CB-1651: every schema_version>=2 receipt carries agent provenance.
pub(crate) fn check_receipt_provenance_present(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1651: Receipt Provenance Present";
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return skip_check(name, "No .pmat-work directory");
    }

    let mut v2_total = 0usize;
    let mut violations: Vec<String> = Vec::new();

    for receipt_path in macs_receipt_files(&work_dir) {
        let Ok(text) = std::fs::read_to_string(&receipt_path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        let schema_version = value
            .get("schema_version")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(1);
        if schema_version < 2 {
            continue; // pre-MACS receipts are exempt (provenance did not exist)
        }
        v2_total += 1;
        let has_agent = value.get("agent").is_some_and(|a| !a.is_null());
        if !has_agent {
            violations.push(format!(
                "{} (schema_version={}, no agent)",
                receipt_path.display(),
                schema_version
            ));
        }
    }

    if v2_total == 0 {
        return skip_check(name, "No schema_version>=2 receipts yet");
    }
    if violations.is_empty() {
        return ComplianceCheck {
            name: name.to_string(),
            status: CheckStatus::Pass,
            message: format!("{v2_total} v2 receipt(s), all carry agent provenance"),
            severity: Severity::Info,
        };
    }
    ComplianceCheck {
        name: name.to_string(),
        status: CheckStatus::Fail,
        message: format!(
            "{} v2 receipt(s) lack agent provenance:\n{}",
            violations.len(),
            format_violation_list(&violations)
        ),
        severity: Severity::Error,
    }
}

/// CB-1654: no ticket carries an unacknowledged Refusal event.
pub(crate) fn check_refusal_events_acked(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1654: Refusal Events Acked";
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return skip_check(name, "No .pmat-work directory");
    }

    let mut journals = 0usize;
    let mut violations: Vec<String> = Vec::new();

    for (ticket, events_path) in macs_event_journals(&work_dir) {
        journals += 1;
        let Ok(text) = std::fs::read_to_string(&events_path) else {
            continue;
        };
        let mut refusals: Vec<String> = Vec::new();
        let mut acked: std::collections::HashSet<String> = std::collections::HashSet::new();
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
                continue;
            };
            let record_id = value
                .get("id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_string();
            match value
                .get("event")
                .and_then(|e| e.get("type"))
                .and_then(serde_json::Value::as_str)
            {
                Some("refusal") => refusals.push(record_id),
                Some("ack") => {
                    if let Some(ack_of) = value
                        .get("event")
                        .and_then(|e| e.get("ack_of"))
                        .and_then(serde_json::Value::as_str)
                    {
                        acked.insert(ack_of.to_string());
                    }
                }
                _ => {}
            }
        }
        for refusal_id in refusals.into_iter().filter(|r| !acked.contains(r)) {
            violations.push(format!("{ticket}: unacked refusal {refusal_id}"));
        }
    }

    if journals == 0 {
        return skip_check(name, "No event journals (.pmat-work/*/events.jsonl) yet");
    }
    if violations.is_empty() {
        return ComplianceCheck {
            name: name.to_string(),
            status: CheckStatus::Pass,
            message: format!("{journals} event journal(s), no unacked refusals"),
            severity: Severity::Info,
        };
    }
    ComplianceCheck {
        name: name.to_string(),
        status: CheckStatus::Fail,
        message: format!(
            "{} unacked refusal event(s) (MACS E5 — acknowledge with `pmat work event --ack-event`):\n{}",
            violations.len(),
            format_violation_list(&violations)
        ),
        severity: Severity::Error,
    }
}

/// All falsification receipt files under `.pmat-work/*/falsification/receipt-*.json`.
fn macs_receipt_files(work_dir: &Path) -> Vec<std::path::PathBuf> {
    let mut receipts = Vec::new();
    let Ok(entries) = std::fs::read_dir(work_dir) else {
        return receipts;
    };
    for entry in entries.flatten() {
        let falsification_dir = entry.path().join("falsification");
        let Ok(files) = std::fs::read_dir(&falsification_dir) else {
            continue;
        };
        for file in files.flatten() {
            let path = file.path();
            let is_receipt = path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("receipt-") && n.ends_with(".json"));
            if is_receipt {
                receipts.push(path);
            }
        }
    }
    receipts.sort();
    receipts
}

/// All `(ticket_id, events.jsonl path)` journals under `.pmat-work/`.
fn macs_event_journals(work_dir: &Path) -> Vec<(String, std::path::PathBuf)> {
    let mut journals = Vec::new();
    let Ok(entries) = std::fs::read_dir(work_dir) else {
        return journals;
    };
    for entry in entries.flatten() {
        let path = entry.path().join("events.jsonl");
        if path.exists() {
            let ticket = entry.file_name().to_string_lossy().to_string();
            journals.push((ticket, path));
        }
    }
    journals.sort();
    journals
}
