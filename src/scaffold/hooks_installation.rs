/// Install pre-commit hook to project directory
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 3
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn install_pre_commit_hook(project_dir: &Path, script: &str) -> Result<()> {
    debug_assert!(project_dir.exists(), "project_dir must exist: {}", project_dir.display());
    use std::fs;

    let hook_path = project_dir.join(".git/hooks/pre-commit");

    // Atomic write: temp file + rename (CB-1334)
    let tmp_path = hook_path.with_extension("tmp");
    fs::write(&tmp_path, script).map_err(ScaffoldError::IoError)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&tmp_path)
            .map_err(ScaffoldError::IoError)?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&tmp_path, perms).map_err(ScaffoldError::IoError)?;
    }

    fs::rename(&tmp_path, &hook_path).map_err(ScaffoldError::IoError)?;

    Ok(())
}

/// Install post-commit hook to project directory
///
/// # TICKET-PMAT-5013
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 2
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn install_post_commit_hook(project_dir: &Path) -> Result<()> {
    debug_assert!(project_dir.exists(), "project_dir must exist: {}", project_dir.display());
    use std::fs;

    let hook_path = project_dir.join(".git/hooks/post-commit");
    let script = generate_post_commit_hook();

    // Atomic write: temp file + rename (CB-1334)
    let tmp_path = hook_path.with_extension("tmp");
    fs::write(&tmp_path, &script).map_err(ScaffoldError::IoError)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&tmp_path)
            .map_err(ScaffoldError::IoError)?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&tmp_path, perms).map_err(ScaffoldError::IoError)?;
    }

    fs::rename(&tmp_path, &hook_path).map_err(ScaffoldError::IoError)?;

    Ok(())
}
