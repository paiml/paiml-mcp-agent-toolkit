// MCP server implementation
/// Mcp server.
pub struct McpServer {
    context: Arc<McpContext>,
    config: ServerConfig,
    shutdown: Arc<tokio::sync::Notify>,
}

#[derive(Clone)]
/// Configuration for server.
pub struct ServerConfig {
    pub name: String,
    pub version: String,
    pub bind_address: String,
    pub unix_socket: Option<String>,
    pub max_connections: usize,
    pub request_timeout: std::time::Duration,
    pub enable_logging: bool,

    // Semantic search configuration (PMAT-SEARCH-012)
    /// Enable semantic search (uses local embeddings, no API keys required)
    pub semantic_enabled: bool,
    pub semantic_db_path: Option<String>,
    pub semantic_workspace: Option<std::path::PathBuf>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        // Semantic search uses local embeddings - no API keys required
        let semantic_enabled = std::env::var("PMAT_SEMANTIC_ENABLED")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);
        let semantic_workspace = std::env::var("PMAT_WORKSPACE")
            .ok()
            .map(std::path::PathBuf::from)
            .or_else(|| std::env::current_dir().ok());
        // Per-workspace, not machine-global: embedding chunk paths are stored
        // workspace-relative, so a `~/.pmat/embeddings.db` shared by every
        // project served one project's chunks to all the others. Same default
        // function the CLI uses, so the two surfaces cannot drift apart.
        let semantic_db_path = std::env::var("PMAT_VECTOR_DB_PATH").ok().or_else(|| {
            semantic_workspace
                .as_deref()
                .map(crate::services::configuration_service::default_vector_db_path)
        });

        Self {
            name: "PMAT MCP Server".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            bind_address: "127.0.0.1:3000".to_string(),
            unix_socket: None,
            max_connections: 100,
            request_timeout: std::time::Duration::from_secs(30),
            enable_logging: true,

            // Semantic search (PMAT-SEARCH-012) - local embeddings, no API keys
            semantic_enabled,
            semantic_db_path,
            semantic_workspace,
        }
    }
}
