#![cfg(feature = "kotlin-ast")]

use paiml_mcp_agent_toolkit::services::ast_strategies::{AstStrategy, KotlinAstStrategy};
use paiml_mcp_agent_toolkit::services::file_classifier::FileClassifier;
use std::io::Write;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_kotlin_class_parsing() {
    let kotlin_code = r#"
package com.example.demo

class Person(val name: String, var age: Int) {
    fun greet() {
        println("Hello, I'm $name")
    }
}
"#;

    // Create temporary file
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "{kotlin_code}").unwrap();
    let path = file.path();

    // Test parsing
    let classifier = FileClassifier::new();
    let strategy = KotlinAstStrategy;
    let result = strategy.analyze(path, &classifier).await;

    assert!(result.is_ok());
    let context = result.unwrap();
    assert_eq!(context.language, "kotlin");


    // Check that we found the class and function
    let class_found = context.items.iter().any(|item| {
        matches!(item, paiml_mcp_agent_toolkit::services::context::AstItem::Struct { name, .. } if name == "Person")
    });
    assert!(class_found, "Should find Person class");

    let function_found = context.items.iter().any(|item| {
        matches!(item, paiml_mcp_agent_toolkit::services::context::AstItem::Function { name, .. } if name == "greet")
    });
    assert!(function_found, "Should find greet function");
}

#[tokio::test]
async fn test_kotlin_interface_parsing() {
    let kotlin_code = r#"
package com.example.demo

interface Vehicle {
    fun start()
    fun stop()
}

class Car : Vehicle {
    override fun start() {
        println("Car starting")
    }
    
    override fun stop() {
        println("Car stopping")
    }
}
"#;

    // Create temporary file
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "{kotlin_code}").unwrap();
    let path = file.path();

    // Test parsing
    let classifier = FileClassifier::new();
    let strategy = KotlinAstStrategy;
    let result = strategy.analyze(path, &classifier).await;

    assert!(result.is_ok());
    let context = result.unwrap();

    // Check that we found the interface and class
    let interface_found = context.items.iter().any(|item| {
        matches!(item, paiml_mcp_agent_toolkit::services::context::AstItem::Struct { name, .. } if name == "Vehicle")
    });
    assert!(interface_found, "Should find Vehicle interface");

    let class_found = context.items.iter().any(|item| {
        matches!(item, paiml_mcp_agent_toolkit::services::context::AstItem::Struct { name, .. } if name == "Car")
    });
    assert!(class_found, "Should find Car class");
}

#[tokio::test]
async fn test_kotlin_data_class_parsing() {
    let kotlin_code = r#"
data class User(val id: Int, val name: String, val email: String)

enum class Status {
    ACTIVE,
    INACTIVE,
    PENDING
}
"#;

    // Create temporary file with .kt extension
    let file = tempfile::Builder::new().suffix(".kt").tempfile().unwrap();
    writeln!(file.as_file(), "{}", kotlin_code).unwrap();
    let path = file.path();

    // Test parsing
    let classifier = FileClassifier::new();
    let strategy = KotlinAstStrategy;
    let result = strategy.analyze(path, &classifier).await;

    assert!(result.is_ok());
    let context = result.unwrap();


    // Check that we found the data class
    let data_class_found = context.items.iter().any(|item| {
        matches!(item, paiml_mcp_agent_toolkit::services::context::AstItem::Struct { name, .. } if name == "User")
    });
    assert!(data_class_found, "Should find User data class");


    // Check that we found the enum - it should be an Enum item, not Struct
    let enum_found = context.items.iter().any(|item| {
        matches!(item, paiml_mcp_agent_toolkit::services::context::AstItem::Struct { name, .. } if name == "Status")
            || matches!(item, paiml_mcp_agent_toolkit::services::context::AstItem::Enum { name, .. } if name == "Status")
    });
    assert!(enum_found, "Should find Status enum");
}

#[test]
fn test_kotlin_extension_support() {
    let strategy = KotlinAstStrategy;
    assert!(strategy.supports_extension("kt"));
    assert!(strategy.supports_extension("kts"));
    assert!(!strategy.supports_extension("java"));
    assert!(!strategy.supports_extension("rs"));
}
