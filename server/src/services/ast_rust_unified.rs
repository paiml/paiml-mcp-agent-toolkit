//! Rust AST Parser implementing UnifiedAstParser trait
//!
//! This module provides a unified interface to the Rust AST parser,
//! following the architectural consolidation pattern.

use super::ast_rust;
use super::unified_ast_parser::{ParseResult, ParserCapabilities, ParserConfig, UnifiedAstParser};
use crate::models::error::PmatError;
use crate::models::unified_ast::{
    AstKind, FunctionKind, Language, NodeFlags, NodeMetadata, TypeKind, UnifiedAstNode,
};
use crate::services::{
    complexity::ComplexityMetrics,
    context::{AstItem, FileContext},
    file_classifier::FileClassifier,
};
use async_trait::async_trait;
use std::path::Path;
use syn::Item;

/// Rust parser implementing the unified interface
pub struct RustAstParser {}

impl RustAstParser {
    pub fn new() -> Self {
        Self {}
    }

    fn extract_function_item(func: &syn::ItemFn) -> AstItem {
        AstItem::Function {
            name: func.sig.ident.to_string(),
            visibility: match func.vis {
                syn::Visibility::Public(_) => "pub".to_string(),
                _ => "private".to_string(),
            },
            is_async: func.sig.asyncness.is_some(),
            line: 0, // Would extract from span in full implementation
        }
    }

    fn extract_struct_item(s: &syn::ItemStruct) -> AstItem {
        AstItem::Struct {
            name: s.ident.to_string(),
            visibility: match s.vis {
                syn::Visibility::Public(_) => "pub".to_string(),
                _ => "private".to_string(),
            },
            fields_count: match &s.fields {
                syn::Fields::Named(fields) => fields.named.len(),
                syn::Fields::Unnamed(fields) => fields.unnamed.len(),
                syn::Fields::Unit => 0,
            },
            derives: Vec::new(), // Would extract derive attributes in full implementation
            line: 0,
        }
    }

    fn extract_enum_item(e: &syn::ItemEnum) -> AstItem {
        AstItem::Enum {
            name: e.ident.to_string(),
            visibility: match e.vis {
                syn::Visibility::Public(_) => "pub".to_string(),
                _ => "private".to_string(),
            },
            variants_count: e.variants.len(),
            line: 0,
        }
    }

    fn extract_trait_item(t: &syn::ItemTrait) -> AstItem {
        AstItem::Trait {
            name: t.ident.to_string(),
            visibility: match t.vis {
                syn::Visibility::Public(_) => "pub".to_string(),
                _ => "private".to_string(),
            },
            line: 0,
        }
    }

    fn extract_impl_item(i: &syn::ItemImpl) -> Option<AstItem> {
        if let syn::Type::Path(type_path) = &*i.self_ty {
            if let Some(segment) = type_path.path.segments.last() {
                return Some(AstItem::Impl {
                    type_name: segment.ident.to_string(),
                    trait_name: i.trait_.as_ref().map(|(_, trait_path, _)| {
                        if let Some(segment) = trait_path.segments.last() {
                            segment.ident.to_string()
                        } else {
                            "Unknown".to_string()
                        }
                    }),
                    line: 0,
                });
            }
        }
        None
    }

    fn extract_use_item(_u: &syn::ItemUse) -> AstItem {
        AstItem::Use {
            path: "use_path".to_string(), // Simplified for now
            line: 0,
        }
    }

    fn create_function_node() -> UnifiedAstNode {
        UnifiedAstNode {
            kind: AstKind::Function(FunctionKind::Regular),
            lang: Language::Rust,
            flags: NodeFlags::default(),
            parent: 0,
            first_child: 0,
            next_sibling: 0,
            source_range: 0..0,                       // Would extract from span
            semantic_hash: 0,                         // Would compute semantic hash
            structural_hash: 0,                       // Would compute structural hash
            name_vector: 0,                           // Would compute name embedding
            metadata: NodeMetadata { complexity: 1 }, // Basic complexity
            proof_annotations: None,
        }
    }

    fn create_struct_node() -> UnifiedAstNode {
        UnifiedAstNode {
            kind: AstKind::Type(TypeKind::Struct),
            lang: Language::Rust,
            flags: NodeFlags::default(),
            parent: 0,
            first_child: 0,
            next_sibling: 0,
            source_range: 0..0,
            semantic_hash: 0,
            structural_hash: 0,
            name_vector: 0,
            metadata: NodeMetadata { raw: 0 },
            proof_annotations: None,
        }
    }
}

impl Default for RustAstParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedAstParser for RustAstParser {
    async fn parse_file(
        &self,
        path: &Path,
        config: &ParserConfig,
        classifier: Option<&FileClassifier>,
    ) -> Result<ParseResult, PmatError> {
        // Validate file first
        self.validate_file(path, config, classifier)?;

        // Read file content
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(PmatError::Io)?;

        self.parse_content(&content, path, config).await
    }

