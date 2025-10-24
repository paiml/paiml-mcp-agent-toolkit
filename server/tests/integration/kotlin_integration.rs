#![cfg(all(test, feature = "kotlin-ast", feature = "integration-tests"))]

use anyhow::Result;
use pmat::services::ast::AstRegistry;
use pmat::services::file_classifier::FileClassifier;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;
use tokio::test;

/// Comprehensive Kotlin language integration test
/// 
/// This test validates the full Kotlin integration in the AST framework by:
/// 1. Creating a variety of Kotlin code constructs
/// 2. Ensuring each construct is properly parsed
/// 3. Verifying the unified AST representation
#[test]
async fn test_kotlin_full_integration() -> Result<()> {
    // Create a temporary directory for our test files
    let temp_dir = TempDir::new()?;
    let kotlin_file_path = temp_dir.path().join("test_integration.kt");

    // Create a Kotlin file with various language features
    let kotlin_code = r#"
package com.example.pmat.integration

import java.util.concurrent.CompletableFuture

// Basic data class
data class User(
    val id: Int,
    val name: String,
    val email: String
)

// Interface with properties and methods
interface Repository<T> {
    val count: Int
    
    fun save(item: T): Boolean
    fun findById(id: Int): T?
}

// Enum class with properties
enum class Status(val code: Int) {
    ACTIVE(1),
    INACTIVE(0),
    PENDING(2);
    
    fun isActive(): Boolean = this == ACTIVE
}

// Class that implements interface
class UserRepository : Repository<User> {
    private val users = mutableListOf<User>()
    
    override val count: Int
        get() = users.size
        
    override fun save(item: User): Boolean {
        users.add(item)
        return true
    }
    
    override fun findById(id: Int): User? {
        return users.find { it.id == id }
    }
}

// Singleton object
object DatabaseConnection {
    fun connect() = println("Connected to database")
}

// Extension function
fun String.toTitleCase(): String {
    return this.split(" ")
        .map { it.capitalize() }
        .joinToString(" ")
}

// Higher order function 
fun <T, R> List<T>.transform(transformer: (T) -> R): List<R> {
    return this.map(transformer)
}

// Coroutine function
suspend fun fetchData(url: String): String {
    delay(100) // Simulate network delay
    return "Data from $url"
}

// Kotlin DSL example
class HTMLBuilder {
    fun h1(text: String) = "<h1>$text</h1>"
    fun p(text: String) = "<p>$text</p>"
}

fun html(init: HTMLBuilder.() -> String): String {
    val builder = HTMLBuilder()
    return builder.init()
}

// Main function
fun main() {
    val user = User(1, "John Doe", "john@example.com")
    val repo = UserRepository()
    repo.save(user)
    
    val greeting = "hello world"
    println(greeting.toTitleCase())
    
    DatabaseConnection.connect()
}
"#;

    // Write the Kotlin code to the file
    let mut file = File::create(&kotlin_file_path)?;
    file.write_all(kotlin_code.as_bytes())?;
    drop(file); // Close the file

    // Create registry and classifier
    let registry = AstRegistry::new();
    let classifier = FileClassifier::new();

    // Get the Kotlin strategy and analyze the file
    let strategy = registry.get_strategy("kt")
        .ok_or_else(|| anyhow::anyhow!("Kotlin strategy not found"))?;
    
    let result = strategy.analyze(&kotlin_file_path, &classifier).await?;
    
    // Verify the results
    assert_eq!(result.language, "kotlin");
    assert!(!result.items.is_empty(), "Should find AST items in Kotlin file");
    
    // Count the different types of items
    let data_classes = result.items.iter().filter(|item| {
        matches!(item, pmat::services::context::AstItem::Struct { name, .. } if name.contains("User"))
    }).count();
    
    let interfaces = result.items.iter().filter(|item| {
        matches!(item, pmat::services::context::AstItem::Trait { name, .. } if name.contains("Repository"))
    }).count();
    
    let enums = result.items.iter().filter(|item| {
        matches!(item, pmat::services::context::AstItem::Enum { name, .. } if name.contains("Status"))
            || matches!(item, pmat::services::context::AstItem::Struct { name, .. } if name.contains("Status"))
    }).count();
    
    let functions = result.items.iter().filter(|item| {
        matches!(item, pmat::services::context::AstItem::Function { .. })
    }).count();
    
    let classes = result.items.iter().filter(|item| {
        matches!(item, pmat::services::context::AstItem::Struct { name, .. } 
            if !name.contains("User") && !name.contains("Status"))
    }).count();
    
    // Check that we found the expected items
    assert!(data_classes > 0, "Should find User data class");
    assert!(interfaces > 0, "Should find Repository interface");
    assert!(enums > 0, "Should find Status enum");
    assert!(functions > 0, "Should find functions");
    assert!(classes > 0, "Should find classes");
    
    // Success!
    println!("✅ Kotlin integration test passed with {} AST items found", result.items.len());
    println!("  - Data classes: {}", data_classes);
    println!("  - Interfaces: {}", interfaces);
    println!("  - Enums: {}", enums);
    println!("  - Functions: {}", functions);
    println!("  - Classes: {}", classes);
    
    Ok(())
}

/// Test Kotlin file detection and classification 
#[test]
async fn test_kotlin_file_classification() -> Result<()> {
    // Create a temporary directory for our test files
    let temp_dir = TempDir::new()?;
    
    // Create files with different Kotlin extensions
    let kt_file_path = temp_dir.path().join("regular.kt");
    let kts_file_path = temp_dir.path().join("script.kts");
    
    // Write some minimal content
    let kotlin_code = "fun main() { println(\"Hello\") }";
    let kotlin_script = "println(\"Hello from script\")";
    
    File::create(&kt_file_path)?.write_all(kotlin_code.as_bytes())?;
    File::create(&kts_file_path)?.write_all(kotlin_script.as_bytes())?;
    
    // Create registry and classifier
    let registry = AstRegistry::new();
    let classifier = FileClassifier::new();
    
    // Verify that both file types are supported
    let kt_strategy = registry.get_strategy("kt");
    let kts_strategy = registry.get_strategy("kts");
    
    assert!(kt_strategy.is_some(), "Should support .kt files");
    assert!(kts_strategy.is_some(), "Should support .kts files");
    
    // Analyze both files
    let kt_result = kt_strategy.unwrap().analyze(&kt_file_path, &classifier).await?;
    let kts_result = kts_strategy.unwrap().analyze(&kts_file_path, &classifier).await?;
    
    // Both should be identified as Kotlin
    assert_eq!(kt_result.language, "kotlin");
    assert_eq!(kts_result.language, "kotlin");
    
    println!("✅ Kotlin file classification test passed");
    Ok(())
}