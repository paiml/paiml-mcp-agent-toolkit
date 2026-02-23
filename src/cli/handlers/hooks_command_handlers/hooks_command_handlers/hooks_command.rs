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
    pub fn new(hooks_dir: PathBuf, _config_path: PathBuf) -> Self {
        Self { hooks_dir }
    }

    /// Get default hooks command for current repository
    pub fn for_current_repo() -> Result<Self> {
        let current_dir = std::env::current_dir()?;
        let git_dir = current_dir.join(".git");
        let hooks_dir = git_dir.join("hooks");
        let config_path = current_dir.join("pmat.toml");

        Ok(Self::new(hooks_dir, config_path))
    }

    /// Install or update pre-commit hooks
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

        Ok(HookInstallResult {
            success: true,
            hook_created: true,
            backup_created,
            message: "Pre-commit hook installed successfully".to_string(),
        })
    }

    /// Uninstall PMAT-managed hooks
    pub async fn uninstall(&self, restore_backup: bool) -> Result<HookUninstallResult> {
        let hook_path = self.hooks_dir.join("pre-commit");
        let backup_path = self.hooks_dir.join("pre-commit.pmat-backup");

        if !hook_path.exists() {
            return Ok(HookUninstallResult {
                success: true,
                hook_removed: false,
                backup_restored: false,
                message: "No hook to uninstall".to_string(),
            });
        }

        // Check if it's PMAT-managed
        if !self.is_pmat_managed(&hook_path)? {
            return Ok(HookUninstallResult {
                success: false,
                hook_removed: false,
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
            message: "Pre-commit hook uninstalled successfully".to_string(),
        })
    }

    /// Show hook installation status
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
}
