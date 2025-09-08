//! Hooks command handlers for pre-commit hook management
//!
//! Following TDD approach for Sprint 80: Pre-commit Hook Management as Core Feature
//! Implements dynamic hook management as specified in:
//! docs/specifications/pre-commit-hooks-spec.md

use crate::cli::commands::HooksCommands;
use crate::services::configuration_service::{configuration, PmatConfig};
use anyhow::Result;
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};

/// Hooks command implementation
pub struct HooksCommand {
    hooks_dir: PathBuf,
}

impl HooksCommand {
    /// Create new hooks command with specified directories
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
    pub async fn install(&self, force: bool, backup: bool) -> Result<HookInstallResult> {
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

        Ok(HookStatus {
            installed: true,
            is_pmat_managed,
            config_up_to_date: is_pmat_managed, // TODO: Check actual config hash
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
                self.install(false, true).await?;
                fixes_applied.push("Installed missing hook".to_string());
            }
        } else if !self.is_pmat_managed(&hook_path)? {
            issues.push("Hook not PMAT-managed".to_string());
        } else {
            // Check if hook content is up-to-date
            let current_content = fs::read_to_string(&hook_path)?;
            let expected_content = self.generate_hook_content().await?;

            if current_content != expected_content {
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

    /// Check if a hook is PMAT-managed
    fn is_pmat_managed(&self, hook_path: &Path) -> Result<bool> {
        if !hook_path.exists() {
            return Ok(false);
        }

        let content = fs::read_to_string(hook_path)?;
        Ok(content.contains("auto-managed by PMAT") && content.contains("DO NOT EDIT"))
    }

    /// Generate hook content from template and configuration
    async fn generate_hook_content(&self) -> Result<String> {
        let config_service = configuration();
        let config = config_service.get_config()?;

        let header = self.generate_hook_header();
        let env_vars = self.generate_env_vars(&config);
        let checks = self.generate_quality_checks();

        let hook_content = format!("{header}\n{env_vars}\n{checks}");
        Ok(hook_content)
    }

    /// Generate hook header section
    fn generate_hook_header(&self) -> String {
        format!(
            r#"#!/bin/bash
# Generated pre-commit hook (auto-managed by PMAT)
# DO NOT EDIT: This file is automatically generated
# Generated at: {}

set -e

echo "🔍 PMAT Pre-commit Quality Gates"
echo "================================"
"#,
            Local::now().format("%Y-%m-%d %H:%M:%S")
        )
    }

    /// Generate environment variables section
    fn generate_env_vars(&self, config: &PmatConfig) -> String {
        format!(
            r#"# Load current configuration dynamically
export PMAT_MAX_CYCLOMATIC_COMPLEXITY={}
export PMAT_MAX_COGNITIVE_COMPLEXITY={}
export PMAT_MIN_TEST_COVERAGE={}
export PMAT_MAX_SATD_COMMENTS=5
export PMAT_TASK_ID_PATTERN="PMAT-[0-9]{{4}}"
"#,
            config.quality.max_complexity,
            config.quality.max_cognitive_complexity,
            config.quality.min_coverage as u32
        )
    }

    /// Generate quality check sections
    fn generate_quality_checks(&self) -> String {
        r#"# Check if pmat is available
if ! command -v pmat &> /dev/null; then
    echo "⚠️  Warning: pmat not found in PATH"
    echo "   Install with: cargo install pmat"
    exit 0  # Allow commit but warn
fi

echo "📊 Running quality gate checks..."

# 1. Complexity analysis
echo -n "  Complexity check... "
if pmat analyze complexity --max-cyclomatic $PMAT_MAX_CYCLOMATIC_COMPLEXITY --quiet; then
    echo "✅"
else
    echo "❌"
    echo "   Complexity exceeds threshold ($PMAT_MAX_CYCLOMATIC_COMPLEXITY)"
    exit 1
fi

# 2. SATD (Self-Admitted Technical Debt) check
echo -n "  SATD check... "
if pmat analyze satd --quiet; then
    echo "✅"
else
    SATD_COUNT=$(pmat analyze satd --format json 2>/dev/null | grep -c "TODO\|FIXME\|HACK" || echo "0")
    if [ "$SATD_COUNT" -gt "$PMAT_MAX_SATD_COMMENTS" ]; then
        echo "❌"
        echo "   SATD comments exceed threshold ($PMAT_MAX_SATD_COMMENTS)"
        exit 1
    fi
    echo "✅"
fi

# 3. Documentation synchronization
echo -n "  Documentation check... "
if [ -f "docs/execution/roadmap.md" ] && [ -f "CHANGELOG.md" ]; then
    echo "✅"
else
    echo "⚠️"
    echo "   Warning: Required documentation files missing"
fi

# 4. Task ID validation (if commit message available)
if [ -n "$1" ]; then
    echo -n "  Task ID check... "
    if echo "$1" | grep -qE "$PMAT_TASK_ID_PATTERN"; then
        echo "✅"
    else
        echo "⚠️"
        echo "   Warning: Commit message should contain task ID matching $PMAT_TASK_ID_PATTERN"
    fi
fi

echo ""
echo "✅ All quality gates passed!"
echo ""

# Success
exit 0
"#.to_string()
    }
}

/// Hook installation result
#[derive(Debug, PartialEq)]
pub struct HookInstallResult {
    pub success: bool,
    pub hook_created: bool,
    pub backup_created: bool,
    pub message: String,
}

/// Hook uninstall result
#[derive(Debug, PartialEq)]
pub struct HookUninstallResult {
    pub success: bool,
    pub hook_removed: bool,
    pub backup_restored: bool,
    pub message: String,
}

/// Hook status information
#[derive(Debug, PartialEq)]
pub struct HookStatus {
    pub installed: bool,
    pub is_pmat_managed: bool,
    pub config_up_to_date: bool,
    pub last_updated: Option<String>,
    pub hook_content_preview: Option<String>,
}

/// Hook verification result
#[derive(Debug, PartialEq)]
pub struct HookVerificationResult {
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub fixes_applied: Vec<String>,
}

/// Hook refresh result
#[derive(Debug, PartialEq)]
pub struct HookRefreshResult {
    pub success: bool,
    pub hook_updated: bool,
    pub config_changes_detected: bool,
    pub message: String,
}

/// Handle hooks subcommand
pub async fn handle_hooks_command(cmd: &HooksCommands) -> Result<()> {
    let hooks_cmd = HooksCommand::for_current_repo()?;

    match cmd {
        HooksCommands::Install { force, backup } => {
            handle_install(&hooks_cmd, *force, *backup).await
        }
        HooksCommands::Uninstall { restore_backup } => {
            handle_uninstall(&hooks_cmd, *restore_backup).await
        }
        HooksCommands::Status => handle_status(&hooks_cmd).await,
        HooksCommands::Verify { fix } => handle_verify(&hooks_cmd, *fix).await,
        HooksCommands::Refresh => handle_refresh(&hooks_cmd).await,
    }
}

/// Handle hooks install command
async fn handle_install(hooks_cmd: &HooksCommand, force: bool, backup: bool) -> Result<()> {
    println!("🔧 Installing pre-commit hooks...");
    if force {
        println!("  Force installation enabled");
    }
    if backup {
        println!("  Backup existing hooks enabled");
    }

    let result = hooks_cmd.install(force, backup).await?;

    if result.success {
        if result.backup_created {
            println!("  📁 Backup created: .git/hooks/pre-commit.pmat-backup");
        }
        println!("✅ {}", result.message);
    } else {
        println!("❌ {}", result.message);
        return Err(anyhow::anyhow!(result.message));
    }

    Ok(())
}

/// Handle hooks uninstall command
async fn handle_uninstall(hooks_cmd: &HooksCommand, restore_backup: bool) -> Result<()> {
    println!("🗑️ Uninstalling pre-commit hooks...");
    if restore_backup {
        println!("  Restoring backup enabled");
    }

    let result = hooks_cmd.uninstall(restore_backup).await?;

    if result.success {
        if result.backup_restored {
            println!("  📁 Backup restored");
        }
        println!("✅ {}", result.message);
    } else {
        println!("❌ {}", result.message);
        return Err(anyhow::anyhow!(result.message));
    }

    Ok(())
}

/// Handle hooks status command
async fn handle_status(hooks_cmd: &HooksCommand) -> Result<()> {
    let status = hooks_cmd.status().await?;

    println!("📊 Pre-commit Hook Status:");
    println!(
        "  Installed: {}",
        if status.installed {
            "✅ Yes"
        } else {
            "❌ No"
        }
    );

    if status.installed {
        print_installed_status(&status);
    }

    Ok(())
}

/// Print detailed status for installed hook
fn print_installed_status(status: &HookStatus) {
    println!(
        "  PMAT-managed: {}",
        if status.is_pmat_managed {
            "✅ Yes"
        } else {
            "❌ No"
        }
    );
    println!(
        "  Config up-to-date: {}",
        if status.config_up_to_date {
            "✅ Yes"
        } else {
            "⚠️ No"
        }
    );

    if let Some(last_updated) = &status.last_updated {
        println!("  Last updated: {}", last_updated);
    }

    if let Some(preview) = &status.hook_content_preview {
        println!("\n  Hook preview:");
        for line in preview.lines() {
            println!("    {}", line);
        }
    }
}

/// Handle hooks verify command
async fn handle_verify(hooks_cmd: &HooksCommand, fix: bool) -> Result<()> {
    println!("🔍 Verifying pre-commit hooks...");

    if fix {
        println!("  Auto-fix enabled");
    }

    let result = hooks_cmd.verify(fix).await?;

    print_verification_issues(&result);
    print_verification_fixes(&result);

    if result.is_valid {
        println!("✅ Pre-commit hooks verified successfully");
    } else {
        println!("❌ Pre-commit hooks verification failed");
        if !fix {
            println!("   Run with --fix to attempt automatic repairs");
        }
        return Err(anyhow::anyhow!("Hook verification failed"));
    }

    Ok(())
}

/// Print verification issues
fn print_verification_issues(result: &HookVerificationResult) {
    if !result.issues.is_empty() {
        println!("  Issues found:");
        for issue in &result.issues {
            println!("    ⚠️ {}", issue);
        }
    }
}

/// Print verification fixes applied
fn print_verification_fixes(result: &HookVerificationResult) {
    if !result.fixes_applied.is_empty() {
        println!("  Fixes applied:");
        for fix_msg in &result.fixes_applied {
            println!("    🔧 {}", fix_msg);
        }
    }
}

/// Handle hooks refresh command
async fn handle_refresh(hooks_cmd: &HooksCommand) -> Result<()> {
    println!("🔄 Refreshing pre-commit hooks from configuration...");

    let result = hooks_cmd.refresh().await?;

    if result.success {
        if result.config_changes_detected {
            println!("  📝 Configuration changes detected");
        }
        if result.hook_updated {
            println!("  🔄 Hook updated with new configuration");
        }
        println!("✅ {}", result.message);
    } else {
        println!("❌ {}", result.message);
        return Err(anyhow::anyhow!(result.message));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hooks_install() {
        let cmd = HooksCommands::Install {
            force: false,
            backup: true,
        };

        let result = handle_hooks_command(&cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_hooks_status() {
        let cmd = HooksCommands::Status;

        let result = handle_hooks_command(&cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_hooks_verify() {
        let cmd = HooksCommands::Verify { fix: false };

        let result = handle_hooks_command(&cmd).await;
        assert!(result.is_ok());
    }
}
