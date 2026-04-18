// CB-1615: Kani harness SHA drift — bind-time harness hash snapshot must
// match the current YAML's per-harness `sha:` fields. Included into
// `check_work_ladder.rs`.

/// CB-1615 (L4): Kani harness hash in ticket must match harness hash in YAML
/// at bind time — catches post-bind drift where a harness body is edited
/// without re-binding the ticket.
///
/// # Schema
///
/// Bind-time snapshot lives at `.pmat-work/<ID>/kani-harness-shas.json`.
/// Component 27's `pmat work bind` is expected to emit it; this check
/// tolerates two shapes until the writer converges:
///
/// ```json
/// { "harnesses": [ { "name": "verify_foo", "sha": "abc…" } ] }
/// ```
///
/// ```json
/// { "harnesses": { "verify_foo": "abc…" } }
/// ```
///
/// Current harness hash is read from the bound YAML's `kani_harnesses:`
/// block, accepting object-form entries with `sha:` siblings:
///
/// ```yaml
/// kani_harnesses:
///   - name: verify_foo
///     sha: abc…
/// ```
///
/// # Skip semantics (tiered)
///
/// * no `.pmat-work/` tickets                       → Skip
/// * no L4+ ticket on any active contract           → Skip
/// * no L4+ ticket has `implements:` bindings       → Skip
/// * no L4+ ticket has a snapshot file yet          → Skip (Component 27
///                                                   bind writer pending)
/// * snapshot(s) present but all empty              → Skip
///
/// # Fail
///
/// * snapshot present, harness present in snapshot but removed from the
///   current YAML, OR sha values disagree
pub(crate) fn check_ladder_kani_harness_sha(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1615: Kani Harness SHA";
    let contracts = load_active_contracts(project_path);
    if contracts.is_empty() {
        return skip_no_contracts(name);
    }

    let l4_plus: Vec<&WorkContract> = contracts.iter().filter(|c| is_l4_or_higher(c)).collect();
    if l4_plus.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No L4+ ticket present".into(),
            severity: Severity::Info,
        };
    }

    let with_bindings: Vec<&&WorkContract> = l4_plus
        .iter()
        .filter(|c| !c.implements.is_empty())
        .collect();
    if with_bindings.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No L4+ ticket has `implements:` bindings".into(),
            severity: Severity::Info,
        };
    }

    let mut any_snapshot = false;
    let mut checked_tickets = 0usize;
    let mut drift: Vec<String> = Vec::new();

    for c in &with_bindings {
        let snapshot_path = project_path
            .join(".pmat-work")
            .join(&c.work_item_id)
            .join("kani-harness-shas.json");
        if !snapshot_path.exists() {
            continue;
        }
        any_snapshot = true;

        let Ok(snapshot_body) = std::fs::read_to_string(&snapshot_path) else {
            drift.push(format!(
                "  {} (unreadable kani-harness-shas.json)",
                c.work_item_id
            ));
            continue;
        };
        let Some(snapshot) = parse_kani_harness_sha_snapshot(&snapshot_body) else {
            drift.push(format!(
                "  {} (malformed kani-harness-shas.json)",
                c.work_item_id
            ));
            continue;
        };
        if snapshot.is_empty() {
            continue;
        }
        checked_tickets += 1;

        // Compare against each binding's current YAML. A harness name may be
        // scoped to a specific binding; if a binding's YAML doesn't declare
        // it, that's handled by the "removed post-bind" check below only if
        // no other binding's YAML claims it either.
        let mut union_current: std::collections::HashMap<String, (String, String, String)> =
            std::collections::HashMap::new();
        for binding in &c.implements {
            let yaml_path = if binding.file.is_absolute() {
                binding.file.clone()
            } else {
                project_path.join(&binding.file)
            };
            let Ok(yaml) = std::fs::read_to_string(&yaml_path) else {
                continue;
            };
            let Some(current) = yaml_kani_harness_shas(&yaml) else {
                continue;
            };
            for (n, s) in current {
                union_current.insert(n, (s, binding.contract.clone(), binding.equation.clone()));
            }
        }

        for (hname, hsha) in &snapshot {
            match union_current.get(hname) {
                None => drift.push(format!(
                    "  {} harness `{}` absent from any bound YAML (removed post-bind)",
                    c.work_item_id, hname
                )),
                Some((now, contract, equation)) if now != hsha => {
                    let snap_prefix: String = hsha.chars().take(8).collect();
                    let now_prefix: String = now.chars().take(8).collect();
                    drift.push(format!(
                        "  {} [{}/{}] harness `{}` SHA drifted: {}… → {}…",
                        c.work_item_id, contract, equation, hname, snap_prefix, now_prefix
                    ));
                }
                _ => {}
            }
        }
    }

    if !any_snapshot {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: format!(
                "No L4+ ticket has `kani-harness-shas.json` yet ({} eligible)",
                with_bindings.len()
            ),
            severity: Severity::Info,
        };
    }

    if !drift.is_empty() {
        let mut msg = format!("{} Kani harness SHA drift(s):\n", drift.len());
        for line in &drift {
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
            message: "Bind-time snapshot(s) found but all empty".into(),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!(
            "{} ticket(s) — Kani harness SHAs match bind-time snapshot",
            checked_tickets
        ),
        severity: Severity::Info,
    }
}

