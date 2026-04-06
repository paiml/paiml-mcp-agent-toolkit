// TDG handler implementations - included by tdg_handlers.rs
// NO `use` imports or `#!` inner attributes allowed here.

// === TdgSystemDiagnosticsTool ===

impl TdgSystemDiagnosticsTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self
    }
}

impl Default for TdgSystemDiagnosticsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for TdgSystemDiagnosticsTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug_assert!(true, "contract: handle");
        debug!("Handling tdg_system_diagnostics with args: {}", args);

        let params: TdgSystemDiagnosticsArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let result = tdg_system_diagnostics(params.detailed, params.components)
            .await
            .map_err(|e| Error::internal(format!("TDG diagnostics failed: {e}")))?;

        Ok(result)
    }
}

// === TdgStorageManagementTool ===

impl TdgStorageManagementTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self
    }
}

impl Default for TdgStorageManagementTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for TdgStorageManagementTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug_assert!(true, "contract: handle");
        debug!("Handling tdg_storage_management with args: {}", args);

        let params: TdgStorageManagementArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let result = tdg_storage_management(params.action, params.options)
            .await
            .map_err(|e| Error::internal(format!("TDG storage management failed: {e}")))?;

        Ok(result)
    }
}

// === TdgAnalyzeWithStorageTool ===

impl TdgAnalyzeWithStorageTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self
    }
}

impl Default for TdgAnalyzeWithStorageTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for TdgAnalyzeWithStorageTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug_assert!(true, "contract: handle");
        debug!("Handling tdg_analyze_with_storage with args: {}", args);

        let params: TdgAnalyzeWithStorageArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths: Vec<PathBuf> = params.paths.into_iter().map(PathBuf::from).collect();

        if paths.is_empty() {
            return Err(Error::validation(
                "At least one path must be specified".to_string(),
            ));
        }

        let result = tdg_analyze_with_storage(paths, params.storage_backend, params.priority)
            .await
            .map_err(|e| Error::internal(format!("TDG analysis failed: {e}")))?;

        Ok(result)
    }
}

// === TdgPerformanceMetricsTool ===

impl TdgPerformanceMetricsTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self
    }
}

impl Default for TdgPerformanceMetricsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for TdgPerformanceMetricsTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug_assert!(true, "contract: handle");
        debug!("Handling tdg_performance_metrics with args: {}", args);

        let _params: TdgPerformanceMetricsArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let result = tdg_performance_metrics()
            .await
            .map_err(|e| Error::internal(format!("TDG performance metrics failed: {e}")))?;

        Ok(result)
    }
}

// === TdgConfigureStorageTool ===

impl TdgConfigureStorageTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self
    }
}

impl Default for TdgConfigureStorageTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for TdgConfigureStorageTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug_assert!(true, "contract: handle");
        debug!("Handling tdg_configure_storage with args: {}", args);

        let params: TdgConfigureStorageArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let result = tdg_configure_storage(
            params.backend_type,
            params.path,
            params.cache_size_mb,
            params.compression,
        )
        .await
        .map_err(|e| Error::internal(format!("TDG storage configuration failed: {e}")))?;

        Ok(result)
    }
}

// === TdgHealthCheckTool ===

impl TdgHealthCheckTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self
    }
}

impl Default for TdgHealthCheckTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for TdgHealthCheckTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        debug_assert!(true, "contract: handle");
        debug!("Handling tdg_health_check with args: {}", args);

        let _params: TdgHealthCheckArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let result = tdg_health_check()
            .await
            .map_err(|e| Error::internal(format!("TDG health check failed: {e}")))?;

        Ok(result)
    }
}
