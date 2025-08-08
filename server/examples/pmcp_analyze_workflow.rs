//! Example demonstrating a complete analysis workflow using the pmcp-based MCP server.
//!
//! This example shows how to:
//! 1. Set up the pmcp server
//! 2. Use various analysis tools
//! 3. Process and display results
//!
//! Run with:
//! ```bash
//! cargo run --example pmcp_analyze_workflow
//! ```

use pmat::mcp_pmcp::PmcpServer;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== PMAT pmcp Analysis Workflow Example ===\n");

    // Create the pmcp server
    println!("1. Creating pmcp-based MCP server...");
    let _server = PmcpServer::new();
    println!("   ✓ Server created successfully");

    // In a real scenario, the server would be running and accepting requests
    // For this example, we'll demonstrate the tool argument formats

    println!("\n2. Example tool invocations:");

    // Complexity analysis example
    println!("\n   a) Complexity Analysis:");
    let complexity_args = json!({
        "paths": ["src/"],
        "top_files": 10,
        "threshold": 20
    });
    println!("      Request: analyze_complexity");
    println!("      Args: {}", serde_json::to_string_pretty(&complexity_args)?);
    println!("      Expected: Returns top 10 files with complexity > 20");

    // SATD detection example
    println!("\n   b) Technical Debt Detection:");
    let satd_args = json!({
        "paths": ["src/", "tests/"],
        "include_resolved": false
    });
    println!("      Request: analyze_satd");
    println!("      Args: {}", serde_json::to_string_pretty(&satd_args)?);
    println!("      Expected: Returns TODO, FIXME, HACK comments");

    // Dead code analysis example
    println!("\n   c) Dead Code Analysis:");
    let dead_code_args = json!({
        "paths": ["src/"],
        "include_tests": false
    });
    println!("      Request: analyze_dead_code");
    println!("      Args: {}", serde_json::to_string_pretty(&dead_code_args)?);
    println!("      Expected: Returns unused functions and variables");

    // Quality gate example
    println!("\n   d) Quality Gate Check:");
    let quality_args = json!({
        "paths": ["src/"],
        "checks": ["complexity", "satd", "dead_code"],
        "strict": true
    });
    println!("      Request: quality_gate");
    println!("      Args: {}", serde_json::to_string_pretty(&quality_args)?);
    println!("      Expected: Returns pass/fail for each quality check");

    println!("\n3. Server Features:");
    println!("   - High-performance async request handling");
    println!("   - Type-safe tool implementations");
    println!("   - Automatic JSON-RPC protocol handling");
    println!("   - Built-in error propagation and logging");

    println!("\n4. Running the server:");
    println!("   To run the pmcp server in production:");
    println!("   ```");
    println!("   PMAT_PMCP_MCP=1 cargo run --features pmcp-mcp --bin pmat");
    println!("   ```");

    println!("\n5. Connecting to the server:");
    println!("   The server communicates over stdio using JSON-RPC 2.0");
    println!("   Example client request:");
    println!("   ```json");
    println!("   {{");
    println!("     \"jsonrpc\": \"2.0\",");
    println!("     \"id\": 1,");
    println!("     \"method\": \"tools/call\",");
    println!("     \"params\": {{");
    println!("       \"name\": \"analyze_complexity\",");
    println!("       \"arguments\": {}", serde_json::to_string(&complexity_args)?);
    println!("     }}");
    println!("   }}");
    println!("   ```");

    println!("\n✅ Example completed successfully!");
    println!("\nNote: This example demonstrates the pmcp server API.");
    println!("In production, the server would handle actual MCP requests.");

    Ok(())
}