/// Parse `.pmat-work/<ID>/kani-harness-shas.json`. Accepts either an array
/// of `{name, sha}` objects or a `{<name>: <sha>}` mapping under the
/// top-level `harnesses` key. Returns `None` on schema mismatch; empty on
/// present-but-empty.
fn parse_kani_harness_sha_snapshot(
    contents: &str,
) -> Option<std::collections::HashMap<String, String>> {
    let v: serde_json::Value = serde_json::from_str(contents).ok()?;
    let harnesses = v.get("harnesses")?;
    let mut map = std::collections::HashMap::new();
    if let Some(arr) = harnesses.as_array() {
        for item in arr {
            let Some(n) = item.get("name").and_then(|n| n.as_str()) else {
                continue;
            };
            let Some(s) = item.get("sha").and_then(|s| s.as_str()) else {
                continue;
            };
            if !n.is_empty() && !s.is_empty() {
                map.insert(n.to_string(), s.to_string());
            }
        }
        Some(map)
    } else if let Some(obj) = harnesses.as_object() {
        for (k, val) in obj {
            if let Some(s) = val.as_str() {
                if !k.is_empty() && !s.is_empty() {
                    map.insert(k.clone(), s.to_string());
                }
            }
        }
        Some(map)
    } else {
        None
    }
}

/// Scan a YAML's top-level `kani_harnesses:` block and extract
/// `{name → sha}` pairs where the list item is an object carrying both a
/// `name:` and a `sha:` key. String-form entries (`- verify_foo`) and
/// object-form entries without a `sha:` sibling are silently skipped —
/// they don't participate in drift detection because there is no bind-
/// time hash to compare against.
///
/// Returns `None` when no `kani_harnesses:` key is present at all (caller
/// skips the binding). An empty map means the section exists but declares
/// no shas.
fn yaml_kani_harness_shas(content: &str) -> Option<std::collections::HashMap<String, String>> {
    let mut in_section = false;
    let mut saw_section = false;
    let mut map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut current_name: Option<String> = None;
    let mut current_sha: Option<String> = None;

    fn commit(
        map: &mut std::collections::HashMap<String, String>,
        name: &mut Option<String>,
        sha: &mut Option<String>,
    ) {
        if let (Some(n), Some(s)) = (name.take(), sha.take()) {
            map.insert(n, s);
        } else {
            *name = None;
            *sha = None;
        }
    }

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // Top-level key (column 0, not a list item).
        if !line.starts_with(' ') && !line.starts_with('-') {
            commit(&mut map, &mut current_name, &mut current_sha);
            if trimmed == "kani_harnesses:" {
                in_section = true;
                saw_section = true;
            } else {
                in_section = false;
            }
            continue;
        }
        if !in_section {
            continue;
        }
        if trimmed.starts_with('-') {
            // New list item — flush the previous.
            commit(&mut map, &mut current_name, &mut current_sha);
            let item = trimmed[1..].trim();
            if item.is_empty() {
                continue;
            }
            // `- name: value`
            if let Some(rest) = item.strip_prefix("name:") {
                let val = rest.trim().trim_matches('"').trim_matches('\'').to_string();
                if !val.is_empty() {
                    current_name = Some(val);
                }
                continue;
            }
            // `- sha: value` is unusual but tolerate it — sha without a name
            // can't be committed, so we drop it.
            if item.strip_prefix("sha:").is_some() {
                continue;
            }
            // `- verify_foo` string form: no sha available in this item.
            if !item.contains(':') {
                // Leave current_name/current_sha at None.
                continue;
            }
            continue;
        }
        // Indented continuation of the current list item.
        if let Some(rest) = trimmed.strip_prefix("name:") {
            let val = rest.trim().trim_matches('"').trim_matches('\'').to_string();
            if !val.is_empty() {
                current_name = Some(val);
            }
        } else if let Some(rest) = trimmed.strip_prefix("sha:") {
            let val = rest.trim().trim_matches('"').trim_matches('\'').to_string();
            if !val.is_empty() {
                current_sha = Some(val);
            }
        }
    }
    commit(&mut map, &mut current_name, &mut current_sha);
    if saw_section {
        Some(map)
    } else {
        None
    }
}
