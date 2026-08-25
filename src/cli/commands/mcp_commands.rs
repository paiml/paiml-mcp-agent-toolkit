use clap::Subcommand;

/// Commands for MCP manifest management and client onboarding
#[derive(Debug, Clone, Subcommand, PartialEq)]
pub enum McpCommands {
    /// Manage the MCP manifest file
    Manifest {
        /// Write the manifest to mcp.json
        #[arg(long)]
        write: bool,
    },

    /// Print how to connect pmat to an MCP client — every transport, in one place
    ///
    /// pmat speaks MCP over stdio AND over streamable HTTP from one binary, and
    /// both serve the same tools. `pmat --mode mcp`, `MCP_VERSION=1 pmat` and
    /// `pmat serve --transport http` are three spellings a user previously had
    /// to already know in order to find. This prints all of them, says when to
    /// use which, and states the four properties of the HTTP endpoint that are
    /// not guessable and that cost this project real time: MCP is served at the
    /// ROOT path (`/mcp` is 404), there is NO `/health` to use as a readiness
    /// probe, `PMAT_MCP_HTTP_TOKEN` has a 16-character minimum, and every RPC
    /// call needs `Accept: application/json, text/event-stream` or the server
    /// answers 406.
    #[command(visible_aliases = &["info", "setup"])]
    Connect,

    /// Print a fresh conforming bearer token and nothing else
    ///
    /// Substitutes directly, which is the point of printing nothing else:
    /// `export PMAT_MCP_HTTP_TOKEN=$(pmat mcp token)`.
    ///
    /// The token is 48 hex characters from the OS CSPRNG — three times the
    /// 16-character floor `pmat serve --transport http` enforces.
    Token,
}
