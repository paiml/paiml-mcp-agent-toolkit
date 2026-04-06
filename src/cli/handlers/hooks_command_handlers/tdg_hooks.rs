//! TDG enforcement hooks installation (Sprint 66 Phase 3)

#![cfg_attr(coverage_nightly, coverage(off))]

use crate::tdg::TdgHooksConfig;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Atomic write: write to temp file then rename (CB-1334 fix).
/// Ensures hook file is never partial — old or new, never corrupt.
fn atomic_write_hook(hook_path: &Path, content: &str) -> Result<()> {
    let tmp_path = hook_path.with_extension("tmp");
    fs::write(&tmp_path, content).context("Failed to write temp hook file")?;

    // Set executable before rename so the file is ready immediately
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&tmp_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&tmp_path, perms)?;
    }

    fs::rename(&tmp_path, hook_path).context("Failed to rename temp hook to final path")?;
    Ok(())
}

/// Escape shell metacharacters in template substitution values (CB-1336 fix).
/// Prevents injection when config values like baseline_path contain shell metacharacters.
fn shell_escape(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '$' | '`' | '\\' | '"' | '!' | '(' | ')' | '{' | '}' | '|' | '&' | ';' | '<' | '>'
            | '\'' | '\n' => {
                escaped.push('\\');
                escaped.push(c);
            }
            _ => escaped.push(c),
        }
    }
    escaped
}

/// Wrapper function for TDG hooks installation
pub(crate) async fn install_tdg_hooks_wrapper() -> Result<()> {
    let project_root = std::env::current_dir()?;
    install_tdg_hooks(&project_root).await?;

    println!("✅ TDG enforcement hooks installed successfully");
    println!();
    println!("Hooks installed:");
    println!("  - .git/hooks/pre-commit (TDG quality checks)");
    println!("  - .git/hooks/post-commit (baseline auto-update)");
    println!();
    println!("Configuration: .pmat/tdg-rules.toml");

    Ok(())
}

/// Install TDG enforcement hooks
pub(crate) async fn install_tdg_hooks(project_root: &Path) -> Result<()> {
    let git_dir = project_root.join(".git");
    let hooks_dir = git_dir.join("hooks");

    // Verify .git directory exists
    if !git_dir.exists() {
        return Err(anyhow::anyhow!(
            "Not a git repository (no .git directory found)"
        ));
    }

    // Create hooks directory if it doesn't exist
    if !hooks_dir.exists() {
        fs::create_dir_all(&hooks_dir).context("Failed to create .git/hooks directory")?;
    }

    // Load or create TDG configuration
    let config = match TdgHooksConfig::load(project_root) {
        Ok(cfg) => cfg,
        Err(_) => {
            println!("📝 Creating default TDG configuration...");
            TdgHooksConfig::create_default(project_root)?;
            TdgHooksConfig::load(project_root)?
        }
    };

    // Install pre-commit hook
    install_tdg_pre_commit_hook(&hooks_dir, &config)?;

    // Install post-commit hook
    install_tdg_post_commit_hook(&hooks_dir, &config)?;

    Ok(())
}

/// Install pre-commit hook with TDG enforcement
pub(crate) fn install_tdg_pre_commit_hook(hooks_dir: &Path, config: &TdgHooksConfig) -> Result<()> {
    debug_assert!(
        hooks_dir.exists(),
        "hooks_dir must exist: {}",
        hooks_dir.display()
    );
    let hook_path = hooks_dir.join("pre-commit");

    // Read template
    let template = include_str!("../../../../templates/hooks/pre-commit-tdg.sh");

    // Substitute configuration values (shell-escaped to prevent injection per CB-1336)
    let hook_content = template
        .replace(
            "{{BASELINE_PATH}}",
            &shell_escape(&config.baseline.baseline_path),
        )
        .replace(
            "{{MIN_GRADE}}",
            &shell_escape(config.quality_gates.get_default_min_grade()),
        )
        .replace(
            "{{MAX_SCORE_DROP}}",
            &config.quality_gates.max_score_drop.to_string(),
        )
        .replace(
            "{{ALLOW_GRADE_DROP}}",
            &config.quality_gates.allow_grade_drop.to_string(),
        )
        .replace(
            "{{MODE}}",
            &shell_escape(&config.quality_gates.mode.to_string()),
        )
        .replace(
            "{{BLOCK_ON_REGRESSION}}",
            &config.quality_gates.block_on_regression.to_string(),
        )
        .replace(
            "{{BLOCK_ON_NEW_FILES}}",
            &config
                .quality_gates
                .block_on_new_files_below_threshold
                .to_string(),
        );

    // Atomic write: temp file + rename (CB-1334)
    atomic_write_hook(&hook_path, &hook_content)?;

    Ok(())
}

/// Install post-commit hook for baseline auto-update
pub(crate) fn install_tdg_post_commit_hook(
    hooks_dir: &Path,
    config: &TdgHooksConfig,
) -> Result<()> {
    debug_assert!(
        hooks_dir.exists(),
        "hooks_dir must exist: {}",
        hooks_dir.display()
    );
    let hook_path = hooks_dir.join("post-commit");

    // Read template
    let template = include_str!("../../../../templates/hooks/post-commit-tdg.sh");

    // Substitute configuration values (shell-escaped to prevent injection per CB-1336)
    let hook_content = template
        .replace(
            "{{BASELINE_PATH}}",
            &shell_escape(&config.baseline.baseline_path),
        )
        .replace(
            "{{AUTO_UPDATE}}",
            &config.baseline.auto_update_on_commit.to_string(),
        )
        .replace(
            "{{STORE_IN_GIT}}",
            &config.baseline.store_in_git.to_string(),
        );

    // Atomic write: temp file + rename (CB-1334)
    atomic_write_hook(&hook_path, &hook_content)?;

    Ok(())
}
