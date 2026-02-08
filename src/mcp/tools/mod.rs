#![cfg_attr(coverage_nightly, coverage(off))]
// MCP Tools Module
// PMAT-SEARCH-006: MCP Tools Integration
// PMAT-470: Agent Context Tools

pub mod agent_context_tools;
pub mod semantic_search_tools;

// Phase 4: Organizational Intelligence Plugin Integration
#[cfg(feature = "org-intelligence")]
pub mod oip_tools;

pub use agent_context_tools::{
    FindSimilarTool, GetFunctionTool, IndexManager, IndexStatsTool, QueryCodeTool,
};
pub use semantic_search_tools::{
    AnalyzeTopicsTool, ClusterCodeTool, FindSimilarCodeTool, McpTool, SemanticSearchTool,
};

#[cfg(feature = "org-intelligence")]
pub use oip_tools::{
    analyze_oip_summary, generate_defect_aware_prompt, generate_prevention_prompt,
    AnalyzeOipSummaryRequest, AnalyzeOipSummaryResponse, GenerateDefectAwarePromptRequest,
    GenerateDefectAwarePromptResponse, GeneratePreventionPromptRequest,
    GeneratePreventionPromptResponse,
};
