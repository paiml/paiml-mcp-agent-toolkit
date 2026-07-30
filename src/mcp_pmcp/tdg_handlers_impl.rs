// TDG handler implementations - included by tdg_handlers.rs
// NO `use` imports or `#!` inner attributes allowed here.

// === TdgSystemDiagnosticsTool ===

impl TdgSystemDiagnosticsTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
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
        debug!("Handling tdg_system_diagnostics with args: {}", args);

        let params: TdgSystemDiagnosticsArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let result = tdg_system_diagnostics(params.detailed, params.components)
            .await
            .map_err(|e| Error::internal(format!("TDG diagnostics failed: {e}")))?;

        Ok(result)
    }

    fn metadata(&self) -> Option<ToolInfo> {
        let schema = json!({
            "type": "object",
            "properties": {
                "detailed":   { "type": "boolean", "description": "Include detailed per-component diagnostics" },
                "components": {
                    "type": "array", "items": { "type": "string" },
                    "description": "Subset of components to inspect (empty = all)"
                }
            }
        });
        Some(build_tool_info(
            "tdg_system_diagnostics",
            "Run comprehensive diagnostics on the TDG (Technical Debt Grading) system.",
            schema,
        ))
    }
}

// === TdgStorageManagementTool ===

impl TdgStorageManagementTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
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
        debug!("Handling tdg_storage_management with args: {}", args);

        let params: TdgStorageManagementArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let result = tdg_storage_management(params.action, params.options)
            .await
            .map_err(|e| Error::internal(format!("TDG storage management failed: {e}")))?;

        Ok(result)
    }

    fn metadata(&self) -> Option<ToolInfo> {
        let schema = json!({
            "type": "object",
            "properties": {
                "action":  { "type": "string",  "description": "Storage action (compact, purge, stats, vacuum, etc.)" },
                "options": { "type": "object",  "description": "Freeform options object for the given action" }
            },
            "required": ["action"]
        });
        Some(build_tool_info(
            "tdg_storage_management",
            "Perform a storage management action (compact, purge, vacuum, etc.) on the TDG backend.",
            schema,
        ))
    }
}

// === TdgAnalyzeWithStorageTool ===

impl TdgAnalyzeWithStorageTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
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
        debug!("Handling tdg_analyze_with_storage with args: {}", args);

        let params: TdgAnalyzeWithStorageArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let paths = crate::mcp_pmcp::tool_schemas::resolve_existing_paths(params.paths)?;

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

    fn metadata(&self) -> Option<ToolInfo> {
        let extra = json!({
            "storage_backend": { "type": "string", "description": "Storage backend name (e.g. sqlite, in-memory)" },
            "priority":        { "type": "string", "description": "Analysis priority (low, normal, high)" }
        });
        Some(build_tool_info(
            "tdg_analyze_with_storage",
            "Analyze the given paths through the transactional TDG storage backend.",
            paths_object_schema(extra, vec!["paths"]),
        ))
    }
}

// === TdgPerformanceMetricsTool ===

impl TdgPerformanceMetricsTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
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
        debug!("Handling tdg_performance_metrics with args: {}", args);

        let _params: TdgPerformanceMetricsArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let result = tdg_performance_metrics()
            .await
            .map_err(|e| Error::internal(format!("TDG performance metrics failed: {e}")))?;

        Ok(result)
    }

    fn metadata(&self) -> Option<ToolInfo> {
        let schema = json!({
            "type": "object",
            "properties": {
                "include_history": { "type": "boolean", "description": "Include historical trend data" },
                "metrics": {
                    "type": "array", "items": { "type": "string" },
                    "description": "Subset of metric names to return (empty = all)"
                }
            }
        });
        Some(build_tool_info(
            "tdg_performance_metrics",
            "Return real-time TDG system performance metrics.",
            schema,
        ))
    }
}

// === TdgConfigureStorageTool ===

impl TdgConfigureStorageTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
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

    fn metadata(&self) -> Option<ToolInfo> {
        let schema = json!({
            "type": "object",
            "properties": {
                "backend_type":  { "type": "string",  "description": "Backend type identifier (sqlite, rocks, memory, etc.)" },
                "path":          { "type": "string",  "description": "Storage path" },
                "cache_size_mb": { "type": "integer", "description": "In-memory cache size in MB" },
                "compression":   { "type": "boolean", "description": "Enable on-disk compression" }
            },
            "required": ["backend_type"]
        });
        Some(build_tool_info(
            "tdg_configure_storage",
            "Configure and validate the TDG storage backend.",
            schema,
        ))
    }
}

// === TdgHealthCheckTool ===

impl TdgHealthCheckTool {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
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
        debug!("Handling tdg_health_check with args: {}", args);

        let _params: TdgHealthCheckArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {e}")))?;

        let result = tdg_health_check()
            .await
            .map_err(|e| Error::internal(format!("TDG health check failed: {e}")))?;

        Ok(result)
    }

    fn metadata(&self) -> Option<ToolInfo> {
        let schema = json!({
            "type": "object",
            "properties": {
                "include_recommendations": { "type": "boolean", "description": "Include remediation recommendations" },
                "check_storage":           { "type": "boolean", "description": "Run storage-backend health probe" },
                "check_performance":       { "type": "boolean", "description": "Run performance-budget health probe" }
            }
        });
        Some(build_tool_info(
            "tdg_health_check",
            "Run a comprehensive TDG-system health check (storage, performance, recommendations).",
            schema,
        ))
    }
}
