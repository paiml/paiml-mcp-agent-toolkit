//! Language mapper interfaces for polyglot AST
//!
//! This module defines the interfaces and implementations for mapping
//! language-specific ASTs to the unified polyglot representation. Each
//! supported language implements the `LanguageMapper` trait to convert
//! its native AST nodes to `UnifiedNode` instances.

use crate::services::context::AstItem;
use crate::ast::polyglot::{Language, NodeKind, UnifiedNode, PolyglotPathValidator};
use anyhow::{Result, anyhow};
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
        let extensions: Vec<String> = self.language()
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
    pub fn new() -> Self {
        Self {
            base: BaseLanguageMapper::new(Language::Java),
        }
    }
    
    /// Process Java-specific nodes
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
                },
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
    
    async fn map_source(&self, source: &str, path: &Path) -> Result<Vec<UnifiedNode>> {
        use crate::services::languages::java::JavaAstVisitor;
        
        let visitor = JavaAstVisitor::new(path);
        match visitor.analyze_java_source(source) {
            Ok(items) => {
                let mut nodes = self.convert_ast_items(&items, path);
                self.process_java_specific(&mut nodes);
                Ok(nodes)
            },
            Err(e) => Err(anyhow!("Failed to analyze Java source: {}", e)),
        }
    }
    
    fn convert_ast_items(&self, items: &[AstItem], path: &Path) -> Vec<UnifiedNode> {
        self.base.convert_ast_items(items, path)
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
                },
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
    pub fn new() -> Self {
        Self {
            base: BaseLanguageMapper::new(Language::Scala),
        }
    }
    
    /// Process Scala-specific nodes
    fn process_scala_specific(&self, nodes: &mut [UnifiedNode]) {
        for node in nodes.iter_mut() {
            // Add Scala-specific metadata
            match node.kind {
                NodeKind::Class => {
                    // Handle case classes
                    if node.has_modifier("case") {
                        node.kind = NodeKind::CaseClass;
                    }
                },
                NodeKind::Module => {
                    // Check if this is a Scala object
                    node.add_metadata("scala:isObject", "true");
                },
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
    
    async fn map_source(&self, source: &str, path: &Path) -> Result<Vec<UnifiedNode>> {
        use crate::services::languages::scala::ScalaAstVisitor;
        
        let visitor = ScalaAstVisitor::new(path);
        match visitor.analyze_scala_source(source) {
            Ok(items) => {
                let mut nodes = self.convert_ast_items(&items, path);
                self.process_scala_specific(&mut nodes);
                Ok(nodes)
            },
            Err(e) => Err(anyhow!("Failed to analyze Scala source: {}", e)),
        }
    }
    
    fn convert_ast_items(&self, items: &[AstItem], path: &Path) -> Vec<UnifiedNode> {
        self.base.convert_ast_items(items, path)
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
                },
                NodeKind::Class => {
                    if node.has_modifier("abstract") {
                        node.add_metadata("typescript:isAbstract", "true");
                    }
                },
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
            },
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
                },
                NodeKind::Function => {
                    // Check for arrow functions
                    if node.has_modifier("arrow") {
                        node.kind = NodeKind::Lambda;
                    }
                },
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
            },
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
        assert_eq!(nodes[0].kind, NodeKind::Class);
        assert_eq!(nodes[0].name, "TestClass");
        assert_eq!(nodes[1].kind, NodeKind::Method);
        assert_eq!(nodes[1].name, "testMethod");
    }
}