// AST parsing strategies for multi-language code analysis.
//
// Uses a strategy pattern with language-specific implementations:
// Rust (syn), TypeScript/JS (swc), Python (regex), C/C++ (tree-sitter), Kotlin (tree-sitter).
//
// C, C++, and Kotlin strategy implementations are in separate include files:
// - ast_strategies_impl_c.rs
// - ast_strategies_impl_cpp.rs
// - ast_strategies_impl_kotlin.rs

use anyhow::Result;
use async_trait::async_trait;
use rustc_hash::FxHashMap;
use std::path::Path;
use std::sync::Arc;

use crate::services::context::FileContext;
use crate::services::file_classifier::FileClassifier;

// Strategy trait for language-specific AST analysis
#[async_trait]
pub trait AstStrategy: Send + Sync {
    async fn analyze(&self, path: &Path, classifier: &FileClassifier) -> Result<FileContext>;
    fn supports_extension(&self, ext: &str) -> bool;
}

// Rust language strategy
pub struct RustAstStrategy;

#[async_trait]
impl AstStrategy for RustAstStrategy {
    async fn analyze(&self, path: &Path, classifier: &FileClassifier) -> Result<FileContext> {
        crate::services::ast_rust::analyze_rust_file_with_classifier(path, Some(classifier))
            .await
            .map_err(|e| anyhow::anyhow!("Rust AST analysis error: {e}"))
    }

    fn supports_extension(&self, ext: &str) -> bool {
        ext == "rs"
    }
}

#[cfg(feature = "typescript-ast")]
// TypeScript/TSX strategy
pub struct TypeScriptAstStrategy;

#[cfg(feature = "typescript-ast")]
#[async_trait]
impl AstStrategy for TypeScriptAstStrategy {
    async fn analyze(&self, path: &Path, classifier: &FileClassifier) -> Result<FileContext> {
        crate::services::ast_typescript::analyze_typescript_file_with_classifier(
            path,
            Some(classifier),
        )
        .await
        .map_err(|e| anyhow::anyhow!("TypeScript AST analysis error: {e}"))
    }

    fn supports_extension(&self, ext: &str) -> bool {
        matches!(ext, "ts" | "tsx")
    }
}

#[cfg(feature = "typescript-ast")]
// JavaScript/JSX strategy
pub struct JavaScriptAstStrategy;

#[cfg(feature = "typescript-ast")]
#[async_trait]
impl AstStrategy for JavaScriptAstStrategy {
    async fn analyze(&self, path: &Path, classifier: &FileClassifier) -> Result<FileContext> {
        crate::services::ast_typescript::analyze_javascript_file_with_classifier(
            path,
            Some(classifier),
        )
        .await
        .map_err(|e| anyhow::anyhow!("JavaScript AST analysis error: {e}"))
    }

    fn supports_extension(&self, ext: &str) -> bool {
        matches!(ext, "js" | "jsx")
    }
}

#[cfg(feature = "python-ast")]
// Python strategy
pub struct PythonAstStrategy;

#[cfg(feature = "python-ast")]
#[async_trait]
impl AstStrategy for PythonAstStrategy {
    async fn analyze(&self, path: &Path, classifier: &FileClassifier) -> Result<FileContext> {
        crate::services::ast_python::analyze_python_file_with_classifier(path, Some(classifier))
            .await
            .map_err(|e| anyhow::anyhow!("Python AST analysis error: {e}"))
    }

    fn supports_extension(&self, ext: &str) -> bool {
        ext == "py"
    }
}

// Strategy registry to manage all language strategies
pub struct StrategyRegistry {
    strategies: FxHashMap<String, Arc<dyn AstStrategy>>,
}

impl StrategyRegistry {
    #[must_use]
    pub fn new() -> Self {
        let mut strategies: FxHashMap<String, Arc<dyn AstStrategy>> = FxHashMap::default();

        // Register all supported language strategies
        let rust_strategy = Arc::new(RustAstStrategy) as Arc<dyn AstStrategy>;
        strategies.insert("rs".to_string(), rust_strategy);

        #[cfg(feature = "typescript-ast")]
        {
            let ts_strategy = Arc::new(TypeScriptAstStrategy) as Arc<dyn AstStrategy>;
            strategies.insert("ts".to_string(), ts_strategy.clone());
            strategies.insert("tsx".to_string(), ts_strategy);

            let js_strategy = Arc::new(JavaScriptAstStrategy) as Arc<dyn AstStrategy>;
            strategies.insert("js".to_string(), js_strategy.clone());
            strategies.insert("jsx".to_string(), js_strategy);
        }

        #[cfg(feature = "python-ast")]
        {
            let py_strategy = Arc::new(PythonAstStrategy) as Arc<dyn AstStrategy>;
            strategies.insert("py".to_string(), py_strategy);
        }

        #[cfg(feature = "c-ast")]
        {
            let c_strategy = Arc::new(CAstStrategy) as Arc<dyn AstStrategy>;
            strategies.insert("c".to_string(), c_strategy.clone());
            strategies.insert("h".to_string(), c_strategy);

            let cpp_strategy = Arc::new(CppAstStrategy) as Arc<dyn AstStrategy>;
            strategies.insert("cpp".to_string(), cpp_strategy.clone());
            strategies.insert("cc".to_string(), cpp_strategy.clone());
            strategies.insert("cxx".to_string(), cpp_strategy.clone());
            strategies.insert("hpp".to_string(), cpp_strategy.clone());
            strategies.insert("hxx".to_string(), cpp_strategy);
        }

        #[cfg(feature = "kotlin-ast")]
        {
            let kotlin_strategy = Arc::new(KotlinAstStrategy) as Arc<dyn AstStrategy>;
            strategies.insert("kt".to_string(), kotlin_strategy.clone());
            strategies.insert("kts".to_string(), kotlin_strategy);
        }

        Self { strategies }
    }

    #[must_use]
    pub fn get_strategy(&self, extension: &str) -> Option<Arc<dyn AstStrategy>> {
        self.strategies.get(extension).cloned()
    }

    pub fn register_strategy(&mut self, extension: String, strategy: Arc<dyn AstStrategy>) {
        self.strategies.insert(extension, strategy);
    }
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Language-specific strategy implementations
include!("ast_strategies_impl_c.rs");
include!("ast_strategies_impl_cpp.rs");
include!("ast_strategies_impl_kotlin.rs");
