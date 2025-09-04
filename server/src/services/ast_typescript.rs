//! TypeScript AST analysis - MIGRATION IN PROGRESS
//!
//! This module is being migrated to the new unified AST architecture.
//! It now acts as a facade, delegating to the compatibility layer.
//!
//! Migration status: Using compatibility shim
//! Target: server/src/ast/languages/typescript.rs

// Re-export compatibility functions
pub use super::ast_typescript_compat::{
    analyze_javascript_file, analyze_javascript_file_with_classifier,
    analyze_javascript_file_with_complexity,
    analyze_javascript_file_with_complexity_and_classifier, analyze_typescript_file,
    analyze_typescript_file_with_classifier, analyze_typescript_file_with_complexity,
    analyze_typescript_file_with_complexity_and_classifier,
};

// Dispatch parser removed - functionality moved to new AST module

// Legacy compatibility types (may be referenced by other modules)
pub struct TypeScriptParser {}

impl Default for TypeScriptParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeScriptParser {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone)]
pub struct TypeScriptSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub line: usize,
    pub is_exported: bool,
    pub is_async: bool,
    pub variants_count: usize,
    pub fields_count: usize,
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Function,
    Class,
    Interface,
    TypeAlias,
    Enum,
    Variable,
    Import,
    Export,
    Method,
    Property,
}

// Keep the analyze_typescript_file_with_complexity_cached function for backward compat
pub async fn analyze_typescript_file_with_complexity_cached(
    path: &std::path::Path,
    _cache_manager: Option<
        std::sync::Arc<crate::services::cache::persistent_manager::PersistentCacheManager>,
    >,
) -> Result<crate::services::complexity::FileComplexityMetrics, crate::models::error::TemplateError>
{
    // Delegate to main complexity function (caching to be implemented in dispatch parser)
    analyze_typescript_file_with_complexity(path).await
}

// Keep any other types that were exported
// (The rest of the original implementation is preserved in ast_typescript_compat.rs)
