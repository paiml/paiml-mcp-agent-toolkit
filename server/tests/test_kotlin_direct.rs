#[cfg(feature = "kotlin-ast")]
#[tokio::test]
async fn test_kotlin_parser_directly() {
    use pmat::services::ast_strategies::{AstStrategy, KotlinAstStrategy};
    use pmat::services::file_classifier::FileClassifier;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create a temporary Kotlin file with test content
    let kotlin_code = r#"
package com.example.test

fun main() {
    println("Hello, Kotlin!")
}

class TestClass {
    fun testMethod(): String {
        return "test"
    }
}
"#;

    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "{}", kotlin_code).unwrap();
    let path = file.path();

    let classifier = FileClassifier::new();
    let strategy = KotlinAstStrategy;

    match strategy.analyze(path, &classifier).await {
        Ok(context) => {
            println!(
                "Success! Language: {}, Items: {}",
                context.language,
                context.items.len()
            );
            assert_eq!(context.language, "kotlin");
            assert!(
                !context.items.is_empty(),
                "Should have found at least one function"
            );

            // Verify we found the main function
            let main_found = context.items.iter().any(|item| {
                matches!(item, pmat::services::context::AstItem::Function { name, .. } if name == "main")
            });
            assert!(main_found, "Should find main function");

            // Verify we found the TestClass
            let class_found = context.items.iter().any(|item| {
                matches!(item, pmat::services::context::AstItem::Struct { name, .. } if name == "TestClass")
            });
            assert!(class_found, "Should find TestClass");

            // Verify we found the testMethod
            let method_found = context.items.iter().any(|item| {
                matches!(item, pmat::services::context::AstItem::Function { name, .. } if name == "testMethod")
            });
            assert!(method_found, "Should find testMethod");
        }
        Err(e) => {
            panic!("Kotlin parsing failed: {e}");
        }
    }
}
