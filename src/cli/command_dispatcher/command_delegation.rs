// Simple command delegation methods
//
// Each method delegates to a specialized handler module, reducing
// cognitive complexity in the main dispatch table.

impl CommandDispatcher {
    /// Execute analyze commands using handler pattern (reduces CC)
    pub async fn execute_analyze_command(analyze_cmd: AnalyzeCommands) -> anyhow::Result<()> {
        // Delegate to the modular analysis handlers
        super::handlers::route_analyze_command(analyze_cmd).await
    }

    /// Execute QDD commands using handler pattern (reduces CC)
    pub async fn execute_qdd_command(qdd_cmd: QddCommands) -> anyhow::Result<()> {
        // Delegate to the QDD handlers
        super::handlers::qdd_handlers::handle_qdd_command(qdd_cmd).await
    }

    /// Execute refactor commands using handler pattern (reduces CC)
    pub async fn execute_refactor_command(refactor_cmd: RefactorCommands) -> anyhow::Result<()> {
        // Delegate to the refactor handlers
        super::handlers::route_refactor_command(refactor_cmd).await
    }

    /// Execute memory management commands using handler pattern (reduces CC)
    pub async fn execute_memory_command(memory_cmd: MemoryCommand) -> anyhow::Result<()> {
        // Delegate to the memory handler
        super::handlers::handle_memory_command(&memory_cmd).await
    }

    /// Execute cache management commands using handler pattern (reduces CC)
    pub async fn execute_cache_command(cache_cmd: CacheCommand) -> anyhow::Result<()> {
        // Delegate to the cache handler
        super::handlers::handle_cache_command(&cache_cmd).await
    }
}
