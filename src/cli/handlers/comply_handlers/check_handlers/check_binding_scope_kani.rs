// CB-1605 Kani Harness Execution — included from check_binding_scope.rs.
// Do NOT add `use` imports or `#!` attributes here.

/// Scan a YAML's top-level `kani_harnesses:` block for list-item harness
/// names. Recognizes two shapes:
///
/// ```yaml
/// kani_harnesses:
/// - verify_foo
/// - verify_bar
/// ```
///
/// and the indented variant:
///
/// ```yaml
/// kani_harnesses:
///   - verify_foo
/// ```
///
/// Object form (`- name: verify_foo`) is also accepted; any other keys on the
/// same item are ignored. Returns `None` if the section is absent (or flow-
/// style empty `[]`) — that signals "no harness obligations declared" and
/// callers should skip the binding, not fail.
fn yaml_kani_harness_names(content: &str) -> Option<Vec<String>> {
    let mut in_section = false;
    let mut saw_section = false;
    let mut names: Vec<String> = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // Top-level key resets the section state
        if !line.starts_with(' ') && !line.starts_with('-') {
            if trimmed == "kani_harnesses:" {
                in_section = true;
                saw_section = true;
                continue;
            }
            // `kani_harnesses: []` — flow-style empty. Caller treats as "no decls".
            if let Some(rest) = trimmed.strip_prefix("kani_harnesses:") {
                let rest = rest.trim();
                if rest.starts_with('[') && rest.ends_with(']') {
                    let inner = &rest[1..rest.len() - 1];
                    if inner.trim().is_empty() {
                        return None;
                    }
                    // Flow-style populated list: split on commas
                    return Some(
                        inner
                            .split(',')
                            .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                            .filter(|s| !s.is_empty())
                            .collect(),
                    );
                }
                // Non-flow scalar form — unusual; bail out conservatively
                return None;
            }
            in_section = false;
            continue;
        }
        if !in_section {
            continue;
        }
        // List items under `kani_harnesses:` — either `- name` (string) or
        // `- name: name_val` (object). Accept both.
        if let Some(after_dash) = trimmed.strip_prefix('-') {
            let item = after_dash.trim();
            if item.is_empty() {
                continue;
            }
            // Object form: `- name: verify_foo`
            if let Some(rest) = item.strip_prefix("name:") {
                let value = rest.trim().trim_matches('"').trim_matches('\'').to_string();
                if !value.is_empty() {
                    names.push(value);
                }
                continue;
            }
            // Key-prefixed object with `name:` later: don't try to parse; skip
            if item.contains(':') {
                continue;
            }
            // String scalar
            let cleaned = item.trim_matches('"').trim_matches('\'').to_string();
            if !cleaned.is_empty() {
                names.push(cleaned);
            }
        }
    }
    if saw_section {
        Some(names)
    } else {
        None
    }
}

/// Parse a kani-report.json into a map of harness-name → success bool.
///
/// Accepts two shapes the Component 29 runner is likely to emit:
///
/// 1. `harnesses: [{name, success}]` — canonical
/// 2. `results:   [{name, success}]` — alternate naming
///
/// For each item, `success` may be a bool OR a status string (`"proved"`,
/// `"failed"`, etc.) so this reader coerces to bool conservatively.
fn parse_kani_harness_results(contents: &str) -> Option<Vec<(String, bool)>> {
    let v: serde_json::Value = serde_json::from_str(contents).ok()?;
    let array = v
        .get("harnesses")
        .or_else(|| v.get("results"))
        .and_then(|v| v.as_array())?;
    let mut out = Vec::new();
    for item in array {
        let Some(name) = item.get("name").and_then(|n| n.as_str()) else {
            continue;
        };
        // Prefer `success: bool`; fall back to `status: "proved"` ≈ true.
        let success = if let Some(b) = item.get("success").and_then(|v| v.as_bool()) {
            b
        } else if let Some(s) = item.get("status").and_then(|v| v.as_str()) {
            matches!(s, "proved" | "success" | "pass" | "passed")
        } else {
            false
        };
        out.push((name.to_string(), success));
    }
    Some(out)
}

