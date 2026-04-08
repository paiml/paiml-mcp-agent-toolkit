//! Command Dispatcher - Reduces CLI complexity through handler pattern
//!
//! This module implements a dispatch table pattern to reduce cyclomatic complexity
//! in the CLI module by delegating command execution to specialized handlers.
#![cfg_attr(coverage_nightly, coverage(off))]

use super::commands::QddCommands;
use super::{AnalyzeCommands, Commands, RefactorCommands};
use crate::cli::handlers;
use crate::cli::handlers::cache::CacheCommand;
use crate::cli::handlers::memory::MemoryCommand;
use crate::stateless_server::StatelessTemplateServer;
use std::sync::Arc;

// Extracted modules for CB-040 file health compliance
mod config_commands;
#[cfg(feature = "demo")]
mod demo_commands;
mod metrics_commands;
mod quality_commands;
mod roadmap_commands;
mod scaffold_commands;
mod semantic_commands;
mod test_commands;
pub(crate) mod test_record;

// Work and spec handlers extracted for file health compliance (CB-040)
#[path = "command_dispatcher_work.rs"]
mod command_dispatcher_work;

// Contract query handlers extracted for CB-040 file health (command_routing.rs was F-grade)
pub(crate) mod contract_query_handlers;

// Scoring and infrastructure handlers extracted for cognitive complexity reduction
#[path = "command_dispatcher_scoring.rs"]
mod command_dispatcher_scoring;

/// Trait for command handlers to reduce complexity through delegation
#[allow(async_fn_in_trait)]
pub trait CommandHandler: Send + Sync {
    async fn execute(&self, server: Arc<StatelessTemplateServer>) -> anyhow::Result<()>;
}

/// Trait for analyze command handlers
#[allow(async_fn_in_trait)]
pub trait AnalyzeCommandHandler: Send + Sync {
    async fn execute(&self) -> anyhow::Result<()>;
}

/// Command dispatcher that reduces complexity by delegating to handlers
pub struct CommandDispatcher;

impl CommandDispatcher {
    /// Execute a command using the handler pattern (reduces CC from dispatch match)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn execute_command(
        command: Commands,
        server: Arc<StatelessTemplateServer>,
    ) -> anyhow::Result<()> {
        Self::route_command(command, server).await
    }
}

// --- Main command routing dispatch table ---
include!("command_routing.rs");

// --- Simple delegation methods (analyze, qdd, refactor, memory, cache) ---
include!("command_delegation.rs");

// --- Quality and analysis command routing ---
include!("quality_routing.rs");

// Tests extracted for file health compliance (CB-040)
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
