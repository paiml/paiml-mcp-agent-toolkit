// Tool handler implementations for refactor MCP tools.
// Included from handlers.rs — shares parent module scope.

#[async_trait]
impl ToolHandler for RefactorStartTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> PmcpResult<Value> {
        debug_assert!(true, "contract: handle");
        debug!("Handling refactor.start with args: {}", args);

        let params: RefactorStartArgs = serde_json::from_value(args)
            .map_err(|e| PmcpError::validation(format!("Invalid arguments: {e}")))?;

        let targets: Vec<PathBuf> = params.targets.into_iter().map(PathBuf::from).collect();

        let config = params.config.unwrap_or_default();

        let mut manager = self.state_manager.lock().await;
        manager
            .start_session(targets, config)
            .map_err(|e| PmcpError::internal(format!("Failed to start session: {e}")))?;

        let state = manager
            .get_state()
            .map_err(|e| PmcpError::internal(format!("Failed to get state: {e}")))?;
        let session_id = manager.get_session_id().to_string();

        let state_value = serialize_state(state)
            .map_err(|e| PmcpError::internal(format!("Failed to serialize state: {e}")))?;

        Ok(serde_json::to_value(RefactorStartResult {
            session_id,
            state: state_value,
        })?)
    }
}

#[async_trait]
impl ToolHandler for RefactorNextIterationTool {
    async fn handle(&self, _args: Value, _extra: RequestHandlerExtra) -> PmcpResult<Value> {
        debug_assert!(true, "contract: handle");
        debug!("Handling refactor.nextIteration");

        let mut manager = self.state_manager.lock().await;
        manager
            .advance()
            .map_err(|e| PmcpError::internal(format!("Failed to advance: {e}")))?;

        let state = manager
            .get_state()
            .map_err(|e| PmcpError::internal(format!("Failed to get state: {e}")))?;

        serialize_state(state)
            .map_err(|e| PmcpError::internal(format!("Failed to serialize state: {e}")))
    }
}

#[async_trait]
impl ToolHandler for RefactorGetStateTool {
    async fn handle(&self, _args: Value, _extra: RequestHandlerExtra) -> PmcpResult<Value> {
        debug_assert!(true, "contract: handle");
        debug!("Handling refactor.getState");

        let manager = self.state_manager.lock().await;
        let state = manager
            .get_state()
            .map_err(|e| PmcpError::internal(format!("Failed to get state: {e}")))?;

        serialize_state(state)
            .map_err(|e| PmcpError::internal(format!("Failed to serialize state: {e}")))
    }
}

#[async_trait]
impl ToolHandler for RefactorStopTool {
    async fn handle(&self, _args: Value, _extra: RequestHandlerExtra) -> PmcpResult<Value> {
        debug_assert!(true, "contract: handle");
        debug!("Handling refactor.stop");

        let mut manager = self.state_manager.lock().await;
        manager
            .stop_session()
            .map_err(|e| PmcpError::internal(format!("Failed to stop session: {e}")))?;

        Ok(json!({
            "status": "stopped",
            "message": "Refactoring session stopped successfully"
        }))
    }
}
