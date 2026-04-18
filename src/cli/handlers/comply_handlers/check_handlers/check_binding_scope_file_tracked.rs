// CB-1609 Binding YAML Git-Tracked — included from check_binding_scope.rs.
// Do NOT add `use` imports or `#!` attributes here.

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
