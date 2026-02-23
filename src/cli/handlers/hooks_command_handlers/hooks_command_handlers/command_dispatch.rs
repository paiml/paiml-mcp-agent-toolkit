//! Command dispatch and handler functions for hooks subcommands

#![cfg_attr(coverage_nightly, coverage(off))]

use super::hooks_command::HooksCommand;
use super::tdg_hooks::install_tdg_hooks_wrapper;
use super::types::{HookStatus, HookVerificationResult};
use crate::cli::commands::HooksCommands;
use anyhow::Result;

/// Handle hooks subcommand
pub async fn handle_hooks_command(cmd: &HooksCommands) -> Result<()> {
    let hooks_cmd = HooksCommand::for_current_repo()?;

    match cmd {
        HooksCommands::Init {
            interactive,
            force,
            backup,
            tdg_enforcement,
        } => handle_install(&hooks_cmd, *force, *backup, *interactive, *tdg_enforcement).await,
        HooksCommands::Install {
            interactive,
            force,
            backup,
            tdg_enforcement,
        } => handle_install(&hooks_cmd, *force, *backup, *interactive, *tdg_enforcement).await,
        HooksCommands::Uninstall { restore_backup } => {
            handle_uninstall(&hooks_cmd, *restore_backup).await
        }
        HooksCommands::Status => handle_status(&hooks_cmd).await,
        HooksCommands::Verify { fix } => handle_verify(&hooks_cmd, *fix).await,
        HooksCommands::Refresh => handle_refresh(&hooks_cmd).await,
        HooksCommands::Run {
            all_files,
            verbose,
            cache,
        } => handle_run(&hooks_cmd, *all_files, *verbose, *cache).await,
        HooksCommands::Cache { action } => super::cache_handlers::handle_cache(action).await,
    }
}

/// Handle hooks install command
async fn handle_install(
    hooks_cmd: &HooksCommand,
    force: bool,
    backup: bool,
    interactive: bool,
    tdg_enforcement: bool,
) -> Result<()> {
    // Handle TDG enforcement installation (Sprint 66 Phase 3)
    if tdg_enforcement {
        println!("🔧 Installing PMAT hooks with TDG enforcement...");
        return install_tdg_hooks_wrapper().await;
    }

    println!("🔧 Installing pre-commit hooks...");
    if interactive {
        println!("  Interactive mode enabled");
    }
    if force {
        println!("  Force installation enabled");
    }
    // Don't print backup message here - only print after actual backup happens

    let result = hooks_cmd.install(force, backup, interactive).await?;

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
        println!("  Last updated: {last_updated}");
    }

    if let Some(preview) = &status.hook_content_preview {
        println!("\n  Hook preview:");
        for line in preview.lines() {
            println!("    {line}");
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
            println!("    ⚠️ {issue}");
        }
    }
}

/// Print verification fixes applied
fn print_verification_fixes(result: &HookVerificationResult) {
    if !result.fixes_applied.is_empty() {
        println!("  Fixes applied:");
        for fix_msg in &result.fixes_applied {
            println!("    🔧 {fix_msg}");
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

/// Handle hooks run command with O(1) cache check
async fn handle_run(
    hooks_cmd: &HooksCommand,
    all_files: bool,
    verbose: bool,
    use_cache: bool,
) -> Result<()> {
    let start_time = std::time::Instant::now();

    // O(1) cache check if enabled
    if use_cache {
        let project_root = std::env::current_dir()?;
        let cache_manager =
            crate::tdg::hooks_cache::HooksCacheManager::new(&project_root);

        // Initialize cache if it doesn't exist
        if !project_root.join(".pmat/hooks-cache").exists() {
            let _ = cache_manager.init();
        }

        match cache_manager.check() {
            Ok(crate::tdg::hooks_cache::CacheCheckResult::Hit { result, cached_at }) => {
                let elapsed = start_time.elapsed();

                if verbose {
                    println!("🎯 O(1) Cache HIT - Skipping full analysis");
                    println!("   Cached result: {:?}", result);
                    println!(
                        "   Cached at: {}",
                        cached_at.format("%Y-%m-%d %H:%M:%S UTC")
                    );
                    println!("   Check time: {:.2}ms", elapsed.as_secs_f64() * 1000.0);
                }

                // Record the cache hit
                let _ = cache_manager.record_run(true, elapsed.as_millis() as u64);

                match result {
                    crate::tdg::hooks_cache::CacheResult::Pass => {
                        println!("✅ All quality gates passed (cached)");
                        return Ok(());
                    }
                    crate::tdg::hooks_cache::CacheResult::Fail => {
                        println!("❌ Quality gates failed (cached)");
                        return Err(anyhow::anyhow!("Pre-commit checks failed (cached)"));
                    }
                    crate::tdg::hooks_cache::CacheResult::Warn => {
                        println!("⚠️  Quality gates passed with warnings (cached)");
                        return Ok(());
                    }
                }
            }
            Ok(crate::tdg::hooks_cache::CacheCheckResult::Miss { reason }) => {
                if verbose {
                    println!("📝 Cache MISS: {}", reason);
                    println!("   Running full analysis...");
                }
            }
            Ok(crate::tdg::hooks_cache::CacheCheckResult::Partial { .. }) => {
                if verbose {
                    println!("⚡ Partial cache hit - running remaining gates...");
                }
            }
            Err(e) => {
                if verbose {
                    println!("⚠️  Cache check failed: {}", e);
                    println!("   Running full analysis...");
                }
            }
        }
    }

    // Run full hooks
    let result = hooks_cmd.run(all_files, verbose).await?;
    let elapsed = start_time.elapsed();

    if verbose {
        println!("\n📊 Results:");
        println!("  Checks passed: {}", result.checks_passed);
        println!("  Checks failed: {}", result.checks_failed);
        println!("  Duration: {:.2}ms", elapsed.as_secs_f64() * 1000.0);
        println!("\nOutput:");
        println!("{}", result.output);
    } else {
        // Non-verbose: just print output
        println!("{}", result.output);
    }

    // Update cache if enabled
    if use_cache {
        let project_root = std::env::current_dir()?;
        let cache_manager =
            crate::tdg::hooks_cache::HooksCacheManager::new(&project_root);

        let cache_result = if result.success {
            crate::tdg::hooks_cache::CacheResult::Pass
        } else {
            crate::tdg::hooks_cache::CacheResult::Fail
        };

        // Update cache with results
        if let Err(e) = cache_manager.update(cache_result, std::collections::HashMap::new()) {
            if verbose {
                println!("⚠️  Failed to update cache: {}", e);
            }
        }

        // Record the cache miss run
        let _ = cache_manager.record_run(false, elapsed.as_millis() as u64);
    }

    if result.success {
        println!("\n✅ All pre-commit checks passed");
        Ok(())
    } else {
        println!("\n❌ Pre-commit checks failed");
        Err(anyhow::anyhow!("Pre-commit checks failed"))
    }
}
