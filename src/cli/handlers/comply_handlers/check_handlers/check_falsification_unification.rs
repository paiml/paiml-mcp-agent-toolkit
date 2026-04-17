// Work Falsification Unification checks (CB-1620..1629) — Component 29
//
// Sub-spec: docs/specifications/components/pmat-work-falsification-unification.md
//
// Bridges the existing bespoke `FalsificationMethod` enum with the
// per-equation `falsification_tests[]` arrays defined in provable-contracts
// YAML. A ticket bound to a YAML equation (Component 27) automatically
// inherits `FalsificationMethod::ProvableContract{}` entries — one per YAML
// test. These checks audit that the inherited roster stays consistent with
// the YAML source and the ticket's own manual claims.
//
// Today's cut implements the checks that can run against today's infrastructure:
//
//   CB-1620 (L1) — every binding has matching `ProvableContract{}` entries
//                  per YAML `falsification_tests[]` id (WARN during migration)
//   CB-1623 (L3) — no duplicate `(yaml_path, test_id)` across a ticket's roster
//   CB-1626 (L1) — referenced `test_id` exists in the YAML at scan time
//
// The remaining checks (CB-1621 expected snapshot drift, CB-1622 roster
// execution coverage, CB-1624 deletion audit, CB-1625 fatal inherited
// failures, CB-1627 post-bind YAML drift, CB-1628 per-run log emission,
// CB-1629 L4 timeout gate) surface as Skip with a "Deferred — requires X"
// message so config plumbing is wired for the follow-up work.

use std::path::{Path, PathBuf};

use super::types::*;
use crate::cli::handlers::work_contract::{FalsificationMethod, WorkContract};

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn load_active_contracts(project_path: &Path) -> Vec<WorkContract> {
    let dir = project_path.join(".pmat-work");
    if !dir.exists() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return out;
    };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let Some(ticket_id) = entry.file_name().to_str().map(|s| s.to_string()) else {
            continue;
        };
        if ticket_id.starts_with('.') || ticket_id == "ledger" {
            continue;
        }
        if let Ok(c) = WorkContract::load(project_path, &ticket_id) {
            out.push(c);
        }
    }
    out
}

fn deferred(name: &str, reason: &str) -> ComplianceCheck {
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Skip,
        message: format!("Deferred — {}", reason),
        severity: Severity::Info,
    }
}

/// Collect every `FalsificationMethod::ProvableContract` entry in a contract's
/// claims. Returns `(yaml_path, equation, test_id)` triples.
fn provable_contract_entries(c: &WorkContract) -> Vec<(PathBuf, String, String)> {
    c.claims
        .iter()
        .filter_map(|claim| match &claim.falsification_method {
            FalsificationMethod::ProvableContract {
                yaml_path,
                equation,
                test_id,
                ..
            } => Some((yaml_path.clone(), equation.clone(), test_id.clone())),
            _ => None,
        })
        .collect()
}

/// Extract the `id:` values of list items under a top-level `falsification_tests:`
/// section of a provable-contracts YAML file. Line-scan only — no full YAML
/// parser. Exits the section on the first unindented line after entry.
/// Matches common shapes:
///   falsification_tests:
///     - id: rope_periodicity_test
///     - id: "rope_linearity_test"
pub(crate) fn yaml_falsification_test_ids(yaml: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut in_section = false;
    for line in yaml.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !line.starts_with(' ') {
            in_section = trimmed.starts_with("falsification_tests:");
            continue;
        }
        if !in_section {
            continue;
        }
        // "- id: foo" or continuation "id: foo"
        if let Some(rest) = trimmed
            .strip_prefix("- id:")
            .or_else(|| trimmed.strip_prefix("id:"))
        {
            let id = rest
                .trim()
                .trim_matches(|c: char| c == '"' || c == '\'')
                .trim();
            if !id.is_empty() {
                ids.push(id.to_string());
            }
        }
    }
    ids
}

fn has_any_provable_contract_entries(contracts: &[WorkContract]) -> bool {
    contracts
        .iter()
        .any(|c| !provable_contract_entries(c).is_empty())
}

// ─── CB-1620: Inherited roster coverage (WARN during migration) ──────────────

