// DEBUG-002: Debug Command Handlers
// Sprint 74 - GREEN Phase
//
// Handlers for `pmat debug` subcommands

use crate::services::dap::DapServer;
use anyhow::Result;

/// Handle `pmat debug serve` command
///
/// Starts a DAP (Debug Adapter Protocol) server on the specified port
/// allowing debuggers like VSCode to connect for time-travel debugging.
///
/// # Arguments
/// * `port` - Port number to bind the DAP server (default: 5678)
/// * `host` - Host address to bind (default: "127.0.0.1")
///
/// # Returns
/// * `Ok(())` if server starts successfully
/// * `Err` if port is already in use or other startup errors
pub async fn handle_debug_serve(port: u16, host: String) -> Result<()> {
    println!("🔍 Starting DAP server...");
    println!("   Host: {}", host);
    println!("   Port: {}", port);
    println!();
    println!("Connect your debugger to: {}:{}", host, port);
    println!("Press Ctrl+C to stop the server");
    println!();

    let server = DapServer::new();
    server.run(port, host).await?;

    Ok(())
}
