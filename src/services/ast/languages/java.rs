//! Java language strategy for the unified AST framework
//!
//! This module provides a Java language strategy that integrates with the
//! unified AST framework, leveraging the JavaAstVisitor from services/languages/java.rs.

#![allow(clippy::module_name_repetitions)]

use crate::services::ast::AstStrategy;
use crate::services::context::FileContext;
use crate::services::file_classifier::FileClassifier;
use crate::services::languages::java::JavaAstVisitor;
use async_trait::async_trait;
use std::path::Path;
use tokio::fs;
use anyhow::Result;

/// Java language strategy for AST parsing
pub struct JavaStrategy;

impl JavaStrategy {
    /// Creates a new Java language strategy
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AstStrategy for JavaStrategy {
    /// Analyzes a Java file and returns a FileContext with AST information
    async fn analyze(&self, file_path: &Path, _classifier: &FileClassifier) -> Result<FileContext> {
        // Read the file content
        let content = fs::read_to_string(file_path).await?;
        
        // Use JavaAstVisitor to analyze the Java file
        let visitor = JavaAstVisitor::new(file_path);
        match visitor.analyze_java_source(&content) {
            Ok(items) => {
                // Create a FileContext with the extracted items
                Ok(FileContext {
                    path: file_path.to_string_lossy().to_string(),
                    language: "java".to_string(),
                    items,
                    complexity_metrics: None,
                })
            },
            Err(e) => {
                // Log error and return empty context
                tracing::warn!("Failed to parse Java file {}: {}", file_path.display(), e);
                Ok(FileContext {
                    path: file_path.to_string_lossy().to_string(),
                    language: "java".to_string(),
                    items: vec![],
                    complexity_metrics: None,
                })
            }
        }
    }
    
    /// Returns the primary file extension for Java
    fn primary_extension(&self) -> &'static str {
        "java"
    }
    
    /// Returns all file extensions supported by this strategy
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["java"]
    }
    
    /// Returns the language name
    fn language_name(&self) -> &'static str {
        "Java"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::context::AstItem;
    use std::path::PathBuf;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;
    
    /// Creates a temporary Java file with the given content for testing
    fn create_temp_java_file(content: &str) -> (PathBuf, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("Test.java");
        
        let mut file = File::create(&file_path).expect("Failed to create temp file");
        file.write_all(content.as_bytes()).expect("Failed to write to temp file");
        
        (file_path, temp_dir)
    }
    
    #[tokio::test]
    async fn test_java_strategy_analyze() {
        // Create a simple Java class for testing
        let java_content = r#"
        package com.example;
        
        public class Test {
            public void exampleMethod() {
                System.out.println("Hello, Java!");
            }
        }
        "#;
        
        let (file_path, _temp_dir) = create_temp_java_file(java_content);
        
        // Create strategy and classifier
        let strategy = JavaStrategy::new();
        let classifier = FileClassifier::new();
        
        // Analyze the file
        let context = strategy.analyze(&file_path, &classifier).await.unwrap();
        
        // Verify the results
        assert_eq!(context.language, "java");
        assert!(!context.items.is_empty());
        
        // Check for the class
        let class_items: Vec<_> = context.items.iter()
            .filter(|item| matches!(item, AstItem::Struct { .. }))
            .collect();
        
        assert!(!class_items.is_empty());
        
        // Check for the method
        let method_items: Vec<_> = context.items.iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();
        
        assert!(!method_items.is_empty());
    }
    
    #[test]
    fn test_java_strategy_extensions() {
        let strategy = JavaStrategy::new();
        
        assert_eq!(strategy.primary_extension(), "java");
        assert!(strategy.supported_extensions().contains(&"java"));
        assert_eq!(strategy.language_name(), "Java");
    }
}