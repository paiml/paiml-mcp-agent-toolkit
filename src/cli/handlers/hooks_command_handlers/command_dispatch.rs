//! Command dispatch and handler functions for hooks subcommands

#![cfg_attr(coverage_nightly, coverage(off))]

use super::hooks_command::HooksCommand;
use super::tdg_hooks::install_tdg_hooks_wrapper;
use super::types::{HookStatus, HookVerificationResult};
use crate::cli::colors as c;
use crate::cli::commands::HooksCommands;
use anyhow::Result;

/// Handle hooks subcommand
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
            stack,
            update,
        } => {
            if *stack {
                return crate::cli::handlers::hooks_stack_handler::handle_hooks_install_stack(
                    *update,
                )
                .await;
            }
            handle_install(&hooks_cmd, *force, *backup, *interactive, *tdg_enforcement).await
        }
        HooksCommands::Uninstall { restore_backup } => {
            handle_uninstall(&hooks_cmd, *restore_backup).await
        }
        HooksCommands::Status { stack, format } => {
            if *stack {
                return crate::cli::handlers::hooks_stack_handler::handle_hooks_status_stack(
                    format,
                )
                .await;
            }
            handle_status(&hooks_cmd).await
        }
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
    debug_assert!(true, "contract: handle_install");
    // Handle TDG enforcement installation (Sprint 66 Phase 3)
    if tdg_enforcement {
        println!(
            "{}",
            c::label("Installing PMAT hooks with TDG enforcement...")
        );
        return install_tdg_hooks_wrapper().await;
    }

    println!("{}", c::label("Installing pre-commit hooks..."));
    if interactive {
        println!("  {}", c::dim("Interactive mode enabled"));
    }
    if force {
        println!("  {}", c::dim("Force installation enabled"));
    }
    // Don't print backup message here - only print after actual backup happens

    let result = hooks_cmd.install(force, backup, interactive).await?;

    if result.success {
        if result.backup_created {
            println!(
                "  {} Backup created: {}",
                c::pass(""),
                c::path(".git/hooks/pre-commit.pmat-backup")
            );
        }
        println!("{}", c::pass(&result.message));
    } else {
        println!("{}", c::fail(&result.message));
        return Err(anyhow::anyhow!(result.message));
    }

    Ok(())
}

/// Handle hooks uninstall command
async fn handle_uninstall(hooks_cmd: &HooksCommand, restore_backup: bool) -> Result<()> {
    debug_assert!(true, "contract: handle_uninstall");
    println!("{}", c::label("Uninstalling pre-commit hooks..."));
    if restore_backup {
        println!("  {}", c::dim("Restoring backup enabled"));
    }

    let result = hooks_cmd.uninstall(restore_backup).await?;

    if result.success {
        if result.backup_restored {
            println!("  {}", c::pass("Backup restored"));
        }
        println!("{}", c::pass(&result.message));
    } else {
        println!("{}", c::fail(&result.message));
        return Err(anyhow::anyhow!(result.message));
    }

    Ok(())
}

/// Handle hooks status command
async fn handle_status(hooks_cmd: &HooksCommand) -> Result<()> {
    debug_assert!(true, "contract: handle_status");
    let status = hooks_cmd.status().await?;

    println!("{}", c::header("Pre-commit Hook Status:"));
    println!(
        "  {}: {}",
        c::dim("Installed"),
        if status.installed {
            c::pass("Yes")
        } else {
            c::fail("No")
        }
    );

    if status.installed {
        print_installed_status(&status);
    }

    Ok(())
}

/// Print detailed status for installed hook
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn print_installed_status(status: &HookStatus) {
    println!(
        "  {}: {}",
        c::dim("PMAT-managed"),
        if status.is_pmat_managed {
            c::pass("Yes")
        } else {
            c::fail("No")
        }
    );
    println!(
        "  {}: {}",
        c::dim("Config up-to-date"),
        if status.config_up_to_date {
            c::pass("Yes")
        } else {
            c::warn("No")
        }
    );

    if let Some(last_updated) = &status.last_updated {
        println!("  {}: {last_updated}", c::dim("Last updated"));
    }

    if let Some(preview) = &status.hook_content_preview {
        println!("\n  {}:", c::dim("Hook preview"));
        for line in preview.lines() {
            println!("    {}", c::dim(line));
        }
    }
}

/// Handle hooks verify command
async fn handle_verify(hooks_cmd: &HooksCommand, fix: bool) -> Result<()> {
    debug_assert!(true, "contract: handle_verify");
    println!("{}", c::label("Verifying pre-commit hooks..."));

    if fix {
        println!("  {}", c::dim("Auto-fix enabled"));
    }

    let result = hooks_cmd.verify(fix).await?;

    print_verification_issues(&result);
    print_verification_fixes(&result);

    if result.is_valid {
        println!("{}", c::pass("Pre-commit hooks verified successfully"));
    } else {
        println!("{}", c::fail("Pre-commit hooks verification failed"));
        if !fix {
            println!(
                "   {}",
                c::dim("Run with --fix to attempt automatic repairs")
            );
        }
        return Err(anyhow::anyhow!("Hook verification failed"));
    }

    Ok(())
}

/// Print verification issues
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn print_verification_issues(result: &HookVerificationResult) {
    if !result.issues.is_empty() {
        println!("  {}:", c::subheader("Issues found"));
        for issue in &result.issues {
            println!("    {}", c::warn(issue));
        }
    }
}

