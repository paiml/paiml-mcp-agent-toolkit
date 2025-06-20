#[cfg(feature = "kotlin-ast")]
mod kotlin_tests {
    use paiml_mcp_agent_toolkit::services::ast_kotlin::KotlinAstParser;

    #[test]
    fn test_kotlin_parsing() {
        let kotlin_code = r#"
package com.example.test

fun main() {
    println("Hello Kotlin!")
    val greeting = "Welcome to PAIML"
    println(greeting)
}

class TestClass {
    fun testMethod(): String {
        return "Test successful"
    }
}
"#;

        let mut parser = KotlinAstParser::new();
        let path = std::path::Path::new("test.kt");
        match parser.parse_file(path, kotlin_code) {
            Ok(ast) => {
                println!("✅ Kotlin parsing successful!");
                println!("AST nodes: {}", ast.nodes.len());
                for (i, node) in ast.nodes.iter().enumerate() {
                    println!(
                        "Node {}: kind={:?}, range={}..{}",
                        i, node.kind, node.source_range.start, node.source_range.end
                    );
                }
                assert!(!ast.nodes.is_empty(), "AST should have nodes");
            }
            Err(e) => {
                panic!("❌ Kotlin parsing failed: {}", e);
            }
        }
    }
}
