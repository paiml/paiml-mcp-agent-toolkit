#![cfg(feature = "kotlin-ast")]

use std::path::Path;

#[test]
fn test_minimal_kotlin_parsing() {
    println!("Testing minimal Kotlin parsing...");

    // Test 1: Simple class parsing
    let kotlin_code = r#"
class Hello {
    fun greet() {
        println("Hello, World!")
    }
}
"#;

    // Create a temporary path
    let path = Path::new("test.kt");

    // Test parser creation
    let mut parser = paiml_mcp_agent_toolkit::services::ast_kotlin::KotlinAstParser::new();
    println!("✓ Parser created successfully");

    // Test parsing
    match parser.parse_file(path, kotlin_code) {
        Ok(ast_dag) => {
            println!("✓ Parsing succeeded");
            println!("  - Found {} nodes", ast_dag.nodes.len());
        }
        Err(e) => {
            println!("✗ Parsing failed: {e}");
        }
    }

    println!("\nTest completed!");
}
