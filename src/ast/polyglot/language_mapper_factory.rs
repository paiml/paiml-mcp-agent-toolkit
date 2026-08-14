#![cfg_attr(coverage_nightly, coverage(off))]
//! Factory for creating language-specific mappers
//!
//! This module provides a factory for creating language-specific mappers
//! that can transform language-specific ASTs into the unified representation.

#[cfg(feature = "polyglot-typescript")]
use crate::ast::polyglot::language_mapper::TypeScriptMapper;
// Sprint 46 Phase 6: Feature-gated unused mapper imports
#[cfg(feature = "polyglot-csharp")]
use crate::ast::polyglot::language_mapper::CSharpMapper;
// Gated on the feature that USES it (`create` matches on `polyglot-java`), not
// on `java-ast`. Sprint 46 Phase 6 turned `java-ast` into an empty feature and
// left this import behind, so `cargo check --lib --features java-ast` failed
// outright under `#![deny(unused_imports)]` — the whole feature was unbuildable,
// and CI could not see it because it builds default features only (#966).
#[cfg(feature = "polyglot-java")]
use crate::ast::polyglot::language_mapper::JavaMapper;
#[cfg(feature = "javascript-ast")]
use crate::ast::polyglot::language_mapper::JavaScriptMapper;
#[cfg(feature = "polyglot-kotlin")]
use crate::ast::polyglot::language_mapper::KotlinMapper;
#[cfg(feature = "polyglot-ruby")]
use crate::ast::polyglot::language_mapper::RubyMapper;
#[cfg(feature = "polyglot-scala")]
use crate::ast::polyglot::language_mapper::ScalaMapper;
use crate::ast::polyglot::{Language, LanguageMapper, PolyglotPathValidator, UnifiedNode};
use crate::services::context::AstItem;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

/// Factory for creating language mappers
pub struct LanguageMapperFactory;

impl LanguageMapperFactory {
    /// Create a language mapper for a specific language
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    include!("language_mapper_factory_tests.rs");
    include!("language_mapper_factory_tests_extended.rs");
}