    async fn parse_content(
        &self,
        content: &str,
        file_path: &Path,
        config: &ParserConfig,
    ) -> Result<ParseResult, PmatError> {
        // Parse with existing Rust parser for complexity
        let complexity = if config.extract_complexity {
            ast_rust::analyze_rust_file_with_complexity(file_path)
                .await
                .map_err(PmatError::Template)?
        } else {
            // Return empty complexity metrics if not requested
            crate::services::complexity::FileComplexityMetrics {
                path: file_path.display().to_string(),
                total_complexity: ComplexityMetrics::default(),
                functions: Vec::new(),
                classes: Vec::new(),
            }
        };

        // Parse AST for unified analysis
        let ast = syn::parse_file(content).map_err(|e| PmatError::ParseError {
            file: file_path.to_path_buf(),
            line: None,
            message: format!("Rust parse error: {e}"),
        })?;

        let mut context_builder = FileContextBuilder::new(file_path);
        let mut unified_nodes = Vec::new();
        let warnings = Vec::new();

        // Extract items and create unified nodes
        for item in &ast.items {
            if let Some(ast_item) = self.extract_ast_item(item, &mut []) {
                context_builder.add_item(ast_item);
            }

            if config.compute_semantic_hashes {
                if let Some(unified_node) = self.create_unified_node(item, &mut []) {
                    unified_nodes.push(unified_node);
                }
            }
        }

        let context = context_builder.build();

        Ok(ParseResult {
            context,
            complexity,
            unified_nodes,
            warnings,
        })
    }

    fn capabilities(&self) -> ParserCapabilities {
        ParserCapabilities {
            language: Language::Rust,
            extensions: vec!["rs"],
            supports_complexity: true,
            supports_dependencies: true,
            supports_semantics: true,
            simd_optimized: false, // Will be optimized in Phase 7
        }
    }

    fn name(&self) -> &'static str {
        "RustAstParser"
    }
}

impl RustAstParser {
    fn extract_ast_item(&self, item: &Item, _warnings: &mut [String]) -> Option<AstItem> {
        match item {
            Item::Fn(func) => Some(Self::extract_function_item(func)),
            Item::Struct(s) => Some(Self::extract_struct_item(s)),
            Item::Enum(e) => Some(Self::extract_enum_item(e)),
            Item::Trait(t) => Some(Self::extract_trait_item(t)),
            Item::Impl(i) => Self::extract_impl_item(i),
            Item::Use(u) => Some(Self::extract_use_item(u)),
            _ => None,
        }
    }

    fn create_unified_node(&self, item: &Item, _warnings: &mut [String]) -> Option<UnifiedAstNode> {
        match item {
            Item::Fn(_func) => Some(Self::create_function_node()),
            Item::Struct(_) => Some(Self::create_struct_node()),
            // Add other item types as needed
            _ => None,
        }
    }
}

/// Helper for building FileContext incrementally
struct FileContextBuilder {
    file_path: String,
    items: Vec<AstItem>,
}

impl FileContextBuilder {
    fn new(path: &Path) -> Self {
        Self {
            file_path: path.display().to_string(),
            items: Vec::new(),
        }
    }

    fn add_item(&mut self, item: AstItem) {
        self.items.push(item);
    }

    fn build(self) -> FileContext {
        FileContext {
            path: self.file_path,
            language: "rust".to_string(),
            items: self.items,
            complexity_metrics: None, // Will be set separately
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rust_parser_capabilities() {
        let parser = RustAstParser::new();
        let caps = parser.capabilities();
        assert_eq!(caps.language, Language::Rust);
        assert_eq!(caps.extensions, vec!["rs"]);
        assert!(caps.supports_complexity);
        assert!(caps.supports_dependencies);
        assert!(caps.supports_semantics);
    }

    #[tokio::test]
    async fn test_rust_parser_can_parse() {
        let parser = RustAstParser::new();
        assert!(parser.can_parse(Path::new("test.rs")));
        assert!(!parser.can_parse(Path::new("test.py")));
        assert!(!parser.can_parse(Path::new("test.js")));
    }

    #[tokio::test]
    async fn test_rust_parser_parse_content() {
        let parser = RustAstParser::new();
        let config = ParserConfig {
            extract_complexity: false, // Don't extract complexity to avoid file I/O
            ..Default::default()
        };

        let rust_code = r#"
            fn hello_world() {
                println!("Hello, world!");
            }
            
            struct Point {
                x: i32,
                y: i32,
            }
        "#;

        let result = parser
            .parse_content(rust_code, Path::new("test.rs"), &config)
            .await;

        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        let parse_result = result.unwrap();

        assert_eq!(parse_result.context.items.len(), 2);
        assert_eq!(parse_result.context.language, "rust");

        // Check for function
        assert!(parse_result.context.items.iter().any(|item| {
            matches!(item, AstItem::Function { name, .. } if name == "hello_world")
        }));

        // Check for struct
        assert!(parse_result
            .context
            .items
            .iter()
            .any(|item| { matches!(item, AstItem::Struct { name, .. } if name == "Point") }));
    }
}
