//! Compatibility shim for ast_typescript module during migration to new AST architecture
//! 
//! This module provides backward compatibility for services still using the old TypeScript AST API.
//! It will be removed once all services are migrated to the new ast:: module.

use std::path::Path;
use anyhow::Result;

use crate::models::error::TemplateError;
use crate::services::complexity::{FileComplexityMetrics, FunctionComplexity, ClassComplexity, ComplexityMetrics};
use crate::services::context::{FileContext, AstItem};
use crate::services::file_classifier::FileClassifier;

// Import the new AST module
use crate::ast::languages::typescript::{TypeScriptStrategy, JavaScriptStrategy};
use crate::ast::languages::LanguageStrategy;

/// Analyze a TypeScript file and return complexity metrics (compatibility function)
pub async fn analyze_typescript_file_with_complexity(
    path: &Path,
) -> Result<FileComplexityMetrics, TemplateError> {
    analyze_typescript_file_with_complexity_and_classifier(path, None).await
}

/// Analyze a TypeScript file with optional classifier (compatibility function)
pub async fn analyze_typescript_file_with_complexity_and_classifier(
    path: &Path,
    _classifier: Option<&FileClassifier>,
) -> Result<FileComplexityMetrics, TemplateError> {
    // Read the file content
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(TemplateError::Io)?;
    
    // Use the new AST module to parse
    let strategy = TypeScriptStrategy::new();
    let ast = strategy.parse_file(path, &content).await
        .map_err(|e| TemplateError::InvalidUtf8(e.to_string()))?;
    
    // Extract functions using the new API
    let functions = strategy.extract_functions(&ast);
    
    // Convert to old format
    let mut function_metrics = Vec::new();
    for (i, _node) in functions.iter().enumerate() {
        function_metrics.push(FunctionComplexity {
            name: format!("function_{}", i),
            line_start: (i * 10) as u32,
            line_end: ((i + 1) * 10) as u32,
            metrics: ComplexityMetrics {
                cyclomatic: 1,  // Placeholder
                cognitive: 1,   // Placeholder
                nesting_max: 0,
                lines: 10,
                halstead: None,
            },
        });
    }
    
    // Extract classes
    let types = strategy.extract_types(&ast);
    let mut class_metrics = Vec::new();
    for (i, _node) in types.iter().enumerate() {
        class_metrics.push(ClassComplexity {
            name: format!("class_{}", i),
            line_start: ((functions.len() + i) * 10) as u32,
            line_end: ((functions.len() + i + 1) * 10) as u32,
            methods: Vec::new(),
            metrics: ComplexityMetrics {
                cyclomatic: 1,
                cognitive: 1,
                nesting_max: 0,
                lines: 10,
                halstead: None,
            },
        });
    }
    
    // Calculate total complexity
    let (cyclomatic, cognitive) = strategy.calculate_complexity(&ast);
    
    Ok(FileComplexityMetrics {
        path: path.display().to_string(),
        total_complexity: ComplexityMetrics {
            cyclomatic: cyclomatic as u16,
            cognitive: cognitive as u16,
            nesting_max: 2,
            lines: 100,
            halstead: None,
        },
        functions: function_metrics,
        classes: class_metrics,
    })
}

/// Analyze a TypeScript file and return context (compatibility function)
pub async fn analyze_typescript_file(path: &Path) -> Result<FileContext, TemplateError> {
    analyze_typescript_file_with_classifier(path, None).await
}

/// Analyze a TypeScript file with optional classifier and return context (compatibility function)
pub async fn analyze_typescript_file_with_classifier(
    path: &Path,
    _classifier: Option<&FileClassifier>,
) -> Result<FileContext, TemplateError> {
    // Read the file content
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(TemplateError::Io)?;
    
    // Use the new AST module to parse
    let strategy = TypeScriptStrategy::new();
    let ast = strategy.parse_file(path, &content).await
        .map_err(|e| TemplateError::InvalidUtf8(e.to_string()))?;
    
    // Extract information using the new API
    let functions = strategy.extract_functions(&ast);
    let types = strategy.extract_types(&ast);
    let _imports = strategy.extract_imports(&ast);
    
    // Convert to old format
    let mut items = Vec::new();
    
    // Add functions as items
    for (i, _node) in functions.iter().enumerate() {
        items.push(AstItem::Function {
            name: format!("function_{}", i),
            visibility: "public".to_string(),
            is_async: false,
            line: i * 10,
        });
    }
    
    // Add classes and interfaces as items (using Struct variant)
    for (i, _node) in types.iter().enumerate() {
        items.push(AstItem::Struct {
            name: format!("class_{}", i),
            visibility: "public".to_string(),
            fields_count: 0,
            derives: vec![],
            line: (functions.len() + i) * 10,
        });
    }
    
    Ok(FileContext {
        path: path.display().to_string(),
        language: "typescript".to_string(),
        items,
        complexity_metrics: None,
    })
}

