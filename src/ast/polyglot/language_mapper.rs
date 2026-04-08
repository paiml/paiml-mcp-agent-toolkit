#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
//! Language mapper interfaces for polyglot AST
//!
//! This module defines the interfaces and implementations for mapping
//! language-specific ASTs to the unified polyglot representation. Each
//! supported language implements the `LanguageMapper` trait to convert
//! its native AST nodes to `UnifiedNode` instances.
//!
//! Submodules (via include!()):
//! - language_mapper_base.rs: BaseLanguageMapper trait implementation
//! - language_mapper_jvm.rs: Java, Kotlin, Scala mappers
//! - language_mapper_web.rs: TypeScript, JavaScript mappers
//! - language_mapper_misc.rs: CSharp, Ruby mappers

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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn create_all() -> HashMap<Language, Arc<dyn LanguageMapper>> {
        let mut mappers = HashMap::new();

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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(language: Language) -> Self {
        Self { language }
    }
}

// --- Submodule includes ---

include!("language_mapper_base.rs");
include!("language_mapper_jvm.rs");
include!("language_mapper_web.rs");
include!("language_mapper_misc.rs");

// Tests extracted to language_mapper_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "language_mapper_tests.rs"]
mod tests;
