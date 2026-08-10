#![cfg_attr(coverage_nightly, coverage(off))]
//! Extended dispatch: TDG, validation, semantic search, and feature-gated commands

use super::super::CommandExecutor;
use crate::cli::Commands;
use anyhow::Result;

impl CommandExecutor {
    /// Execute TDG, validation, semantic, and feature-gated commands
    pub(super) async fn execute_ext_advanced(&self, command: Commands) -> Result<()> {
        match command {
            Commands::Tdg {
                path,
                command,
                format,
                config,
                quiet,
                include_components,
                min_grade,
                output,
                with_git_context,
                explain,
                threshold,
                baseline,
                ml: _, // GH-97: ML flag (not yet implemented in handler)
                viz,
                viz_theme,
            } => {
                let tdg_config = crate::cli::handlers::tdg_handlers::TdgCommandConfig {
                    path,
                    command,
                    format,
                    config,
                    quiet,
                    include_components,
                    min_grade,
                    output,
                    with_git_context,
                    explain,
                    threshold,
                    baseline,
                    viz,
                    viz_theme,
                };
                crate::cli::handlers::handle_tdg_command(tdg_config).await
            }
            Commands::ValidateDocs(cmd) => {
                let exit_code = cmd.execute().await?;
                if exit_code != std::process::ExitCode::SUCCESS {
                    std::process::exit(1);
                }
                Ok(())
            }
            Commands::ValidateReadme(cmd) => {
                // Sprint 38: Hallucination detection (synchronous)
                let exit_code = cmd.execute()?;
                if exit_code != std::process::ExitCode::SUCCESS {
                    std::process::exit(1);
                }
                Ok(())
            }
            Commands::RedTeam(cmd) => {
                // Red Team Mode: Commit hallucination detection
                let exit_code = cmd.execute()?;
                if exit_code != std::process::ExitCode::SUCCESS {
                    std::process::exit(1);
                }
                Ok(())
            }
            Commands::Org(_org_cmd) => {
                #[cfg(feature = "org-intelligence")]
                {
                    crate::cli::handlers::handle_org_command(_org_cmd).await
                }
                #[cfg(not(feature = "org-intelligence"))]
                {
                    // See command_routing.rs: `org analyze` no longer exists in
                    // any build, so this arm must not promise a rebuild will
                    // bring it back.
                    crate::cli::handlers::org_handlers::org_not_enabled_error(&_org_cmd)
                }
            }
            Commands::Prompt(prompt_cmd) => {
                crate::cli::handlers::handle_prompt_command(prompt_cmd).await
            }
            Commands::Hooks(hooks_cmd) => {
                crate::cli::handlers::handle_hooks_command(&hooks_cmd).await
            }
            // Semantic search commands (PMAT-SEARCH-011)
            Commands::Embed(embed_cmd) => {
                crate::cli::command_dispatcher::CommandDispatcher::execute_embed_command(embed_cmd)
                    .await
            }
            Commands::Semantic(semantic_cmd) => {
                crate::cli::command_dispatcher::CommandDispatcher::execute_semantic_command(
                    semantic_cmd,
                )
                .await
            }
            // Mutation testing command (Sprint 61)
            #[cfg(feature = "mutation-testing")]
            Commands::Mutate(args) => {
                crate::cli::handlers::mutate::handle(args, self.server.clone()).await
            }
            // Time-travel debugging commands (Sprint 74)
            Commands::Debug { command } => Self::execute_debug(command).await,
            // Commands handled by command_dispatcher.rs
            other => Self::forward_to_dispatcher(other),
        }
    }
}
