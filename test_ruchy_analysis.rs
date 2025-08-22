use paiml_mcp_agent_toolkit::services::languages::ruchy::analyze_ruchy_file;
use std::path::Path;

#[tokio::main]
async fn main() {
    let path = Path::new("../ruchy/examples/fibonacci.ruchy");
    match analyze_ruchy_file(path).await {
        Ok(metrics) => {
            println!("Success! Found {} functions", metrics.functions.len());
            for func in &metrics.functions {
                println!("  Function '{}': cyclomatic={}", func.name, func.metrics.cyclomatic);
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}
