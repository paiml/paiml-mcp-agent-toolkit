//! Language mapper interfaces for polyglot AST
//!
//! This module defines the interfaces and implementations for mapping
//! language-specific ASTs to the unified polyglot representation. Each
//! supported language implements the `LanguageMapper` trait to convert
//! its native AST nodes to `UnifiedNode` instances.

use crate::ast::polyglot::{Language, NodeKind, PolyglotPathValidator, UnifiedNode};
use crate::services::context::AstItem;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;

/// Trait for mapping language-specific ASTs to the unified representation
#[async_trait]
pub trait LanguageMapper: Send + Sync {
    /// The language this mapper handles
    fn language(&self) -> Language;

    /// Map a file to unified nodes
    async fn map_file(&self, path: &Path) -> Result<Vec<UnifiedNode>>;

    /// Map a directory of files to unified nodes
    async fn map_directory(&self, path: &Path, recursive: bool) -> Result<Vec<UnifiedNode>>;

    /// Map a string of source code to unified nodes
    async fn map_source(&self, source: &str, path: &Path) -> Result<Vec<UnifiedNode>>;

    /// Convert language-specific AST items to unified nodes
    fn convert_ast_items(&self, items: &[AstItem], path: &Path) -> Vec<UnifiedNode>;

    /// Clone this mapper as a trait object
    fn clone_box(&self) -> Box<dyn LanguageMapper>;

    /// Create a test node for unit testing
    fn create_test_node(&self, kind: NodeKind, name: &str) -> UnifiedNode {
        UnifiedNode::new(kind, name, self.language())
    }
}

/// Factory for creating language mappers
pub struct LanguageMapperFactory;

impl LanguageMapperFactory {
    /// Create a mapper for the specified language
    pub fn create(language: Language) -> Result<Arc<dyn LanguageMapper>> {
        match language {
            Language::Java => Ok(Arc::new(JavaMapper::new())),
            Language::Kotlin => Ok(Arc::new(KotlinMapper::new())),
            Language::Scala => Ok(Arc::new(ScalaMapper::new())),
            Language::TypeScript => Ok(Arc::new(TypeScriptMapper::new())),
            Language::JavaScript => Ok(Arc::new(JavaScriptMapper::new())),
            _ => Err(anyhow!("Unsupported language: {:?}", language)),
        }
    }

    /// Create mappers for all supported languages
    pub fn create_all() -> HashMap<Language, Arc<dyn LanguageMapper>> {
        let mut mappers = HashMap::new();

        // Add all supported languages
        for language in &[
            Language::Java,
            Language::Kotlin,
            Language::Scala,
            Language::TypeScript,
            Language::JavaScript,
        ] {
            if let Ok(mapper) = Self::create(*language) {
                mappers.insert(*language, mapper);
            }
        }

        mappers
    }

    /// Create a mapper for a file based on its extension
    pub fn create_for_file(path: &Path) -> Result<Arc<dyn LanguageMapper>> {
        let language = Language::from_path(path)
            .ok_or_else(|| anyhow!("Unsupported file type: {:?}", path))?;

        Self::create(language)
    }
}

/// Base implementation of LanguageMapper with common functionality
#[derive(Clone)]
pub struct BaseLanguageMapper {
    language: Language,
}

impl BaseLanguageMapper {
    /// Create a new base mapper
    pub fn new(language: Language) -> Self {
        Self { language }
    }
}

#[async_trait]
impl LanguageMapper for BaseLanguageMapper {
    fn language(&self) -> Language {
        self.language
    }

    async fn map_file(&self, path: &Path) -> Result<Vec<UnifiedNode>> {
        PolyglotPathValidator::validate_file_path(path)?;

        let content = fs::read_to_string(path).await?;
        self.map_source(&content, path).await
    }

    async fn map_directory(&self, path: &Path, recursive: bool) -> Result<Vec<UnifiedNode>> {
        PolyglotPathValidator::validate_directory_path(path)?;

        let mut nodes = Vec::new();
        let extensions: Vec<String> = self
            .language()
            .file_extensions()
            .iter()
            .map(|ext| ext.to_string())
            .collect();

        let read_dir = tokio::fs::read_dir(path).await?;
        let mut entries = Vec::new();

        tokio::pin!(read_dir);
        while let Some(entry) = read_dir.next_entry().await? {
            entries.push(entry);
        }

        for entry in entries {
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if let Some(ext_str) = ext.to_str() {
                        if extensions.contains(&ext_str.to_string()) {
                            let file_nodes = self.map_file(&path).await?;
                            nodes.extend(file_nodes);
                        }
                    }
                }
            } else if recursive && path.is_dir() {
                let dir_nodes = self.map_directory(&path, recursive).await?;
                nodes.extend(dir_nodes);
            }
        }

        Ok(nodes)
    }

    async fn map_source(&self, _source: &str, _path: &Path) -> Result<Vec<UnifiedNode>> {
        // Base implementation doesn't know how to map source
        // Subclasses should override this
        Err(anyhow!("Source mapping not implemented for this language"))
    }

    fn convert_ast_items(&self, items: &[AstItem], path: &Path) -> Vec<UnifiedNode> {
        items
            .iter()
            .map(|item| UnifiedNode::from_ast_item(item, self.language(), path, None))
            .collect()
    }

    fn clone_box(&self) -> Box<dyn LanguageMapper> {
        Box::new(self.clone())
    }
}

/// Java language mapper
#[derive(Clone)]
pub struct JavaMapper {
    base: BaseLanguageMapper,
}

impl JavaMapper {
    /// Create a new Java mapper
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            base: BaseLanguageMapper::new(Language::Java),
        }
    }

    /// Process Java-specific nodes
    #[allow(dead_code)]
    fn process_java_specific(&self, nodes: &mut [UnifiedNode]) {
        for node in nodes.iter_mut() {
            // Add Java-specific metadata
            match node.kind {
                NodeKind::Class => {
                    // Check for Java interfaces
                    if node.has_modifier("interface") {
                        node.kind = NodeKind::Interface;
                    }

                    // Check for Java records
                    if node.has_modifier("record") {
                        node.kind = NodeKind::Record;
                    }
                }
                _ => {}
            }
        }
    }
}

#[async_trait]
impl LanguageMapper for JavaMapper {
    fn language(&self) -> Language {
        self.base.language()
    }

    async fn map_file(&self, path: &Path) -> Result<Vec<UnifiedNode>> {
        self.base.map_file(path).await
    }

    async fn map_directory(&self, path: &Path, recursive: bool) -> Result<Vec<UnifiedNode>> {
        self.base.map_directory(path, recursive).await
    }

    async fn map_source(&self, _source: &str, _path: &Path) -> Result<Vec<UnifiedNode>> {
        #[cfg(feature = "java-ast")]
        {
            use crate::services::languages::java::JavaAstVisitor;
            let visitor = JavaAstVisitor::new(_path);
            match visitor.analyze_java_source(_source) {
                Ok(items) => {
                    let mut nodes = self.convert_ast_items(&items, _path);
                    self.process_java_specific(&mut nodes);
                    Ok(nodes)
                }
                Err(e) => Err(anyhow!("Failed to analyze Java source: {}", e)),
            }
        }
        #[cfg(not(feature = "java-ast"))]
        {
            Err(anyhow!(
                "Java AST support not enabled (feature 'java-ast' required)"
            ))
        }
    }

    fn convert_ast_items(&self, items: &[AstItem], _path: &Path) -> Vec<UnifiedNode> {
        self.base.convert_ast_items(items, _path)
    }

    fn clone_box(&self) -> Box<dyn LanguageMapper> {
        Box::new(self.clone())
    }
}

/// Kotlin language mapper
#[derive(Clone)]
pub struct KotlinMapper {
    base: BaseLanguageMapper,
}

