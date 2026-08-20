//! Core HooksCommand struct and lifecycle methods (install, uninstall, status, verify, refresh, run)

#![cfg_attr(coverage_nightly, coverage(off))]

use super::types::{
    HookInstallResult, HookRefreshResult, HookRunResult, HookStatus, HookUninstallResult,
    HookVerificationResult,
};
use anyhow::Result;
use chrono::Local;
use std::fs;
use std::path::PathBuf;

/// Hooks command implementation
pub struct HooksCommand {
    pub(super) hooks_dir: PathBuf,
}

impl HooksCommand {
    /// Create new hooks command with specified directories
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn new(hooks_dir: PathBuf, _config_path: PathBuf) -> Self {
        Self { hooks_dir }
    }

    /// Get default hooks command for current repository
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn for_current_repo() -> Result<Self> {
        let current_dir = std::env::current_dir()?;
        let git_dir = current_dir.join(".git");
        let hooks_dir = git_dir.join("hooks");
        let config_path = current_dir.join("pmat.toml");

        Ok(Self::new(hooks_dir, config_path))
    }

    /// Install or update pre-commit hooks
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn install(
        &self,
        force: bool,
        backup: bool,
        interactive: bool,
    ) -> Result<HookInstallResult> {
        // Interactive mode: prompt user for configuration preferences
        if interactive {
            self.run_interactive_setup()?;
        }
        let hook_path = self.hooks_dir.join("pre-commit");
        let backup_path = self.hooks_dir.join("pre-commit.pmat-backup");

        // Create hooks directory if it doesn't exist
        fs::create_dir_all(&self.hooks_dir)?;

        let mut backup_created = false;

        // Handle existing hook
        if hook_path.exists() {
            if backup && !backup_path.exists() {
                fs::copy(&hook_path, &backup_path)?;
                backup_created = true;
            } else if !force && !self.is_pmat_managed(&hook_path)? {
                return Ok(HookInstallResult {
                    success: false,
                    hook_created: false,
                    backup_created: false,
                    message: "Existing hook not PMAT-managed. Use --force to overwrite."
                        .to_string(),
                });
            }
        }

        // Generate hook content from template
        let hook_content = self.generate_hook_content().await?;

        // Write hook file
        fs::write(&hook_path, &hook_content)?;

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&hook_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms)?;
        }

        // Install pre-push hook (fast local gate)
        self.install_pre_push_hook()?;

        Ok(HookInstallResult {
            success: true,
            hook_created: true,
            backup_created,
            message: "Pre-commit and pre-push hooks installed successfully".to_string(),
        })
    }

    /// Uninstall PMAT-managed hooks
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn uninstall(&self, restore_backup: bool) -> Result<HookUninstallResult> {
        let hook_path = self.hooks_dir.join("pre-commit");
        let backup_path = self.hooks_dir.join("pre-commit.pmat-backup");

        // `install` writes pre-commit AND pre-push, but uninstall only ever
        // touched pre-commit: users who uninstalled were told "Pre-commit hook
        // uninstalled successfully" (and `hooks status` then said "Installed:
        // ✗ No") while the PMAT pre-push gate kept firing on every push, with
        // no command that reported or removed it. Remove every hook install
        // writes.
        let pre_push_removed = self.remove_pre_push_hook()?;

        if !hook_path.exists() {
            return Ok(HookUninstallResult {
                success: true,
                hook_removed: pre_push_removed,
                backup_restored: false,
                message: if pre_push_removed {
                    "Pre-push hook uninstalled successfully (no pre-commit hook)".to_string()
                } else {
                    "No hook to uninstall".to_string()
                },
            });
        }

        // Check if it's PMAT-managed
        if !self.is_pmat_managed(&hook_path)? {
            return Ok(HookUninstallResult {
                success: false,
                hook_removed: pre_push_removed,
                backup_restored: false,
                message: "Hook is not PMAT-managed".to_string(),
            });
        }

        // Remove hook
        fs::remove_file(&hook_path)?;

        let mut backup_restored = false;
        if restore_backup && backup_path.exists() {
            fs::rename(&backup_path, &hook_path)?;
            backup_restored = true;
        }

        Ok(HookUninstallResult {
            success: true,
            hook_removed: true,
            backup_restored,
            message: if pre_push_removed {
                "Pre-commit and pre-push hooks uninstalled successfully".to_string()
            } else {
                "Pre-commit hook uninstalled successfully".to_string()
            },
        })
    }

    /// Remove the PMAT-managed pre-push hook, if that is what is there.
    ///
    /// Returns whether a hook was removed. A pre-push hook PMAT did not write
    /// is left alone.
    fn remove_pre_push_hook(&self) -> Result<bool> {
        let hook_path = self.hooks_dir.join("pre-push");

        if !hook_path.exists() || !self.is_pmat_managed(&hook_path)? {
            return Ok(false);
        }

        fs::remove_file(&hook_path)?;
        Ok(true)
    }

    /// Show hook installation status
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn status(&self) -> Result<HookStatus> {
        let hook_path = self.hooks_dir.join("pre-commit");

        if !hook_path.exists() {
            return Ok(HookStatus {
                installed: false,
                is_pmat_managed: false,
                config_up_to_date: false,
                last_updated: None,
                hook_content_preview: None,
            });
        }

        let is_pmat_managed = self.is_pmat_managed(&hook_path)?;
        let content = fs::read_to_string(&hook_path)?;
        let preview = content.lines().take(10).collect::<Vec<_>>().join("\n");

        // Get modification time
        let metadata = fs::metadata(&hook_path)?;
        let modified = metadata.modified()?;
        let datetime = chrono::DateTime::<Local>::from(modified);

        // Check if config is up-to-date by comparing normalized content
        let config_up_to_date = if is_pmat_managed {
            match self.generate_hook_content().await {
                Ok(expected) => {
                    Self::normalize_hook_content(&content)
                        == Self::normalize_hook_content(&expected)
                }
                Err(_) => false, // Can't generate expected content, assume outdated
            }
        } else {
            false
        };

        Ok(HookStatus {
            installed: true,
            is_pmat_managed,
            config_up_to_date,
            last_updated: Some(datetime.format("%Y-%m-%d %H:%M:%S").to_string()),
            hook_content_preview: Some(preview),
        })
    }

    /// Verify hooks work with current configuration
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn verify(&self, fix: bool) -> Result<HookVerificationResult> {
        let hook_path = self.hooks_dir.join("pre-commit");
        let mut issues = Vec::new();
        let mut fixes_applied = Vec::new();

        if !hook_path.exists() {
            issues.push("Hook not installed".to_string());
            if fix {
                self.install(false, true, false).await?;
                fixes_applied.push("Installed missing hook".to_string());
            }
        } else if !self.is_pmat_managed(&hook_path)? {
            issues.push("Hook not PMAT-managed".to_string());
        } else {
            // Check if hook content is up-to-date (TICKET-PMAT-6011)
            // Strip timestamps before comparing to avoid false positives
            let current_content = fs::read_to_string(&hook_path)?;
            let expected_content = self.generate_hook_content().await?;

            let current_normalized = Self::normalize_hook_content(&current_content);
            let expected_normalized = Self::normalize_hook_content(&expected_content);

            if current_normalized != expected_normalized {
                issues.push("Hook content outdated".to_string());
                if fix {
                    fs::write(&hook_path, &expected_content)?;
                    fixes_applied.push("Updated hook content".to_string());
                }
            }
        }

        Ok(HookVerificationResult {
            is_valid: issues.is_empty() || (!fixes_applied.is_empty() && fix),
            issues,
            fixes_applied,
        })
    }

    /// Regenerate hooks from current configuration
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn refresh(&self) -> Result<HookRefreshResult> {
        let hook_path = self.hooks_dir.join("pre-commit");

        if !hook_path.exists() {
            return Ok(HookRefreshResult {
                success: false,
                hook_updated: false,
                config_changes_detected: false,
                message: "No hook to refresh".to_string(),
            });
        }

        if !self.is_pmat_managed(&hook_path)? {
            return Ok(HookRefreshResult {
                success: false,
                hook_updated: false,
                config_changes_detected: false,
                message: "Hook is not PMAT-managed".to_string(),
            });
        }

        let current_content = fs::read_to_string(&hook_path)?;
        let new_content = self.generate_hook_content().await?;

        let config_changes_detected = current_content != new_content;

        if config_changes_detected {
            fs::write(&hook_path, &new_content)?;

            // Ensure executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&hook_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&hook_path, perms)?;
            }
        }

        Ok(HookRefreshResult {
            success: true,
            hook_updated: config_changes_detected,
            config_changes_detected,
            message: if config_changes_detected {
                "Hook refreshed with configuration changes".to_string()
            } else {
                "Hook already up-to-date".to_string()
            },
        })
    }

    /// Run hooks on files (for CI/CD integration)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn run(&self, all_files: bool, verbose: bool) -> Result<HookRunResult> {
        use std::process::Command;

        let hook_path = self.hooks_dir.join("pre-commit");

        if !hook_path.exists() {
            return Ok(HookRunResult {
                success: false,
                checks_passed: 0,
                checks_failed: 0,
                output: "Pre-commit hook not installed".to_string(),
            });
        }

        if verbose {
            println!("🔍 Running pre-commit hooks...");
            if all_files {
                println!("  Mode: All files");
            } else {
                println!("  Mode: Staged files only");
            }
        }

        // Run the hook script
        let output = Command::new("bash").arg(&hook_path).output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined_output = format!("{stdout}{stderr}");

        let success = output.status.success();

        // Count passed/failed checks from output
        let checks_passed = combined_output.matches("✅").count();
        let checks_failed = combined_output.matches("❌").count();

        Ok(HookRunResult {
            success,
            checks_passed,
            checks_failed,
            output: combined_output,
        })
    }

    /// Install pre-push hook (fast local quality gate).
    fn install_pre_push_hook(&self) -> Result<()> {
        let hook_path = self.hooks_dir.join("pre-push");

        let hook_content = r#"#!/usr/bin/env bash
# PMAT Pre-Push Quality Gate (O(1))
# auto-managed by PMAT — DO NOT EDIT
#
# Fast local gate (<2s). CI owns build/clippy/test.
# Bypass with: git push --no-verify

set -euo pipefail

# Skip for non-Rust repos
if [ ! -f Cargo.toml ]; then
    exit 0
fi

echo "🔍 PMAT Pre-Push Quality Gate"
echo "=============================="

FAILED=0

# 1. Format check (fast, ~1s — stylistic only, pre-commit handles per-file)
echo -n "  Format check... "
if cargo fmt --all -- --check > /dev/null 2>&1; then
    echo "✅"
else
    echo "❌"
    echo "   Run: cargo fmt --all"
    FAILED=1
fi

# Clippy is NOT re-run here: PMAT-630 moved it into the pre-commit hook, where
# it is content-addressed and costs ~0.3s on an already-proven tree. Every commit
# being pushed has therefore already been linted with `ci / lint`'s exact flags.
# Build and test still belong to CI, so `git push` stays O(1).

if [ "$FAILED" -ne 0 ]; then
    echo ""
    echo "❌ Pre-push gate FAILED — fix issues before pushing"
    echo "   Bypass (emergency): git push --no-verify"
    exit 1
fi

echo ""
echo "✅ Pre-push gate passed"
"#;

        fs::write(&hook_path, hook_content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&hook_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod uninstall_tests {
    //! `hooks install` writes two hooks; uninstall used to remove one.
    use super::HooksCommand;

    fn managed_hook(label: &str) -> String {
        format!(
            "#!/usr/bin/env bash\n# PMAT {label}\n# auto-managed by PMAT — DO NOT EDIT\nexit 0\n"
        )
    }

    #[tokio::test]
    async fn test_uninstall_removes_the_pre_push_hook_install_wrote() {
        let dir = tempfile::tempdir().expect("tempdir");
        let hooks_dir = dir.path().join("hooks");
        std::fs::create_dir_all(&hooks_dir).expect("create hooks dir");
        std::fs::write(hooks_dir.join("pre-commit"), managed_hook("pre-commit"))
            .expect("write pre-commit");
        std::fs::write(hooks_dir.join("pre-push"), managed_hook("pre-push"))
            .expect("write pre-push");

        let cmd = HooksCommand::new(hooks_dir.clone(), dir.path().join("pmat.toml"));
        let result = cmd.uninstall(false).await.expect("uninstall");

        assert!(result.success, "{}", result.message);
        assert!(!hooks_dir.join("pre-commit").exists());
        assert!(
            !hooks_dir.join("pre-push").exists(),
            "uninstall reported \"{}\" but the PMAT pre-push gate is still installed",
            result.message
        );
    }

    #[tokio::test]
    async fn test_uninstall_leaves_a_foreign_pre_push_hook_alone() {
        let dir = tempfile::tempdir().expect("tempdir");
        let hooks_dir = dir.path().join("hooks");
        std::fs::create_dir_all(&hooks_dir).expect("create hooks dir");
        std::fs::write(hooks_dir.join("pre-commit"), managed_hook("pre-commit"))
            .expect("write pre-commit");
        std::fs::write(hooks_dir.join("pre-push"), "#!/bin/sh\n# mine\nexit 0\n")
            .expect("write pre-push");

        let cmd = HooksCommand::new(hooks_dir.clone(), dir.path().join("pmat.toml"));
        cmd.uninstall(false).await.expect("uninstall");

        assert!(
            hooks_dir.join("pre-push").exists(),
            "a pre-push hook PMAT did not write must survive uninstall"
        );
    }
}