/// Print verification fixes applied
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn print_verification_fixes(result: &HookVerificationResult) {
    if !result.fixes_applied.is_empty() {
        println!("  {}:", c::subheader("Fixes applied"));
        for fix_msg in &result.fixes_applied {
            println!("    {}", c::pass(fix_msg));
        }
    }
}

/// Handle hooks refresh command
async fn handle_refresh(hooks_cmd: &HooksCommand) -> Result<()> {
    debug_assert!(true, "contract: handle_refresh");
    println!(
        "{}",
        c::label("Refreshing pre-commit hooks from configuration...")
    );

    let result = hooks_cmd.refresh().await?;

    if result.success {
        if result.config_changes_detected {
            println!("  {}", c::pass("Configuration changes detected"));
        }
        if result.hook_updated {
            println!("  {}", c::pass("Hook updated with new configuration"));
        }
        println!("{}", c::pass(&result.message));
    } else {
        println!("{}", c::fail(&result.message));
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
    debug_assert!(true, "contract: handle_run");
    let start_time = std::time::Instant::now();

    // O(1) cache check if enabled
    if use_cache {
        let project_root = std::env::current_dir()?;
        let cache_manager = crate::tdg::hooks_cache::HooksCacheManager::new(&project_root);

        // Initialize cache if it doesn't exist
        if !project_root.join(".pmat/hooks-cache").exists() {
            let _ = cache_manager.init();
        }

        match cache_manager.check() {
            Ok(crate::tdg::hooks_cache::CacheCheckResult::Hit { result, cached_at }) => {
                let elapsed = start_time.elapsed();

                if verbose {
                    println!("{} O(1) Cache HIT - Skipping full analysis", c::pass(""));
                    println!("   {}: {:?}", c::dim("Cached result"), result);
                    println!(
                        "   {}: {}",
                        c::dim("Cached at"),
                        cached_at.format("%Y-%m-%d %H:%M:%S UTC")
                    );
                    println!(
                        "   {}: {:.2}ms",
                        c::dim("Check time"),
                        elapsed.as_secs_f64() * 1000.0
                    );
                }

                // Record the cache hit
                let _ = cache_manager.record_run(true, elapsed.as_millis() as u64);

                match result {
                    crate::tdg::hooks_cache::CacheResult::Pass => {
                        println!(
                            "{} {}",
                            c::pass("All quality gates passed"),
                            c::dim("(cached)")
                        );
                        return Ok(());
                    }
                    crate::tdg::hooks_cache::CacheResult::Fail => {
                        println!("{} {}", c::fail("Quality gates failed"), c::dim("(cached)"));
                        return Err(anyhow::anyhow!("Pre-commit checks failed (cached)"));
                    }
                    crate::tdg::hooks_cache::CacheResult::Warn => {
                        println!(
                            "{} {}",
                            c::warn("Quality gates passed with warnings"),
                            c::dim("(cached)")
                        );
                        return Ok(());
                    }
                }
            }
            Ok(crate::tdg::hooks_cache::CacheCheckResult::Miss { reason }) => {
                if verbose {
                    println!("{} Cache MISS: {}", c::dim(""), reason);
                    println!("   {}", c::dim("Running full analysis..."));
                }
            }
            Ok(crate::tdg::hooks_cache::CacheCheckResult::Partial { .. }) => {
                if verbose {
                    println!(
                        "{}",
                        c::warn("Partial cache hit - running remaining gates...")
                    );
                }
            }
            Err(e) => {
                if verbose {
                    println!("{}", c::warn(&format!("Cache check failed: {}", e)));
                    println!("   {}", c::dim("Running full analysis..."));
                }
            }
        }
    }

    // Run full hooks
    let result = hooks_cmd.run(all_files, verbose).await?;
    let elapsed = start_time.elapsed();

    if verbose {
        println!();
        println!("{}", c::subheader("Results:"));
        println!(
            "  {}: {}",
            c::dim("Checks passed"),
            c::number(&result.checks_passed.to_string())
        );
        println!(
            "  {}: {}",
            c::dim("Checks failed"),
            if result.checks_failed > 0 {
                format!("{}{}{}", c::RED, result.checks_failed, c::RESET)
            } else {
                result.checks_failed.to_string()
            }
        );
        println!(
            "  {}: {:.2}ms",
            c::dim("Duration"),
            elapsed.as_secs_f64() * 1000.0
        );
        println!();
        println!("{}:", c::dim("Output"));
        println!("{}", result.output);
    } else {
        // Non-verbose: just print output
        println!("{}", result.output);
    }

    // Update cache if enabled
    if use_cache {
        let project_root = std::env::current_dir()?;
        let cache_manager = crate::tdg::hooks_cache::HooksCacheManager::new(&project_root);

        let cache_result = if result.success {
            crate::tdg::hooks_cache::CacheResult::Pass
        } else {
            crate::tdg::hooks_cache::CacheResult::Fail
        };

        // Update cache with results
        if let Err(e) = cache_manager.update(cache_result, std::collections::HashMap::new()) {
            if verbose {
                println!("{}", c::warn(&format!("Failed to update cache: {}", e)));
            }
        }

        // Record the cache miss run
        let _ = cache_manager.record_run(false, elapsed.as_millis() as u64);
    }

    if result.success {
        println!();
        println!("{}", c::pass("All pre-commit checks passed"));
        Ok(())
    } else {
        println!();
        println!("{}", c::fail("Pre-commit checks failed"));
        Err(anyhow::anyhow!("Pre-commit checks failed"))
    }
}
