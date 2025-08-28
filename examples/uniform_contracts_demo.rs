//! Demonstration of uniform contracts across all interfaces
//! This example shows CLI, MCP, and HTTP all using identical contracts

use pmat::contracts::{
    AnalyzeComplexityContract, BaseAnalysisContract, OutputFormat,
    simple_service::SimpleContractService,
    mcp_simple::SimpleMcpHandler,
    http_impl::create_router,
    adapter::BackwardCompatibility,
};
use serde_json::json;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Uniform Contracts Demo");
    println!("Demonstrating identical contracts across CLI, MCP, and HTTP interfaces");
    println!();
    
    // Create a contract - this is the SAME contract used by all interfaces
    let contract = AnalyzeComplexityContract {
        base: BaseAnalysisContract {
            path: PathBuf::from("."),
            format: OutputFormat::Json,
            output: None,
            top_files: Some(5),
            include_tests: false,
            timeout: 30,
        },
        max_cyclomatic: Some(10),
        max_cognitive: Some(8),
        max_halstead: Some(5.0),
    };
    
    println!("📋 Contract Definition (used by ALL interfaces):");
    println!("{}", serde_json::to_string_pretty(&contract)?);
    println!();
    
    // 1. Direct Service Usage (what CLI calls internally)
    println!("1️⃣ Direct Service Call (CLI backend):");
    let service = SimpleContractService::new()?;
    let result = service.analyze_complexity(contract.clone()).await?;
    println!("Result: {}", serde_json::to_string_pretty(&result)?);
    println!();
    
    // 2. MCP Tool Usage
    println!("2️⃣ MCP Tool Call (same contract as JSON):");
    let mcp_handler = SimpleMcpHandler::new()?;
    let mcp_params = serde_json::to_value(&contract)?;
    let mcp_result = mcp_handler.handle_tool_call("analyze_complexity", mcp_params).await?;
    println!("MCP Result: {}", serde_json::to_string_pretty(&mcp_result)?);
    println!();
    
    // 3. HTTP Endpoint Usage (same contract as JSON body)
    println!("3️⃣ HTTP Endpoint (would receive same JSON as body):");
    let http_body = serde_json::to_value(&contract)?;
    println!("HTTP Body: {}", serde_json::to_string_pretty(&http_body)?);
    println!("(This would be sent to POST /api/analyze/complexity)");
    println!();
    
    // 4. Backward Compatibility Demo
    println!("4️⃣ Backward Compatibility Mapping:");
    let old_params = json!({
        "project_path": ".",  // OLD parameter name
        "file": "main.rs",    // OLD single file parameter
        "format": "human"     // OLD format name
    });
    println!("Old parameters: {}", serde_json::to_string_pretty(&old_params)?);
    
    let new_params = BackwardCompatibility::map_json_params(old_params);
    println!("Mapped to new: {}", serde_json::to_string_pretty(&new_params)?);
    println!();
    
    // 5. Contract Validation
    println!("5️⃣ Contract Validation:");
    match contract.validate() {
        Ok(_) => println!("✅ Contract is valid!"),
        Err(e) => println!("❌ Contract validation failed: {}", e),
    }
    println!();
    
    // 6. Schema Generation for MCP
    println!("6️⃣ MCP Tool Schema (auto-generated from contract):");
    let schema = mcp_handler.get_tool_definitions();
    if let Some(tools) = schema["tools"].as_array() {
        if let Some(complexity_tool) = tools.iter().find(|t| t["name"] == "analyze_complexity") {
            println!("{}", serde_json::to_string_pretty(&complexity_tool["parameters"])?);
        }
    }
    println!();
    
    println!("🎉 Demo Complete!");
    println!("Key Benefits Demonstrated:");
    println!("✅ Single contract definition used by ALL interfaces");
    println!("✅ Identical parameter names across CLI, MCP, and HTTP");
    println!("✅ Automatic backward compatibility mapping");
    println!("✅ Contract validation ensures data integrity");
    println!("✅ Schema generation from contracts reduces duplication");
    println!("✅ Type safety across the entire system");
    
    Ok(())
}