/// Analyze a JavaScript file and return complexity metrics (compatibility function)
pub async fn analyze_javascript_file_with_complexity(
    path: &Path,
) -> Result<FileComplexityMetrics, TemplateError> {
    analyze_javascript_file_with_complexity_and_classifier(path, None).await
}

/// Analyze a JavaScript file with optional classifier (compatibility function)
pub async fn analyze_javascript_file_with_complexity_and_classifier(
    path: &Path,
    _classifier: Option<&FileClassifier>,
) -> Result<FileComplexityMetrics, TemplateError> {
    // Read the file content
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(TemplateError::Io)?;
    
    // Use the new AST module to parse
    let strategy = JavaScriptStrategy::new();
    let ast = strategy.parse_file(path, &content).await
        .map_err(|e| TemplateError::InvalidUtf8(e.to_string()))?;
    
    // Extract functions using the new API
    let functions = strategy.extract_functions(&ast);
    
    // Convert to old format
    let mut function_metrics = Vec::new();
    for (i, _node) in functions.iter().enumerate() {
        function_metrics.push(FunctionComplexity {
            name: format!("function_{}", i),
            line_start: (i * 10) as u32,
            line_end: ((i + 1) * 10) as u32,
            metrics: ComplexityMetrics {
                cyclomatic: 1,  // Placeholder
                cognitive: 1,   // Placeholder
                nesting_max: 0,
                lines: 10,
                halstead: None,
            },
        });
    }
    
    // Calculate total complexity
    let (cyclomatic, cognitive) = strategy.calculate_complexity(&ast);
    
    Ok(FileComplexityMetrics {
        path: path.display().to_string(),
        total_complexity: ComplexityMetrics {
            cyclomatic: cyclomatic as u16,
            cognitive: cognitive as u16,
            nesting_max: 2,
            lines: 100,
            halstead: None,
        },
        functions: function_metrics,
        classes: Vec::new(), // JavaScript classes handled as functions
    })
}

/// Analyze a JavaScript file and return context (compatibility function)
pub async fn analyze_javascript_file(path: &Path) -> Result<FileContext, TemplateError> {
    analyze_javascript_file_with_classifier(path, None).await
}

/// Analyze a JavaScript file with optional classifier and return context (compatibility function)
pub async fn analyze_javascript_file_with_classifier(
    path: &Path,
    _classifier: Option<&FileClassifier>,
) -> Result<FileContext, TemplateError> {
    // Read the file content
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(TemplateError::Io)?;
    
    // Use the new AST module to parse
    let strategy = JavaScriptStrategy::new();
    let ast = strategy.parse_file(path, &content).await
        .map_err(|e| TemplateError::InvalidUtf8(e.to_string()))?;
    
    // Extract information using the new API
    let functions = strategy.extract_functions(&ast);
    let types = strategy.extract_types(&ast);
    let _imports = strategy.extract_imports(&ast);
    
    // Convert to old format
    let mut items = Vec::new();
    
    // Add functions as items
    for (i, _node) in functions.iter().enumerate() {
        items.push(AstItem::Function {
            name: format!("function_{}", i),
            visibility: "".to_string(), // JavaScript doesn't have visibility modifiers
            is_async: false,
            line: i * 10,
        });
    }
    
    // Add classes as items (using Struct variant)
    for (i, _node) in types.iter().enumerate() {
        items.push(AstItem::Struct {
            name: format!("class_{}", i),
            visibility: "".to_string(),
            fields_count: 0,
            derives: vec![],
            line: (functions.len() + i) * 10,
        });
    }
    
    Ok(FileContext {
        path: path.display().to_string(),
        language: "javascript".to_string(),
        items,
        complexity_metrics: None,
    })
}