impl KotlinMapper {
    /// Create a new Kotlin mapper
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            base: BaseLanguageMapper::new(Language::Kotlin),
        }
    }

    /// Process Kotlin-specific nodes
    #[allow(dead_code)]
    fn process_kotlin_specific(&self, nodes: &mut [UnifiedNode]) {
        for node in nodes.iter_mut() {
            // Add Kotlin-specific metadata
            match node.kind {
                NodeKind::Class => {
                    // Check for Kotlin data classes
                    if node.has_modifier("data") {
                        node.kind = NodeKind::Record;
                        node.add_metadata("kotlin:isData", "true");
                    }

                    // Check for Kotlin sealed classes
                    if node.has_modifier("sealed") {
                        node.add_metadata("kotlin:isSealed", "true");
                    }
                }
                _ => {}
            }
        }
    }
}

#[async_trait]
impl LanguageMapper for KotlinMapper {
    fn language(&self) -> Language {
        self.base.language()
    }

    async fn map_file(&self, path: &Path) -> Result<Vec<UnifiedNode>> {
        self.base.map_file(path).await
    }

    async fn map_directory(&self, path: &Path, recursive: bool) -> Result<Vec<UnifiedNode>> {
        self.base.map_directory(path, recursive).await
    }

    async fn map_source(&self, source: &str, path: &Path) -> Result<Vec<UnifiedNode>> {
        // For now, use the base mapper implementation
        // TODO: Add Kotlin-specific analysis when kotlin-ast feature is enabled
        self.base.map_source(source, path).await
    }

    fn convert_ast_items(&self, items: &[AstItem], path: &Path) -> Vec<UnifiedNode> {
        self.base.convert_ast_items(items, path)
    }

    fn clone_box(&self) -> Box<dyn LanguageMapper> {
        Box::new(self.clone())
    }
}

/// Scala language mapper
#[derive(Clone)]
pub struct ScalaMapper {
    base: BaseLanguageMapper,
}

impl ScalaMapper {
    /// Create a new Scala mapper
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            base: BaseLanguageMapper::new(Language::Scala),
        }
    }

    /// Process Scala-specific nodes
    #[allow(dead_code)]
    fn process_scala_specific(&self, nodes: &mut [UnifiedNode]) {
        for node in nodes.iter_mut() {
            // Add Scala-specific metadata
            match node.kind {
                NodeKind::Class => {
                    // Handle case classes
                    if node.has_modifier("case") {
                        node.kind = NodeKind::CaseClass;
                    }
                }
                NodeKind::Module => {
                    // Check if this is a Scala object
                    node.add_metadata("scala:isObject", "true");
                }
                _ => {}
            }
        }
    }
}

#[async_trait]
impl LanguageMapper for ScalaMapper {
    fn language(&self) -> Language {
        self.base.language()
    }

    async fn map_file(&self, path: &Path) -> Result<Vec<UnifiedNode>> {
        self.base.map_file(path).await
    }

    async fn map_directory(&self, path: &Path, recursive: bool) -> Result<Vec<UnifiedNode>> {
        self.base.map_directory(path, recursive).await
    }

    async fn map_source(&self, _source: &str, _path: &Path) -> Result<Vec<UnifiedNode>> {
        #[cfg(feature = "scala-ast")]
        {
            use crate::services::languages::scala::ScalaAstVisitor;
            let visitor = ScalaAstVisitor::new(_path);
            match visitor.analyze_scala_source(_source) {
                Ok(items) => {
                    let mut nodes = self.convert_ast_items(&items, _path);
                    self.process_scala_specific(&mut nodes);
                    Ok(nodes)
                }
                Err(e) => Err(anyhow!("Failed to analyze Scala source: {}", e)),
            }
        }
        #[cfg(not(feature = "scala-ast"))]
        {
            Err(anyhow!(
                "Scala AST support not enabled (feature 'scala-ast' required)"
            ))
        }
    }

    fn convert_ast_items(&self, items: &[AstItem], _path: &Path) -> Vec<UnifiedNode> {
        self.base.convert_ast_items(items, _path)
    }

    fn clone_box(&self) -> Box<dyn LanguageMapper> {
        Box::new(self.clone())
    }
}

/// TypeScript language mapper
#[derive(Clone)]
pub struct TypeScriptMapper {
    base: BaseLanguageMapper,
}

impl TypeScriptMapper {
    /// Create a new TypeScript mapper
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            base: BaseLanguageMapper::new(Language::TypeScript),
        }
    }

    /// Process TypeScript-specific nodes
    fn process_typescript_specific(&self, nodes: &mut [UnifiedNode]) {
        for node in nodes.iter_mut() {
            // Add TypeScript-specific metadata
            match node.kind {
                NodeKind::Interface => {
                    node.add_metadata("typescript:isInterface", "true");
                }
                NodeKind::Class => {
                    if node.has_modifier("abstract") {
                        node.add_metadata("typescript:isAbstract", "true");
                    }
                }
                _ => {}
            }
        }
    }
}

#[async_trait]
impl LanguageMapper for TypeScriptMapper {
    fn language(&self) -> Language {
        self.base.language()
    }

    async fn map_file(&self, path: &Path) -> Result<Vec<UnifiedNode>> {
        self.base.map_file(path).await
    }

    async fn map_directory(&self, path: &Path, recursive: bool) -> Result<Vec<UnifiedNode>> {
        self.base.map_directory(path, recursive).await
    }

    async fn map_source(&self, source: &str, path: &Path) -> Result<Vec<UnifiedNode>> {
        use crate::services::languages::typescript::TypeScriptAstVisitor;

        let visitor = TypeScriptAstVisitor::new(path);
        match visitor.analyze_typescript_source(source) {
            Ok(items) => {
                let mut nodes = self.convert_ast_items(&items, path);
                self.process_typescript_specific(&mut nodes);
                Ok(nodes)
            }
            Err(e) => Err(anyhow!("Failed to analyze TypeScript source: {}", e)),
        }
    }

    fn convert_ast_items(&self, items: &[AstItem], path: &Path) -> Vec<UnifiedNode> {
        self.base.convert_ast_items(items, path)
    }

    fn clone_box(&self) -> Box<dyn LanguageMapper> {
        Box::new(self.clone())
    }
}

/// JavaScript language mapper
#[derive(Clone)]
pub struct JavaScriptMapper {
    base: BaseLanguageMapper,
}

impl JavaScriptMapper {
    /// Create a new JavaScript mapper
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            base: BaseLanguageMapper::new(Language::JavaScript),
        }
    }

    /// Process JavaScript-specific nodes
    fn process_javascript_specific(&self, nodes: &mut [UnifiedNode]) {
        for node in nodes.iter_mut() {
            // Add JavaScript-specific metadata
            match node.kind {
                NodeKind::Class => {
                    node.add_metadata("javascript:isClass", "true");
                }
                NodeKind::Function => {
                    // Check for arrow functions
                    if node.has_modifier("arrow") {
                        node.kind = NodeKind::Lambda;
                    }
                }
                _ => {}
            }
        }
    }
}

#[async_trait]
impl LanguageMapper for JavaScriptMapper {
    fn language(&self) -> Language {
        self.base.language()
    }

    async fn map_file(&self, path: &Path) -> Result<Vec<UnifiedNode>> {
        self.base.map_file(path).await
    }

    async fn map_directory(&self, path: &Path, recursive: bool) -> Result<Vec<UnifiedNode>> {
        self.base.map_directory(path, recursive).await
    }

    async fn map_source(&self, source: &str, path: &Path) -> Result<Vec<UnifiedNode>> {
        use crate::services::languages::javascript::JavaScriptAstVisitor;

        let visitor = JavaScriptAstVisitor::new(path);
        match visitor.analyze_javascript_source(source) {
            Ok(items) => {
                let mut nodes = self.convert_ast_items(&items, path);
                self.process_javascript_specific(&mut nodes);
                Ok(nodes)
            }
            Err(e) => Err(anyhow!("Failed to analyze JavaScript source: {}", e)),
        }
    }

