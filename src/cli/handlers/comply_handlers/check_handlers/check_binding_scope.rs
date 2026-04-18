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
//   CB-1600 (L1) — orphan detection: staged files w/ bindings → active ticket
//                  must declare `implements:` covering them
//   CB-1601 (L1) — SHA drift against current YAML bytes
//   CB-1602 (L1) — unbind ledger: every `.pmat-work/ledger/unbinds.json`
//                  entry must carry a DEBT ticket reference (skip-if-absent)
//   CB-1603 (L3) — inherited clause integrity: contract.require contains each
//                  bound equation's YAML-declared preconditions
//   CB-1605 (L4) — per-binding kani_harnesses[] declared in YAML each appear
//                  with success: true in `.pmat-work/<ID>/kani-report.json`.
//                  Skip-if-absent (no YAML harness decls OR no reports yet).
//   CB-1607 (L3) — equation identifier exists in referenced YAML
//   CB-1609 (L1) — YAML file is tracked in git
//
// The remaining checks (CB-1604 postcondition weakening, CB-1606 lean,
// CB-1608 cross-binding consistency) surface as Skip with a
// "deferred: requires X" message so config plumbing is already wired for
// the follow-up work.

use std::path::Path;

use sha2::{Digest, Sha256};

use super::types::*;
use crate::cli::handlers::work_contract::{ContractBinding, WorkContract};
use crate::services::contract_index::ContractIndex;

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

/// Collect paths staged for commit (via `git diff --cached --name-only`),
/// returning them as forward-slash project-relative strings suitable for
/// lookup in the binding index.
fn staged_files(project_path: &Path) -> Vec<String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(project_path)
        .args(["diff", "--cached", "--name-only"])
        .stderr(std::process::Stdio::null())
        .output();
    let Ok(output) = out else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// CB-1600 (L1): Tickets modifying contracted files MUST declare `implements`.
/// Reads `.pmat/binding-index.json` (CB-1208), intersects with git-staged files,
/// and fails if any active `.pmat-work/<ID>/contract.json` touches a bound file
/// without declaring `implements[].file` for it. Per spec §Migration Path,
/// tickets with no bindings remain valid as long as they aren't staging
/// contracted files.
pub(crate) fn check_binding_scope_orphan(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1600: Binding Scope Orphan";
    let Some(index) = ContractIndex::load(project_path) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No .pmat/binding-index.json — run `pmat comply refresh-bindings`".into(),
            severity: Severity::Info,
        };
    };
    let staged = staged_files(project_path);
    if staged.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No staged files to check for binding orphans".into(),
            severity: Severity::Info,
        };
    }

    let staged_bound: Vec<&String> = staged.iter().filter(|f| index.has_bindings(f)).collect();
    if staged_bound.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} staged file(s); none intersect {} bound file(s)",
                staged.len(),
                index.total_files
            ),
            severity: Severity::Info,
        };
    }

    // Build set of `implements[].file` (project-relative, forward-slash) across
    // all active work contracts — any active ticket declaring `implements:` for
    // a bound file is sufficient coverage per the spec's pre-commit semantics.
    let contracts = load_active_contracts(project_path);
    let declared: std::collections::HashSet<String> = contracts
        .iter()
        .flat_map(|c| &c.implements)
        .map(|b| b.file.to_string_lossy().replace('\\', "/"))
        .collect();

    let orphans: Vec<&String> = staged_bound
        .iter()
        .filter(|f| !declared.contains(f.as_str()))
        .copied()
        .collect();

    if orphans.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} staged bound file(s) covered by active `implements:` entries",
                staged_bound.len()
            ),
            severity: Severity::Info,
        };
    }

    let mut msg = format!(
        "{} staged bound file(s) not declared in any active `implements:`\n",
        orphans.len()
    );
    for f in &orphans {
        let bindings = index.get_bindings(f);
        msg.push_str(&format!("  {} → bindings: {}\n", f, bindings.join(", ")));
    }
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Fail,
        message: msg,
        severity: Severity::Error,
    }
}

