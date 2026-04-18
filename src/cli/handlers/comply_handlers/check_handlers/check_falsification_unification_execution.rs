// Work Falsification Unification — CB-1622 roster execution coverage and
// CB-1625 inherited failure fatal.
//
// Included from `check_falsification_unification.rs` — do NOT add `use`
// imports or `#!` attributes here.
//
// Contains:
//   parse_falsification_log — shared JSONL parser helper.
//   CB-1622 — check_roster_execution_coverage (L3): every roster entry has
//             a receipt line in .pmat-work/<ID>/falsification.log.
//   CB-1625 — check_inherited_failure_fatal (L3): any inherited log line
//             with status != "pass" is fatal regardless of level.

/// Parse a `falsification.log` JSONL file into the set of
/// `(yaml_path, test_id)` pairs it covers. The format (per Component 29
/// §falsification.log) is one JSON object per line:
/// `{"yaml":"...","equation":"...","test_id":"...","status":"pass",...}`.
/// Malformed or non-inherited lines (manual-source entries have no `yaml`)
/// are silently ignored — we only care which roster entries executed.
fn parse_falsification_log(contents: &str) -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let Some(yaml) = v.get("yaml").and_then(|y| y.as_str()) else {
            continue;
        };
        let Some(test_id) = v.get("test_id").and_then(|t| t.as_str()) else {
            continue;
        };
        out.push((PathBuf::from(yaml), test_id.to_string()));
    }
    out
}

/// CB-1622 (L3): every `ProvableContract` roster entry must have at least
/// one execution line in `.pmat-work/<ID>/falsification.log`. A roster
/// entry without a receipt is an "untested claim" — it declares coverage
/// the runner never verified.
///
/// Skip-if-absent: the unified falsification runner (Component 29) that
/// emits `falsification.log` hasn't landed, so today's check skips
/// cleanly when no ticket has the log. Once any ticket gains a log, this
/// check lights up for that ticket while others still skip.
pub(crate) fn check_roster_execution_coverage(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1622: Roster Execution Coverage";
    let contracts = load_active_contracts(project_path);

    let mut uncovered: Vec<String> = Vec::new();
    let mut checked = 0usize;

    for c in &contracts {
        let entries = provable_contract_entries(c);
        if entries.is_empty() {
            continue;
        }
        let log_path = project_path
            .join(".pmat-work")
            .join(&c.work_item_id)
            .join("falsification.log");
        let Ok(contents) = std::fs::read_to_string(&log_path) else {
            // No log for this ticket → skip per-ticket, don't fail
            continue;
        };
        let covered = parse_falsification_log(&contents);
        checked += 1;
        for (yaml_path, _eq, test_id) in entries {
            let present = covered
                .iter()
                .any(|(y, t)| y == &yaml_path && t == &test_id);
            if !present {
                uncovered.push(format!(
                    "{} → {}#{}",
                    c.work_item_id,
                    yaml_path.display(),
                    test_id
                ));
            }
        }
    }

    if checked == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/<ID>/falsification.log` files — unified runner hasn't executed rosters yet".into(),
            severity: Severity::Info,
        };
    }

    if uncovered.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} ticket(s): every roster entry has an execution receipt",
                checked
            ),
            severity: Severity::Info,
        };
    }

    let preview: Vec<String> = uncovered.iter().take(5).cloned().collect();
    let more = uncovered.len().saturating_sub(5);
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Fail,
        message: format!(
            "{} roster entry/entries never executed — run `pmat work falsify`: {}{}",
            uncovered.len(),
            preview.join(", "),
            if more > 0 {
                format!(", +{} more", more)
            } else {
                String::new()
            }
        ),
        severity: Severity::Error,
    }
}

/// CB-1625 (L3): any *inherited* falsification log line with `status` other
/// than `"pass"` is fatal, regardless of the ticket's verification level.
/// An inherited entry that failed once stays failed until a subsequent pass
/// supersedes it — the runner appends, never mutates, so a trailing `fail`
/// means the bound equation is still unproven.
///
/// Scope:
///   • Only *inherited* lines (those carrying `yaml`+`test_id`) count —
///     manual `method`-keyed lines are governed by CB-1629's L4 timeout gate
///     and caller-specific logic, not this blanket rule
///   • Scans every ticket regardless of `verification_level` — if you ran
///     a roster test, its result is binding
///
/// Skip-if-absent: no `.pmat-work/*/falsification.log` files → overall skip.
pub(crate) fn check_inherited_failure_fatal(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1625: Inherited Failure Fatal";
    let contracts = load_active_contracts(project_path);

    let mut checked_logs = 0usize;
    let mut checked_lines = 0usize;
    let mut failing: Vec<String> = Vec::new();

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
            let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) else {
                continue; // CB-1628 owns malformed JSON detection
            };
            if !is_inherited_receipt(&v) {
                continue;
            }
            let Some(status) = v.get("status").and_then(|s| s.as_str()) else {
                continue; // CB-1628 owns missing-field detection
            };
            checked_lines += 1;
            if status != "pass" {
                let label = v
                    .get("test_id")
                    .and_then(|s| s.as_str())
                    .unwrap_or("<no-test-id>");
                let yaml = v
                    .get("yaml")
                    .and_then(|s| s.as_str())
                    .unwrap_or("<no-yaml>");
                failing.push(format!(
                    "  {}:{} {}::{} status={}",
                    c.work_item_id,
                    idx + 1,
                    yaml,
                    label,
                    status
                ));
            }
        }
    }

    if checked_logs == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/<ID>/falsification.log` files to inspect".into(),
            severity: Severity::Info,
        };
    }

    if !failing.is_empty() {
        let mut msg = format!(
            "{} inherited falsification line(s) did not pass — contract is falsified:\n",
            failing.len()
        );
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

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!(
            "All {} inherited line(s) across {} log(s) passed",
            checked_lines, checked_logs
        ),
        severity: Severity::Info,
    }
}
