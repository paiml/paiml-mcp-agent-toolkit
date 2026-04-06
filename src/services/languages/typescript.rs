#![cfg_attr(coverage_nightly, coverage(off))]
//! TypeScript language support module
//!
//! This module provides support for analyzing TypeScript code, including
//! AST parsing, syntax analysis, and code structure extraction.

use crate::services::context::AstItem;
use anyhow::Result;
use std::path::Path;

#[cfg(feature = "typescript-ast")]
use crate::services::ast_typescript::analyze_typescript_file;

/// Visitor for TypeScript AST analysis
pub struct TypeScriptAstVisitor {
    #[allow(dead_code)]
    path: std::path::PathBuf,
}

impl TypeScriptAstVisitor {
    /// Create a new TypeScript AST visitor
    pub fn new(path: &Path) -> Self {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        Self {
            path: path.to_path_buf(),
        }
    }

    /// Analyze TypeScript source code
    ///
    /// This method parses TypeScript source code and extracts AST items.
    /// It creates a temporary file to leverage the existing file-based parser.
    #[cfg(feature = "typescript-ast")]
    pub fn analyze_typescript_source(&self, source: &str) -> Result<Vec<AstItem>> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        // Create temporary file with .ts extension (builder pattern)
        let temp_file = tempfile::Builder::new()
            .suffix(".ts")
            .tempfile()
            .map_err(|e| anyhow::anyhow!("Failed to create temp file: {}", e))?;

        // Write source code to ephemeral file
        std::fs::write(temp_file.path(), source.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to write source to temp file: {}", e))?;

        // Use existing file-based parser
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| anyhow::anyhow!("Failed to create runtime: {}", e))?;

        runtime.block_on(async {
            let context = analyze_typescript_file(temp_file.path())
                .await
                .map_err(|e| anyhow::anyhow!("TypeScript parsing failed: {}", e))?;
            Ok(context.items)
        })
    }

    /// Analyze TypeScript source code (feature not enabled)
    #[cfg(not(feature = "typescript-ast"))]
    pub fn analyze_typescript_source(&self, _source: &str) -> Result<Vec<AstItem>> {
        // Return empty result when TypeScript AST feature is not enabled
        Ok(Vec::new())
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_typescript_ast_visitor_new() {
        let path = Path::new("test.ts");
        let visitor = TypeScriptAstVisitor::new(path);
        // Verify the visitor is created without panicking
        let _ = visitor;
    }

    #[test]
    fn test_typescript_ast_visitor_new_with_complex_path() {
        let path = Path::new("/src/components/App.tsx");
        let visitor = TypeScriptAstVisitor::new(path);
        let _ = visitor;
    }

    #[test]
    fn test_typescript_ast_visitor_new_with_relative_path() {
        let path = Path::new("./src/index.ts");
        let visitor = TypeScriptAstVisitor::new(path);
        let _ = visitor;
    }

    #[test]
    fn test_analyze_typescript_empty_source() {
        let path = Path::new("test.ts");
        let visitor = TypeScriptAstVisitor::new(path);

        let result = visitor.analyze_typescript_source("");
        assert!(result.is_ok());
        // When feature is not enabled, returns empty vec
        // When feature is enabled, might return empty vec for empty source
        let items = result.unwrap();
        // Verify result is valid; empty is acceptable for empty source
        let _ = items.len();
    }

    #[test]
    fn test_analyze_typescript_simple_function() {
        let path = Path::new("test.ts");
        let visitor = TypeScriptAstVisitor::new(path);

        let source = "function hello(): string { return 'world'; }";
        let result = visitor.analyze_typescript_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_typescript_interface() {
        let path = Path::new("test.ts");
        let visitor = TypeScriptAstVisitor::new(path);

        let source = r#"
            interface User {
                id: number;
                name: string;
                email: string;
            }
        "#;
        let result = visitor.analyze_typescript_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_typescript_class_with_types() {
        let path = Path::new("test.ts");
        let visitor = TypeScriptAstVisitor::new(path);

        let source = r#"
            class Calculator {
                add(a: number, b: number): number {
                    return a + b;
                }
                subtract(a: number, b: number): number {
                    return a - b;
                }
            }
        "#;
        let result = visitor.analyze_typescript_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_typescript_generics() {
        let path = Path::new("test.ts");
        let visitor = TypeScriptAstVisitor::new(path);

        let source = r#"
            function identity<T>(arg: T): T {
                return arg;
            }

            interface Container<T> {
                value: T;
                getValue(): T;
            }
        "#;
        let result = visitor.analyze_typescript_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_typescript_with_imports() {
        let path = Path::new("test.ts");
        let visitor = TypeScriptAstVisitor::new(path);

        // Use plain TypeScript without JSX
        let source = r#"
            import axios, { AxiosResponse } from 'axios';
            import type { Config, Options } from './types';

            interface Props {
                name: string;
                count: number;
            }

            const processData = (props: Props): string => {
                const { name, count } = props;
                return `${name}: ${count}`;
            };

            export default processData;
        "#;
        let result = visitor.analyze_typescript_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_typescript_type_alias() {
        let path = Path::new("test.ts");
        let visitor = TypeScriptAstVisitor::new(path);

        let source = r#"
            type StringOrNumber = string | number;
            type AsyncResult<T> = Promise<T>;
            type UserCallback = (user: User) => void;
        "#;
        let result = visitor.analyze_typescript_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_typescript_enum() {
        let path = Path::new("test.ts");
        let visitor = TypeScriptAstVisitor::new(path);

        let source = r#"
            enum Direction {
                Up = "UP",
                Down = "DOWN",
                Left = "LEFT",
                Right = "RIGHT"
            }
        "#;
        let result = visitor.analyze_typescript_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_typescript_ast_visitor_preserves_path() {
        let path = Path::new("/absolute/path/to/file.ts");
        let _visitor = TypeScriptAstVisitor::new(path);
        // The path is stored internally, this just verifies no panic
    }
}