/// CB-1620 (L1): For every `ContractBinding`, the ticket's claim roster must
/// contain a matching `ProvableContract{}` entry for each test in the bound
/// YAML's `falsification_tests[]`. Missing entries surface as **warnings**
/// during the 30-day migration window (§Migration) so users have time to run
/// `pmat work migrate --seed-inherited-falsification` before fail-mode.
pub(crate) fn check_inherited_roster_coverage(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1620: Inherited Roster Coverage";
    let contracts = load_active_contracts(project_path);
    let any_binding = contracts.iter().any(|c| !c.implements.is_empty());
    if !any_binding {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ticket has `implements:` bindings to enforce".into(),
            severity: Severity::Info,
        };
    }
    let mut gaps: Vec<String> = Vec::new();
    for c in &contracts {
        for binding in &c.implements {
            let file = if binding.file.is_absolute() {
                binding.file.clone()
            } else {
                project_path.join(&binding.file)
            };
            let Ok(yaml) = std::fs::read_to_string(&file) else {
                continue;
            };
            let expected_ids = yaml_falsification_test_ids(&yaml);
            if expected_ids.is_empty() {
                continue;
            }
            let entries = provable_contract_entries(c);
            for id in &expected_ids {
                let present = entries.iter().any(|(path, eq, test_id)| {
                    path == &binding.file && eq == &binding.equation && test_id == id
                });
                if !present {
                    gaps.push(format!(
                        "{} → {}/{}#{}",
                        c.work_item_id, binding.contract, binding.equation, id
                    ));
                }
            }
        }
    }
    if gaps.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: "All bound tickets have inherited ProvableContract entries".into(),
            severity: Severity::Info,
        }
    } else {
        let preview: Vec<String> = gaps.iter().take(5).cloned().collect();
        let more = gaps.len().saturating_sub(5);
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} inherited test(s) missing from roster (e.g. {}{}). Run `pmat work migrate --seed-inherited-falsification`.",
                gaps.len(),
                preview.join(", "),
                if more > 0 { format!(", +{} more", more) } else { String::new() }
            ),
            severity: Severity::Warning,
        }
    }
}

// ─── CB-1623: No duplicate (yaml_path, test_id) across a ticket's roster ─────

/// CB-1623 (L3): `FalsificationMethod::ProvableContract` entries with the
/// same `(yaml_path, test_id)` pair count as double-coverage and inflate
/// passing claim counts. The check walks every ticket's claim roster and
/// fails on the first duplicate found.
pub(crate) fn check_no_duplicate_provable_entries(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1623: No Duplicate ProvableContract Entries";
    let contracts = load_active_contracts(project_path);
    if !has_any_provable_contract_entries(&contracts) {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ProvableContract entries in any ticket yet".into(),
            severity: Severity::Info,
        };
    }
    let mut dupes: Vec<String> = Vec::new();
    for c in &contracts {
        let entries = provable_contract_entries(c);
        let mut seen: Vec<(PathBuf, String)> = Vec::new();
        for (path, _eq, test_id) in entries {
            let key = (path.clone(), test_id.clone());
            if seen.contains(&key) {
                dupes.push(format!(
                    "{} → {}#{}",
                    c.work_item_id,
                    path.display(),
                    test_id
                ));
            } else {
                seen.push(key);
            }
        }
    }
    if dupes.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: "No duplicate (yaml_path, test_id) pairs in any ticket roster".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!("Duplicate ProvableContract entries: {}", dupes.join("; ")),
            severity: Severity::Error,
        }
    }
}

// ─── CB-1626: Referenced test_id exists in YAML ──────────────────────────────

/// CB-1626 (L1): Each `ProvableContract.test_id` must exist in the bound
/// YAML's `falsification_tests[]` at scan time. A stale reference means the
/// YAML removed the test post-bind and the ticket's claim is unanchored.
pub(crate) fn check_test_id_exists_in_yaml(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1626: Referenced test_id Exists in YAML";
    let contracts = load_active_contracts(project_path);
    if !has_any_provable_contract_entries(&contracts) {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ProvableContract entries in any ticket yet".into(),
            severity: Severity::Info,
        };
    }
    let mut stale: Vec<String> = Vec::new();
    for c in &contracts {
        for (yaml_path, _eq, test_id) in provable_contract_entries(c) {
            let file = if yaml_path.is_absolute() {
                yaml_path.clone()
            } else {
                project_path.join(&yaml_path)
            };
            let Ok(yaml) = std::fs::read_to_string(&file) else {
                stale.push(format!(
                    "{} → {} (YAML missing)",
                    c.work_item_id,
                    yaml_path.display()
                ));
                continue;
            };
            let ids = yaml_falsification_test_ids(&yaml);
            if !ids.iter().any(|i| i == &test_id) {
                stale.push(format!(
                    "{} → {}#{}",
                    c.work_item_id,
                    yaml_path.display(),
                    test_id
                ));
            }
        }
    }
    if stale.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: "All referenced test_ids exist in their YAML sources".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!("Stale test_id references: {}", stale.join("; ")),
            severity: Severity::Error,
        }
    }
}

