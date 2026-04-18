// CB-1618: Level monotonicity across checkpoints — a ticket's recorded
// `verification_level` across `.pmat-work/<ID>/checkpoints/*.json` must be
// non-decreasing unless a downgrade ledger entry audits the regression.
// Included into `check_work_ladder.rs`.

/// CB-1618 (L1): level monotonicity across ticket checkpoints — a ticket
/// cannot drop from L3 → L1 → L3 without an audit. Scans
/// `.pmat-work/<ID>/checkpoints/*.json` for the optional `verification_level`
/// field. Any regression in the per-ticket checkpoint timeline must be
/// matched by an entry in `.pmat-work/ledger/downgrades.json` (the CB-1617
/// audit surface — presence alone is enough here; CB-1617 validates the
/// `reason` content).
///
/// Skip semantics (tiered):
///   • no `.pmat-work/` directory                      → Skip
///   • no ticket has a non-empty `checkpoints/` dir    → Skip
///   • checkpoints exist but none carry
///     `verification_level` yet (writer pending)       → Skip
///   • per-ticket: fewer than 2 leveled checkpoints    → ignored (not enough
///                                                        history to judge)
///   • regression found AND ticket missing from
///     downgrade ledger                                → Fail
pub(crate) fn check_ladder_monotonicity(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1618: Level Monotonicity";
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/` directory".into(),
            severity: Severity::Info,
        };
    }

    let tickets = collect_ticket_checkpoints(&work_dir);
    if tickets.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ticket has a non-empty `checkpoints/` directory".into(),
            severity: Severity::Info,
        };
    }

    let audited = load_audited_tickets(project_path);
    let mut checked_tickets = 0usize;
    let mut unaudited_regressions: Vec<String> = Vec::new();

    for (ticket_id, cp_files) in &tickets {
        let leveled = load_leveled_timeline(cp_files);
        if leveled.len() < 2 {
            continue; // per-ticket skip: insufficient history
        }
        checked_tickets += 1;

        if has_regression(&leveled) && !audited.contains(ticket_id) {
            unaudited_regressions.push(format!(
                "  {}: checkpoint level regresses without a downgrade ledger entry ({})",
                ticket_id,
                summarize_timeline(&leveled),
            ));
        }
    }

    if !unaudited_regressions.is_empty() {
        let mut msg = format!(
            "{} ticket(s) regressed without audit:\n",
            unaudited_regressions.len()
        );
        for line in &unaudited_regressions {
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

    if checked_tickets == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No checkpoint records the `verification_level` field yet".into(),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!("{} ticket(s) monotonic across checkpoints", checked_tickets),
        severity: Severity::Info,
    }
}

/// Scan `.pmat-work/<ID>/checkpoints/*.json`, ignoring the `ledger` and
/// hidden directories. Returns `(ticket_id, sorted_checkpoint_paths)` pairs
/// for tickets with at least one JSON file. Path sort is by filename, which
/// — given the `checkpoint-<uuid>.json` shape — does not encode time. The
/// caller sorts further by the `timestamp` field when loading.
fn collect_ticket_checkpoints(work_dir: &Path) -> Vec<(String, Vec<std::path::PathBuf>)> {
    let Ok(entries) = std::fs::read_dir(work_dir) else {
        return Vec::new();
    };
    let mut out: Vec<(String, Vec<std::path::PathBuf>)> = Vec::new();
    for e in entries.flatten() {
        if !e.path().is_dir() {
            continue;
        }
        let Some(id) = e.file_name().to_str().map(String::from) else {
            continue;
        };
        if id.starts_with('.') || id == "ledger" {
            continue;
        }
        let cp_dir = e.path().join("checkpoints");
        if !cp_dir.exists() {
            continue;
        }
        let Ok(files) = std::fs::read_dir(&cp_dir) else {
            continue;
        };
        let mut cp_files: Vec<std::path::PathBuf> = files
            .flatten()
            .filter(|f| f.path().extension().and_then(|s| s.to_str()) == Some("json"))
            .map(|f| f.path())
            .collect();
        if cp_files.is_empty() {
            continue;
        }
        cp_files.sort();
        out.push((id, cp_files));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Load each checkpoint, keep only those carrying `verification_level`,
/// sort by `timestamp` (stable). Unparseable files are silently dropped.
fn load_leveled_timeline(cp_files: &[std::path::PathBuf]) -> Vec<(String, VerificationLevel)> {
    let mut rows: Vec<(String, VerificationLevel)> = Vec::new();
    for path in cp_files {
        let Ok(body) = std::fs::read_to_string(path) else {
            continue;
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) else {
            continue;
        };
        let ts = v
            .get("timestamp")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string();
        let Some(lvl) = v
            .get("verification_level")
            .and_then(|s| s.as_str())
            .and_then(VerificationLevel::parse_lenient)
        else {
            continue;
        };
        rows.push((ts, lvl));
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows
}

fn has_regression(timeline: &[(String, VerificationLevel)]) -> bool {
    timeline.windows(2).any(|pair| pair[1].1 < pair[0].1)
}

fn summarize_timeline(timeline: &[(String, VerificationLevel)]) -> String {
    let levels: Vec<&str> = timeline.iter().map(|(_, l)| l.as_str()).collect();
    levels.join(" → ")
}

/// Set of `ticket` ids appearing at least once in the downgrade ledger.
/// Absent/malformed ledger returns an empty set (CB-1617 reports that).
fn load_audited_tickets(project_path: &Path) -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    let ledger = project_path
        .join(".pmat-work")
        .join("ledger")
        .join("downgrades.json");
    let Ok(content) = std::fs::read_to_string(&ledger) else {
        return out;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return out;
    };
    let Some(array) = value.as_array() else {
        return out;
    };
    for e in array {
        if let Some(t) = e.get("ticket").and_then(|v| v.as_str()) {
            out.insert(t.to_string());
        }
    }
    out
}