/// CB-1602 (L1): `pmat work unbind` without a DEBT follow-up ticket reference
/// indicates silent contract abandonment. Every entry in the unbind ledger
/// must cite the debt ticket that'll restore the binding.
///
/// Ledger schema (minimum): JSON array at `.pmat-work/ledger/unbinds.json`
/// where each entry has:
///   • `ticket`       — the work-item-id that unbound
///   • `contract`     — YAML path (or contract name) that was unbound from
///   • `debt_ticket`  — follow-up ticket id (e.g., "DEBT-123"), non-empty
///
/// Skip-if-absent: the ledger file is optional — until `pmat work unbind`
/// lands, it doesn't exist, and this check is Skip.
pub(crate) fn check_binding_unbind_audit(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1602: Unbind Audit";
    let ledger = project_path
        .join(".pmat-work")
        .join("ledger")
        .join("unbinds.json");
    if !ledger.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No unbind ledger at .pmat-work/ledger/unbinds.json".into(),
            severity: Severity::Info,
        };
    }
    let Ok(content) = std::fs::read_to_string(&ledger) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Warn,
            message: format!("Unreadable unbind ledger: {}", ledger.display()),
            severity: Severity::Warning,
        };
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: "Unbind ledger is not valid JSON".into(),
            severity: Severity::Error,
        };
    };

    let entries: &[serde_json::Value] = match &value {
        serde_json::Value::Array(a) => a.as_slice(),
        _ => {
            return ComplianceCheck {
                name: name.into(),
                status: CheckStatus::Fail,
                message: "Unbind ledger must be a JSON array of entries".into(),
                severity: Severity::Error,
            };
        }
    };

    if entries.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: "Unbind ledger present but empty — no audits pending".into(),
            severity: Severity::Info,
        };
    }

    let mut bad: Vec<String> = Vec::new();
    for (idx, entry) in entries.iter().enumerate() {
        let ticket = entry
            .get("ticket")
            .and_then(|t| t.as_str())
            .unwrap_or("<no-ticket>");
        let debt = entry
            .get("debt_ticket")
            .and_then(|t| t.as_str())
            .unwrap_or("");
        if debt.trim().is_empty() {
            bad.push(format!(
                "  entry {}: ticket={} missing/empty debt_ticket",
                idx, ticket
            ));
        }
    }

    if !bad.is_empty() {
        let mut msg = format!("{} unbind(s) lack DEBT ticket reference:\n", bad.len());
        for line in &bad {
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
            "{} unbind(s) all carry DEBT ticket reference",
            entries.len()
        ),
        severity: Severity::Info,
    }
}

/// Extract `preconditions:` entries for a specific equation from a source
/// YAML's `equations:` block. Returns `None` if the equation or its
/// `preconditions:` key is absent (nothing to inherit); `Some(vec![])` if the
/// list exists but is empty.
///
/// Recognized shape:
/// ```yaml
/// equations:
///   rope:
///     preconditions:
///     - "foo"
///     - "bar"
/// ```
fn yaml_equation_preconditions(content: &str, equation: &str) -> Option<Vec<String>> {
    let mut in_equations = false;
    let mut in_target = false;
    let mut in_preconditions = false;
    let mut preconditions: Option<Vec<String>> = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // Indent level 0: top-level key
        if !line.starts_with(' ') {
            in_equations = trimmed == "equations:";
            in_target = false;
            in_preconditions = false;
            continue;
        }
        if !in_equations {
            continue;
        }
        let indent = line.len() - line.trim_start_matches(' ').len();
        // Indent 2 = equation key under `equations:`
        if indent == 2 && !trimmed.starts_with('-') {
            if let Some(idx) = trimmed.find(':') {
                let name = trimmed[..idx].trim();
                in_target = name == equation;
                in_preconditions = false;
            }
            continue;
        }
        if !in_target {
            continue;
        }
        // Indent 4 = field under the target equation
        if indent == 4 && !trimmed.starts_with('-') {
            if let Some(idx) = trimmed.find(':') {
                let key = trimmed[..idx].trim();
                in_preconditions = key == "preconditions";
                if in_preconditions && preconditions.is_none() {
                    preconditions = Some(Vec::new());
                }
                continue;
            }
        }
        // Indent ≥4, starts with `-` and we're in preconditions: a list item
        if in_preconditions && indent >= 4 && trimmed.starts_with('-') {
            let item = trimmed[1..].trim();
            let unquoted = item
                .trim_start_matches('"')
                .trim_end_matches('"')
                .trim_start_matches('\'')
                .trim_end_matches('\'')
                .to_string();
            if !unquoted.is_empty() {
                if let Some(v) = preconditions.as_mut() {
                    v.push(unquoted);
                }
            }
            continue;
        }
        // Any other line under the target equation that isn't a sub-field
        // continuation ends the preconditions list.
        if in_preconditions && indent < 4 {
            in_preconditions = false;
        }
    }
    preconditions
}

