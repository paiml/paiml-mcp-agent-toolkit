use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
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
            .map_err(|e| anyhow::anyhow!("Rust AST analysis error: {}", e))
    }

    fn supports_extension(&self, ext: &str) -> bool {
        ext == "rs"
    }
}

// TypeScript/TSX strategy
pub struct TypeScriptAstStrategy;

#[async_trait]
impl AstStrategy for TypeScriptAstStrategy {
    async fn analyze(&self, path: &Path, classifier: &FileClassifier) -> Result<FileContext> {
        crate::services::ast_typescript::analyze_typescript_file_with_classifier(
            path,
            Some(classifier),
        )
        .await
        .map_err(|e| anyhow::anyhow!("TypeScript AST analysis error: {}", e))
    }

    fn supports_extension(&self, ext: &str) -> bool {
        matches!(ext, "ts" | "tsx")
    }
}

// JavaScript/JSX strategy
pub struct JavaScriptAstStrategy;

#[async_trait]
impl AstStrategy for JavaScriptAstStrategy {
    async fn analyze(&self, path: &Path, classifier: &FileClassifier) -> Result<FileContext> {
        crate::services::ast_typescript::analyze_javascript_file_with_classifier(
            path,
            Some(classifier),
        )
        .await
        .map_err(|e| anyhow::anyhow!("JavaScript AST analysis error: {}", e))
    }

    fn supports_extension(&self, ext: &str) -> bool {
        matches!(ext, "js" | "jsx")
    }
}

// Python strategy
pub struct PythonAstStrategy;

#[async_trait]
impl AstStrategy for PythonAstStrategy {
    async fn analyze(&self, path: &Path, classifier: &FileClassifier) -> Result<FileContext> {
        crate::services::ast_python::analyze_python_file_with_classifier(path, Some(classifier))
            .await
            .map_err(|e| anyhow::anyhow!("Python AST analysis error: {}", e))
    }

    fn supports_extension(&self, ext: &str) -> bool {
        ext == "py"
    }
}

// Strategy registry to manage all language strategies
pub struct StrategyRegistry {
    strategies: HashMap<String, Arc<dyn AstStrategy>>,
}

impl StrategyRegistry {
    pub fn new() -> Self {
        let mut strategies: HashMap<String, Arc<dyn AstStrategy>> = HashMap::new();

        // Register all supported language strategies
        let rust_strategy = Arc::new(RustAstStrategy) as Arc<dyn AstStrategy>;
        strategies.insert("rs".to_string(), rust_strategy);

        let ts_strategy = Arc::new(TypeScriptAstStrategy) as Arc<dyn AstStrategy>;
        strategies.insert("ts".to_string(), ts_strategy.clone());
        strategies.insert("tsx".to_string(), ts_strategy);

        let js_strategy = Arc::new(JavaScriptAstStrategy) as Arc<dyn AstStrategy>;
        strategies.insert("js".to_string(), js_strategy.clone());
        strategies.insert("jsx".to_string(), js_strategy);

        let py_strategy = Arc::new(PythonAstStrategy) as Arc<dyn AstStrategy>;
        strategies.insert("py".to_string(), py_strategy);

        Self { strategies }
    }

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