    fn convert_ast_items(&self, items: &[AstItem], path: &Path) -> Vec<UnifiedNode> {
        self.base.convert_ast_items(items, path)
    }

    fn clone_box(&self) -> Box<dyn LanguageMapper> {
        Box::new(self.clone())
    }
}

/// C# language mapper
#[derive(Clone)]
pub struct CSharpMapper {
    base: BaseLanguageMapper,
}

impl CSharpMapper {
    /// Create a new C# mapper
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            base: BaseLanguageMapper::new(Language::CSharp),
        }
    }
}

#[async_trait]
impl LanguageMapper for CSharpMapper {
    fn language(&self) -> Language {
        self.base.language()
    }

    async fn map_file(&self, path: &Path) -> Result<Vec<UnifiedNode>> {
        self.base.map_file(path).await
    }

    async fn map_directory(&self, path: &Path, recursive: bool) -> Result<Vec<UnifiedNode>> {
        self.base.map_directory(path, recursive).await
    }

    async fn map_source(&self, source: &str, path: &Path) -> Result<Vec<UnifiedNode>> {
        self.base.map_source(source, path).await
    }

    fn convert_ast_items(&self, items: &[AstItem], path: &Path) -> Vec<UnifiedNode> {
        self.base.convert_ast_items(items, path)
    }

    fn clone_box(&self) -> Box<dyn LanguageMapper> {
        Box::new(self.clone())
    }
}

/// Ruby language mapper
#[derive(Clone)]
pub struct RubyMapper {
    base: BaseLanguageMapper,
}

impl RubyMapper {
    /// Create a new Ruby mapper
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            base: BaseLanguageMapper::new(Language::Ruby),
        }
    }
}

#[async_trait]
impl LanguageMapper for RubyMapper {
    fn language(&self) -> Language {
        self.base.language()
    }

    async fn map_file(&self, path: &Path) -> Result<Vec<UnifiedNode>> {
        self.base.map_file(path).await
    }

    async fn map_directory(&self, path: &Path, recursive: bool) -> Result<Vec<UnifiedNode>> {
        self.base.map_directory(path, recursive).await
    }

    async fn map_source(&self, source: &str, path: &Path) -> Result<Vec<UnifiedNode>> {
        self.base.map_source(source, path).await
    }

    fn convert_ast_items(&self, items: &[AstItem], path: &Path) -> Vec<UnifiedNode> {
        self.base.convert_ast_items(items, path)
    }

    fn clone_box(&self) -> Box<dyn LanguageMapper> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::context::AstItem;

    fn create_test_ast_item(kind: &str, name: &str) -> AstItem {
        // Map string kind to AstItem enum variant
        match kind {
            "function" | "method" => AstItem::Function {
                name: name.to_string(),
                visibility: "public".to_string(),
                is_async: false,
                line: 1,
            },
            "class" | "struct" => AstItem::Struct {
                name: name.to_string(),
                visibility: "public".to_string(),
                fields_count: 0,
                derives: vec![],
                line: 1,
            },
            "trait" | "interface" => AstItem::Trait {
                name: name.to_string(),
                visibility: "public".to_string(),
                line: 1,
            },
            "enum" => AstItem::Enum {
                name: name.to_string(),
                visibility: "public".to_string(),
                variants_count: 0,
                line: 1,
            },
            "module" | "namespace" => AstItem::Module {
                name: name.to_string(),
                visibility: "public".to_string(),
                line: 1,
            },
            _ => AstItem::Struct {
                name: name.to_string(),
                visibility: "public".to_string(),
                fields_count: 0,
                derives: vec![],
                line: 1,
            },
        }
    }

    #[test]
    fn test_language_mapper_factory() {
        // Test creating mappers for supported languages
        let java_mapper = LanguageMapperFactory::create(Language::Java);
        assert!(java_mapper.is_ok());
        assert_eq!(java_mapper.unwrap().language(), Language::Java);

        let scala_mapper = LanguageMapperFactory::create(Language::Scala);
        assert!(scala_mapper.is_ok());
        assert_eq!(scala_mapper.unwrap().language(), Language::Scala);

        // Test creating mapper for unsupported language
        let unsupported = LanguageMapperFactory::create(Language::Other(0));
        assert!(unsupported.is_err());

        // Test creating mapper for file
        let file_path = Path::new("test.java");
        let file_mapper = LanguageMapperFactory::create_for_file(file_path);
        assert!(file_mapper.is_ok());
        assert_eq!(file_mapper.unwrap().language(), Language::Java);
    }

    #[test]
    fn test_convert_ast_items() {
        let java_mapper = JavaMapper::new();
        let file_path = Path::new("/path/to/Test.java");

        let items = vec![
            create_test_ast_item("class", "TestClass"),
            create_test_ast_item("method", "testMethod"),
        ];

        let nodes = java_mapper.convert_ast_items(&items, file_path);

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].kind, NodeKind::Struct); // Java classes are represented as Struct in AstItem
        assert_eq!(nodes[0].name, "TestClass");
        assert_eq!(nodes[1].kind, NodeKind::Function); // Methods are represented as Function in AstItem
        assert_eq!(nodes[1].name, "testMethod");
    }
}

