//! MCP tools for Deep WASM analysis
//!
//! Provides AI-powered WebAssembly pipeline inspection tools for MCP clients.

use super::*;
use crate::agents::registry::AgentRegistry;
use serde_json::json;
use std::sync::Arc;

#[cfg(feature = "deep-wasm")]
use crate::services::deep_wasm::{
    AnalysisFocus, DeepWasmAnalysisRequest, DeepWasmService, ReportGenerator, SourceLanguage,
};

/// Deep WASM analysis tool - comprehensive pipeline inspection
pub struct DeepWasmAnalyzeTool {
    _registry: Arc<AgentRegistry>,
}

impl DeepWasmAnalyzeTool {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }
}

/// Query source-to-WASM mappings
pub struct DeepWasmQueryMappingTool {
    _registry: Arc<AgentRegistry>,
}

impl DeepWasmQueryMappingTool {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }
}

/// Trace execution through pipeline layers
pub struct DeepWasmTraceExecutionTool {
    _registry: Arc<AgentRegistry>,
}

impl DeepWasmTraceExecutionTool {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }
}

/// Compare optimization levels
pub struct DeepWasmCompareOptimizationsTool {
    _registry: Arc<AgentRegistry>,
}

impl DeepWasmCompareOptimizationsTool {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }
}

/// Detect WASM-specific issues
pub struct DeepWasmDetectIssuesTool {
    _registry: Arc<AgentRegistry>,
}

impl DeepWasmDetectIssuesTool {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }
}

// --- Method implementations split into include files ---

include!("deep_wasm_tools_analyze.rs");
include!("deep_wasm_tools_query_mapping.rs");
include!("deep_wasm_tools_stub_tools.rs");
include!("deep_wasm_tools_tests.rs");
