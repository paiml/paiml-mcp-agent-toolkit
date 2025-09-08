//! TDD tests for `pmat hooks` command implementation
//!
//! Following Toyota Way TDD approach:
//! 1. RED: Write failing tests first
//! 2. GREEN: Implement minimum code to pass
//! 3. REFACTOR: Keep complexity ≤30 cyclomatic, ≤25 cognitive

use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio;

/// Test fixture for hooks command testing
struct HooksTestFixture {
    temp_dir: TempDir,
    git_dir: PathBuf,
    hooks_dir: PathBuf,
    config_path: PathBuf,
}

impl HooksTestFixture {
    /// Create test fixture with git repository and pmat.toml
    fn new() -> Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let git_dir = temp_dir.path().join(".git");
        let hooks_dir = git_dir.join("hooks");
        let config_path = temp_dir.path().join("pmat.toml");

        // Create .git/hooks directory
        fs::create_dir_all(&hooks_dir)?;

        let sample_config = r#"
[hooks]
enabled = true
auto_install = true
backup_existing = true

[hooks.quality_gates]
max_cyclomatic_complexity = 30
max_cognitive_complexity = 25
max_satd_comments = 5
min_test_coverage = 80.0
max_clippy_warnings = 100

[hooks.documentation]
required_files = [
    "docs/execution/roadmap.md",
    "CHANGELOG.md"
]
task_id_pattern = "PMAT-[0-9]{4}"
        "#;

        std::fs::write(&config_path, sample_config)?;

