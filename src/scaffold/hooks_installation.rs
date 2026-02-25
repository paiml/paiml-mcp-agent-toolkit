/// Install pre-commit hook to project directory
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 3
pub fn install_pre_commit_hook(project_dir: &Path, script: &str) -> Result<()> {
    use std::fs;

    let hook_path = project_dir.join(".git/hooks/pre-commit");

    // Write hook script
    fs::write(&hook_path, script).map_err(ScaffoldError::IoError)?;

    // Make executable (chmod +x)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_path)
            .map_err(ScaffoldError::IoError)?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms).map_err(ScaffoldError::IoError)?;
    }

    Ok(())
}

/// Install post-commit hook to project directory
///
/// # TICKET-PMAT-5013
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 2
pub fn install_post_commit_hook(project_dir: &Path) -> Result<()> {
    use std::fs;

    let hook_path = project_dir.join(".git/hooks/post-commit");
    let script = generate_post_commit_hook();

    fs::write(&hook_path, script).map_err(ScaffoldError::IoError)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_path)
            .map_err(ScaffoldError::IoError)?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms).map_err(ScaffoldError::IoError)?;
    }

    Ok(())
}
