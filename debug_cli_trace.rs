// Debug program to trace the CLI complexity analysis issue
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let test_content = r#"fn test_function() {
    println!("hello");
}

pub fn second_function() {
    if true {
        println!("world");
    }
}
"#;
    
    let path = Path::new("test.rs");
    
    println!("=== DEBUGGING CLI COMPLEXITY ANALYSIS ===");
    println!("Test content:");
    println!("{}", test_content);
    
    // Test the language analyzer directly
    println!("\n1. Testing language_analyzer::analyze_file_complexity directly:");
    match pmat::cli::language_analyzer::analyze_file_complexity(path, test_content).await {
        Ok(metrics) => {
            println!("   SUCCESS: Found {} functions", metrics.functions.len());
            for func in &metrics.functions {
                println!("   - {}: lines {}-{}", func.name, func.line_start, func.line_end);
            }
        }
        Err(e) => println!("   ERROR: {}", e),
    }
    
    // Test the stubs layer
    println!("\n2. Testing stubs::analyze_file_complexity_async via load_file_and_analyze:");
    // We can't call analyze_file_complexity_async directly as it's private
    // But we can trace through load_file_and_analyze which calls it
    
    // Write test content to a real file
    std::fs::write("debug_test.rs", test_content)?;
    let test_path = Path::new("debug_test.rs");
    
    match pmat::cli::stubs::load_file_and_analyze(test_path, 20, 15).await {
        Ok(Some(metrics)) => {
            println!("   SUCCESS: Found {} functions", metrics.functions.len());
            for func in &metrics.functions {
                println!("   - {}: lines {}-{}", func.name, func.line_start, func.line_end);
            }
        }
        Ok(None) => println!("   RESULT: None (file not found or error)"),
        Err(e) => println!("   ERROR: {}", e),
    }
    
    // Clean up
    std::fs::remove_file("debug_test.rs").ok();
    
    println!("\n=== ROOT CAUSE ANALYSIS ===");
    println!("If #1 works but #2 fails, the bug is in stubs.rs");
    println!("If both work, the bug is in the CLI command handling layer");
    
    Ok(())
}