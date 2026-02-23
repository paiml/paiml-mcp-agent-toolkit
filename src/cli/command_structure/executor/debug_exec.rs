#![cfg_attr(coverage_nightly, coverage(off))]
//! Debug subcommand execution

use super::super::CommandExecutor;
use anyhow::Result;

impl CommandExecutor {
    /// Execute debug subcommands
    pub(super) async fn execute_debug(command: crate::cli::commands::DebugCommands) -> Result<()> {
        use crate::cli::commands::DebugCommands;
        match command {
            DebugCommands::Serve {
                port,
                host,
                record_dir,
            } => {
                crate::cli::handlers::debug_handlers::handle_debug_serve(port, host, record_dir)
                    .await
            }
            DebugCommands::Replay {
                recording,
                position,
                interactive,
            } => {
                crate::cli::handlers::debug_handlers::handle_debug_replay(
                    recording,
                    position,
                    interactive,
                )
                .await
            }
        }
    }
}
