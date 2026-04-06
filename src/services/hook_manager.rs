#![cfg_attr(coverage_nightly, coverage(off))]
// Hook manager for workflow integration (Issue #75 Phase 6)
//
// Manages git hooks for commit message validation and quality gates.

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Git hook template for commit-msg validation
const COMMIT_MSG_HOOK: &str = r#"#!/bin/bash
# PMAT Workflow Commit Message Hook (Issue #75)
# Validates commit messages reference work items

COMMIT_MSG_FILE=$1
COMMIT_MSG=$(cat "$COMMIT_MSG_FILE")

# Skip merge commits and revert commits
if echo "$COMMIT_MSG" | grep -qE "^(Merge|Revert)"; then
    exit 0
fi

# Check for work item reference: "Refs #123" or "Refs TICKET-ID"
if ! echo "$COMMIT_MSG" | grep -qE "(Refs #[0-9]+|Refs [A-Za-z0-9_-]+)"; then
    echo "❌ Commit message must reference a work item"
    echo ""
    echo "   Expected format:"
    echo "   - GitHub issue: 'feat: Add feature (Refs #123)'"
    echo "   - YAML ticket: 'feat: Add feature (Refs my-ticket)'"
    echo ""
    echo "   See: pmat work status"
    exit 1
fi

# Extract work item ID
WORK_ITEM=$(echo "$COMMIT_MSG" | grep -oE "(Refs #[0-9]+|Refs [A-Za-z0-9_-]+)" | sed 's/Refs //' | sed 's/#/GH-/')

# Verify work item exists in roadmap
ROADMAP="docs/roadmaps/roadmap.yaml"
if [ -f "$ROADMAP" ]; then
    if ! grep -q "id: $WORK_ITEM" "$ROADMAP"; then
        echo "⚠️  Warning: Work item '$WORK_ITEM' not found in roadmap"
        echo "   Run: pmat work start $WORK_ITEM"
        # Don't block, just warn
    fi
fi

exit 0
"#;

/// Install commit-msg hook in git repository
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn install_commit_msg_hook(project_path: &PathBuf) -> Result<()> {
    let git_dir = project_path.join(".git");
    if !git_dir.exists() {
        anyhow::bail!("Not a git repository");
    }

    let hooks_dir = git_dir.join("hooks");
    fs::create_dir_all(&hooks_dir).context("Failed to create .git/hooks directory")?;

    let hook_path = hooks_dir.join("commit-msg");

    // Check if hook already exists
    if hook_path.exists() {
        let existing = fs::read_to_string(&hook_path)?;
        if existing.contains("PMAT Workflow Commit Message Hook") {
            // Already installed
            return Ok(());
        } else {
            // Backup existing hook
            let backup_path = hooks_dir.join("commit-msg.backup");
            fs::copy(&hook_path, &backup_path)
                .context("Failed to backup existing commit-msg hook")?;
            println!("   ℹ️  Backed up existing hook to commit-msg.backup");
        }
    }

    // Write hook
    fs::write(&hook_path, COMMIT_MSG_HOOK).context("Failed to write commit-msg hook")?;

    // Make executable (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms)?;
    }

    Ok(())
}

/// Uninstall commit-msg hook
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn uninstall_commit_msg_hook(project_path: &PathBuf) -> Result<()> {
    let hook_path = project_path.join(".git/hooks/commit-msg");

    if !hook_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&hook_path)?;
    if !content.contains("PMAT Workflow Commit Message Hook") {
        anyhow::bail!("commit-msg hook is not a PMAT hook, refusing to uninstall");
    }

    fs::remove_file(&hook_path).context("Failed to remove commit-msg hook")?;

    // Restore backup if exists
    let backup_path = project_path.join(".git/hooks/commit-msg.backup");
    if backup_path.exists() {
        fs::rename(&backup_path, &hook_path).context("Failed to restore backup hook")?;
        println!("   ✅ Restored backup hook");
    }

    Ok(())
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_repo() -> TempDir {
        let temp = TempDir::new().unwrap();
        let git_dir = temp.path().join(".git/hooks");
        fs::create_dir_all(&git_dir).unwrap();
        temp
    }

    #[test]
    fn test_install_commit_msg_hook() {
        let temp = create_test_repo();
        let result = install_commit_msg_hook(&temp.path().to_path_buf());
        assert!(result.is_ok());

        let hook_path = temp.path().join(".git/hooks/commit-msg");
        assert!(hook_path.exists());

        let content = fs::read_to_string(&hook_path).unwrap();
        assert!(content.contains("PMAT Workflow Commit Message Hook"));
    }

    #[test]
    fn test_install_twice_idempotent() {
        let temp = create_test_repo();
        let path = temp.path().to_path_buf();

        install_commit_msg_hook(&path).unwrap();
        install_commit_msg_hook(&path).unwrap(); // Should not error

        let hook_path = temp.path().join(".git/hooks/commit-msg");
        assert!(hook_path.exists());
    }

    #[test]
    fn test_uninstall_commit_msg_hook() {
        let temp = create_test_repo();
        let path = temp.path().to_path_buf();

        install_commit_msg_hook(&path).unwrap();
        uninstall_commit_msg_hook(&path).unwrap();

        let hook_path = temp.path().join(".git/hooks/commit-msg");
        assert!(!hook_path.exists());
    }

    #[test]
    fn test_backup_existing_hook() {
        let temp = create_test_repo();
        let path = temp.path().to_path_buf();
        let hook_path = temp.path().join(".git/hooks/commit-msg");

        // Create existing hook
        fs::write(&hook_path, "#!/bin/bash\necho 'existing hook'\n").unwrap();

        install_commit_msg_hook(&path).unwrap();

        let backup_path = temp.path().join(".git/hooks/commit-msg.backup");
        assert!(backup_path.exists());

        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert!(backup_content.contains("existing hook"));
    }
}
