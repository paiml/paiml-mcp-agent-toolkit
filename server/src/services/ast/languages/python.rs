// Toyota Way: Unified Python AST Strategy

use super::super::AstStrategy;
use crate::services::context::FileContext;
use crate::services::file_classifier::FileClassifier;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

/// Python AST analysis strategy
#[cfg(feature = "python-ast")]
pub struct PythonStrategy;

#[cfg(feature = "python-ast")]
impl Default for PythonStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl PythonStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "python-ast")]
#[async_trait]
impl AstStrategy for PythonStrategy {
    async fn analyze(&self, file_path: &Path, _classifier: &FileClassifier) -> Result<FileContext> {
        // Delegate to existing Python analysis
        // Convert TemplateError to anyhow::Error
        let context = crate::services::ast_python::analyze_python_file(file_path)
            .await
            .map_err(|e| anyhow::anyhow!("Python analysis failed: {}", e))?;
        Ok(context)
    }

    fn primary_extension(&self) -> &'static str {
        "py"
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["py", "pyi", "pyw"]
    }

    fn language_name(&self) -> &'static str {
        "Python"
    }
}
