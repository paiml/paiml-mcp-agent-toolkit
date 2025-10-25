//! Factory for creating language-specific mappers
//! 
//! This module provides a factory for creating language-specific mappers
//! that can transform language-specific ASTs into the unified representation.

use crate::ast::polyglot::{Language, UnifiedNode, LanguageMapper, PolyglotPathValidator, NodeKind};
use crate::ast::polyglot::language_mapper::{
    JavaMapper, KotlinMapper, ScalaMapper, TypeScriptMapper,
    JavaScriptMapper, CSharpMapper, RubyMapper
};
use crate::services::context::AstItem;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;

/// Factory for creating language mappers
pub struct LanguageMapperFactory;

impl LanguageMapperFactory {
    /// Create a language mapper for a specific language
    pub fn create(language: Language) -> Result<Arc<dyn LanguageMapper>> {
        match language {
            #[cfg(feature = "polyglot-java")]
            Language::Java => Ok(Arc::new(JavaMapper::new())),
            
            #[cfg(feature = "polyglot-kotlin")]
            Language::Kotlin => Ok(Arc::new(KotlinMapper::new())),
            
            #[cfg(feature = "polyglot-scala")]
            Language::Scala => Ok(Arc::new(ScalaMapper::new())),
            
            #[cfg(feature = "polyglot-typescript")]
            Language::TypeScript => Ok(Arc::new(TypeScriptMapper::new())),
            
            #[cfg(feature = "polyglot-javascript")]
            Language::JavaScript => Ok(Arc::new(JavaScriptMapper::new())),
            
            #[cfg(feature = "polyglot-csharp")]
            Language::CSharp => Ok(Arc::new(CSharpMapper::new())),
            
            #[cfg(feature = "polyglot-ruby")]
            Language::Ruby => Ok(Arc::new(RubyMapper::new())),
            
            _ => {
                // For now, use a stub mapper for testing
                Ok(Arc::new(StubMapper::new(language)))
            }
        }
    }
}

/// Stub mapper for testing and languages without full implementation
#[derive(Clone)]
pub struct StubMapper {
    language: Language,
}

impl StubMapper {
    pub fn new(language: Language) -> Self {
        Self { language }
    }
}

#[async_trait]
impl LanguageMapper for StubMapper {
    fn language(&self) -> Language {
        self.language
    }
    
    async fn map_file(&self, path: &Path) -> Result<Vec<UnifiedNode>> {
        // Validate path first
        PolyglotPathValidator::validate_file_path(path)?;
        // Return an empty list for now - this is just a stub
        Ok(Vec::new())
    }
    
    async fn map_directory(&self, path: &Path, _recursive: bool) -> Result<Vec<UnifiedNode>> {
        // Validate path first
        PolyglotPathValidator::validate_directory_path(path)?;
        // Return an empty list for now - this is just a stub
        Ok(Vec::new())
    }
    
    async fn map_source(&self, _source: &str, _path: &Path) -> Result<Vec<UnifiedNode>> {
        // Return an empty list for now - this is just a stub
        Ok(Vec::new())
    }
    
    fn convert_ast_items(&self, _items: &[AstItem], _path: &Path) -> Vec<UnifiedNode> {
        // Return an empty list for now - this is just a stub
        Vec::new()
    }
    
    fn clone_box(&self) -> Box<dyn LanguageMapper> {
        Box::new(self.clone())
    }
}
