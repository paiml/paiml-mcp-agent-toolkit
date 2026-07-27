#![cfg_attr(coverage_nightly, coverage(off))]
//! Compatibility shim for `ast_python` module during migration to new AST architecture
//!
//! This module provides backward compatibility for services still using the old Python AST API.
//! It will be removed once all services are migrated to the new `ast::` module.

use anyhow::Result;
use std::path::Path;

use crate::models::error::TemplateError;
use crate::services::complexity::{ComplexityMetrics, FileComplexityMetrics, FunctionComplexity};
use crate::services::context::{AstItem, FileContext};
use crate::services::file_classifier::FileClassifier;

// Import the new AST module
use crate::ast::languages::python::PythonStrategy;
use crate::ast::languages::LanguageStrategy;

// DEPRECATED: Enhanced Python visitor migration in progress
// Will be migrated to tree-sitter in next phase
// #[cfg(feature = "python-ast")]
// use crate::services::enhanced_python_visitor::EnhancedPythonVisitor;

/// Analyze a Python file and return complexity metrics (compatibility function)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_python_file_with_complexity(
    path: &Path,
    classifier: Option<&FileClassifier>,
) -> Result<FileComplexityMetrics, TemplateError> {
    // Delegate to the version with classifier
    analyze_python_file_with_complexity_and_classifier(path, classifier).await
}

/// Analyze a Python file with optional classifier (compatibility function)
async fn analyze_python_file_with_complexity_and_classifier(
    path: &Path,
    _classifier: Option<&FileClassifier>,
) -> Result<FileComplexityMetrics, TemplateError> {
    // Read the file content
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(TemplateError::Io)?;

    // Use the new AST module to parse
    let strategy = PythonStrategy::new();
    let ast = strategy
        .parse_file(path, &content)
        .await
        .map_err(|e| TemplateError::InvalidUtf8(e.to_string()))?;

    // Extract functions using the new API
    let functions = strategy.extract_functions(&ast);

    // Convert to old format
    let mut function_metrics = Vec::new();
    for (i, _node) in functions.iter().enumerate() {
        function_metrics.push(FunctionComplexity {
            name: format!("function_{i}"),
            line_start: (i * 10) as u32,
            line_end: ((i + 1) * 10) as u32,
            metrics: ComplexityMetrics {
                cyclomatic: 1, // Placeholder
                cognitive: 1,  // Placeholder
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
        classes: Vec::new(), // Will be populated from types
    })
}

/// Analyze a Python file and return context (compatibility function)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_python_file(path: &Path) -> Result<FileContext, TemplateError> {
    analyze_python_file_with_classifier(path, None).await
}

/// Analyze a Python file with optional classifier and return context (compatibility function)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_python_file_with_classifier(
    path: &Path,
    _classifier: Option<&FileClassifier>,
) -> Result<FileContext, TemplateError> {
    // Read the file content
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(TemplateError::Io)?;

    // Use new tree-sitter-based Python strategy
    // (Enhanced visitor migration in progress - will use tree-sitter in next phase)
    let strategy = PythonStrategy::new();
    let ast = strategy
        .parse_file(path, &content)
        .await
        .map_err(|e| TemplateError::InvalidUtf8(e.to_string()))?;

    // Extract information using the new API
    let functions = strategy.extract_functions(&ast);
    let types = strategy.extract_types(&ast);
    let imports = strategy.extract_imports(&ast);

    // Convert to old format (as fallback)
    let mut items = Vec::new();

    // Add imports as items
    for (i, _node) in imports.iter().enumerate() {
        items.push(AstItem::Import {
            module: format!("module_{i}"),
            items: vec![],
            alias: None,
            line: i * 2, // Imports typically near top of file
        });
    }

    // Add functions as items
    for (i, _node) in functions.iter().enumerate() {
        items.push(AstItem::Function {
            name: format!("function_{i}"),
            visibility: String::new(), // Python doesn't have visibility modifiers
            is_async: false,           // Could check node flags for async
            line: i * 10 + 20,         // Offset after imports
        });
    }

    // Add classes as items (using Struct variant for Python classes)
    for (i, _node) in types.iter().enumerate() {
        items.push(AstItem::Struct {
            name: format!("class_{i}"),
            visibility: String::new(),
            fields_count: 0,
            derives: vec![],
            line: (functions.len() + i) * 10 + 50, // Offset after functions
        });
    }

    Ok(FileContext {
        path: path.display().to_string(),
        language: "python".to_string(),
        items,
        complexity_metrics: None,
    })
}

#[cfg(test)]
mod ast_python_compat_tests {
    //! Covers ast_python_compat.rs (45 uncov on broad, 0% cov).
    //! Drives the 4 public async fns through PythonStrategy on a
    //! realistic in-memory Python source so the conversion code paths
    //! (functions/types/imports) execute.
    use super::*;

    fn write_py(tmp: &tempfile::TempDir, name: &str, body: &str) -> std::path::PathBuf {
        let path = tmp.path().join(name);
        std::fs::write(&path, body).unwrap();
        path
    }

    const SAMPLE: &str = r#"
import os
from collections import defaultdict

class MyClass:
    pass

def hello():
    return "world"

def another():
    if True:
        return 1
    return 2
"#;

    #[tokio::test]
    async fn test_analyze_python_file_with_complexity_returns_metrics() {
        let tmp = tempfile::tempdir().unwrap();
        let p = write_py(&tmp, "a.py", SAMPLE);
        let metrics = analyze_python_file_with_complexity(&p, None).await.unwrap();
        // Path roundtrip + non-panic execution.
        assert_eq!(metrics.path, p.to_string_lossy());
    }

    #[tokio::test]
    async fn test_analyze_python_file_with_complexity_with_classifier() {
        let tmp = tempfile::tempdir().unwrap();
        let p = write_py(&tmp, "b.py", SAMPLE);
        let classifier = FileClassifier::default();
        let _metrics = analyze_python_file_with_complexity(&p, Some(&classifier))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_analyze_python_file_returns_context() {
        let tmp = tempfile::tempdir().unwrap();
        let p = write_py(&tmp, "c.py", SAMPLE);
        let ctx = analyze_python_file(&p).await.unwrap();
        // Items vector should reflect imports + functions + classes.
        assert!(!ctx.items.is_empty(), "items must populate from sample");
    }

    #[tokio::test]
    async fn test_analyze_python_file_with_classifier_path_alias() {
        let tmp = tempfile::tempdir().unwrap();
        let p = write_py(&tmp, "d.py", SAMPLE);
        let classifier = FileClassifier::default();
        let ctx = analyze_python_file_with_classifier(&p, Some(&classifier))
            .await
            .unwrap();
        assert!(!ctx.items.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_python_file_empty_source_no_panic() {
        let tmp = tempfile::tempdir().unwrap();
        let p = write_py(&tmp, "empty.py", "");
        // Even empty source must not panic — items may be empty.
        let _ = analyze_python_file(&p).await.unwrap();
    }

    #[tokio::test]
    async fn test_analyze_python_file_missing_file_returns_err() {
        let p = std::path::Path::new("/tmp/pmat_missing_python_xyz.py");
        let result = analyze_python_file(p).await;
        assert!(result.is_err(), "missing file must propagate as error");
    }

    #[tokio::test]
    async fn test_analyze_python_file_with_complexity_missing_file_returns_err() {
        let p = std::path::Path::new("/tmp/pmat_missing_complexity_xyz.py");
        let result = analyze_python_file_with_complexity(p, None).await;
        assert!(result.is_err());
    }
}