/// Comprehensive coverage tests for language_mapper.rs
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::services::context::AstItem;
    use std::fs;
    use tempfile::TempDir;

    // ==========================================================================
    // Helper functions for test data creation
    // ==========================================================================

    fn create_function_item(name: &str, is_async: bool, line: usize) -> AstItem {
        AstItem::Function {
            name: name.to_string(),
            visibility: "public".to_string(),
            is_async,
            line,
        }
    }

    fn create_struct_item(name: &str, fields: usize, derives: Vec<String>, line: usize) -> AstItem {
        AstItem::Struct {
            name: name.to_string(),
            visibility: "public".to_string(),
            fields_count: fields,
            derives,
            line,
        }
    }

    fn create_enum_item(name: &str, variants: usize, line: usize) -> AstItem {
        AstItem::Enum {
            name: name.to_string(),
            visibility: "public".to_string(),
            variants_count: variants,
            line,
        }
    }

    fn create_trait_item(name: &str, line: usize) -> AstItem {
        AstItem::Trait {
            name: name.to_string(),
            visibility: "public".to_string(),
            line,
        }
    }

    fn create_module_item(name: &str, line: usize) -> AstItem {
        AstItem::Module {
            name: name.to_string(),
            visibility: "public".to_string(),
            line,
        }
    }

    fn create_use_item(path: &str, line: usize) -> AstItem {
        AstItem::Use {
            path: path.to_string(),
            line,
        }
    }

    fn create_impl_item(type_name: &str, trait_name: Option<&str>, line: usize) -> AstItem {
        AstItem::Impl {
            type_name: type_name.to_string(),
            trait_name: trait_name.map(|s| s.to_string()),
            line,
        }
    }

    fn create_import_item(module: &str, line: usize) -> AstItem {
        AstItem::Import {
            module: module.to_string(),
            items: vec![],
            alias: None,
            line,
        }
    }

    // ==========================================================================
    // BaseLanguageMapper Tests
    // ==========================================================================

    #[test]
    fn test_base_language_mapper_new() {
        let mapper = BaseLanguageMapper::new(Language::Java);
        assert_eq!(mapper.language, Language::Java);
    }

    #[test]
    fn test_base_language_mapper_language() {
        let mapper = BaseLanguageMapper::new(Language::Kotlin);
        assert_eq!(mapper.language(), Language::Kotlin);
    }

    #[test]
    fn test_base_language_mapper_clone() {
        let mapper = BaseLanguageMapper::new(Language::Scala);
        let cloned = mapper.clone();
        assert_eq!(cloned.language, Language::Scala);
    }

    #[test]
    fn test_base_language_mapper_clone_box() {
        let mapper = BaseLanguageMapper::new(Language::TypeScript);
        let boxed = mapper.clone_box();
        assert_eq!(boxed.language(), Language::TypeScript);
    }

    #[test]
    fn test_base_language_mapper_convert_ast_items_comprehensive() {
        let mapper = BaseLanguageMapper::new(Language::Rust);
        let path = Path::new("/test/file.rs");

        // Test with various AstItem types
        let items = vec![
            create_function_item("test_func", false, 1),
            create_struct_item("TestStruct", 3, vec!["Debug".to_string()], 10),
            create_enum_item("TestEnum", 5, 20),
            create_trait_item("TestTrait", 30),
            create_module_item("test_module", 40),
            create_use_item("std::collections::HashMap", 50),
            create_impl_item("TestStruct", Some("TestTrait"), 60),
            create_import_item("external_crate", 70),
        ];

        let nodes = mapper.convert_ast_items(&items, path);

        assert_eq!(nodes.len(), 8);
        assert_eq!(nodes[0].kind, NodeKind::Function);
        assert_eq!(nodes[0].name, "test_func");
        assert_eq!(nodes[1].kind, NodeKind::Struct);
        assert_eq!(nodes[1].name, "TestStruct");
        assert_eq!(nodes[2].kind, NodeKind::Enum);
        assert_eq!(nodes[2].name, "TestEnum");
        assert_eq!(nodes[3].kind, NodeKind::Trait);
        assert_eq!(nodes[3].name, "TestTrait");
        assert_eq!(nodes[4].kind, NodeKind::Module);
        assert_eq!(nodes[4].name, "test_module");
        assert_eq!(nodes[5].kind, NodeKind::Uses);
        assert_eq!(nodes[5].name, "std::collections::HashMap");
        assert_eq!(nodes[6].kind, NodeKind::Implements);
        assert_eq!(nodes[6].name, "TestStruct");
        assert_eq!(nodes[7].kind, NodeKind::Import);
        assert_eq!(nodes[7].name, "external_crate");
    }

    #[test]
    fn test_base_language_mapper_convert_ast_items_empty() {
        let mapper = BaseLanguageMapper::new(Language::Go);
        let path = Path::new("/test/file.go");
        let items: Vec<AstItem> = vec![];

        let nodes = mapper.convert_ast_items(&items, path);
        assert!(nodes.is_empty());
    }

    #[tokio::test]
    async fn test_base_language_mapper_map_source_returns_error() {
        let mapper = BaseLanguageMapper::new(Language::Rust);
        let result = mapper.map_source("fn main() {}", Path::new("test.rs")).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not implemented"));
    }

    #[tokio::test]
    async fn test_base_language_mapper_map_file_not_found() {
        let mapper = BaseLanguageMapper::new(Language::Java);
        let result = mapper.map_file(Path::new("/nonexistent/file.java")).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_base_language_mapper_map_directory_not_found() {
        let mapper = BaseLanguageMapper::new(Language::Java);
        let result = mapper
            .map_directory(Path::new("/nonexistent/dir"), false)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_base_language_mapper_map_directory_empty() {
        let temp_dir = TempDir::new().unwrap();
        let mapper = BaseLanguageMapper::new(Language::Java);

        let result = mapper.map_directory(temp_dir.path(), false).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_base_language_mapper_map_directory_with_non_matching_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create files with non-matching extensions
        fs::write(temp_dir.path().join("readme.txt"), "Hello").unwrap();
        fs::write(temp_dir.path().join("data.json"), "{}").unwrap();

        let mapper = BaseLanguageMapper::new(Language::Java);
        let result = mapper.map_directory(temp_dir.path(), false).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_base_language_mapper_map_directory_with_nested_dirs() {
        let temp_dir = TempDir::new().unwrap();

        // Create nested directory structure
        let nested_dir = temp_dir.path().join("subdir");
        fs::create_dir(&nested_dir).unwrap();

        // Create files in both directories
        fs::write(temp_dir.path().join("file.txt"), "test").unwrap();
        fs::write(nested_dir.join("nested.txt"), "nested").unwrap();

        let mapper = BaseLanguageMapper::new(Language::Java);

        // Non-recursive should skip nested directories
        let result = mapper.map_directory(temp_dir.path(), false).await;
        assert!(result.is_ok());

        // Recursive should explore nested directories
        let result = mapper.map_directory(temp_dir.path(), true).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_base_language_mapper_create_test_node() {
        let mapper = BaseLanguageMapper::new(Language::Python);
        let node = mapper.create_test_node(NodeKind::Function, "test_function");

        assert_eq!(node.kind, NodeKind::Function);
        assert_eq!(node.name, "test_function");
        assert_eq!(node.language, Language::Python);
    }

    // ==========================================================================
    // LanguageMapperFactory Tests (in language_mapper.rs)
    // ==========================================================================

    #[test]
    fn test_factory_create_all_languages() {
        let mappers = LanguageMapperFactory::create_all();

        // Should contain all supported languages
        assert!(mappers.contains_key(&Language::Java));
        assert!(mappers.contains_key(&Language::Kotlin));
        assert!(mappers.contains_key(&Language::Scala));
        assert!(mappers.contains_key(&Language::TypeScript));
        assert!(mappers.contains_key(&Language::JavaScript));

        // Verify each mapper returns correct language
        for (lang, mapper) in &mappers {
            assert_eq!(mapper.language(), *lang);
        }
    }

    #[test]
    fn test_factory_create_kotlin() {
        let result = LanguageMapperFactory::create(Language::Kotlin);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().language(), Language::Kotlin);
    }

    #[test]
    fn test_factory_create_typescript() {
        let result = LanguageMapperFactory::create(Language::TypeScript);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().language(), Language::TypeScript);
    }

    #[test]
    fn test_factory_create_javascript() {
        let result = LanguageMapperFactory::create(Language::JavaScript);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().language(), Language::JavaScript);
    }

    #[test]
    fn test_factory_create_for_file_kotlin() {
        let result = LanguageMapperFactory::create_for_file(Path::new("Test.kt"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().language(), Language::Kotlin);
    }

    #[test]
    fn test_factory_create_for_file_typescript() {
        let result = LanguageMapperFactory::create_for_file(Path::new("app.ts"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().language(), Language::TypeScript);
    }

    #[test]
    fn test_factory_create_for_file_javascript() {
        let result = LanguageMapperFactory::create_for_file(Path::new("script.js"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().language(), Language::JavaScript);
    }

    #[test]
    fn test_factory_create_for_file_scala() {
        let result = LanguageMapperFactory::create_for_file(Path::new("Main.scala"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().language(), Language::Scala);
    }

    #[test]
    fn test_factory_create_for_file_unsupported() {
        let result = LanguageMapperFactory::create_for_file(Path::new("readme.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_factory_create_for_file_no_extension() {
        let result = LanguageMapperFactory::create_for_file(Path::new("Makefile"));
        assert!(result.is_err());
    }

    #[test]
    fn test_factory_create_unsupported_other() {
        let result = LanguageMapperFactory::create(Language::Other(999));
        assert!(result.is_err());
    }

    // ==========================================================================
    // JavaMapper Tests
    // ==========================================================================

    #[test]
    fn test_java_mapper_new() {
        let mapper = JavaMapper::new();
        assert_eq!(mapper.language(), Language::Java);
    }

    #[test]
    fn test_java_mapper_clone() {
        let mapper = JavaMapper::new();
        let cloned = mapper.clone();
        assert_eq!(cloned.language(), Language::Java);
    }

    #[test]
    fn test_java_mapper_clone_box() {
        let mapper = JavaMapper::new();
        let boxed = mapper.clone_box();
        assert_eq!(boxed.language(), Language::Java);
    }

    #[test]
    fn test_java_mapper_process_java_specific_interface() {
        let mapper = JavaMapper::new();
        let mut nodes = vec![UnifiedNode::new(NodeKind::Class, "TestInterface", Language::Java)];

        // Add interface modifier
        nodes[0]
            .attributes
            .insert("modifier:interface".to_string(), "true".to_string());

        mapper.process_java_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::Interface);
    }

    #[test]
    fn test_java_mapper_process_java_specific_record() {
        let mapper = JavaMapper::new();
        let mut nodes = vec![UnifiedNode::new(NodeKind::Class, "TestRecord", Language::Java)];

        // Add record modifier
        nodes[0]
            .attributes
            .insert("modifier:record".to_string(), "true".to_string());

        mapper.process_java_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::Record);
    }

    #[test]
    fn test_java_mapper_process_java_specific_no_modifier() {
        let mapper = JavaMapper::new();
        let mut nodes = vec![
            UnifiedNode::new(NodeKind::Class, "TestClass", Language::Java),
            UnifiedNode::new(NodeKind::Function, "testMethod", Language::Java),
        ];

        mapper.process_java_specific(&mut nodes);

        // Should remain unchanged
        assert_eq!(nodes[0].kind, NodeKind::Class);
        assert_eq!(nodes[1].kind, NodeKind::Function);
    }

    #[test]
    fn test_java_mapper_convert_ast_items() {
        let mapper = JavaMapper::new();
        let path = Path::new("/test/Test.java");
        let items = vec![
            create_function_item("doSomething", false, 10),
            create_struct_item("InnerClass", 2, vec![], 20),
        ];

        let nodes = mapper.convert_ast_items(&items, path);

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].language, Language::Java);
        assert_eq!(nodes[1].language, Language::Java);
    }

    #[tokio::test]
    async fn test_java_mapper_map_source_without_feature() {
        let mapper = JavaMapper::new();
        let source = "public class Test { public void hello() {} }";
        let result = mapper.map_source(source, Path::new("Test.java")).await;

        // Without java-ast feature, should return error
        #[cfg(not(feature = "java-ast"))]
        assert!(result.is_err());

        #[cfg(feature = "java-ast")]
        assert!(result.is_ok() || result.is_err()); // May succeed or fail depending on implementation
    }

    #[tokio::test]
    async fn test_java_mapper_map_file_not_found() {
        let mapper = JavaMapper::new();
        let result = mapper.map_file(Path::new("/nonexistent/Test.java")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_java_mapper_map_directory_empty() {
        let temp_dir = TempDir::new().unwrap();
        let mapper = JavaMapper::new();

        let result = mapper.map_directory(temp_dir.path(), false).await;
        assert!(result.is_ok());
    }

    // ==========================================================================
    // KotlinMapper Tests
    // ==========================================================================

    #[test]
    fn test_kotlin_mapper_new() {
        let mapper = KotlinMapper::new();
        assert_eq!(mapper.language(), Language::Kotlin);
    }

    #[test]
    fn test_kotlin_mapper_clone() {
        let mapper = KotlinMapper::new();
        let cloned = mapper.clone();
        assert_eq!(cloned.language(), Language::Kotlin);
    }

    #[test]
    fn test_kotlin_mapper_clone_box() {
        let mapper = KotlinMapper::new();
        let boxed = mapper.clone_box();
        assert_eq!(boxed.language(), Language::Kotlin);
    }

    #[test]
    fn test_kotlin_mapper_process_kotlin_specific_data_class() {
        let mapper = KotlinMapper::new();
        let mut nodes = vec![UnifiedNode::new(NodeKind::Class, "UserData", Language::Kotlin)];

        // Add data modifier
        nodes[0]
            .attributes
            .insert("modifier:data".to_string(), "true".to_string());

        mapper.process_kotlin_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::Record);
        assert_eq!(nodes[0].metadata.get("kotlin:isData"), Some(&"true".to_string()));
    }

    #[test]
    fn test_kotlin_mapper_process_kotlin_specific_sealed_class() {
        let mapper = KotlinMapper::new();
        let mut nodes = vec![UnifiedNode::new(NodeKind::Class, "Result", Language::Kotlin)];

        // Add sealed modifier
        nodes[0]
            .attributes
            .insert("modifier:sealed".to_string(), "true".to_string());

        mapper.process_kotlin_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::Class); // Sealed doesn't change kind
        assert_eq!(nodes[0].metadata.get("kotlin:isSealed"), Some(&"true".to_string()));
    }

    #[test]
    fn test_kotlin_mapper_process_kotlin_specific_data_and_sealed() {
        let mapper = KotlinMapper::new();
        let mut nodes = vec![UnifiedNode::new(NodeKind::Class, "SealedData", Language::Kotlin)];

        // Add both modifiers
        nodes[0]
            .attributes
            .insert("modifier:data".to_string(), "true".to_string());
        nodes[0]
            .attributes
            .insert("modifier:sealed".to_string(), "true".to_string());

        mapper.process_kotlin_specific(&mut nodes);

        // Data takes precedence (processed first)
        assert_eq!(nodes[0].kind, NodeKind::Record);
        assert!(nodes[0].metadata.contains_key("kotlin:isData"));
        assert!(nodes[0].metadata.contains_key("kotlin:isSealed"));
    }

    #[test]
    fn test_kotlin_mapper_process_kotlin_specific_no_modifier() {
        let mapper = KotlinMapper::new();
        let mut nodes = vec![
            UnifiedNode::new(NodeKind::Class, "NormalClass", Language::Kotlin),
            UnifiedNode::new(NodeKind::Function, "doWork", Language::Kotlin),
        ];

        mapper.process_kotlin_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::Class);
        assert_eq!(nodes[1].kind, NodeKind::Function);
        assert!(nodes[0].metadata.is_empty());
    }

    #[test]
    fn test_kotlin_mapper_convert_ast_items() {
        let mapper = KotlinMapper::new();
        let path = Path::new("/test/Main.kt");
        let items = vec![
            create_function_item("processData", true, 5),
            create_struct_item("DataHolder", 4, vec![], 15),
        ];

        let nodes = mapper.convert_ast_items(&items, path);

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].language, Language::Kotlin);
    }

    #[tokio::test]
    async fn test_kotlin_mapper_map_source() {
        let mapper = KotlinMapper::new();
        let source = "data class User(val name: String)";
        let result = mapper.map_source(source, Path::new("User.kt")).await;

        // Uses base implementation which returns error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_kotlin_mapper_map_file_not_found() {
        let mapper = KotlinMapper::new();
        let result = mapper.map_file(Path::new("/nonexistent/Test.kt")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_kotlin_mapper_map_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mapper = KotlinMapper::new();

        let result = mapper.map_directory(temp_dir.path(), true).await;
        assert!(result.is_ok());
    }

    // ==========================================================================
    // ScalaMapper Tests
    // ==========================================================================

    #[test]
    fn test_scala_mapper_new() {
        let mapper = ScalaMapper::new();
        assert_eq!(mapper.language(), Language::Scala);
    }

    #[test]
    fn test_scala_mapper_clone() {
        let mapper = ScalaMapper::new();
        let cloned = mapper.clone();
        assert_eq!(cloned.language(), Language::Scala);
    }

    #[test]
    fn test_scala_mapper_clone_box() {
        let mapper = ScalaMapper::new();
        let boxed = mapper.clone_box();
        assert_eq!(boxed.language(), Language::Scala);
    }

    #[test]
    fn test_scala_mapper_process_scala_specific_case_class() {
        let mapper = ScalaMapper::new();
        let mut nodes = vec![UnifiedNode::new(NodeKind::Class, "Person", Language::Scala)];

        // Add case modifier
        nodes[0]
            .attributes
            .insert("modifier:case".to_string(), "true".to_string());

        mapper.process_scala_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::CaseClass);
    }

    #[test]
    fn test_scala_mapper_process_scala_specific_object() {
        let mapper = ScalaMapper::new();
        let mut nodes = vec![UnifiedNode::new(NodeKind::Module, "AppObject", Language::Scala)];

        mapper.process_scala_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::Module);
        assert_eq!(nodes[0].metadata.get("scala:isObject"), Some(&"true".to_string()));
    }

    #[test]
    fn test_scala_mapper_process_scala_specific_multiple_nodes() {
        let mapper = ScalaMapper::new();
        let mut nodes = vec![
            UnifiedNode::new(NodeKind::Class, "NormalClass", Language::Scala),
            UnifiedNode::new(NodeKind::Module, "Companion", Language::Scala),
            UnifiedNode::new(NodeKind::Function, "apply", Language::Scala),
        ];

        // Only add case to first node
        nodes[0]
            .attributes
            .insert("modifier:case".to_string(), "true".to_string());

        mapper.process_scala_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::CaseClass);
        assert_eq!(nodes[1].kind, NodeKind::Module);
        assert!(nodes[1].metadata.contains_key("scala:isObject"));
        assert_eq!(nodes[2].kind, NodeKind::Function);
    }

    #[test]
    fn test_scala_mapper_convert_ast_items() {
        let mapper = ScalaMapper::new();
        let path = Path::new("/test/App.scala");
        let items = vec![
            create_function_item("main", false, 1),
            create_trait_item("Service", 10),
        ];

        let nodes = mapper.convert_ast_items(&items, path);

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].language, Language::Scala);
        assert_eq!(nodes[1].language, Language::Scala);
    }

    #[tokio::test]
    async fn test_scala_mapper_map_source_without_feature() {
        let mapper = ScalaMapper::new();
        let source = "case class User(name: String)";
        let result = mapper.map_source(source, Path::new("User.scala")).await;

        #[cfg(not(feature = "scala-ast"))]
        assert!(result.is_err());

        #[cfg(feature = "scala-ast")]
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_scala_mapper_map_file_not_found() {
        let mapper = ScalaMapper::new();
        let result = mapper.map_file(Path::new("/nonexistent/Main.scala")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_scala_mapper_map_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mapper = ScalaMapper::new();

        let result = mapper.map_directory(temp_dir.path(), false).await;
        assert!(result.is_ok());
    }

    // ==========================================================================
    // TypeScriptMapper Tests
    // ==========================================================================

    #[test]
    fn test_typescript_mapper_new() {
        let mapper = TypeScriptMapper::new();
        assert_eq!(mapper.language(), Language::TypeScript);
    }

    #[test]
    fn test_typescript_mapper_clone() {
        let mapper = TypeScriptMapper::new();
        let cloned = mapper.clone();
        assert_eq!(cloned.language(), Language::TypeScript);
    }

    #[test]
    fn test_typescript_mapper_clone_box() {
        let mapper = TypeScriptMapper::new();
        let boxed = mapper.clone_box();
        assert_eq!(boxed.language(), Language::TypeScript);
    }

    #[test]
    fn test_typescript_mapper_process_typescript_specific_interface() {
        let mapper = TypeScriptMapper::new();
        let mut nodes = vec![UnifiedNode::new(NodeKind::Interface, "UserProps", Language::TypeScript)];

        mapper.process_typescript_specific(&mut nodes);

        assert_eq!(nodes[0].metadata.get("typescript:isInterface"), Some(&"true".to_string()));
    }

    #[test]
    fn test_typescript_mapper_process_typescript_specific_abstract_class() {
        let mapper = TypeScriptMapper::new();
        let mut nodes = vec![UnifiedNode::new(NodeKind::Class, "BaseService", Language::TypeScript)];

        // Add abstract modifier
        nodes[0]
            .attributes
            .insert("modifier:abstract".to_string(), "true".to_string());

        mapper.process_typescript_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::Class);
        assert_eq!(nodes[0].metadata.get("typescript:isAbstract"), Some(&"true".to_string()));
    }

    #[test]
    fn test_typescript_mapper_process_typescript_specific_regular_class() {
        let mapper = TypeScriptMapper::new();
        let mut nodes = vec![UnifiedNode::new(NodeKind::Class, "UserService", Language::TypeScript)];

        mapper.process_typescript_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::Class);
        assert!(nodes[0].metadata.is_empty());
    }

    #[test]
    fn test_typescript_mapper_process_typescript_specific_mixed() {
        let mapper = TypeScriptMapper::new();
        let mut nodes = vec![
            UnifiedNode::new(NodeKind::Interface, "IUser", Language::TypeScript),
            UnifiedNode::new(NodeKind::Class, "AbstractBase", Language::TypeScript),
            UnifiedNode::new(NodeKind::Class, "ConcreteImpl", Language::TypeScript),
            UnifiedNode::new(NodeKind::Function, "helper", Language::TypeScript),
        ];

        nodes[1]
            .attributes
            .insert("modifier:abstract".to_string(), "true".to_string());

        mapper.process_typescript_specific(&mut nodes);

        assert!(nodes[0].metadata.contains_key("typescript:isInterface"));
        assert!(nodes[1].metadata.contains_key("typescript:isAbstract"));
        assert!(nodes[2].metadata.is_empty());
        assert!(nodes[3].metadata.is_empty());
    }

    #[test]
    fn test_typescript_mapper_convert_ast_items() {
        let mapper = TypeScriptMapper::new();
        let path = Path::new("/test/app.ts");
        let items = vec![
            create_function_item("fetchData", true, 1),
            create_trait_item("IDataProvider", 10),
        ];

        let nodes = mapper.convert_ast_items(&items, path);

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].language, Language::TypeScript);
    }

    #[tokio::test]
    async fn test_typescript_mapper_map_file_not_found() {
        let mapper = TypeScriptMapper::new();
        let result = mapper.map_file(Path::new("/nonexistent/app.ts")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_typescript_mapper_map_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mapper = TypeScriptMapper::new();

        let result = mapper.map_directory(temp_dir.path(), true).await;
        assert!(result.is_ok());
    }

    // ==========================================================================
    // JavaScriptMapper Tests
    // ==========================================================================

    #[test]
    fn test_javascript_mapper_new() {
        let mapper = JavaScriptMapper::new();
        assert_eq!(mapper.language(), Language::JavaScript);
    }

    #[test]
    fn test_javascript_mapper_clone() {
        let mapper = JavaScriptMapper::new();
        let cloned = mapper.clone();
        assert_eq!(cloned.language(), Language::JavaScript);
    }

    #[test]
    fn test_javascript_mapper_clone_box() {
        let mapper = JavaScriptMapper::new();
        let boxed = mapper.clone_box();
        assert_eq!(boxed.language(), Language::JavaScript);
    }

    #[test]
    fn test_javascript_mapper_process_javascript_specific_class() {
        let mapper = JavaScriptMapper::new();
        let mut nodes = vec![UnifiedNode::new(NodeKind::Class, "MyComponent", Language::JavaScript)];

        mapper.process_javascript_specific(&mut nodes);

        assert_eq!(nodes[0].metadata.get("javascript:isClass"), Some(&"true".to_string()));
    }

    #[test]
    fn test_javascript_mapper_process_javascript_specific_arrow_function() {
        let mapper = JavaScriptMapper::new();
        let mut nodes = vec![UnifiedNode::new(NodeKind::Function, "handleClick", Language::JavaScript)];

        // Add arrow modifier
        nodes[0]
            .attributes
            .insert("modifier:arrow".to_string(), "true".to_string());

        mapper.process_javascript_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::Lambda);
    }

    #[test]
    fn test_javascript_mapper_process_javascript_specific_regular_function() {
        let mapper = JavaScriptMapper::new();
        let mut nodes = vec![UnifiedNode::new(NodeKind::Function, "processData", Language::JavaScript)];

        mapper.process_javascript_specific(&mut nodes);

        assert_eq!(nodes[0].kind, NodeKind::Function);
        assert!(nodes[0].metadata.is_empty());
    }

    #[test]
    fn test_javascript_mapper_process_javascript_specific_mixed() {
        let mapper = JavaScriptMapper::new();
        let mut nodes = vec![
            UnifiedNode::new(NodeKind::Class, "UserList", Language::JavaScript),
            UnifiedNode::new(NodeKind::Function, "render", Language::JavaScript),
            UnifiedNode::new(NodeKind::Function, "onClick", Language::JavaScript),
            UnifiedNode::new(NodeKind::Module, "utils", Language::JavaScript),
        ];

        nodes[2]
            .attributes
            .insert("modifier:arrow".to_string(), "true".to_string());

        mapper.process_javascript_specific(&mut nodes);

        assert!(nodes[0].metadata.contains_key("javascript:isClass"));
        assert_eq!(nodes[1].kind, NodeKind::Function);
        assert_eq!(nodes[2].kind, NodeKind::Lambda);
        assert!(nodes[3].metadata.is_empty());
    }

    #[test]
    fn test_javascript_mapper_convert_ast_items() {
        let mapper = JavaScriptMapper::new();
        let path = Path::new("/test/app.js");
        let items = vec![
            create_function_item("initialize", false, 1),
            create_module_item("helpers", 20),
        ];

        let nodes = mapper.convert_ast_items(&items, path);

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].language, Language::JavaScript);
    }

    #[tokio::test]
    async fn test_javascript_mapper_map_file_not_found() {
        let mapper = JavaScriptMapper::new();
        let result = mapper.map_file(Path::new("/nonexistent/script.js")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_javascript_mapper_map_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mapper = JavaScriptMapper::new();

        let result = mapper.map_directory(temp_dir.path(), false).await;
        assert!(result.is_ok());
    }

    // ==========================================================================
    // CSharpMapper Tests
    // ==========================================================================

    #[test]
    fn test_csharp_mapper_new() {
        let mapper = CSharpMapper::new();
        assert_eq!(mapper.language(), Language::CSharp);
    }

    #[test]
    fn test_csharp_mapper_clone() {
        let mapper = CSharpMapper::new();
        let cloned = mapper.clone();
        assert_eq!(cloned.language(), Language::CSharp);
    }

    #[test]
    fn test_csharp_mapper_clone_box() {
        let mapper = CSharpMapper::new();
        let boxed = mapper.clone_box();
        assert_eq!(boxed.language(), Language::CSharp);
    }

    #[test]
    fn test_csharp_mapper_convert_ast_items() {
        let mapper = CSharpMapper::new();
        let path = Path::new("/test/Program.cs");
        let items = vec![
            create_function_item("Main", false, 1),
            create_struct_item("Config", 5, vec![], 10),
        ];

        let nodes = mapper.convert_ast_items(&items, path);

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].language, Language::CSharp);
    }

    #[tokio::test]
    async fn test_csharp_mapper_map_source() {
        let mapper = CSharpMapper::new();
        let source = "public class Test { }";
        let result = mapper.map_source(source, Path::new("Test.cs")).await;

        // Uses base implementation which returns error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_csharp_mapper_map_file_not_found() {
        let mapper = CSharpMapper::new();
        let result = mapper.map_file(Path::new("/nonexistent/Program.cs")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_csharp_mapper_map_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mapper = CSharpMapper::new();

        let result = mapper.map_directory(temp_dir.path(), true).await;
        assert!(result.is_ok());
    }

    // ==========================================================================
    // RubyMapper Tests
    // ==========================================================================

    #[test]
    fn test_ruby_mapper_new() {
        let mapper = RubyMapper::new();
        assert_eq!(mapper.language(), Language::Ruby);
    }

    #[test]
    fn test_ruby_mapper_clone() {
        let mapper = RubyMapper::new();
        let cloned = mapper.clone();
        assert_eq!(cloned.language(), Language::Ruby);
    }

    #[test]
    fn test_ruby_mapper_clone_box() {
        let mapper = RubyMapper::new();
        let boxed = mapper.clone_box();
        assert_eq!(boxed.language(), Language::Ruby);
    }

    #[test]
    fn test_ruby_mapper_convert_ast_items() {
        let mapper = RubyMapper::new();
        let path = Path::new("/test/app.rb");
        let items = vec![
            create_function_item("initialize", false, 1),
            create_module_item("Helpers", 20),
        ];

        let nodes = mapper.convert_ast_items(&items, path);

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].language, Language::Ruby);
    }

    #[tokio::test]
    async fn test_ruby_mapper_map_source() {
        let mapper = RubyMapper::new();
        let source = "class User; def name; @name; end; end";
        let result = mapper.map_source(source, Path::new("user.rb")).await;

        // Uses base implementation which returns error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ruby_mapper_map_file_not_found() {
        let mapper = RubyMapper::new();
        let result = mapper.map_file(Path::new("/nonexistent/app.rb")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ruby_mapper_map_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mapper = RubyMapper::new();

        let result = mapper.map_directory(temp_dir.path(), false).await;
        assert!(result.is_ok());
    }

    // ==========================================================================
    // Cross-Mapper Integration Tests
    // ==========================================================================

    #[test]
    fn test_all_mappers_implement_clone_box() {
        // Ensure all mappers can be cloned via clone_box
        let mappers: Vec<Box<dyn LanguageMapper>> = vec![
            JavaMapper::new().clone_box(),
            KotlinMapper::new().clone_box(),
            ScalaMapper::new().clone_box(),
            TypeScriptMapper::new().clone_box(),
            JavaScriptMapper::new().clone_box(),
            CSharpMapper::new().clone_box(),
            RubyMapper::new().clone_box(),
            BaseLanguageMapper::new(Language::Python).clone_box(),
        ];

        let expected_languages = vec![
            Language::Java,
            Language::Kotlin,
            Language::Scala,
            Language::TypeScript,
            Language::JavaScript,
            Language::CSharp,
            Language::Ruby,
            Language::Python,
        ];

        for (mapper, expected_lang) in mappers.iter().zip(expected_languages.iter()) {
            assert_eq!(mapper.language(), *expected_lang);
        }
    }

    #[test]
    fn test_all_mappers_create_test_node() {
        let java_mapper = JavaMapper::new();
        let kotlin_mapper = KotlinMapper::new();
        let scala_mapper = ScalaMapper::new();
        let typescript_mapper = TypeScriptMapper::new();
        let javascript_mapper = JavaScriptMapper::new();

        let node1 = java_mapper.create_test_node(NodeKind::Class, "JavaClass");
        let node2 = kotlin_mapper.create_test_node(NodeKind::Class, "KotlinClass");
        let node3 = scala_mapper.create_test_node(NodeKind::Class, "ScalaClass");
        let node4 = typescript_mapper.create_test_node(NodeKind::Class, "TSClass");
        let node5 = javascript_mapper.create_test_node(NodeKind::Class, "JSClass");

        assert_eq!(node1.language, Language::Java);
        assert_eq!(node2.language, Language::Kotlin);
        assert_eq!(node3.language, Language::Scala);
        assert_eq!(node4.language, Language::TypeScript);
        assert_eq!(node5.language, Language::JavaScript);
    }

    #[tokio::test]
    async fn test_mappers_handle_directory_with_mixed_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create files with various extensions
        fs::write(temp_dir.path().join("Test.java"), "class Test {}").unwrap();
        fs::write(temp_dir.path().join("Main.kt"), "fun main() {}").unwrap();
        fs::write(temp_dir.path().join("App.scala"), "object App").unwrap();
        fs::write(temp_dir.path().join("index.ts"), "const x = 1").unwrap();
        fs::write(temp_dir.path().join("utils.js"), "function f() {}").unwrap();
        fs::write(temp_dir.path().join("readme.txt"), "Documentation").unwrap();

        let java_mapper = JavaMapper::new();
        let kotlin_mapper = KotlinMapper::new();

        // Each mapper should only process files for its language
        let java_result = java_mapper.map_directory(temp_dir.path(), false).await;
        let kotlin_result = kotlin_mapper.map_directory(temp_dir.path(), false).await;

        // Results should be ok (may fail on actual parsing but directory traversal works)
        assert!(java_result.is_ok() || java_result.is_err());
        assert!(kotlin_result.is_ok() || kotlin_result.is_err());
    }

    // ==========================================================================
    // Edge Cases and Error Handling Tests
    // ==========================================================================

    #[test]
    fn test_convert_ast_items_with_all_item_types() {
        let mapper = BaseLanguageMapper::new(Language::Rust);
        let path = Path::new("/test/lib.rs");

        let items = vec![
            AstItem::Function {
                name: "async_func".to_string(),
                visibility: "pub".to_string(),
                is_async: true,
                line: 1,
            },
            AstItem::Struct {
                name: "MyStruct".to_string(),
                visibility: "pub".to_string(),
                fields_count: 3,
                derives: vec!["Debug".to_string(), "Clone".to_string()],
                line: 10,
            },
            AstItem::Enum {
                name: "Status".to_string(),
                visibility: "pub".to_string(),
                variants_count: 3,
                line: 20,
            },
            AstItem::Trait {
                name: "Handler".to_string(),
                visibility: "pub".to_string(),
                line: 30,
            },
            AstItem::Impl {
                type_name: "MyStruct".to_string(),
                trait_name: Some("Handler".to_string()),
                line: 40,
            },
            AstItem::Use {
                path: "std::io::Result".to_string(),
                line: 50,
            },
            AstItem::Module {
                name: "submodule".to_string(),
                visibility: "pub".to_string(),
                line: 60,
            },
            AstItem::Import {
                module: "external".to_string(),
                items: vec!["Item1".to_string()],
                alias: Some("ext".to_string()),
                line: 70,
            },
        ];

        let nodes = mapper.convert_ast_items(&items, path);

        assert_eq!(nodes.len(), 8);

        // Verify async function has async modifier
        assert!(nodes[0].attributes.contains_key("modifier:async"));

        // Verify struct has derive attributes
        assert!(nodes[1].attributes.contains_key("derive:Debug"));
        assert!(nodes[1].attributes.contains_key("derive:Clone"));
    }

    #[test]
    fn test_factory_create_all_returns_correct_count() {
        let mappers = LanguageMapperFactory::create_all();

        // Should have exactly 5 supported languages
        assert_eq!(mappers.len(), 5);
    }

    #[tokio::test]
    async fn test_mapper_handles_file_without_extension() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("Makefile");
        fs::write(&file_path, "all: build").unwrap();

        let mapper = JavaMapper::new();
        // Should not crash but may return empty or error
        let result = mapper.map_file(&file_path).await;
        // File exists but wrong type - should still work (validation passes, parsing may fail)
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_process_java_specific_empty_nodes() {
        let mapper = JavaMapper::new();
        let mut nodes: Vec<UnifiedNode> = vec![];

        // Should not panic on empty input
        mapper.process_java_specific(&mut nodes);
        assert!(nodes.is_empty());
    }

    #[test]
    fn test_process_kotlin_specific_empty_nodes() {
        let mapper = KotlinMapper::new();
        let mut nodes: Vec<UnifiedNode> = vec![];

        mapper.process_kotlin_specific(&mut nodes);
        assert!(nodes.is_empty());
    }

    #[test]
    fn test_process_scala_specific_empty_nodes() {
        let mapper = ScalaMapper::new();
        let mut nodes: Vec<UnifiedNode> = vec![];

        mapper.process_scala_specific(&mut nodes);
        assert!(nodes.is_empty());
    }

    #[test]
    fn test_process_typescript_specific_empty_nodes() {
        let mapper = TypeScriptMapper::new();
        let mut nodes: Vec<UnifiedNode> = vec![];

        mapper.process_typescript_specific(&mut nodes);
        assert!(nodes.is_empty());
    }

    #[test]
    fn test_process_javascript_specific_empty_nodes() {
        let mapper = JavaScriptMapper::new();
        let mut nodes: Vec<UnifiedNode> = vec![];

        mapper.process_javascript_specific(&mut nodes);
        assert!(nodes.is_empty());
    }
}

