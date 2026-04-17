//! Simple command delegation methods.
//!
//! Each method delegates to a specialized handler module, reducing
//! cognitive complexity in the main dispatch table.
#![cfg_attr(coverage_nightly, coverage(off))]

use super::CommandDispatcher;
use crate::cli::commands::QddCommands;
use crate::cli::handlers::cache::CacheCommand;
use crate::cli::handlers::memory::MemoryCommand;
use crate::cli::{AnalyzeCommands, RefactorCommands};

impl CommandDispatcher {
    /// Execute analyze commands using handler pattern (reduces CC)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn execute_analyze_command(analyze_cmd: AnalyzeCommands) -> anyhow::Result<()> {
        // Delegate to the modular analysis handlers
        crate::cli::handlers::route_analyze_command(analyze_cmd).await
    }

    /// Execute QDD commands using handler pattern (reduces CC)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn execute_qdd_command(qdd_cmd: QddCommands) -> anyhow::Result<()> {
        // Delegate to the QDD handlers
        crate::cli::handlers::qdd_handlers::handle_qdd_command(qdd_cmd).await
    }

    /// Execute refactor commands using handler pattern (reduces CC)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn execute_refactor_command(refactor_cmd: RefactorCommands) -> anyhow::Result<()> {
        // Delegate to the refactor handlers
        crate::cli::handlers::route_refactor_command(refactor_cmd).await
    }

    /// Execute memory management commands using handler pattern (reduces CC)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn execute_memory_command(memory_cmd: MemoryCommand) -> anyhow::Result<()> {
        // Delegate to the memory handler
        crate::cli::handlers::handle_memory_command(&memory_cmd).await
    }

    /// Execute cache management commands using handler pattern (reduces CC)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn execute_cache_command(cache_cmd: CacheCommand) -> anyhow::Result<()> {
        // Delegate to the cache handler
        crate::cli::handlers::handle_cache_command(&cache_cmd).await
    }
}
