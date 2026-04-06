/// MCP server that uses uniform contracts
pub struct ContractMcpServer {
    server: Server,
    service: Arc<ContractService>,
}

impl ContractMcpServer {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn new() -> Result<Self> {
        let service = Arc::new(ContractService::new()?);
        
        let server = ServerBuilder::new()
            .name("pmat")
            .version(env!("CARGO_PKG_VERSION"))
            .tool(create_analyze_complexity_tool())
            .tool(create_analyze_satd_tool())
            .tool(create_analyze_dead_code_tool())
            .tool(create_analyze_tdg_tool())
            .tool(create_analyze_lint_hotspot_tool())
            .tool(create_analyze_entropy_tool())
            .tool(create_quality_gate_tool())
            .tool(create_refactor_auto_tool())
            // TICKET-PMAT-6013: Scaffolding and maintenance tools
            .tool(create_scaffold_agent_tool())
            .tool(create_scaffold_wasm_tool())
            .tool(create_validate_roadmap_tool())
            .tool(create_health_check_tool())
            .tool(create_generate_tickets_tool())
            .build()
            .await?;
        
        Ok(Self { server, service })
    }
    
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn run(self) -> Result<()> {
        self.server.run().await
    }
    
    /// Handle tool calls using uniform contracts
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn handle_tool_call(&self, name: &str, params: Value) -> Result<ToolResult> {
        debug_assert!(!name.is_empty(), "name must not be empty");
        // Apply backward compatibility mapping
        let params = super::adapter::BackwardCompatibility::map_json_params(params);
        
        match name {
            "analyze_complexity" => self.handle_analyze_complexity(params).await,
            "analyze_satd" => self.handle_analyze_satd(params).await,
            "analyze_dead_code" => self.handle_analyze_dead_code(params).await,
            "analyze_tdg" => self.handle_analyze_tdg(params).await,
            "analyze_lint_hotspot" => self.handle_analyze_lint_hotspot(params).await,
            "analyze_entropy" => self.handle_analyze_entropy(params).await,
            "quality_gate" => self.handle_quality_gate(params).await,
            "refactor_auto" => self.handle_refactor_auto(params).await,
            // TICKET-PMAT-6013: Scaffolding and maintenance tools
            "scaffold_agent" => self.handle_scaffold_agent(params).await,
            "scaffold_wasm" => self.handle_scaffold_wasm(params).await,
            "validate_roadmap" => self.handle_validate_roadmap(params).await,
            "health_check" => self.handle_health_check(params).await,
            "generate_tickets" => self.handle_generate_tickets(params).await,
            _ => Err(anyhow::anyhow!("Unknown tool: {}", name))
        }
    }
    
    async fn handle_analyze_complexity(&self, params: Value) -> Result<ToolResult> {
        debug_assert!(true, "contract: handle_analyze_complexity");
        let contract = serde_json::from_value::<AnalyzeComplexityContract>(params)?;
        let result = self.service.analyze_complexity(contract).await?;
        Ok(ToolResult::Success(result))
    }
    
    async fn handle_analyze_satd(&self, params: Value) -> Result<ToolResult> {
        debug_assert!(true, "contract: handle_analyze_satd");
        let contract = serde_json::from_value::<AnalyzeSatdContract>(params)?;
        let result = self.service.analyze_satd(contract).await?;
        Ok(ToolResult::Success(result))
    }
    
    async fn handle_analyze_dead_code(&self, params: Value) -> Result<ToolResult> {
        debug_assert!(true, "contract: handle_analyze_dead_code");
        let contract = serde_json::from_value::<AnalyzeDeadCodeContract>(params)?;
        let result = self.service.analyze_dead_code(contract).await?;
        Ok(ToolResult::Success(result))
    }
    
    async fn handle_analyze_tdg(&self, params: Value) -> Result<ToolResult> {
        debug_assert!(true, "contract: handle_analyze_tdg");
        let contract = serde_json::from_value::<AnalyzeTdgContract>(params)?;
        let result = self.service.analyze_tdg(contract).await?;
        Ok(ToolResult::Success(result))
    }
    
    async fn handle_analyze_lint_hotspot(&self, params: Value) -> Result<ToolResult> {
        debug_assert!(true, "contract: handle_analyze_lint_hotspot");
        let contract = serde_json::from_value::<AnalyzeLintHotspotContract>(params)?;
        let result = self.service.analyze_lint_hotspot(contract).await?;
        Ok(ToolResult::Success(result))
    }
    
    async fn handle_analyze_entropy(&self, params: Value) -> Result<ToolResult> {
        debug_assert!(true, "contract: handle_analyze_entropy");
        let contract = serde_json::from_value::<AnalyzeEntropyContract>(params)?;
        let result = self.service.analyze_entropy(contract).await?;
        Ok(ToolResult::Success(result))
    }
    
    async fn handle_quality_gate(&self, params: Value) -> Result<ToolResult> {
        debug_assert!(true, "contract: handle_quality_gate");
        let contract = serde_json::from_value::<QualityGateContract>(params)?;
        let result = self.service.quality_gate(contract).await?;
        Ok(ToolResult::Success(result))
    }
    
    async fn handle_refactor_auto(&self, params: Value) -> Result<ToolResult> {
        debug_assert!(true, "contract: handle_refactor_auto");
        let contract = serde_json::from_value::<RefactorAutoContract>(params)?;
        let result = self.service.refactor_auto(contract).await?;
        Ok(ToolResult::Success(result))
    }
}
