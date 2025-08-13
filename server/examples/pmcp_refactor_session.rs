//! Example demonstrating a complete refactoring session using the pmcp-based MCP server.
//!
//! This example shows the refactoring state machine workflow:
//! 1. Start a refactoring session
//! 2. Iterate through states
//! 3. Get current state
//! 4. Stop the session
//!
//! Run with:
//! ```bash
//! cargo run --example pmcp_refactor_session
//! ```

use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== PMAT pmcp Refactoring Session Example ===\n");

    println!("This example demonstrates the refactoring workflow through the pmcp MCP server.");
    println!("The refactoring tools implement a state machine for systematic code improvement.\n");

    // Step 1: Start a refactoring session
    println!("1. Starting a refactoring session:");
    let start_args = json!({
        "targets": ["src/complex_module.rs", "src/legacy_code.rs"],
        "config": {
            "max_complexity": 15,
            "enable_auto_fix": true,
            "preserve_comments": true
        }
    });
    println!("   Request: refactor.start");
    println!("   Args: {}", serde_json::to_string_pretty(&start_args)?);
    println!("   Expected Response:");
    println!("   {{");
    println!("     \"session_id\": \"uuid-here\",");
    println!("     \"state\": {{");
    println!("       \"current\": \"Scan\",");
    println!("       \"targets\": [\"src/complex_module.rs\", \"src/legacy_code.rs\"],");
    println!("       \"current_target_index\": 0");
    println!("     }}");
    println!("   }}");

    // Step 2: Advance through states
    println!("\n2. Advancing through refactoring states:");
    println!("   Request: refactor.nextIteration");
    println!("   Args: {{}} (no arguments needed)");
    println!("\n   State progression:");
    println!("   - Scan → Analyze → Plan → Refactor → Test → Lint → Complete");
    println!("   - Each iteration advances to the next state");
    println!("   - The server maintains session state internally");

    // Step 3: Get current state
    println!("\n3. Getting current refactoring state:");
    println!("   Request: refactor.getState");
    println!("   Args: {{}} (no arguments needed)");
    println!("   Expected: Returns current state machine state");

    // Step 4: Example state responses
    println!("\n4. Example state responses:");

    println!("\n   a) Analyze state:");
    let analyze_state = json!({
        "current": "Analyze",
        "current_file": "src/complex_module.rs",
        "targets": ["src/complex_module.rs", "src/legacy_code.rs"],
        "current_target_index": 0
    });
    println!("   {}", serde_json::to_string_pretty(&analyze_state)?);

    println!("\n   b) Plan state:");
    let plan_state = json!({
        "current": "Plan",
        "violations": [
            {
                "file": "src/complex_module.rs",
                "line": 42,
                "type": "complexity",
                "message": "Function 'process_data' has complexity 25 (max: 15)",
                "suggested_fix": "Extract method for validation logic"
            }
        ],
        "targets": ["src/complex_module.rs", "src/legacy_code.rs"],
        "current_target_index": 0
    });
    println!("   {}", serde_json::to_string_pretty(&plan_state)?);

    println!("\n   c) Complete state:");
    let complete_state = json!({
        "current": "Complete",
        "summary": {
            "files_processed": 2,
            "violations_found": 5,
            "violations_fixed": 4,
            "complexity_reduction": "25%",
            "elapsed_time": "2.5s"
        }
    });
    println!("   {}", serde_json::to_string_pretty(&complete_state)?);

    // Step 5: Stop the session
    println!("\n5. Stopping the refactoring session:");
    println!("   Request: refactor.stop");
    println!("   Args: {{}} (no arguments needed)");
    println!("   Expected Response:");
    println!("   {{");
    println!("     \"status\": \"stopped\",");
    println!("     \"message\": \"Refactoring session stopped successfully\"");
    println!("   }}");

    println!("\n6. Best Practices:");
    println!("   - Always start a session before refactoring");
    println!("   - Check state after each iteration");
    println!("   - Handle errors gracefully");
    println!("   - Stop sessions when complete or on error");

    println!("\n✅ Example completed successfully!");
    println!("\nNote: This example shows the refactoring workflow API.");
    println!("Actual refactoring would modify files based on the analysis.");

    Ok(())
}
