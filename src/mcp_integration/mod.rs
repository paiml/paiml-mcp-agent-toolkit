#![cfg_attr(coverage_nightly, coverage(off))]
// MCP (Model Context Protocol) integration for agent system
pub mod ast_item_helpers;
#[cfg(feature = "deep-wasm")]
pub mod deep_wasm_tools;
pub mod hallucination_detection_tools;
#[cfg(feature = "mutation-testing")]
pub mod mutation_tools;
pub mod prompts;
pub mod resources;
pub mod server;
pub mod service_registry;
pub mod tdg_tools;
pub mod tools;
pub mod transport;

// JVM language support (Sprint 51)
#[cfg(feature = "java-ast")]
pub mod java_tools;
#[cfg(feature = "scala-ast")]
pub mod scala_tools;

// Cross-language analysis (Sprint 52)
pub mod polyglot_tools;

#[cfg(all(test, feature = "java-ast", feature = "scala-ast"))]
mod jvm_tools_integration_tests;
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tools_integration_tests;

// Semantic submodules
pub mod registry;
pub mod session;
pub mod types;

#[cfg(test)]
mod protocol_tests;

// Re-export all public items for backward compatibility
pub use registry::{
    ContentPart, McpPrompt, McpResource, McpTool, PromptArgument, PromptContent, PromptMessage,
    PromptMetadata, PromptRegistry, ResourceContent, ResourceContentType, ResourceRegistry,
    ResourceTemplate, ToolMetadata, ToolRegistry,
};
pub use session::{McpContext, McpSession, McpTransport};
pub use types::{
    error_codes, JsonRpcMessage, LoggingCapabilities, McpError, McpMessage, McpNotification,
    McpRequest, McpResponse, PromptsCapability, RequestId, ResourcesCapability, ServerCapabilities,
    ServerInfo, ToolsCapability, MCP_VERSION,
};
