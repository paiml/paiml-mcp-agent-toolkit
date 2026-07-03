use clap::Subcommand;

/// Commands for MCP manifest management
#[derive(Debug, Clone, Subcommand, PartialEq)]
pub enum McpCommands {
    /// Manage the MCP manifest file
    Manifest {
        /// Write the manifest to mcp.json
        #[arg(long)]
        write: bool,
    },
}