/// CB-1605 (L4): when a bound YAML declares `kani_harnesses[]`, every named
/// harness must appear in the ticket's `.pmat-work/<ID>/kani-report.json`
/// with a success result. This complements CB-1614's top-level report gate
/// with per-harness granularity.
///
/// Tiered skip semantics:
///   - no contracts with `implements:`                  → Skip
///   - no binding's YAML declares kani_harnesses[]      → Skip
///   - no eligible ticket has a kani-report.json yet    → Skip
///   - else                                             → Pass/Fail
pub(crate) fn check_binding_kani_harnesses(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1605: Kani Harness Execution";
    let contracts = load_active_contracts(project_path);
    if iter_bindings(&contracts).next().is_none() {
        return skip_no_bindings(name);
    }

    let mut bindings_with_harnesses = 0usize;
    let mut reports_seen = 0usize;
    let mut missing_harnesses: Vec<String> = Vec::new();
    let mut failed_harnesses: Vec<String> = Vec::new();
    let mut malformed_reports: Vec<String> = Vec::new();

    for contract in &contracts {
        for binding in &contract.implements {
            let file = if binding.file.is_absolute() {
                binding.file.clone()
            } else {
                project_path.join(&binding.file)
            };
            let Ok(yaml) = std::fs::read_to_string(&file) else {
                continue;
            };
            let Some(declared) = yaml_kani_harness_names(&yaml) else {
                continue;
            };
            if declared.is_empty() {
                continue;
            }
            bindings_with_harnesses += 1;

            let report = project_path
                .join(".pmat-work")
                .join(&contract.work_item_id)
                .join("kani-report.json");
            if !report.exists() {
                continue; // in-progress L4 ticket — overall presence owned by CB-1614
            }
            let Ok(contents) = std::fs::read_to_string(&report) else {
                malformed_reports.push(format!(
                    "  {} [{}] unreadable kani-report.json",
                    contract.work_item_id,
                    binding.key()
                ));
                continue;
            };
            let Some(results) = parse_kani_harness_results(&contents) else {
                malformed_reports.push(format!(
                    "  {} [{}] kani-report.json missing `harnesses`/`results` array",
                    contract.work_item_id,
                    binding.key()
                ));
                continue;
            };
            reports_seen += 1;

            for harness in &declared {
                match results.iter().find(|(n, _)| n == harness) {
                    None => missing_harnesses.push(format!(
                        "  {} [{}] '{}' not in kani-report.json",
                        contract.work_item_id,
                        binding.key(),
                        harness
                    )),
                    Some((_, false)) => failed_harnesses.push(format!(
                        "  {} [{}] '{}' success=false",
                        contract.work_item_id,
                        binding.key(),
                        harness
                    )),
                    Some((_, true)) => {}
                }
            }
        }
    }

    if bindings_with_harnesses == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No bound YAML declares `kani_harnesses:`".into(),
            severity: Severity::Info,
        };
    }

    if reports_seen == 0 && malformed_reports.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: format!(
                "{} binding(s) declare kani_harnesses — no `.pmat-work/<ID>/kani-report.json` yet",
                bindings_with_harnesses
            ),
            severity: Severity::Info,
        };
    }

    if missing_harnesses.is_empty() && failed_harnesses.is_empty() && malformed_reports.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} binding(s) — all declared kani_harnesses succeeded in kani-report.json",
                bindings_with_harnesses
            ),
            severity: Severity::Info,
        };
    }

    let mut msg = String::new();
    if !missing_harnesses.is_empty() {
        msg.push_str(&format!(
            "{} declared harness(es) absent from kani-report.json:\n",
            missing_harnesses.len()
        ));
        for line in &missing_harnesses {
            msg.push_str(line);
            msg.push('\n');
        }
    }
    if !failed_harnesses.is_empty() {
        msg.push_str(&format!(
            "{} harness(es) failed in kani-report.json:\n",
            failed_harnesses.len()
        ));
        for line in &failed_harnesses {
            msg.push_str(line);
            msg.push('\n');
        }
    }
    if !malformed_reports.is_empty() {
        msg.push_str(&format!(
            "{} malformed kani-report.json file(s):\n",
            malformed_reports.len()
        ));
        for line in &malformed_reports {
            msg.push_str(line);
            msg.push('\n');
        }
    }
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Fail,
        message: msg,
        severity: Severity::Error,
    }
}
