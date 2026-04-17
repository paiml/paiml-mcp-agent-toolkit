// Work Contract Binding enforcement checks (CB-1600..1609) — Component 27
//
// Sub-spec: docs/specifications/components/pmat-work-contract-binding.md
//
// Each work ticket's contract.json may declare `implements: Vec<ContractBinding>`.
// These checks audit that the declared bindings remain internally consistent and
// externally anchored against the provable-contracts YAML files they cite.
//
// This first cut implements the checks that can run against today's infrastructure:
//
//   CB-1601 (L1) — SHA drift against current YAML bytes
//   CB-1607 (L3) — equation identifier exists in referenced YAML
//   CB-1609 (L1) — YAML file is tracked in git
//
// The remaining checks (CB-1600 orphan detection, CB-1602 unbind audit,
// CB-1603/1604 clause inheritance, CB-1605 kani, CB-1606 lean, CB-1608
// cross-binding consistency) surface as Skip with a "deferred: requires X"
// message so config plumbing is already wired for the follow-up work.

use std::path::Path;

use sha2::{Digest, Sha256};

use super::types::*;
use crate::cli::handlers::work_contract::{ContractBinding, WorkContract};

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Walk `.pmat-work/<ID>/contract.json` and load each work contract.
/// Bindings from each contract feed the per-ticket checks below.
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

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let d = h.finalize();
    let mut s = String::with_capacity(d.len() * 2);
    for b in d {
        use std::fmt::Write;
        let _ = write!(&mut s, "{:02x}", b);
    }
    s
}

/// Shape a deferred-but-wired check. Returns Skip with an explanatory reason
/// so the check id still appears in `pmat comply check` output and users can
/// see the enforcement roster.
fn deferred(name: &str, reason: &str) -> ComplianceCheck {
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Skip,
        message: format!("Deferred — {}", reason),
        severity: Severity::Info,
    }
}

fn skip_no_bindings(name: &str) -> ComplianceCheck {
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Skip,
        message: "No `.pmat-work/*/contract.json` with `implements:` entries".into(),
        severity: Severity::Info,
    }
}

fn iter_bindings<'a>(
    contracts: &'a [WorkContract],
) -> impl Iterator<Item = (&'a str, &'a ContractBinding)> + 'a {
    contracts.iter().flat_map(|c| {
        c.implements
            .iter()
            .map(move |b| (c.work_item_id.as_str(), b))
    })
}

// ─── CB-1601: SHA drift ──────────────────────────────────────────────────────

