//! Config Command Handlers for CommandDispatcher
//!
//! Extracted from command_dispatcher mod.rs for file health compliance (CB-040).
//! Contains configuration command execution.
#![cfg_attr(coverage_nightly, coverage(off))]

use super::CommandDispatcher;
use crate::cli::handlers;
use std::path::PathBuf;

impl CommandDispatcher {
    /// Execute configuration command
    pub(crate) async fn execute_config_command(
        show: bool,
        edit: bool,
        validate: bool,
        reset: bool,
        section: Option<String>,
        set: Option<Vec<String>>,
        config_path: Option<PathBuf>,
    ) -> anyhow::Result<()> {
        handlers::handle_configuration(
            show,
            edit,
            validate,
            reset,
            section,
            set.unwrap_or_default(),
            config_path,
        )
        .await
    }
}
