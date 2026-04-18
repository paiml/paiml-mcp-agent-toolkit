// CB-1608 Cross-Binding Consistency — included from check_binding_scope.rs.
// Do NOT add `use` imports or `#!` attributes here.

/// CB-1608 (L1): cross-binding consistency — a multi-bind ticket cannot
/// report some bindings passing while other bindings have failing
/// falsification entries. Either all bindings are green, all unknown, or
/// the ticket is not eligible for completion.
///
/// # Skip semantics (tiered)
///
/// * no `.pmat-work/*/contract.json` tickets                          → Skip
/// * no multi-bind ticket has a `.pmat-work/<ID>/falsification.log`   → Skip
///
/// # Fail
///
/// * any multi-bind ticket has at least one binding where **every**
///   logged entry is `status: "pass"` AND at least one other binding
///   where **any** entry has `status != "pass"`. Bindings with no log
///   evidence at all are ignored here — CB-1622 owns that gap.
///
/// # Pass
///
/// * every multi-bind ticket with a log is either uniformly green
///   across its bindings or has no evidence mixed with failures
pub(crate) fn check_binding_cross_consistency(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1608: Cross-Binding Consistency";
    let contracts = load_active_contracts(project_path);
    if contracts.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/*/contract.json` tickets present".into(),
            severity: Severity::Info,
        };
    }

    let mut evaluated = 0usize;
    let mut violations: Vec<String> = Vec::new();

    for c in &contracts {
        if c.implements.len() < 2 {
            continue;
        }
        let log_path = project_path
            .join(".pmat-work")
            .join(&c.work_item_id)
            .join("falsification.log");
        let Ok(contents) = std::fs::read_to_string(&log_path) else {
            continue;
        };
        evaluated += 1;

        // (yaml, equation) → (has_entry, all_entries_pass)
        let entries = parse_inherited_log_entries(&contents);
        let mut passing: Vec<String> = Vec::new();
        let mut failing: Vec<String> = Vec::new();
        for b in &c.implements {
            let mut saw_any = false;
            let mut saw_fail = false;
            for (yaml, eq, status) in &entries {
                if yaml == &b.file && eq == &b.equation {
                    saw_any = true;
                    if status != "pass" {
                        saw_fail = true;
                    }
                }
            }
            if !saw_any {
                continue;
            }
            let label = format!("{}#{}", b.file.display(), b.equation);
            if saw_fail {
                failing.push(label);
            } else {
                passing.push(label);
            }
        }
        if !passing.is_empty() && !failing.is_empty() {
            violations.push(format!(
                "  {} — passing: [{}] vs failing: [{}]",
                c.work_item_id,
                passing.join(", "),
                failing.join(", ")
            ));
        }
    }

    if evaluated == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No multi-bind ticket has a `.pmat-work/<ID>/falsification.log` yet".into(),
            severity: Severity::Info,
        };
    }

    if !violations.is_empty() {
        let mut msg = format!(
            "{} multi-bind ticket(s) show inconsistent per-binding outcomes:\n",
            violations.len()
        );
        for line in &violations {
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
            "{} multi-bind ticket(s) show consistent binding outcomes",
            evaluated
        ),
        severity: Severity::Info,
    }
}

/// Parse a `falsification.log` JSONL blob into `(yaml_path, equation, status)`
/// tuples. Skips malformed lines and non-inherited entries (those missing
/// `yaml` or `equation`) — CB-1628 owns malformed-line detection.
fn parse_inherited_log_entries(contents: &str) -> Vec<(std::path::PathBuf, String, String)> {
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
        let Some(eq) = v.get("equation").and_then(|y| y.as_str()) else {
            continue;
        };
        let Some(status) = v.get("status").and_then(|s| s.as_str()) else {
            continue;
        };
        out.push((
            std::path::PathBuf::from(yaml),
            eq.to_string(),
            status.to_string(),
        ));
    }
    out
}