        Ok(Self {
            temp_dir,
            git_dir,
            hooks_dir,
            config_path,
        })
    }

    /// Get path to git hooks directory
    fn hooks_dir(&self) -> &PathBuf {
        &self.hooks_dir
    }

    /// Get path to pre-commit hook
    fn pre_commit_hook(&self) -> PathBuf {
        self.hooks_dir.join("pre-commit")
    }

    /// Get path to config file
    fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// Create existing pre-commit hook for testing
    fn create_existing_hook(&self, content: &str) -> Result<()> {
        let hook_path = self.pre_commit_hook();
        std::fs::write(&hook_path, content)?;

        // Make executable on Unix
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

/// Hooks command interface (to be implemented)
struct HooksCommand {
    hooks_dir: PathBuf,
    config_path: PathBuf,
}

impl HooksCommand {
    /// Create new hooks command with specified directories
    fn new(hooks_dir: PathBuf, config_path: PathBuf) -> Self {
        Self {
            hooks_dir,
            config_path,
        }
    }

    /// Install or update pre-commit hooks
    async fn install(&self, force: bool, backup: bool) -> Result<HookInstallResult> {
        // TO BE IMPLEMENTED - this should make test fail (RED phase)
        todo!("Implement hooks install command")
    }

    /// Uninstall PMAT-managed hooks
    async fn uninstall(&self, restore_backup: bool) -> Result<HookUninstallResult> {
        // TO BE IMPLEMENTED - this should make test fail (RED phase)
        todo!("Implement hooks uninstall command")
    }

    /// Show hook installation status
    async fn status(&self) -> Result<HookStatus> {
        // TO BE IMPLEMENTED - this should make test fail (RED phase)
        todo!("Implement hooks status command")
    }

    /// Verify hooks work with current configuration
    async fn verify(&self, fix: bool) -> Result<HookVerificationResult> {
        // TO BE IMPLEMENTED - this should make test fail (RED phase)
        todo!("Implement hooks verify command")
    }

    /// Regenerate hooks from current configuration
    async fn refresh(&self) -> Result<HookRefreshResult> {
        // TO BE IMPLEMENTED - this should make test fail (RED phase)
        todo!("Implement hooks refresh command")
    }
}

/// Hook installation result
#[derive(Debug, PartialEq)]
struct HookInstallResult {
    success: bool,
    hook_created: bool,
    backup_created: bool,
    message: String,
}

/// Hook uninstall result
#[derive(Debug, PartialEq)]
struct HookUninstallResult {
    success: bool,
    hook_removed: bool,
    backup_restored: bool,
    message: String,
}

/// Hook status information
#[derive(Debug, PartialEq)]
struct HookStatus {
    installed: bool,
    is_pmat_managed: bool,
    config_up_to_date: bool,
    last_updated: Option<String>,
    hook_content_preview: Option<String>,
}

/// Hook verification result
#[derive(Debug, PartialEq)]
struct HookVerificationResult {
    is_valid: bool,
    issues: Vec<String>,
    fixes_applied: Vec<String>,
}

/// Hook refresh result
#[derive(Debug, PartialEq)]
struct HookRefreshResult {
    success: bool,
    hook_updated: bool,
    config_changes_detected: bool,
    message: String,
}

// =============================================================================
// TDD TESTS (RED PHASE) - These should fail initially
// =============================================================================

#[tokio::test]
async fn test_hooks_install_basic() -> Result<()> {
    // ARRANGE
    let fixture = HooksTestFixture::new()?;
    let hooks_cmd = HooksCommand::new(fixture.hooks_dir().clone(), fixture.config_path().clone());

    // ACT
    let result = hooks_cmd.install(false, true).await?;

    // ASSERT
    assert_eq!(result.success, true);
    assert_eq!(result.hook_created, true);
    assert_eq!(result.backup_created, false); // No existing hook to backup

    // Hook file should exist and be executable
    let hook_path = fixture.pre_commit_hook();
    assert!(hook_path.exists());

    // Hook should contain PMAT-generated content
    let hook_content = std::fs::read_to_string(&hook_path)?;
    assert!(hook_content.contains("PMAT"));
    assert!(hook_content.contains("auto-generated"));
    assert!(hook_content.contains("DO NOT EDIT"));

    Ok(())
}

#[tokio::test]
async fn test_hooks_install_with_existing_hook() -> Result<()> {
    // ARRANGE
    let fixture = HooksTestFixture::new()?;
    fixture.create_existing_hook("#!/bin/bash\necho 'existing hook'")?;

    let hooks_cmd = HooksCommand::new(fixture.hooks_dir().clone(), fixture.config_path().clone());

    // ACT
    let result = hooks_cmd.install(false, true).await?;

    // ASSERT
    assert_eq!(result.success, true);
    assert_eq!(result.hook_created, true);
    assert_eq!(result.backup_created, true);

    // Backup should exist
    let backup_path = fixture.hooks_dir().join("pre-commit.pmat-backup");
    assert!(backup_path.exists());

    let backup_content = std::fs::read_to_string(&backup_path)?;
    assert!(backup_content.contains("existing hook"));

    Ok(())
}

#[tokio::test]
async fn test_hooks_install_force_overwrite() -> Result<()> {
    // ARRANGE
    let fixture = HooksTestFixture::new()?;
    fixture.create_existing_hook("#!/bin/bash\necho 'existing hook'")?;

    let hooks_cmd = HooksCommand::new(fixture.hooks_dir().clone(), fixture.config_path().clone());

    // ACT
    let result = hooks_cmd.install(true, false).await?;

    // ASSERT
    assert_eq!(result.success, true);
    assert_eq!(result.hook_created, true);
    assert_eq!(result.backup_created, false); // Force install without backup

    // Hook should be replaced
    let hook_content = std::fs::read_to_string(&fixture.pre_commit_hook())?;
    assert!(hook_content.contains("PMAT"));
    assert!(!hook_content.contains("existing hook"));

    Ok(())
}

#[tokio::test]
async fn test_hooks_uninstall_basic() -> Result<()> {
    // ARRANGE
    let fixture = HooksTestFixture::new()?;
    let hooks_cmd = HooksCommand::new(fixture.hooks_dir().clone(), fixture.config_path().clone());

    // Install first
    let _ = hooks_cmd.install(false, true).await?;

    // ACT
    let result = hooks_cmd.uninstall(false).await?;

    // ASSERT
    assert_eq!(result.success, true);
    assert_eq!(result.hook_removed, true);
    assert_eq!(result.backup_restored, false);

    // Hook should be removed
    assert!(!fixture.pre_commit_hook().exists());

    Ok(())
}

#[tokio::test]
async fn test_hooks_uninstall_with_backup_restore() -> Result<()> {
    // ARRANGE
    let fixture = HooksTestFixture::new()?;
    fixture.create_existing_hook("#!/bin/bash\necho 'original hook'")?;

    let hooks_cmd = HooksCommand::new(fixture.hooks_dir().clone(), fixture.config_path().clone());

    // Install (creates backup)
    let _ = hooks_cmd.install(false, true).await?;

    // ACT
    let result = hooks_cmd.uninstall(true).await?;

    // ASSERT
    assert_eq!(result.success, true);
    assert_eq!(result.hook_removed, true);
    assert_eq!(result.backup_restored, true);

    // Original hook should be restored
    let hook_content = std::fs::read_to_string(&fixture.pre_commit_hook())?;
    assert!(hook_content.contains("original hook"));
    assert!(!hook_content.contains("PMAT"));

    Ok(())
}

#[tokio::test]
async fn test_hooks_status_not_installed() -> Result<()> {
    // ARRANGE
    let fixture = HooksTestFixture::new()?;
    let hooks_cmd = HooksCommand::new(fixture.hooks_dir().clone(), fixture.config_path().clone());

    // ACT
    let result = hooks_cmd.status().await?;

    // ASSERT
    assert_eq!(result.installed, false);
    assert_eq!(result.is_pmat_managed, false);
    assert_eq!(result.config_up_to_date, false);
    assert_eq!(result.last_updated, None);
    assert_eq!(result.hook_content_preview, None);

    Ok(())
}

#[tokio::test]
async fn test_hooks_status_installed() -> Result<()> {
    // ARRANGE
    let fixture = HooksTestFixture::new()?;
    let hooks_cmd = HooksCommand::new(fixture.hooks_dir().clone(), fixture.config_path().clone());

    // Install first
    let _ = hooks_cmd.install(false, true).await?;

    // ACT
    let result = hooks_cmd.status().await?;

    // ASSERT
    assert_eq!(result.installed, true);
    assert_eq!(result.is_pmat_managed, true);
    assert_eq!(result.config_up_to_date, true);
    assert!(result.last_updated.is_some());
    assert!(result.hook_content_preview.is_some());

    let preview = result.hook_content_preview.unwrap();
    assert!(preview.contains("PMAT"));

    Ok(())
}

#[tokio::test]
async fn test_hooks_verify_valid() -> Result<()> {
    // ARRANGE
    let fixture = HooksTestFixture::new()?;
    let hooks_cmd = HooksCommand::new(fixture.hooks_dir().clone(), fixture.config_path().clone());

    // Install first
    let _ = hooks_cmd.install(false, true).await?;

    // ACT
    let result = hooks_cmd.verify(false).await?;

    // ASSERT
    assert_eq!(result.is_valid, true);
    assert!(result.issues.is_empty());
    assert!(result.fixes_applied.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_hooks_verify_with_fix() -> Result<()> {
    // ARRANGE
    let fixture = HooksTestFixture::new()?;
    let hooks_cmd = HooksCommand::new(fixture.hooks_dir().clone(), fixture.config_path().clone());

    // Install hook
    let _ = hooks_cmd.install(false, true).await?;

    // Corrupt the hook to test fix
    std::fs::write(&fixture.pre_commit_hook(), "#!/bin/bash\necho 'corrupted'")?;

    // ACT
    let result = hooks_cmd.verify(true).await?;

    // ASSERT
    assert_eq!(result.is_valid, true); // Should be valid after fix
    assert!(!result.issues.is_empty()); // Should have detected issues
    assert!(!result.fixes_applied.is_empty()); // Should have applied fixes

    // Hook should be fixed
    let hook_content = std::fs::read_to_string(&fixture.pre_commit_hook())?;
    assert!(hook_content.contains("PMAT"));
    assert!(!hook_content.contains("corrupted"));

    Ok(())
}

#[tokio::test]
async fn test_hooks_refresh() -> Result<()> {
    // ARRANGE
    let fixture = HooksTestFixture::new()?;
    let hooks_cmd = HooksCommand::new(fixture.hooks_dir().clone(), fixture.config_path().clone());

    // Install first
    let _ = hooks_cmd.install(false, true).await?;

    // Modify config to trigger refresh
    let new_config = r#"
[hooks]
enabled = true
auto_install = true

[hooks.quality_gates]
max_cyclomatic_complexity = 25
max_cognitive_complexity = 20
    "#;
    std::fs::write(&fixture.config_path, new_config)?;

    // ACT
    let result = hooks_cmd.refresh().await?;

    // ASSERT
    assert_eq!(result.success, true);
    assert_eq!(result.hook_updated, true);
    assert_eq!(result.config_changes_detected, true);

    Ok(())
}

#[tokio::test]
async fn test_hooks_template_generation() -> Result<()> {
    // ARRANGE
    let fixture = HooksTestFixture::new()?;
    let hooks_cmd = HooksCommand::new(fixture.hooks_dir().clone(), fixture.config_path().clone());

    // ACT
    let _ = hooks_cmd.install(false, true).await?;

    // ASSERT
    let hook_content = std::fs::read_to_string(&fixture.pre_commit_hook())?;

    // Should contain configuration values from pmat.toml
    assert!(hook_content.contains("30")); // max_cyclomatic_complexity
    assert!(hook_content.contains("25")); // max_cognitive_complexity
    assert!(hook_content.contains("80")); // min_test_coverage
    assert!(hook_content.contains("PMAT-[0-9]{4}")); // task_id_pattern

    // Should contain quality gate enforcement
    assert!(hook_content.contains("pmat") || hook_content.contains("analyze"));
    assert!(hook_content.contains("complexity"));

    Ok(())
}

// =============================================================================
// PROPERTY-BASED TESTS
// =============================================================================

#[tokio::test]
async fn test_hooks_idempotent_install() -> Result<()> {
    // Property: Installing hooks multiple times should be idempotent

    // ARRANGE
    let fixture = HooksTestFixture::new()?;
    let hooks_cmd = HooksCommand::new(fixture.hooks_dir().clone(), fixture.config_path().clone());

    // ACT
    let result1 = hooks_cmd.install(false, true).await?;
    let result2 = hooks_cmd.install(false, true).await?;
    let result3 = hooks_cmd.install(false, true).await?;

    // ASSERT
    assert_eq!(result1.success, true);
    assert_eq!(result2.success, true);
    assert_eq!(result3.success, true);

    // Hook content should be identical
    let hook_content = std::fs::read_to_string(&fixture.pre_commit_hook())?;
    assert!(hook_content.contains("PMAT"));

    Ok(())
}

// =============================================================================
// INTEGRATION TESTS
// =============================================================================

#[tokio::test]
async fn test_hooks_integration_with_config() -> Result<()> {
    // Integration test: hooks should reflect config changes

    // ARRANGE
    let fixture = HooksTestFixture::new()?;
    let hooks_cmd = HooksCommand::new(fixture.hooks_dir().clone(), fixture.config_path().clone());

    // Install with initial config
    let _ = hooks_cmd.install(false, true).await?;
    let initial_content = std::fs::read_to_string(&fixture.pre_commit_hook())?;

    // Change config
    let updated_config = r#"
[hooks]
enabled = true

[hooks.quality_gates]
max_cyclomatic_complexity = 35
max_cognitive_complexity = 30
    "#;
    std::fs::write(&fixture.config_path, updated_config)?;

    // ACT
    let _ = hooks_cmd.refresh().await?;
    let updated_content = std::fs::read_to_string(&fixture.pre_commit_hook())?;

    // ASSERT
    assert_ne!(initial_content, updated_content);
    assert!(updated_content.contains("35")); // New max_cyclomatic_complexity
    assert!(updated_content.contains("30")); // New max_cognitive_complexity

    Ok(())
}

#[tokio::test]
async fn test_hooks_performance_requirements() -> Result<()> {
    // Performance test: hook installation should be <5 seconds

    // ARRANGE
    let fixture = HooksTestFixture::new()?;
    let hooks_cmd = HooksCommand::new(fixture.hooks_dir().clone(), fixture.config_path().clone());

    // ACT
    let start = std::time::Instant::now();
    let _ = hooks_cmd.install(false, true).await?;
    let elapsed = start.elapsed();

    // ASSERT
    assert!(
        elapsed.as_secs() < 5,
        "Hook installation took {}s (should be <5s)",
        elapsed.as_secs()
    );

    Ok(())
}
