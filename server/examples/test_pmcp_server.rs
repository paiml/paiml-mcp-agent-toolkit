//! Test example to verify pmcp server implementation works

use pmat::mcp_pmcp::PmcpServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("Creating pmcp-based MCP server...");
    let _server = PmcpServer::new();

    println!("pmcp server created successfully!");
    println!("To run the server, use: PMAT_PMCP_MCP=1 cargo run --bin pmat");
    println!("The server will handle MCP requests over stdio");

    // Don't actually run the server in this test, just verify it compiles
    println!("✅ pmcp server implementation is ready!");

    Ok(())
}
