#![cfg_attr(coverage_nightly, coverage(off))]
// MCP Module
// Model Context Protocol integration

pub mod tools;

pub use tools::{
    AnalyzeTopicsTool, ClusterCodeTool, FindSimilarCodeTool, FindSimilarTool, GetFunctionTool,
    IndexManager, IndexStatsTool, McpTool, QueryCodeTool, SemanticSearchTool,
};