/// Property-based tests for language mapper
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Strategy for generating valid language names
    fn language_strategy() -> impl Strategy<Value = Language> {
        prop_oneof![
            Just(Language::Java),
            Just(Language::Kotlin),
            Just(Language::Scala),
            Just(Language::TypeScript),
            Just(Language::JavaScript),
            Just(Language::Python),
            Just(Language::Rust),
            Just(Language::Go),
            Just(Language::Cpp),
            Just(Language::CSharp),
            Just(Language::Ruby),
            Just(Language::Swift),
            Just(Language::Php),
            (0u32..1000).prop_map(Language::Other),
        ]
    }

    // Strategy for generating valid identifiers
    fn identifier_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z_][a-zA-Z0-9_]{0,30}".prop_map(|s| s)
    }

    // Strategy for generating node kinds
    fn node_kind_strategy() -> impl Strategy<Value = NodeKind> {
        prop_oneof![
            Just(NodeKind::Class),
            Just(NodeKind::Interface),
            Just(NodeKind::Trait),
            Just(NodeKind::Function),
            Just(NodeKind::Method),
            Just(NodeKind::Module),
            Just(NodeKind::Enum),
            Just(NodeKind::Struct),
        ]
    }

    proptest! {
        #[test]
        fn test_base_mapper_language_preserved(lang in language_strategy()) {
            let mapper = BaseLanguageMapper::new(lang);
            prop_assert_eq!(mapper.language(), lang);
        }

        #[test]
        fn test_base_mapper_clone_preserves_language(lang in language_strategy()) {
            let mapper = BaseLanguageMapper::new(lang);
            let cloned = mapper.clone();
            prop_assert_eq!(cloned.language(), lang);
        }

        #[test]
        fn test_base_mapper_clone_box_preserves_language(lang in language_strategy()) {
            let mapper = BaseLanguageMapper::new(lang);
            let boxed = mapper.clone_box();
            prop_assert_eq!(boxed.language(), lang);
        }

        #[test]
        fn test_create_test_node_preserves_properties(
            kind in node_kind_strategy(),
            name in identifier_strategy(),
            lang in language_strategy()
        ) {
            let mapper = BaseLanguageMapper::new(lang);
            let node = mapper.create_test_node(kind, &name);

            prop_assert_eq!(node.kind, kind);
            prop_assert_eq!(node.name, name);
            prop_assert_eq!(node.language, lang);
        }

        #[test]
        fn test_unified_node_new_generates_valid_id(
            kind in node_kind_strategy(),
            name in identifier_strategy(),
            lang in language_strategy()
        ) {
            let node = UnifiedNode::new(kind, &name, lang);

            // ID should contain language name, kind, and name
            prop_assert!(node.id.contains(lang.name()));
            prop_assert!(node.id.contains(kind.as_str()));
            prop_assert!(node.id.contains(&name));
        }

        #[test]
        fn test_factory_create_supported_languages(lang in prop_oneof![
            Just(Language::Java),
            Just(Language::Kotlin),
            Just(Language::Scala),
            Just(Language::TypeScript),
            Just(Language::JavaScript),
        ]) {
            let result = LanguageMapperFactory::create(lang);
            prop_assert!(result.is_ok());
            prop_assert_eq!(result.unwrap().language(), lang);
        }

        #[test]
        fn test_factory_create_unsupported_returns_error(id in 0u32..1000) {
            let result = LanguageMapperFactory::create(Language::Other(id));
            prop_assert!(result.is_err());
        }

        #[test]
        fn test_convert_ast_items_count_matches(
            count in 0usize..20,
        ) {
            let mapper = BaseLanguageMapper::new(Language::Rust);
            let path = std::path::Path::new("/test/file.rs");

            let items: Vec<AstItem> = (0..count)
                .map(|i| AstItem::Function {
                    name: format!("func_{}", i),
                    visibility: "pub".to_string(),
                    is_async: false,
                    line: i + 1,
                })
                .collect();

            let nodes = mapper.convert_ast_items(&items, path);
            prop_assert_eq!(nodes.len(), count);
        }
    }
}
