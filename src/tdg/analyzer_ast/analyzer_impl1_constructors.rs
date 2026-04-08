impl TdgAnalyzerAst {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Result<Self> {
        Ok(Self {
            config: TdgConfig::default(),
            storage: None,
            scheduler: None,
            adaptive_manager: None,
            resource_controller: None,
            git_context: None,
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// With config.
    pub fn with_config(config: TdgConfig) -> Result<Self> {
        Ok(Self {
            config,
            storage: None,
            scheduler: None,
            adaptive_manager: None,
            resource_controller: None,
            git_context: None,
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// With storage.
    pub fn with_storage(config: TdgConfig) -> Result<Self> {
        let storage = TieredStorageFactory::create_default()?;
        let scheduler = SchedulerFactory::create_balanced();
        let adaptive_manager = AdaptiveThresholdFactory::create_default();
        let resource_controller = ResourceControllerFactory::create_default();
        Ok(Self {
            config,
            storage: Some(storage),
            scheduler: Some(scheduler),
            adaptive_manager: Some(adaptive_manager),
            resource_controller: Some(resource_controller),
            git_context: None,
        })
    }

    /// Create analyzer with in-memory storage for testing (no file I/O conflicts)
    #[cfg(test)]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_in_memory_storage(config: TdgConfig) -> Self {
        let storage = TieredStorageFactory::create_in_memory();
        let scheduler = SchedulerFactory::create_balanced();
        let adaptive_manager = AdaptiveThresholdFactory::create_default();
        let resource_controller = ResourceControllerFactory::create_default();
        Self {
            config,
            storage: Some(storage),
            scheduler: Some(scheduler),
            adaptive_manager: Some(adaptive_manager),
            resource_controller: Some(resource_controller),
            git_context: None,
        }
    }

    /// Create analyzer with full resource management for production
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn with_full_resource_management(config: TdgConfig) -> Result<Self> {
        let storage = TieredStorageFactory::create_default()?;
        let scheduler = SchedulerFactory::create_background_optimized();
        let adaptive_manager = AdaptiveThresholdFactory::create_prod_optimized();
        let resource_controller = ResourceControllerFactory::create_prod_optimized();

        // Start resource monitoring
        resource_controller.start_monitoring().await?;

        Ok(Self {
            config,
            storage: Some(storage),
            scheduler: Some(scheduler),
            adaptive_manager: Some(adaptive_manager),
            resource_controller: Some(resource_controller),
            git_context: None,
        })
    }

    /// Sprint 65: Set git context for commit correlation
    /// This should be called before analyze_file() when --with-git-context flag is enabled
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn set_git_context(&mut self, git_context: Option<crate::models::git_context::GitContext>) {
        self.git_context = git_context;
    }

    /// Sprint 65: Get git context for output formatting
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_git_context(&self) -> Option<&crate::models::git_context::GitContext> {
        self.git_context.as_ref()
    }

    /// Sprint 65 Phase 3: Get storage reference for history queries
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn storage(&self) -> Option<&TieredStore> {
        self.storage.as_ref()
    }
}
