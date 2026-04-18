// CB-1600 Binding Scope Orphan — included from check_binding_scope.rs.
// Do NOT add `use` imports or `#!` attributes here.

// ─── CB-160x check implementations (all active, skip-if-absent) ──────────────

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