/// CB-1603 (L3): verify each bound equation's YAML-declared `preconditions:`
/// are reflected in the ticket's `contract.require[]`. Catches inheritance
/// pipeline regressions where a tightening of the bound equation's precond
/// set isn't propagated to in-flight tickets.
///
/// Spec §Inheritance: a ticket inherits preconditions from each bound
/// equation. This check enforces that inheritance at the contract level —
/// equivalent to verifying `inherited-clauses.json` against source YAML,
/// without requiring the intermediate artifact.
pub(crate) fn check_binding_inherited_clauses(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1603: Inherited Clause Integrity";
    let contracts = load_active_contracts(project_path);
    if iter_bindings(&contracts).next().is_none() {
        return skip_no_bindings(name);
    }

    let mut missing: Vec<String> = Vec::new();
    let mut checked_bindings = 0usize;
    let mut bindings_with_preconds = 0usize;

    for contract in &contracts {
        // Collect require-clause descriptions + ids for this ticket. A YAML
        // precondition string matches if it appears either as a clause id
        // (e.g. "require.compiles") or as the human-readable description.
        let ticket_require: std::collections::HashSet<&str> = contract
            .require
            .iter()
            .flat_map(|c| [c.id.as_str(), c.description.as_str()])
            .collect();

        for binding in &contract.implements {
            checked_bindings += 1;
            let yaml_path = if binding.file.is_absolute() {
                binding.file.clone()
            } else {
                project_path.join(&binding.file)
            };
            let Ok(yaml) = std::fs::read_to_string(&yaml_path) else {
                continue;
            };
            let Some(preconds) = yaml_equation_preconditions(&yaml, &binding.equation) else {
                continue; // No preconditions declared for this equation
            };
            if preconds.is_empty() {
                continue;
            }
            bindings_with_preconds += 1;
            for p in &preconds {
                if !ticket_require.contains(p.as_str()) {
                    missing.push(format!(
                        "  {} [{}] missing inherited: {}",
                        contract.work_item_id,
                        binding.key(),
                        p
                    ));
                }
            }
        }
    }

    if bindings_with_preconds == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: format!(
                "{} binding(s) checked; none declare YAML preconditions to inherit",
                checked_bindings
            ),
            severity: Severity::Info,
        };
    }

    if missing.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} binding(s) with preconditions — all inherited into `require:`",
                bindings_with_preconds
            ),
            severity: Severity::Info,
        };
    }

    let mut msg = format!("{} inherited precondition(s) missing:\n", missing.len());
    for line in &missing {
        msg.push_str(line);
        msg.push('\n');
    }
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Fail,
        message: msg,
        severity: Severity::Error,
    }
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
        if trimmed.starts_with('-') {
            let item = trimmed[1..].trim();
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
            check_binding_postcondition_weakening(tmp.path()),
            check_binding_lean_theorem(tmp.path()),
            check_binding_cross_consistency(tmp.path()),
        ] {
            assert_eq!(r.status, CheckStatus::Skip);
            assert!(r.message.starts_with("Deferred"));
        }
    }

    // ── CB-1602 unbind audit tests ────────────────────────────────────────

    fn write_unbind_ledger(project: &Path, body: &str) {
        let dir = project.join(".pmat-work").join("ledger");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("unbinds.json"), body).unwrap();
    }

    #[test]
    fn unbind_audit_skips_when_ledger_missing() {
        let tmp = tempdir().unwrap();
        let r = check_binding_unbind_audit(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No unbind ledger"));
    }

    #[test]
    fn unbind_audit_passes_when_ledger_empty_array() {
        let tmp = tempdir().unwrap();
        write_unbind_ledger(tmp.path(), "[]");
        let r = check_binding_unbind_audit(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("empty"));
    }

    #[test]
    fn unbind_audit_passes_when_all_have_debt_ticket() {
        let tmp = tempdir().unwrap();
        write_unbind_ledger(
            tmp.path(),
            r#"[
                {"ticket":"T-1","contract":"contracts/rope.yaml","debt_ticket":"DEBT-42"},
                {"ticket":"T-2","contract":"contracts/norm.yaml","debt_ticket":"DEBT-43"}
            ]"#,
        );
        let r = check_binding_unbind_audit(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("2 unbind"));
    }

    #[test]
    fn unbind_audit_fails_on_missing_debt_ticket() {
        let tmp = tempdir().unwrap();
        write_unbind_ledger(
            tmp.path(),
            r#"[{"ticket":"T-1","contract":"contracts/rope.yaml"}]"#,
        );
        let r = check_binding_unbind_audit(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
        assert!(r.message.contains("debt_ticket"));
    }

    #[test]
    fn unbind_audit_fails_on_empty_debt_ticket() {
        let tmp = tempdir().unwrap();
        write_unbind_ledger(
            tmp.path(),
            r#"[{"ticket":"T-1","contract":"c","debt_ticket":"  "}]"#,
        );
        let r = check_binding_unbind_audit(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
    }

    #[test]
    fn unbind_audit_fails_on_malformed_json() {
        let tmp = tempdir().unwrap();
        write_unbind_ledger(tmp.path(), "not-json");
        let r = check_binding_unbind_audit(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("not valid JSON"));
    }

    #[test]
    fn unbind_audit_fails_when_top_level_object() {
        let tmp = tempdir().unwrap();
        write_unbind_ledger(tmp.path(), r#"{"ticket":"T-1","debt_ticket":"DEBT-1"}"#);
        let r = check_binding_unbind_audit(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("JSON array"));
    }

    #[test]
    fn unbind_audit_aggregates_multiple_failures() {
        let tmp = tempdir().unwrap();
        write_unbind_ledger(
            tmp.path(),
            r#"[
                {"ticket":"T-1","contract":"c"},
                {"ticket":"T-2","contract":"c","debt_ticket":"DEBT-2"},
                {"ticket":"T-3","contract":"c","debt_ticket":""}
            ]"#,
        );
        let r = check_binding_unbind_audit(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("2 unbind"));
        assert!(r.message.contains("T-1"));
        assert!(r.message.contains("T-3"));
        assert!(!r.message.contains("T-2 missing"));
    }

    // ── CB-1603 inherited clause integrity tests ─────────────────────────

    #[test]
    fn yaml_precond_parses_list() {
        let s = "equations:\n  rope:\n    preconditions:\n    - \"foo\"\n    - \"bar\"\n  softmax: {}\n";
        let p = yaml_equation_preconditions(s, "rope").unwrap();
        assert_eq!(p, vec!["foo".to_string(), "bar".to_string()]);
    }

    #[test]
    fn yaml_precond_returns_none_for_missing_equation() {
        let s = "equations:\n  rope:\n    preconditions:\n    - \"foo\"\n";
        assert!(yaml_equation_preconditions(s, "softmax").is_none());
    }

    #[test]
    fn yaml_precond_returns_none_when_field_absent() {
        let s = "equations:\n  rope:\n    invariants:\n    - \"x\"\n";
        assert!(yaml_equation_preconditions(s, "rope").is_none());
    }

    #[test]
    fn inherited_clauses_skip_without_bindings() {
        let tmp = tempdir().unwrap();
        let r = check_binding_inherited_clauses(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }

    #[test]
    fn inherited_clauses_skip_when_no_yaml_preconds() {
        let tmp = tempdir().unwrap();
        write_yaml(tmp.path(), "k", "equations:\n  rope: {}\n");
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/k.yaml"),
            "rope",
            "deadbeef",
        );
        let r = check_binding_inherited_clauses(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip, "{}", r.message);
        assert!(r.message.contains("YAML preconditions"));
    }

    #[test]
    fn inherited_clauses_fails_when_require_missing_entry() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "k",
            "equations:\n  rope:\n    preconditions:\n    - \"input normalized\"\n",
        );
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/k.yaml"),
            "rope",
            "deadbeef",
        );
        // Ticket's contract.require is empty → inheritance broken
        let r = check_binding_inherited_clauses(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("input normalized"));
    }

    #[test]
    fn inherited_clauses_passes_when_require_has_description() {
        use crate::cli::handlers::work_contract::{
            ClauseKind, ClauseSource, ContractClause, FalsificationMethod,
        };
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "k",
            "equations:\n  rope:\n    preconditions:\n    - \"input normalized\"\n",
        );
        let yaml = tmp.path().join("contracts/k.yaml");
        let sha = sha256_hex(&std::fs::read(&yaml).unwrap());
        let mut c =
            crate::cli::handlers::work_contract::WorkContract::new("T-1".into(), "deadbeef".into());
        c.implements.push(ContractBinding {
            contract: "k".into(),
            equation: "rope".into(),
            file: PathBuf::from("contracts/k.yaml"),
            sha,
            bound_at: chrono::Utc::now(),
        });
        c.require.push(ContractClause {
            id: "require.normalized".into(),
            kind: ClauseKind::Require,
            description: "input normalized".into(),
            falsification_method: FalsificationMethod::ManifestIntegrity,
            threshold: None,
            blocking: false,
            source: ClauseSource::Manual,
        });
        c.save(tmp.path()).unwrap();
        let r = check_binding_inherited_clauses(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn file_tracked_skips_without_bindings() {
        let tmp = tempdir().unwrap();
        let r = check_binding_file_tracked(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }

    // ── CB-1600 orphan-detection tests ───────────────────────────────────

    fn write_binding_index(project: &Path, entries: &[(&str, &[&str])]) {
        std::fs::create_dir_all(project.join(".pmat")).unwrap();
        let mut obj = serde_json::Map::new();
        for (file, names) in entries {
            let arr = names
                .iter()
                .map(|n| serde_json::Value::String((*n).to_string()))
                .collect::<Vec<_>>();
            obj.insert((*file).to_string(), serde_json::Value::Array(arr));
        }
        let json = serde_json::Value::Object(obj).to_string();
        std::fs::write(project.join(".pmat/binding-index.json"), json).unwrap();
    }

    /// Init a git repo in `project` and stage `files` (each created empty).
    fn init_repo_with_staged(project: &Path, files: &[&str]) {
        use std::process::Command;
        let run = |args: &[&str]| {
            let s = Command::new("git")
                .arg("-C")
                .arg(project)
                .args(args)
                .output()
                .unwrap();
            assert!(s.status.success(), "git {:?}: {:?}", args, s);
        };
        run(&["init", "--quiet", "--initial-branch=main"]);
        run(&["config", "user.email", "t@t"]);
        run(&["config", "user.name", "t"]);
        for f in files {
            let path = project.join(f);
            if let Some(p) = path.parent() {
                std::fs::create_dir_all(p).unwrap();
            }
            std::fs::write(&path, "").unwrap();
            run(&["add", f]);
        }
    }

    #[test]
    fn orphan_skip_when_no_binding_index() {
        let tmp = tempdir().unwrap();
        let r = check_binding_scope_orphan(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("binding-index.json"));
    }

    #[test]
    fn orphan_skip_when_no_staged_files() {
        let tmp = tempdir().unwrap();
        write_binding_index(tmp.path(), &[("src/rope.rs", &["rope"])]);
        // No git repo → staged_files returns empty → Skip
        let r = check_binding_scope_orphan(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No staged files"));
    }

    #[test]
    fn orphan_pass_when_staged_files_not_bound() {
        let tmp = tempdir().unwrap();
        write_binding_index(tmp.path(), &[("src/rope.rs", &["rope"])]);
        init_repo_with_staged(tmp.path(), &["README.md"]);
        let r = check_binding_scope_orphan(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("none intersect"));
    }

    #[test]
    fn orphan_fail_when_bound_file_staged_without_implements() {
        let tmp = tempdir().unwrap();
        write_binding_index(tmp.path(), &[("src/rope.rs", &["rope"])]);
        init_repo_with_staged(tmp.path(), &["src/rope.rs"]);
        // No `.pmat-work/*/contract.json` → no `implements:` coverage
        let r = check_binding_scope_orphan(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("src/rope.rs"));
        assert!(r.message.contains("rope"));
    }

    #[test]
    fn orphan_pass_when_bound_file_covered_by_implements() {
        let tmp = tempdir().unwrap();
        write_binding_index(tmp.path(), &[("src/rope.rs", &["rope"])]);
        init_repo_with_staged(tmp.path(), &["src/rope.rs"]);
        // Active ticket declares implements for src/rope.rs
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("src/rope.rs"),
            "rope",
            "deadbeef",
        );
        let r = check_binding_scope_orphan(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("covered"));
    }

    // ── CB-1605 Kani harness execution tests ─────────────────────────────

    fn write_kani_report(project: &Path, ticket: &str, body: &str) {
        let dir = project.join(".pmat-work").join(ticket);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("kani-report.json"), body).unwrap();
    }

    #[test]
    fn yaml_kani_harnesses_parses_block_style() {
        let s = "equations:\n  rope: {}\nkani_harnesses:\n- verify_a\n- verify_b\n";
        let names = yaml_kani_harness_names(s).unwrap();
        assert_eq!(names, vec!["verify_a".to_string(), "verify_b".to_string()]);
    }

    #[test]
    fn yaml_kani_harnesses_parses_indented_style() {
        let s = "kani_harnesses:\n  - verify_a\n  - verify_b\n";
        let names = yaml_kani_harness_names(s).unwrap();
        assert_eq!(names, vec!["verify_a".to_string(), "verify_b".to_string()]);
    }

    #[test]
    fn yaml_kani_harnesses_parses_object_form() {
        let s = "kani_harnesses:\n- name: verify_a\n- name: verify_b\n";
        let names = yaml_kani_harness_names(s).unwrap();
        assert_eq!(names, vec!["verify_a".to_string(), "verify_b".to_string()]);
    }

    #[test]
    fn yaml_kani_harnesses_flow_empty_returns_none() {
        let s = "kani_harnesses: []\n";
        assert!(yaml_kani_harness_names(s).is_none());
    }

    #[test]
    fn yaml_kani_harnesses_missing_returns_none() {
        let s = "equations:\n  rope: {}\n";
        assert!(yaml_kani_harness_names(s).is_none());
    }

    #[test]
    fn yaml_kani_harnesses_ignores_comments_and_blanks() {
        let s = "kani_harnesses:\n# comment\n- verify_a\n\n- verify_b\n";
        let names = yaml_kani_harness_names(s).unwrap();
        assert_eq!(names, vec!["verify_a".to_string(), "verify_b".to_string()]);
    }

    #[test]
    fn parse_kani_report_canonical_shape() {
        let r = r#"{"harnesses":[{"name":"h1","success":true},{"name":"h2","success":false}]}"#;
        let out = parse_kani_harness_results(r).unwrap();
        assert_eq!(
            out,
            vec![("h1".to_string(), true), ("h2".to_string(), false)]
        );
    }

    #[test]
    fn parse_kani_report_results_alias() {
        let r = r#"{"results":[{"name":"h1","success":true}]}"#;
        let out = parse_kani_harness_results(r).unwrap();
        assert_eq!(out, vec![("h1".to_string(), true)]);
    }

    #[test]
    fn parse_kani_report_status_string_coerces() {
        let r =
            r#"{"harnesses":[{"name":"h1","status":"proved"},{"name":"h2","status":"failed"}]}"#;
        let out = parse_kani_harness_results(r).unwrap();
        assert_eq!(
            out,
            vec![("h1".to_string(), true), ("h2".to_string(), false)]
        );
    }

    #[test]
    fn kani_harnesses_skip_without_bindings() {
        let tmp = tempdir().unwrap();
        let r = check_binding_kani_harnesses(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("implements:"));
    }

    #[test]
    fn kani_harnesses_skip_when_no_yaml_declares() {
        let tmp = tempdir().unwrap();
        write_yaml(tmp.path(), "k", "equations:\n  rope: {}\n");
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/k.yaml"),
            "rope",
            "deadbeef",
        );
        let r = check_binding_kani_harnesses(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip, "{}", r.message);
        assert!(r.message.contains("kani_harnesses"));
    }

    #[test]
    fn kani_harnesses_skip_when_declared_but_no_report() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "k",
            "equations:\n  rope: {}\nkani_harnesses:\n- verify_a\n",
        );
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/k.yaml"),
            "rope",
            "deadbeef",
        );
        let r = check_binding_kani_harnesses(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip, "{}", r.message);
        assert!(r.message.contains("kani-report.json"));
    }

    #[test]
    fn kani_harnesses_pass_when_all_harnesses_succeed() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "k",
            "equations:\n  rope: {}\nkani_harnesses:\n- verify_a\n- verify_b\n",
        );
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/k.yaml"),
            "rope",
            "deadbeef",
        );
        write_kani_report(
            tmp.path(),
            "T-1",
            r#"{"success":true,"harnesses":[{"name":"verify_a","success":true},{"name":"verify_b","success":true}]}"#,
        );
        let r = check_binding_kani_harnesses(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("succeeded"));
    }

    #[test]
    fn kani_harnesses_fails_when_declared_harness_missing() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "k",
            "equations:\n  rope: {}\nkani_harnesses:\n- verify_a\n- verify_b\n",
        );
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/k.yaml"),
            "rope",
            "deadbeef",
        );
        // Report only covers verify_a — verify_b is missing
        write_kani_report(
            tmp.path(),
            "T-1",
            r#"{"harnesses":[{"name":"verify_a","success":true}]}"#,
        );
        let r = check_binding_kani_harnesses(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("verify_b"));
        assert!(r.message.contains("T-1"));
    }

    #[test]
    fn kani_harnesses_fails_when_harness_failed() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "k",
            "equations:\n  rope: {}\nkani_harnesses:\n- verify_a\n",
        );
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/k.yaml"),
            "rope",
            "deadbeef",
        );
        write_kani_report(
            tmp.path(),
            "T-1",
            r#"{"harnesses":[{"name":"verify_a","success":false}]}"#,
        );
        let r = check_binding_kani_harnesses(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("verify_a"));
        assert!(r.message.contains("success=false"));
    }

    #[test]
    fn kani_harnesses_fails_on_malformed_report() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "k",
            "equations:\n  rope: {}\nkani_harnesses:\n- verify_a\n",
        );
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("contracts/k.yaml"),
            "rope",
            "deadbeef",
        );
        // Report is valid JSON but lacks harnesses/results array
        write_kani_report(tmp.path(), "T-1", r#"{"success":true}"#);
        let r = check_binding_kani_harnesses(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("missing `harnesses`"));
    }
}