// ─── CB-1621..1629 deferred stubs ────────────────────────────────────────────

pub(crate) fn check_expected_snapshot_drift(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1621: Expected Snapshot Drift",
        "requires per-test YAML parser emitting canonical JSON for comparison",
    )
}

pub(crate) fn check_roster_execution_coverage(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1622: Roster Execution Coverage",
        "requires `.pmat-work/<ID>/falsification.log` emitted by unified runner",
    )
}

pub(crate) fn check_no_manual_deletion(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1624: No Manual Deletion of Inherited Entries",
        "requires audit ledger integration for roster mutation events",
    )
}

pub(crate) fn check_inherited_failure_fatal(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1625: Inherited Failure Fatal",
        "requires runtime falsification execution via `pmat work falsify`",
    )
}

pub(crate) fn check_post_bind_yaml_drift(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1627: Post-bind YAML Drift",
        "requires bind-time snapshot of YAML `falsification_tests[]` for diff",
    )
}

pub(crate) fn check_per_run_log_line(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1628: Per-run Log Line Emitted",
        "requires unified falsification runner emitting JSONL receipts",
    )
}

pub(crate) fn check_l4_timeout_gate(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1629: L4 Timeout Gate",
        "requires runtime timeout tracking in ProvableContract dispatch",
    )
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn yaml_ids_extracts_list_form() {
        let y = "falsification_tests:\n  - id: a\n  - id: \"b\"\n  - id: 'c'\n";
        assert_eq!(yaml_falsification_test_ids(y), vec!["a", "b", "c"]);
    }

    #[test]
    fn yaml_ids_ignores_other_sections() {
        let y = "equations:\n  - id: not_here\nfalsification_tests:\n  - id: actual\nkani_harnesses:\n  - id: neither\n";
        assert_eq!(yaml_falsification_test_ids(y), vec!["actual"]);
    }

    #[test]
    fn yaml_ids_empty_when_section_absent() {
        assert!(yaml_falsification_test_ids("equations:\n  rope: {}\n").is_empty());
    }

    #[test]
    fn yaml_ids_empty_flow_list() {
        let y = "falsification_tests: []\n";
        assert!(yaml_falsification_test_ids(y).is_empty());
    }

    #[test]
    fn deferred_checks_return_skip_with_reason() {
        let path = Path::new(".");
        for (name, check) in [
            ("CB-1621", check_expected_snapshot_drift(path)),
            ("CB-1622", check_roster_execution_coverage(path)),
            ("CB-1624", check_no_manual_deletion(path)),
            ("CB-1625", check_inherited_failure_fatal(path)),
            ("CB-1627", check_post_bind_yaml_drift(path)),
            ("CB-1628", check_per_run_log_line(path)),
            ("CB-1629", check_l4_timeout_gate(path)),
        ] {
            assert_eq!(check.status, CheckStatus::Skip, "{}", name);
            assert!(
                check.message.starts_with("Deferred — "),
                "{}: {}",
                name,
                check.message
            );
        }
    }

    #[test]
    fn roster_coverage_skips_without_bindings() {
        let tmp = tempdir().unwrap();
        let check = check_inherited_roster_coverage(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn duplicate_entries_skips_without_provable_contract() {
        let tmp = tempdir().unwrap();
        let check = check_no_duplicate_provable_entries(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_id_exists_skips_without_provable_contract() {
        let tmp = tempdir().unwrap();
        let check = check_test_id_exists_in_yaml(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn provable_contract_variant_round_trips_through_serde() {
        let m = FalsificationMethod::ProvableContract {
            yaml_path: PathBuf::from("contracts/rope-kernel-v1.yaml"),
            equation: "rope".into(),
            test_id: "rope_periodicity_test".into(),
            expected: "1.0".into(),
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: FalsificationMethod = serde_json::from_str(&json).unwrap();
        match back {
            FalsificationMethod::ProvableContract {
                yaml_path,
                equation,
                test_id,
                expected,
            } => {
                assert_eq!(yaml_path, PathBuf::from("contracts/rope-kernel-v1.yaml"));
                assert_eq!(equation, "rope");
                assert_eq!(test_id, "rope_periodicity_test");
                assert_eq!(expected, "1.0");
            }
            other => panic!("round-trip landed on wrong variant: {:?}", other),
        }
    }
}
