//! Core SimpleMcpHandler - struct, construction, and tool call dispatch

use crate::contracts::simple_service::SimpleContractService;
use crate::contracts::{
    AnalyzeComplexityContract, AnalyzeDeadCodeContract, AnalyzeEntropyContract,
    AnalyzeLintHotspotContract, AnalyzeSatdContract, AnalyzeTdgContract, QualityGateContract,
    RefactorAutoContract,
};
use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;

/// Simplified MCP handler using uniform contracts
pub struct SimpleMcpHandler {
    service: Arc<SimpleContractService>,
}

impl SimpleMcpHandler {
    pub fn new() -> Result<Self> {
        Ok(Self {
            service: Arc::new(SimpleContractService::new()?),
        })
    }

    /// Handle MCP tool call using uniform contracts
    pub async fn handle_tool_call(&self, name: &str, params: Value) -> Result<Value> {
        // Apply backward compatibility mapping
        let params = crate::contracts::adapter::BackwardCompatibility::map_json_params(params);

        match name {
            "analyze_complexity" => self.handle_analyze_complexity(params).await,
            "analyze_satd" => self.handle_analyze_satd(params).await,
            "analyze_dead_code" => self.handle_analyze_dead_code(params).await,
            "analyze_tdg" => self.handle_analyze_tdg(params).await,
            "analyze_lint_hotspot" => self.handle_analyze_lint_hotspot(params).await,
            "analyze_entropy" => self.handle_analyze_entropy(params).await,
            "quality_gate" => self.handle_quality_gate(params).await,
            "refactor_auto" => self.handle_refactor_auto(params).await,
            _ => Err(anyhow::anyhow!("Unknown tool: {name}")),
        }
    }

    async fn handle_analyze_complexity(&self, params: Value) -> Result<Value> {
        let contract = serde_json::from_value::<AnalyzeComplexityContract>(params)?;
        self.service.analyze_complexity(contract).await
    }

    async fn handle_analyze_satd(&self, params: Value) -> Result<Value> {
        let contract = serde_json::from_value::<AnalyzeSatdContract>(params)?;
        self.service.analyze_satd(contract).await
    }

    async fn handle_analyze_dead_code(&self, params: Value) -> Result<Value> {
        let contract = serde_json::from_value::<AnalyzeDeadCodeContract>(params)?;
        self.service.analyze_dead_code(contract).await
    }

    async fn handle_analyze_tdg(&self, params: Value) -> Result<Value> {
        let contract = serde_json::from_value::<AnalyzeTdgContract>(params)?;
        self.service.analyze_tdg(contract).await
    }

    async fn handle_analyze_lint_hotspot(&self, params: Value) -> Result<Value> {
        let contract = serde_json::from_value::<AnalyzeLintHotspotContract>(params)?;
        self.service.analyze_lint_hotspot(contract).await
    }

    async fn handle_quality_gate(&self, params: Value) -> Result<Value> {
        let contract = serde_json::from_value::<QualityGateContract>(params)?;
        self.service.quality_gate(contract).await
    }

    async fn handle_analyze_entropy(&self, params: Value) -> Result<Value> {
        let contract = serde_json::from_value::<AnalyzeEntropyContract>(params)?;
        self.service.analyze_entropy(contract).await
    }

    async fn handle_refactor_auto(&self, params: Value) -> Result<Value> {
        let contract = serde_json::from_value::<RefactorAutoContract>(params)?;
        self.service.refactor_auto(contract).await
    }

    /// Get tool definitions for MCP discovery
    #[must_use]
    pub fn get_tool_definitions(&self) -> Value {
        json!({
            "tools": [
                {
                    "name": "analyze_complexity",
                    "description": "Analyze code complexity metrics",
                    "parameters": self.get_complexity_schema()
                },
                {
                    "name": "analyze_satd",
                    "description": "Analyze Self-Admitted Technical Debt",
                    "parameters": self.get_satd_schema()
                },
                {
                    "name": "analyze_dead_code",
                    "description": "Analyze dead and unreachable code",
                    "parameters": self.get_dead_code_schema()
                },
                {
                    "name": "analyze_tdg",
                    "description": "Analyze Technical Debt Gradient",
                    "parameters": self.get_tdg_schema()
                },
                {
                    "name": "analyze_lint_hotspot",
                    "description": "Find lint hotspots",
                    "parameters": self.get_lint_hotspot_schema()
                },
                {
                    "name": "analyze_entropy",
                    "description": "Analyze actionable entropy patterns",
                    "parameters": self.get_entropy_schema()
                },
                {
                    "name": "quality_gate",
                    "description": "Run quality gate checks",
                    "parameters": self.get_quality_gate_schema()
                },
                {
                    "name": "refactor_auto",
                    "description": "Auto refactor code",
                    "parameters": self.get_refactor_schema()
                }
            ]
        })
    }
}
