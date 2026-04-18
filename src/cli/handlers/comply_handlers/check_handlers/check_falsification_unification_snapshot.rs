// Work Falsification Unification — CB-1621 expected-snapshot drift.
//
// Included from `check_falsification_unification.rs` — do NOT add `use`
// imports or `#!` attributes here.
//
// Contains:
//   CB-1621 — check_expected_snapshot_drift (L1): ProvableContract{expected}
//             snapshots must match current YAML scalar `expected:`.
//   Helpers — provable_contract_entries_with_expected,
//             yaml_expected_by_test_id, yaml_scalar_to_canonical_json.

// ─── CB-1621: Expected-snapshot drift ───────────────────────────────────────

/// CB-1621 (L1): `ProvableContract{expected}` snapshots taken at bind time
/// must match the current YAML's `expected:` field for that test. A
/// divergence is *silent* test-expectation drift — the test_id is stable,
/// the method executes the same code, but the asserted value changed.
///
/// # Schema
///
/// Snapshot: `ProvableContract.expected` — canonical JSON emitted by the
/// bind step. Today's writer may leave this empty; empty snapshots skip
/// per-entry (nothing to diff against).
///
/// Current value: line-scanned from the bound YAML's `falsification_tests:`
/// block. Only scalar shapes (bool, number, quoted/bare string, null) are
/// compared — inline mappings (`expected: {a: 1}`) and block scalars
/// (`expected: |\n  ...`) need a structural YAML parser and are silently
/// skipped until the pv-yaml-loader (Component 29) is in place.
///
/// # Skip semantics (tiered)
///
/// * no `.pmat-work/*/contract.json` tickets        → Skip
/// * no ticket has a ProvableContract entry with a
///   non-empty `expected` snapshot                  → Skip
/// * YAMLs exist but none declare scalar `expected:`
///   for any bound test_id                          → Skip
///
/// # Fail
///
/// * snapshot and current scalar decode to different JSON values
pub(crate) fn check_expected_snapshot_drift(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1621: Expected Snapshot Drift";
    let contracts = load_active_contracts(project_path);
    if contracts.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/*/contract.json` tickets present".into(),
            severity: Severity::Info,
        };
    }

    let mut any_snapshot = false;
    let mut compared = 0usize;
    let mut drift: Vec<String> = Vec::new();

    for c in &contracts {
        for (yaml_rel, equation, test_id, snapshot) in provable_contract_entries_with_expected(c) {
            if snapshot.trim().is_empty() {
                continue;
            }
            any_snapshot = true;
            let yaml_abs = if yaml_rel.is_absolute() {
                yaml_rel.clone()
            } else {
                project_path.join(&yaml_rel)
            };
            let Ok(yaml_body) = std::fs::read_to_string(&yaml_abs) else {
                continue;
            };
            let map = yaml_expected_by_test_id(&yaml_body);
            let Some(current_json) = map.get(&test_id) else {
                continue;
            };
            compared += 1;

            // Structural compare: both sides canonicalize to JSON; an
            // unparseable snapshot or current value falls back to string
            // equality so we still catch trivial differences.
            let match_ok = match (
                serde_json::from_str::<serde_json::Value>(&snapshot),
                serde_json::from_str::<serde_json::Value>(current_json),
            ) {
                (Ok(a), Ok(b)) => a == b,
                _ => current_json == &snapshot,
            };
            if !match_ok {
                drift.push(format!(
                    "  {} [{}/{}#{}] expected drifted: {} → {}",
                    c.work_item_id,
                    yaml_rel.display(),
                    equation,
                    test_id,
                    snapshot,
                    current_json
                ));
            }
        }
    }

    if !any_snapshot {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ProvableContract entry has a non-empty `expected` snapshot".into(),
            severity: Severity::Info,
        };
    }
    if compared == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No bound YAML declares scalar `expected:` for any seeded test_id".into(),
            severity: Severity::Info,
        };
    }

    if !drift.is_empty() {
        let mut msg = format!("{} expected snapshot drift(s):\n", drift.len());
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

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!(
            "{} ProvableContract snapshot(s) match current YAML `expected:`",
            compared
        ),
        severity: Severity::Info,
    }
}

/// Collect ProvableContract roster entries with their bind-time `expected`
/// snapshot. Sibling of `provable_contract_entries` — that helper drops
/// the snapshot for checks that only care about `(yaml, eq, test_id)`.
fn provable_contract_entries_with_expected(
    c: &WorkContract,
) -> Vec<(PathBuf, String, String, String)> {
    c.claims
        .iter()
        .filter_map(|claim| match &claim.falsification_method {
            FalsificationMethod::ProvableContract {
                yaml_path,
                equation,
                test_id,
                expected,
            } => Some((
                yaml_path.clone(),
                equation.clone(),
                test_id.clone(),
                expected.clone(),
            )),
            _ => None,
        })
        .collect()
}

/// Parse a YAML's top-level `falsification_tests:` block and return a
/// `{test_id → canonical-JSON expected}` map. Only scalar `expected:`
/// values are converted; inline mappings/sequences and block scalars are
/// silently skipped — they'd need a structural parser (Component 29
/// pv-yaml-loader) to round-trip safely.
fn yaml_expected_by_test_id(yaml: &str) -> std::collections::HashMap<String, String> {
    let mut out: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut in_section = false;
    let mut current_id: Option<String> = None;
    let mut current_exp: Option<String> = None;

    fn flush(
        out: &mut std::collections::HashMap<String, String>,
        id: &mut Option<String>,
        exp: &mut Option<String>,
    ) {
        if let (Some(i), Some(e)) = (id.take(), exp.take()) {
            out.insert(i, e);
        } else {
            *id = None;
            *exp = None;
        }
    }

    for line in yaml.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !line.starts_with(' ') && !line.starts_with('-') {
            flush(&mut out, &mut current_id, &mut current_exp);
            in_section = trimmed.starts_with("falsification_tests:");
            continue;
        }
        if !in_section {
            continue;
        }
        if let Some(after_dash) = trimmed.strip_prefix('-') {
            flush(&mut out, &mut current_id, &mut current_exp);
            let item = after_dash.trim();
            if let Some(rest) = item.strip_prefix("id:") {
                let id = rest
                    .trim()
                    .trim_matches(|c: char| c == '"' || c == '\'')
                    .to_string();
                if !id.is_empty() {
                    current_id = Some(id);
                }
            } else if let Some(rest) = item.strip_prefix("expected:") {
                if let Some(j) = yaml_scalar_to_canonical_json(rest) {
                    current_exp = Some(j);
                }
            }
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("id:") {
            let id = rest
                .trim()
                .trim_matches(|c: char| c == '"' || c == '\'')
                .to_string();
            if !id.is_empty() {
                current_id = Some(id);
            }
        } else if let Some(rest) = trimmed.strip_prefix("expected:") {
            if let Some(j) = yaml_scalar_to_canonical_json(rest) {
                current_exp = Some(j);
            }
        }
    }
    flush(&mut out, &mut current_id, &mut current_exp);
    out
}

/// Convert a YAML scalar to canonical JSON. Handles booleans, nulls,
/// integers/floats, quoted strings, and bare strings. Returns `None` for
/// complex shapes (inline mappings, sequences, block scalars) so callers
/// skip the entry instead of comparing wrong shapes.
fn yaml_scalar_to_canonical_json(raw: &str) -> Option<String> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }
    // Inline mapping / sequence / block scalar — bail.
    if s.starts_with('{') || s.starts_with('[') || s.starts_with('|') || s.starts_with('>') {
        return None;
    }
    match s {
        "true" | "false" => return Some(s.to_string()),
        "null" | "~" => return Some("null".to_string()),
        _ => {}
    }
    if let Ok(n) = s.parse::<i64>() {
        return Some(n.to_string());
    }
    if let Ok(n) = s.parse::<f64>() {
        if let Ok(j) = serde_json::to_string(&n) {
            return Some(j);
        }
    }
    if s.len() >= 2 {
        if s.starts_with('"') && s.ends_with('"') {
            let inner = &s[1..s.len() - 1];
            return serde_json::to_string(inner).ok();
        }
        if s.starts_with('\'') && s.ends_with('\'') {
            let inner = &s[1..s.len() - 1];
            return serde_json::to_string(inner).ok();
        }
    }
    serde_json::to_string(s).ok()
}
