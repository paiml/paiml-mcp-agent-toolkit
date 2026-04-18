// CB-1644 — Agent run replayable. Validates `.pmat-work/<ID>/agent-runs/*.json`
// entries carry the replay schema (`prompt_sha`, `tool_calls`, `commit_sha`).
// Included from `check_cot_proof.rs` — do NOT add `use` imports or `#!` attributes here.

// ─── CB-164x check implementations (all active, skip-if-absent) ──────────────

/// CB-1644 (L1): Replayability hinges on three fields per recorded agent run:
///   - `prompt_sha` — content hash of the prompt that produced the run
///   - `tool_calls` — ordered trace of tool invocations (array)
///   - `commit_sha` — git commit the run was anchored against
///
/// We scan `.pmat-work/<ID>/agent-runs/*.json`. Entries missing any required
/// field are reported. The check skips cleanly when no `agent-runs/` folder
/// exists for any ticket — Component 10's writer hasn't emitted traces yet.
pub(crate) fn check_agent_run_replayable(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1644: Agent Run Replayable";
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/` directory — agent run writer hasn't executed yet".into(),
            severity: Severity::Info,
        };
    }

    const REQUIRED_FIELDS: &[&str] = &["prompt_sha", "tool_calls", "commit_sha"];

    let mut checked_runs = 0usize;
    let mut malformed: Vec<String> = Vec::new();
    let mut incomplete: Vec<String> = Vec::new();

    let Ok(ticket_entries) = std::fs::read_dir(&work_dir) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "`.pmat-work/` unreadable — agent run writer hasn't executed yet".into(),
            severity: Severity::Info,
        };
    };

    for entry in ticket_entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let Some(ticket_id) = entry.file_name().to_str().map(|s| s.to_string()) else {
            continue;
        };
        if ticket_id.starts_with('.') || ticket_id == "ledger" {
            continue;
        }
        let runs_dir = entry.path().join("agent-runs");
        if !runs_dir.exists() {
            continue;
        }
        let Ok(run_files) = std::fs::read_dir(&runs_dir) else {
            continue;
        };
        for run in run_files.flatten() {
            let path = run.path();
            if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let run_id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("<unknown>")
                .to_string();
            checked_runs += 1;
            let Ok(bytes) = std::fs::read(&path) else {
                malformed.push(format!("{}:{}", ticket_id, run_id));
                continue;
            };
            let Ok(value) = serde_json::from_slice::<Value>(&bytes) else {
                malformed.push(format!("{}:{}", ticket_id, run_id));
                continue;
            };
            let Value::Object(map) = value else {
                malformed.push(format!("{}:{} (not an object)", ticket_id, run_id));
                continue;
            };
            let missing: Vec<&str> = REQUIRED_FIELDS
                .iter()
                .filter(|f| {
                    // Field is absent or null
                    map.get(**f).map(|v| v.is_null()).unwrap_or(true)
                })
                .copied()
                .collect();
            // `tool_calls` must specifically be an array, not any non-null shape
            let tool_calls_ok = map.get("tool_calls").map(|v| v.is_array()).unwrap_or(false);
            if !missing.is_empty() {
                incomplete.push(format!(
                    "{}:{} missing {}",
                    ticket_id,
                    run_id,
                    missing.join(", ")
                ));
            } else if !tool_calls_ok {
                incomplete.push(format!(
                    "{}:{} tool_calls is not an array",
                    ticket_id, run_id
                ));
            }
        }
    }

    if checked_runs == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/<ID>/agent-runs/*.json` files — Component 10 writer hasn't emitted runs yet".into(),
            severity: Severity::Info,
        };
    }

    if malformed.is_empty() && incomplete.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} agent run(s) carry prompt_sha/tool_calls/commit_sha",
                checked_runs
            ),
            severity: Severity::Info,
        };
    }

    let mut msg = String::new();
    if !malformed.is_empty() {
        msg.push_str(&format!(
            "{} unreadable run(s): {}\n",
            malformed.len(),
            malformed.join(", ")
        ));
    }
    if !incomplete.is_empty() {
        msg.push_str(&format!(
            "{} run(s) incomplete:\n  {}",
            incomplete.len(),
            incomplete.join("\n  ")
        ));
    }
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Fail,
        message: msg,
        severity: Severity::Error,
    }
}