/// CB-1601 (L1): `ContractBinding::sha` must match SHA-256 of the YAML file
/// as it exists on disk today. A mismatch means the contract drifted after
/// the ticket was bound — either the YAML was edited (and the ticket
/// should re-bind), or the ticket is stale.
pub(crate) fn check_binding_sha_drift(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1601: Binding SHA Drift";
    let contracts = load_active_contracts(project_path);
    if iter_bindings(&contracts).next().is_none() {
        return skip_no_bindings(name);
    }

    let mut drifted: Vec<String> = Vec::new();
    let mut missing: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for (ticket, binding) in iter_bindings(&contracts) {
        checked += 1;
        let file = if binding.file.is_absolute() {
            binding.file.clone()
        } else {
            project_path.join(&binding.file)
        };
        let Ok(bytes) = std::fs::read(&file) else {
            missing.push(format!(
                "  {} [{}] -> {} (not found)",
                ticket,
                binding.key(),
                binding.file.display()
            ));
            continue;
        };
        let current = sha256_hex(&bytes);
        if current != binding.sha {
            drifted.push(format!(
                "  {} [{}] recorded {}… current {}…",
                ticket,
                binding.key(),
                &binding.sha[..binding.sha.len().min(8)],
                &current[..current.len().min(8)],
            ));
        }
    }

    if !drifted.is_empty() || !missing.is_empty() {
        let mut msg = format!("SHA drift in {}/{} binding(s)", drifted.len(), checked);
        if !missing.is_empty() {
            msg.push_str(&format!(" + {} missing YAML", missing.len()));
        }
        msg.push_str("\n");
        for line in drifted.iter().chain(missing.iter()) {
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
        message: format!("All {} binding SHA(s) match current YAML bytes", checked),
        severity: Severity::Info,
    }
}

// ─── CB-1607: Equation identifier exists in YAML ─────────────────────────────

/// Scan the YAML line-wise for `equations:` and collect the two-space-indented
/// identifiers below it. Accepts both block-style `rope:` (followed by a
/// nested body) and flow-style `rope: {}` on one line. Returns `None` if
/// no `equations:` section exists (e.g. alternate YAML shape).
fn yaml_equation_names(content: &str) -> Option<Vec<String>> {
    let mut in_equations = false;
    let mut saw_equations = false;
    let mut names = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // Any non-indented, non-blank line ends the equations: section
        if !line.starts_with(' ') {
            in_equations = trimmed == "equations:";
            if in_equations {
                saw_equations = true;
            }
            continue;
        }
        if !in_equations {
            continue;
        }
        // Two-space indent exactly = a key under equations:
        if line.starts_with("  ") && !line.starts_with("    ") && !trimmed.starts_with('-') {
            if let Some(idx) = trimmed.find(':') {
                let name = trimmed[..idx].trim();
                if !name.is_empty() {
                    names.push(name.to_string());
                }
            }
        }
    }
    if saw_equations {
        Some(names)
    } else {
        None
    }
}

/// CB-1607 (L3): the `equation` half of a `<contract>/<equation>` binding
/// must exist as a key under `equations:` in the referenced YAML. Fat-fingered
/// equation names would otherwise bind cleanly (file found, SHA recorded) but
/// point at nothing.
pub(crate) fn check_binding_equation_exists(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1607: Binding Equation Identifier";
    let contracts = load_active_contracts(project_path);
    if iter_bindings(&contracts).next().is_none() {
        return skip_no_bindings(name);
    }

    let mut missing: Vec<String> = Vec::new();
    let mut unreadable: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for (ticket, binding) in iter_bindings(&contracts) {
        checked += 1;
        let file = if binding.file.is_absolute() {
            binding.file.clone()
        } else {
            project_path.join(&binding.file)
        };
        let Ok(content) = std::fs::read_to_string(&file) else {
            unreadable.push(format!("  {} [{}]", ticket, binding.key()));
            continue;
        };
        match yaml_equation_names(&content) {
            Some(names) if names.contains(&binding.equation) => {}
            Some(_) => missing.push(format!(
                "  {} [{}] equation '{}' not found in {}",
                ticket,
                binding.key(),
                binding.equation,
                binding.file.display()
            )),
            None => {
                // No equations: section — treat as skip-equivalent
            }
        }
    }

    if !missing.is_empty() || !unreadable.is_empty() {
        let mut msg = format!("{} equation(s) not found in referenced YAML", missing.len());
        if !unreadable.is_empty() {
            msg.push_str(&format!(" + {} unreadable", unreadable.len()));
        }
        msg.push('\n');
        for line in missing.iter().chain(unreadable.iter()) {
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
        message: format!("All {} equation(s) resolve in their YAML", checked),
        severity: Severity::Info,
    }
}

// ─── CB-1609: YAML file is tracked in git ────────────────────────────────────

/// CB-1609 (L1): `implements[].file` must be a git-tracked path. Transient
/// scratch YAML (e.g. `/tmp/experimental.yaml`) is not a durable anchor for
/// a bound ticket. Uses `git ls-files --error-unmatch`.
pub(crate) fn check_binding_file_tracked(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1609: Binding YAML Git-Tracked";
    let contracts = load_active_contracts(project_path);
    if iter_bindings(&contracts).next().is_none() {
        return skip_no_bindings(name);
    }

    let mut untracked: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for (ticket, binding) in iter_bindings(&contracts) {
        checked += 1;
        let file = if binding.file.is_absolute() {
            binding.file.clone()
        } else {
            project_path.join(&binding.file)
        };
        // Resolve relative to project_path for `git ls-files`
        let rel = file.strip_prefix(project_path).unwrap_or(&binding.file);
        let status = std::process::Command::new("git")
            .arg("-C")
            .arg(project_path)
            .arg("ls-files")
            .arg("--error-unmatch")
            .arg("--")
            .arg(rel)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        match status {
            Ok(s) if s.success() => {}
            _ => untracked.push(format!(
                "  {} [{}] {}",
                ticket,
                binding.key(),
                rel.display()
            )),
        }
    }

    if !untracked.is_empty() {
        let mut msg = format!(
            "{} binding(s) reference untracked YAML files\n",
            untracked.len()
        );
        for line in &untracked {
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
        message: format!("All {} binding YAML(s) tracked in git", checked),
        severity: Severity::Info,
    }
}

// ─── Deferred checks (scheduled follow-up work) ──────────────────────────────

/// CB-1600 (L1): Tickets modifying contracted files MUST declare `implements`.
/// Requires `.pmat/binding-index.json` (CB-1208) to intersect staged files
/// with binding entries. Skipped until that index ships.
pub(crate) fn check_binding_scope_orphan(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1600: Binding Scope Orphan",
        "requires .pmat/binding-index.json (CB-1208) for file↔binding intersection",
    )
}

/// CB-1602 (L1): `pmat work unbind` without a DEBT follow-up ticket reference
/// indicates silent contract abandonment. Activated when the unbind command
/// and its ledger land.
pub(crate) fn check_binding_unbind_audit(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1602: Unbind Audit",
        "requires `pmat work unbind` + .pmat-work/ledger/ implementation",
    )
}

