#![cfg_attr(coverage_nightly, coverage(off))]

pub mod agent_tools;
pub mod context_adapters;
pub mod semantic_adapters;
pub mod workflow_tools;

mod tests;

// Re-export all public types for backward compatibility
pub use agent_tools::{AnalyzeTool, TransformTool, ValidateTool};
pub use context_adapters::{
    FindSimilarToolAdapter, GetFunctionToolAdapter, IndexStatsToolAdapter, QueryCodeToolAdapter,
};
pub use semantic_adapters::{
    AnalyzeTopicsToolAdapter, ClusterCodeToolAdapter, FindSimilarCodeToolAdapter,
    SemanticSearchToolAdapter,
};
pub use workflow_tools::{OrchestrateTool, QualityGateTool};
