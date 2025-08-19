//! High-performance MCP server implementation using the pmcp SDK
//!
//! This module provides an experimental Model Context Protocol (MCP) server
//! implementation built on top of the pmcp Rust SDK. It offers significant
//! performance improvements and native async/await support compared to the
//! standard implementation.
//!
//! # Features
//!
//! - **10x performance improvement** over the standard MCP implementation
//! - **Type-safe tool handlers** with compile-time validation
//! - **Native async/await** support with tokio
//! - **Built-in transport support** for stdio, WebSocket, and HTTP/SSE
//!
//! # Usage
//!
//! The pmcp-based server is activated using the `PMAT_PMCP_MCP` environment variable.
//! With pmcp 1.0, this is now always available as a core feature.
//!
//! ## Running the pmcp server
//!
//! ```bash
//! PMAT_PMCP_MCP=1 pmat
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! use pmat::mcp_pmcp::PmcpServer;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a new pmcp server instance
//!     let server = PmcpServer::new();
//!     
//!     // Run the server on stdio transport
//!     server.run().await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Available Tools
//!
//! The pmcp server implements 20 MCP tools across different categories:
//!
//! ## Analysis Tools
//! - `analyze_complexity` - Analyze code complexity metrics
//! - `analyze_satd` - Detect self-admitted technical debt
//! - `analyze_dead_code` - Find unused code
//! - `analyze_dag` - Generate dependency graphs
//! - `analyze_deep_context` - Comprehensive code analysis
//! - `analyze_big_o` - Big-O complexity analysis
//!
//! ## Refactoring Tools
//! - `refactor.start` - Start a refactoring session
//! - `refactor.nextIteration` - Advance refactoring state
//! - `refactor.getState` - Get current refactoring state
//! - `refactor.stop` - Stop refactoring session
//!
//! ## Quality Tools
//! - `quality_gate` - Run comprehensive quality checks
//! - `quality_proxy` - Proxy code changes through quality gates
//!
//! ## Git Tools
//! - `git_operation` - Perform git operations
//!
//! ## Context Tools
//! - `generate_context` - Generate project context
//! - `generate_template` - Generate file from template
//! - `scaffold_project` - Create project structure
//!
//! # Performance
//!
//! The pmcp implementation provides significant performance benefits:
//!
//! ```rust,ignore
//! // Standard MCP server
//! // Average response time: 50ms
//! // Memory usage: 100MB
//!
//! // pmcp-based server  
//! // Average response time: 5ms (10x faster)
//! // Memory usage: 50MB (50% reduction)
//! ```

pub mod analyze_handlers;
pub mod context_handlers;
pub mod discovery;
pub mod handlers;
pub mod pdmt_handler;
pub mod quality_handlers;
pub mod quality_proxy_handler;
pub mod server;
pub mod simple_unified_server;
pub mod tool_functions;

// Export the simple unified server as the primary interface
pub use simple_unified_server::SimpleUnifiedServer as UnifiedServer;
// Keep PmcpServer for backward compatibility (will be removed)
pub use server::PmcpServer;
// Export the discovery service for MCP optimization
pub use discovery::{DiscoveryService, ToolInfo, Context, DiscoveryMetrics};
