use crate::mcp_server::cache::{CacheConfig, CacheKeyBuilder, McpCache};
use crate::mcp_server::handlers;
use crate::mcp_server::state_manager::StateManager;
use crate::models::mcp::{McpRequest, McpResponse};
use serde_json::json;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

/// Model Context Protocol (MCP) server for PMAT refactoring capabilities.
///
/// This server implements the MCP specification for AI-assisted refactoring,
/// providing a standardized interface for Claude and other AI models to interact
/// with PMAT's analysis and refactoring capabilities. Critical for maintaining
/// MCP protocol compliance and preventing API drift.
///
/// # MCP Protocol Support
///
/// - **Protocol Version**: 2024-11-05
/// - **Transport**: JSON-RPC 2.0 over stdin/stdout
/// - **Capabilities**: Refactoring state machine control
/// - **Error Handling**: Standard JSON-RPC error codes
///
/// # Supported Methods
///
/// ## Core Protocol Methods
/// - `initialize` - Initialize MCP session with capabilities
///
/// ## Refactoring Methods
/// - `refactor.start` - Start new refactoring session
/// - `refactor.nextIteration` - Advance refactoring state machine
/// - `refactor.getState` - Get current refactoring state
/// - `refactor.stop` - Stop current refactoring session
///
/// # State Management
///
/// The server maintains refactoring sessions with:
/// - Target files and configuration
/// - State machine progression (Scan -> Analyze -> Plan -> Refactor -> Complete)
/// - Session isolation and cleanup
/// - Error recovery and rollback
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::mcp_server::server::McpServer;
///
/// # tokio_test::block_on(async {
/// // Create MCP server
/// let server = McpServer::new();
///
/// // Server is ready for MCP communication
/// // In real usage, server.run().await would handle stdin/stdout
/// # });
/// ```
///
/// # MCP Message Examples
///
/// ## Initialize Request
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "id": 1,
///   "method": "initialize",
///   "params": {
///     "protocolVersion": "2024-11-05",
///     "clientInfo": {
///       "name": "claude-desktop",
///       "version": "1.0.0"
///     }
///   }
/// }
/// ```ignore
///
/// ## Refactor Start Request
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "id": 2,
///   "method": "refactor.start",
///   "params": {
///     "targets": ["/path/to/file.rs"],
///     "config": {
///       "target_complexity": 15,
///       "remove_satd": true
///     }
///   }
/// }
/// ```ignore
///
/// ## State Query Request
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "id": 3,
///   "method": "refactor.getState"
/// }
/// ```ignore
pub struct McpServer {
    state_manager: Arc<Mutex<StateManager>>,
    cache: Arc<McpCache>,
}

impl McpServer {
    /// Creates a new MCP server with default state management.
    ///
    /// Initializes the server with a new state manager for handling refactoring
    /// sessions. The server is ready to accept MCP connections and process
    /// refactoring requests according to the MCP protocol specification.
    ///
    /// # Returns
    ///
    /// A new `McpServer` instance with initialized state management.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use pmat::mcp_server::server::McpServer;
    ///
    /// // Create MCP server for AI integration
    /// let server = McpServer::new();
    ///
    /// // Server is ready for MCP protocol communication
    /// // Typically used with: server.run().await
    /// ```
    ///
    /// # MCP Integration
    ///
    /// The server is designed to integrate with MCP clients like:
    /// - Claude Desktop application
    /// - VS Code with MCP extension
    /// - Custom MCP clients
    /// - CI/CD pipeline integrations
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        let cache_config = CacheConfig {
            max_entries: 5000,
            default_ttl: Duration::from_secs(600), // 10 minutes
            enable_metrics: true,
        };

        Self {
            state_manager: Arc::new(Mutex::new(StateManager::new())),
            cache: Arc::new(McpCache::with_config(cache_config)),
        }
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

// MCP protocol communication: run loop, request handling, and cache metrics
include!("server_protocol.rs");

// Tests: property tests and coverage tests
include!("server_tests.rs");
