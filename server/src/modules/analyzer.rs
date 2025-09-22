use super::{ModuleError, PmatModule};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Public interface - the ONLY way other modules interact
#[async_trait]
pub trait AnalyzerModule: Send + Sync {
    async fn analyze(&self, input: &str) -> Result<Metrics, ModuleError>;
    async fn analyze_file(&self, path: &std::path::Path) -> Result<Metrics, ModuleError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub complexity: u32,
    pub lines_of_code: usize,
    pub functions: usize,
    pub classes: usize,
    pub imports: usize,
}

// Private implementation details
mod internal {
    use super::*;
    use syn;

    pub(super) struct AnalyzerCore {
        cache: lru::LruCache<String, Arc<Metrics>>,
    }

    impl AnalyzerCore {
        pub fn new() -> Self {
            Self {
                cache: lru::LruCache::new(std::num::NonZeroUsize::new(100).unwrap()),
            }
        }

        pub fn analyze_ast(&mut self, ast: &syn::File) -> Metrics {
            let mut metrics = Metrics {
                complexity: 1,
                lines_of_code: 0,
                functions: 0,
                classes: 0,
                imports: 0,
            };

            for item in &ast.items {
                match item {
                    syn::Item::Fn(_) => metrics.functions += 1,
                    syn::Item::Struct(_) => metrics.classes += 1,
                    syn::Item::Use(_) => metrics.imports += 1,
                    _ => {}
                }
            }

            metrics
        }
    }
}

// Concrete implementation hidden from other modules
#[derive(Clone)]
pub struct AnalyzerImpl {
    core: Arc<parking_lot::Mutex<internal::AnalyzerCore>>,
}

impl AnalyzerImpl {
    pub fn new() -> Self {
        Self {
            core: Arc::new(parking_lot::Mutex::new(internal::AnalyzerCore::new())),
        }
    }
}

#[async_trait]
impl AnalyzerModule for AnalyzerImpl {
    async fn analyze(&self, input: &str) -> Result<Metrics, ModuleError> {
        let ast =
            syn::parse_file(input).map_err(|e| ModuleError::ExecutionFailed(e.to_string()))?;

        let metrics = self.core.lock().analyze_ast(&ast);
        Ok(metrics)
    }

    async fn analyze_file(&self, path: &std::path::Path) -> Result<Metrics, ModuleError> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| ModuleError::ExecutionFailed(e.to_string()))?;

        self.analyze(&content).await
    }
}

#[async_trait]
impl PmatModule for AnalyzerImpl {
    type Input = String;
    type Output = Metrics;

    fn name(&self) -> &'static str {
        "AnalyzerModule"
    }

    async fn initialize(&mut self) -> Result<(), ModuleError> {
        // Any initialization logic
        Ok(())
    }

    async fn process(&self, input: Self::Input) -> Result<Self::Output, ModuleError> {
        self.analyze(&input).await
    }

    async fn shutdown(&mut self) -> Result<(), ModuleError> {
        // Cleanup if needed
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyzer_module() {
        let analyzer = AnalyzerImpl::new();

        let code = r#"
            fn main() {
                println!("Hello, world!");
            }

            struct Foo {
                bar: i32,
            }

            use std::collections::HashMap;
        "#;

        let metrics = analyzer.analyze(code).await.unwrap();
        assert_eq!(metrics.functions, 1);
        assert_eq!(metrics.classes, 1);
        assert_eq!(metrics.imports, 1);
    }
}
