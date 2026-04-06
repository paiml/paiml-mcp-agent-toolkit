// Deep context orchestrator factory - creation and configuration helpers

/// Factory for creating deep context orchestrator
pub struct DeepContextOrchestratorFactory;

impl DeepContextOrchestratorFactory {
    /// Create orchestrator with default configuration
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn create() -> Result<DeepContextOrchestrator> {
        let ast_engine = Arc::new(UnifiedAstEngine::new());
        let intelligence = Arc::new(CodeIntelligence::new());

        // Create cache manager with default configuration
        let cache_manager = Arc::new(UnifiedCacheManager::default());

        Ok(DeepContextOrchestrator::new(ast_engine, intelligence, cache_manager))
    }

    /// Create minimal orchestrator for testing
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn create_minimal() -> Result<DeepContextOrchestrator> {
        let ast_engine = Arc::new(UnifiedAstEngine::new());
        let intelligence = Arc::new(CodeIntelligence::new());

        // Create minimal cache manager with default config
        let cache_config = UnifiedCacheConfig::default();
        let cache_manager = Arc::new(UnifiedCacheManager::new(cache_config)?);

        Ok(DeepContextOrchestrator::new(ast_engine, intelligence, cache_manager))
    }
}
