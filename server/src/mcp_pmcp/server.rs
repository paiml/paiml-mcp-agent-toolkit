use crate::mcp_pmcp::handlers::{
    RefactorGetStateTool, RefactorNextIterationTool, RefactorStartTool, RefactorStopTool,
};
use crate::mcp_server::state_manager::StateManager;
use pmcp::{Server, ServerCapabilities, ToolCapabilities};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

/// MCP server implementation using pmcp SDK for PMAT refactoring capabilities.
///
/// This server provides a standardized interface for AI-assisted refactoring,
/// implementing the MCP specification with full protocol compliance using the
/// pmcp SDK.
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::mcp_pmcp::server::PmcpServer;
///
/// # tokio_test::block_on(async {
/// let server = PmcpServer::new();
/// // Run with stdio transport
/// server.run().await.unwrap();
/// # });
/// ```
pub struct PmcpServer {
    state_manager: Arc<Mutex<StateManager>>,
}

impl PmcpServer {
    /// Creates a new MCP server instance using pmcp SDK.
    pub fn new() -> Self {
        Self {
            state_manager: Arc::new(Mutex::new(StateManager::new())),
        }
    }

    /// Runs the MCP server with stdio transport.
    ///
    /// This method creates a pmcp Server instance configured with PMAT's
    /// refactoring tools and runs it using the stdio transport, handling
    /// JSON-RPC communication over stdin/stdout.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Server ran successfully and shut down cleanly
    /// * `Err(Box<dyn std::error::Error>)` - Server initialization or runtime error
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use pmat::mcp_pmcp::server::PmcpServer;
    ///
    /// # tokio_test::block_on(async {
    /// let server = PmcpServer::new();
    /// server.run().await.unwrap();
    /// # });
    /// ```
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting PMAT MCP server using pmcp SDK");

        // Build server with refactoring tools
        let server = Server::builder()
            .name("paiml-mcp-agent-toolkit")
            .version(env!("CARGO_PKG_VERSION"))
            .capabilities(ServerCapabilities {
                tools: Some(ToolCapabilities { list_changed: None }),
                ..Default::default()
            })
            .tool(
                "refactor.start",
                RefactorStartTool::new(self.state_manager.clone()),
            )
            .tool(
                "refactor.nextIteration",
                RefactorNextIterationTool::new(self.state_manager.clone()),
            )
            .tool(
                "refactor.getState",
                RefactorGetStateTool::new(self.state_manager.clone()),
            )
            .tool(
                "refactor.stop",
                RefactorStopTool::new(self.state_manager.clone()),
            )
            .build()?;

        info!("PMAT MCP server ready, listening on stdio");

        // Run server with stdio transport
        server.run_stdio().await?;

        info!("PMAT MCP server shutting down");
        Ok(())
    }
}

impl Default for PmcpServer {
    fn default() -> Self {
        Self::new()
    }
}