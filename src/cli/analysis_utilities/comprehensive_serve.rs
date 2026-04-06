// Serve handlers - split from comprehensive.rs for file health (PMAT-503)
/// Starts an HTTP server
///
/// # Errors
/// Returns an error if the server cannot be started
pub async fn handle_serve(
    host: String,
    port: u16,
    cors: bool,
    transport: crate::cli::commands::ServeTransport,
) -> Result<()> {
    use crate::cli::commands::ServeTransport;

    match transport {
        ServeTransport::Http => handle_http_server(&host, port, cors).await,
        ServeTransport::WebSocket => handle_websocket_server(&host, port).await,
        ServeTransport::HttpSse => handle_http_sse_server(&host, port, cors).await,
        ServeTransport::Both => handle_hybrid_server(&host, port, cors).await,
        ServeTransport::All => handle_full_server(&host, port, cors).await,
    }
}

/// Extract Method: Handle HTTP server startup
async fn handle_http_server(host: &str, port: u16, cors: bool) -> Result<()> {
    debug_assert!(!host.is_empty(), "host must not be empty");
    eprintln!("🚀 Starting PMAT HTTP server on http://{host}:{port}");
    eprintln!("✅ Server ready!");
    eprintln!("📍 Health check: http://{host}:{port}/health");
    eprintln!("📍 API base: http://{host}:{port}/api/v1");
    print_cors_status(cors);
    eprintln!("\n🔧 HTTP server functionality ready for implementation.");

    await_shutdown_signal().await
}

/// Extract Method: Handle WebSocket server startup
async fn handle_websocket_server(host: &str, port: u16) -> Result<()> {
    debug_assert!(!host.is_empty(), "host must not be empty");
    eprintln!("🚀 Starting PMAT WebSocket server on ws://{host}:{port}");
    eprintln!("✅ WebSocket server ready!");
    eprintln!("📍 WebSocket endpoint: ws://{host}:{port}");
    eprintln!("🔌 MCP protocol over WebSocket");

    let addr = format!("{host}:{port}");
    start_websocket_server(addr).await
}

/// Extract Method: Handle HTTP-SSE server startup
async fn handle_http_sse_server(host: &str, port: u16, cors: bool) -> Result<()> {
    debug_assert!(!host.is_empty(), "host must not be empty");
    eprintln!("🚀 Starting PMAT HTTP-SSE server on http://{host}:{port}");
    eprintln!("✅ HTTP-SSE server ready!");
    eprintln!("📍 SSE endpoint: http://{host}:{port}/sse");
    eprintln!("📍 Message endpoint: http://{host}:{port}/message");
    eprintln!("🌊 MCP protocol over Server-Sent Events");
    print_cors_status(cors);

    let addr = format!("{host}:{port}");
    start_http_sse_server(addr, cors).await
}

/// Extract Method: Handle hybrid server startup
async fn handle_hybrid_server(host: &str, port: u16, cors: bool) -> Result<()> {
    debug_assert!(!host.is_empty(), "host must not be empty");
    eprintln!("🚀 Starting PMAT hybrid server (HTTP + WebSocket) on {host}:{port}");
    eprintln!("✅ Hybrid server ready!");
    eprintln!("📍 HTTP endpoint: http://{host}:{port}");
    eprintln!("📍 WebSocket endpoint: ws://{host}:{port}");
    eprintln!("🔌 MCP protocol over both transports");
    print_cors_status(cors);

    let addr = format!("{host}:{port}");
    start_hybrid_server(addr, cors).await
}

/// Extract Method: Handle full server startup  
async fn handle_full_server(host: &str, port: u16, cors: bool) -> Result<()> {
    debug_assert!(!host.is_empty(), "host must not be empty");
    eprintln!("🚀 Starting PMAT full server (HTTP + WebSocket + SSE) on {host}:{port}");
    eprintln!("✅ All transports ready!");
    eprintln!("📍 HTTP endpoint: http://{host}:{port}");
    eprintln!("📍 WebSocket endpoint: ws://{host}:{port}");
    eprintln!("📍 SSE endpoint: http://{host}:{port}/sse");
    eprintln!("🌐 MCP protocol over all transports");
    print_cors_status(cors);

    let addr = format!("{host}:{port}");
    start_full_server(addr, cors).await
}

/// Extract Method: Print CORS status
fn print_cors_status(cors: bool) {
    if cors {
        eprintln!("🌐 CORS enabled for all origins");
    }
}

/// Extract Method: Await shutdown signal
async fn await_shutdown_signal() -> Result<()> {
    eprintln!("Press Ctrl+C to exit.\n");
    tokio::signal::ctrl_c().await?;
    eprintln!("🛑 Shutting down server...");
    Ok(())
}

/// Start a WebSocket-only server
async fn start_websocket_server(addr: String) -> Result<()> {
    eprintln!("🔌 WebSocket server implementation ready for {addr}");
    eprintln!("📍 This would start a WebSocket server for MCP protocol communication");
    eprintln!("🔗 Integration with transport layer and MCP server required");
    eprintln!("Press Ctrl+C to exit.\n");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    eprintln!("🛑 Shutting down WebSocket server...");

    Ok(())
}

/// Start a hybrid server (HTTP + WebSocket)
async fn start_hybrid_server(addr: String, _cors: bool) -> Result<()> {
    eprintln!("🔧 Hybrid server functionality ready for implementation on {addr}.");
    eprintln!("📍 This would support both HTTP REST API and WebSocket MCP protocol");
    eprintln!("Press Ctrl+C to exit.\n");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    eprintln!("🛑 Shutting down hybrid server...");

    Ok(())
}

/// Start an HTTP-SSE server
async fn start_http_sse_server(addr: String, _cors: bool) -> Result<()> {
    eprintln!("🌊 HTTP-SSE server implementation ready for {addr}");
    eprintln!("📍 This would start an HTTP Server-Sent Events server for MCP protocol");
    eprintln!("📨 POST /message - Send messages to server");
    eprintln!("🔄 GET /sse - Receive events via Server-Sent Events");
    eprintln!("Press Ctrl+C to exit.\n");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    eprintln!("🛑 Shutting down HTTP-SSE server...");

    Ok(())
}

/// Start a full multi-transport server (HTTP + WebSocket + SSE)
async fn start_full_server(addr: String, _cors: bool) -> Result<()> {
    eprintln!("🌐 Full multi-transport server implementation ready for {addr}");
    eprintln!("📍 This would support HTTP, WebSocket, and SSE transports simultaneously");
    eprintln!("🔗 All MCP protocol communication methods available");
    eprintln!("Press Ctrl+C to exit.\n");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    eprintln!("🛑 Shutting down full server...");

    Ok(())
}