/// CB-1603 (L3): verifies `inherited-clauses.json` matches YAML preconditions.
/// Activated when the inheritance pipeline in Component 27 (the full version)
/// writes that derived file.
pub(crate) fn check_binding_inherited_clauses(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1603: Inherited Clause Integrity",
        "requires precondition inheritance pipeline emitting inherited-clauses.json",
    )
}

/// CB-1604 (L3): a ticket cannot override an inherited postcondition with a
/// weaker threshold. Requires clause-threshold comparison semantics which
/// arrive with the full DbC triad inheritance.
pub(crate) fn check_binding_postcondition_weakening(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1604: Postcondition Weakening",
        "requires inherited-postcondition threshold comparator",
    )
}

/// CB-1605 (L4): if the bound YAML has `kani_harnesses[]`, completion must
/// have executed them. Requires the Kani runner integration (Component 29 L4).
pub(crate) fn check_binding_kani_harnesses(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1605: Kani Harness Execution",
        "requires Kani runner integration (Component 29 L4)",
    )
}

/// CB-1606 (L5): if the bound YAML's `lean_theorem.status != \"proved\"`,
/// the ticket must link a `BLOCK-ON-PROOF` follow-up. Requires Lean proof-
/// status tracking (Component 29 L5).
pub(crate) fn check_binding_lean_theorem(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1606: Lean Theorem Linkage",
        "requires Lean proof-status tracking (Component 29 L5)",
    )
}

