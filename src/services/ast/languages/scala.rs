//! Scala language strategy for the unified AST framework
//!
//! This module provides a Scala language strategy that integrates with the
//! unified AST framework, leveraging the ScalaAstVisitor from services/languages/scala.rs.

#![allow(clippy::module_name_repetitions)]

use crate::services::ast::AstStrategy;
use crate::services::context::FileContext;
use crate::services::file_classifier::FileClassifier;
use crate::services::languages::scala::ScalaAstVisitor;
use async_trait::async_trait;
use std::path::Path;
use tokio::fs;
use anyhow::Result;

/// Scala language strategy for AST parsing
pub struct ScalaStrategy;

impl ScalaStrategy {
    /// Creates a new Scala language strategy
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AstStrategy for ScalaStrategy {
    /// Analyzes a Scala file and returns a FileContext with AST information
    async fn analyze(&self, file_path: &Path, _classifier: &FileClassifier) -> Result<FileContext> {
        // Read the file content
        let content = fs::read_to_string(file_path).await?;
        
        // Use ScalaAstVisitor to analyze the Scala file
        let visitor = ScalaAstVisitor::new(file_path);
        match visitor.analyze_scala_source(&content) {
            Ok(items) => {
                // Create a FileContext with the extracted items
                Ok(FileContext {
                    path: file_path.to_string_lossy().to_string(),
                    language: "scala".to_string(),
                    items,
                    complexity_metrics: None,
                })
            },
            Err(e) => {
                // Log error and return empty context
                tracing::warn!("Failed to parse Scala file {}: {}", file_path.display(), e);
                Ok(FileContext {
                    path: file_path.to_string_lossy().to_string(),
                    language: "scala".to_string(),
                    items: vec![],
                    complexity_metrics: None,
                })
            }
        }
    }
    
    /// Returns the primary file extension for Scala
    fn primary_extension(&self) -> &'static str {
        "scala"
    }
    
    /// Returns all file extensions supported by this strategy
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["scala", "sc"]
    }
    
    /// Returns the language name
    fn language_name(&self) -> &'static str {
        "Scala"
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
    
    /// Creates a temporary Scala file with the given content for testing
    fn create_temp_scala_file(content: &str) -> (PathBuf, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("Test.scala");
        
        let mut file = File::create(&file_path).expect("Failed to create temp file");
        file.write_all(content.as_bytes()).expect("Failed to write to temp file");
        
        (file_path, temp_dir)
    }
    
    #[tokio::test]
    async fn test_scala_strategy_analyze() {
        // Create a simple Scala class for testing
        let scala_content = r#"
        package com.example
        
        class Test {
          def exampleMethod(): String = {
            "Hello, Scala!"
          }
        }
        "#;
        
        let (file_path, _temp_dir) = create_temp_scala_file(scala_content);
        
        // Create strategy and classifier
        let strategy = ScalaStrategy::new();
        let classifier = FileClassifier::new();
        
        // Analyze the file
        let context = strategy.analyze(&file_path, &classifier).await.unwrap();
        
        // Verify the results
        assert_eq!(context.language, "scala");
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
    
    #[tokio::test]
    async fn test_scala_strategy_case_class() {
        // Create a Scala case class for testing
        let scala_content = r#"
        package com.example.models
        
        case class Person(name: String, age: Int)
        
        object Person {
          def apply(name: String): Person = new Person(name, 0)
        }
        "#;
        
        let (file_path, _temp_dir) = create_temp_scala_file(scala_content);
        
        // Create strategy and classifier
        let strategy = ScalaStrategy::new();
        let classifier = FileClassifier::new();
        
        // Analyze the file
        let context = strategy.analyze(&file_path, &classifier).await.unwrap();
        
        // Verify the results
        assert_eq!(context.language, "scala");
        
        // Check for the case class
        let case_class_items: Vec<_> = context.items.iter()
            .filter(|item| {
                if let AstItem::Struct { derives, .. } = item {
                    derives.contains(&"case".to_string())
                } else {
                    false
                }
            })
            .collect();
        
        assert!(!case_class_items.is_empty());
        
        // Check for the companion object
        let object_items: Vec<_> = context.items.iter()
            .filter(|item| matches!(item, AstItem::Module { .. }))
            .collect();
        
        assert!(!object_items.is_empty());
    }
    
    #[test]
    fn test_scala_strategy_extensions() {
        let strategy = ScalaStrategy::new();
        
        assert_eq!(strategy.primary_extension(), "scala");
        assert!(strategy.supported_extensions().contains(&"scala"));
        assert!(strategy.supported_extensions().contains(&"sc"));
        assert_eq!(strategy.language_name(), "Scala");
    }
}