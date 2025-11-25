// MCP Tools Module
// PMAT-SEARCH-006: MCP Tools Integration

pub mod semantic_search_tools;

// Phase 4: Organizational Intelligence Plugin Integration
#[cfg(feature = "org-intelligence")]
pub mod oip_tools;

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