/// CB-1608 (L1): cross-binding consistency — a multi-bind ticket must satisfy
/// all bindings at completion, not just a subset. Activated once falsification
/// unification (Component 29) propagates per-binding pass/fail.
pub(crate) fn check_binding_cross_consistency(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1608: Cross-Binding Consistency",
        "requires per-binding falsification status (Component 29)",
    )
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn write_contract(project: &Path, ticket: &str, yaml_file: &Path, equation: &str, sha: &str) {
        let mut c = WorkContract::new(ticket.to_string(), "deadbeef".to_string());
        c.implements.push(ContractBinding {
            contract: "k".to_string(),
            equation: equation.to_string(),
            file: yaml_file.to_path_buf(),
            sha: sha.to_string(),
            bound_at: chrono::Utc::now(),
        });
        c.save(project).unwrap();
    }

    fn write_yaml(project: &Path, name: &str, body: &str) -> PathBuf {
        let dir = project.join("contracts");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join(format!("{}.yaml", name));
        std::fs::write(&p, body).unwrap();
        p
    }

    #[test]
    fn sha_drift_passes_when_aligned() {
        let tmp = tempdir().unwrap();
        let yaml = write_yaml(tmp.path(), "k", "equations:\n  rope: {}\n");
        let bytes = std::fs::read(&yaml).unwrap();
        let sha = sha256_hex(&bytes);
        let rel = PathBuf::from("contracts/k.yaml");
        write_contract(tmp.path(), "T-1", &rel, "rope", &sha);
        let r = check_binding_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn sha_drift_fails_on_edit() {
        let tmp = tempdir().unwrap();
        let yaml = write_yaml(tmp.path(), "k", "equations:\n  rope: {}\n");
        let bytes = std::fs::read(&yaml).unwrap();
        let sha = sha256_hex(&bytes);
        let rel = PathBuf::from("contracts/k.yaml");
        write_contract(tmp.path(), "T-1", &rel, "rope", &sha);
        // Mutate the YAML
        std::fs::write(&yaml, "equations:\n  rope: {a: 1}\n").unwrap();
        let r = check_binding_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("SHA drift"));
    }

    #[test]
    fn sha_drift_skips_without_bindings() {
        let tmp = tempdir().unwrap();
        let r = check_binding_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }

    #[test]
    fn equation_exists_passes_on_known_name() {
        let tmp = tempdir().unwrap();
        write_yaml(tmp.path(), "k", "equations:\n  rope: {}\n  softmax: {}\n");
        let rel = PathBuf::from("contracts/k.yaml");
        write_contract(tmp.path(), "T-1", &rel, "rope", "deadbeef");
        let r = check_binding_equation_exists(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn equation_exists_fails_on_typo() {
        let tmp = tempdir().unwrap();
        write_yaml(tmp.path(), "k", "equations:\n  rope: {}\n");
        let rel = PathBuf::from("contracts/k.yaml");
        write_contract(tmp.path(), "T-1", &rel, "ropee", "deadbeef");
        let r = check_binding_equation_exists(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("ropee"));
    }

    #[test]
    fn yaml_equation_names_parses_minimal() {
        let s = "version: 1\nequations:\n  rope:\n    preconditions: []\n  softmax: {}\n";
        let names = yaml_equation_names(s).unwrap();
        assert_eq!(names, vec!["rope".to_string(), "softmax".to_string()]);
    }

    #[test]
    fn yaml_equation_names_ignores_comments() {
        let s = "equations:\n  # top-level comment line\n  rope: {}\n";
        let names = yaml_equation_names(s).unwrap();
        assert_eq!(names, vec!["rope".to_string()]);
    }

    #[test]
    fn deferred_checks_return_skip() {
        let tmp = tempdir().unwrap();
        for r in [
            check_binding_scope_orphan(tmp.path()),
            check_binding_unbind_audit(tmp.path()),
            check_binding_inherited_clauses(tmp.path()),
            check_binding_postcondition_weakening(tmp.path()),
            check_binding_kani_harnesses(tmp.path()),
            check_binding_lean_theorem(tmp.path()),
            check_binding_cross_consistency(tmp.path()),
        ] {
            assert_eq!(r.status, CheckStatus::Skip);
            assert!(r.message.starts_with("Deferred"));
        }
    }

    #[test]
    fn file_tracked_skips_without_bindings() {
        let tmp = tempdir().unwrap();
        let r = check_binding_file_tracked(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }
}
