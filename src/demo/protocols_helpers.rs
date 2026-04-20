// Protocol helper functions: conversion, formatting, request building, output printing.
// Included by protocols.rs — shares parent module scope (no `use` imports here).

/// Convert Protocol enum to string
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn protocol_to_string(protocol: &Protocol) -> String {
    match protocol {
        Protocol::Cli => "cli".to_string(),
        Protocol::Http => "http".to_string(),
        Protocol::Mcp => "mcp".to_string(),
        #[cfg(feature = "tui")]
        Protocol::Tui => "tui".to_string(),
        Protocol::All => "all".to_string(),
    }
}

/// Print protocol-specific banner
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn print_protocol_banner(protocol: &Protocol) {
    match protocol {
        Protocol::Cli => println!("🚀 CLI Protocol Demo"),
        Protocol::Http => println!("🌐 HTTP Protocol Demo"),
        Protocol::Mcp => println!("🔌 MCP Protocol Demo"),
        #[cfg(feature = "tui")]
        Protocol::Tui => println!("📺 TUI Protocol Demo"),
        Protocol::All => println!("🎯 All Protocols Demo"),
    }
}

/// Helper to build protocol-specific requests
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn build_protocol_request(
    protocol: &str,
    repo_path: &std::path::Path,
    show_api: bool,
) -> serde_json::Value {
    let path_str = repo_path.to_str().expect("internal error");
    match protocol {
        "cli" => serde_json::json!({
            "path": path_str,
            "show_api": show_api
        }),
        "http" => serde_json::json!({
            "method": "GET",
            "path": "/demo/analyze",
            "query": {"path": path_str},
            "headers": {"Accept": "application/json"}
        }),
        "mcp" => serde_json::json!({
            "jsonrpc": "2.0",
            "method": "demo.analyze",
            "params": {
                "path": path_str,
                "include_trace": show_api
            },
            "id": 1
        }),
        _ => serde_json::json!({}),
    }
}

/// Format and print output based on format type
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn format_and_print_output(
    response: &serde_json::Value,
    format: &crate::cli::OutputFormat,
) -> Result<()> {
    match format {
        crate::cli::OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(response)?);
        }
        crate::cli::OutputFormat::Yaml => {
            println!("{}", serde_yaml_ng::to_string(response)?);
        }
        // Table/Markdown/Csv/Summary/Text/Plain/Junit all fall back to
        // pretty-debug — the demo harness only speaks Json/Yaml structurally;
        // other formats aren't meaningful for a JSON-RPC response blob.
        _ => {
            println!("{response:#?}");
        }
    }
    Ok(())
}

/// Print API metadata for a protocol
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) async fn print_api_metadata(protocol_name: &str) -> Result<()> {
    println!("\n📊 API Introspection");
    // TRACKED: This would require access to the engine reference
    println!("Protocol: {protocol_name}");
    Ok(())
}
