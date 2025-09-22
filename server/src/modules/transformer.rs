use super::analyzer::AnalyzerModule;
use super::{ModuleError, PmatModule};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Public interface - can ONLY use public interfaces of other modules
#[async_trait]
pub trait TransformerModule: Send + Sync {
    async fn transform(&self, ast: &str) -> Result<TransformResult, ModuleError>;
    async fn refactor(&self, code: &str, rules: &[RefactorRule]) -> Result<String, ModuleError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformResult {
    pub original: String,
    pub transformed: String,
    pub changes: Vec<Change>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub line: usize,
    pub description: String,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Addition,
    Deletion,
    Modification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorRule {
    pub pattern: String,
    pub replacement: String,
    pub description: String,
}

// Private implementation
struct TransformerCore {
    transform_log: Vec<TransformResult>,
    max_log_size: usize,
}

impl TransformerCore {
    fn new() -> Self {
        Self {
            transform_log: Vec::new(),
            max_log_size: 100,
        }
    }

    fn apply_transform(&mut self, code: &str) -> TransformResult {
        // Simplified transformation logic
        let transformed = code.replace("fn ", "pub fn ");

        let changes = vec![Change {
            line: 1,
            description: "Made functions public".to_string(),
            change_type: ChangeType::Modification,
        }];

        let result = TransformResult {
            original: code.to_string(),
            transformed,
            changes,
        };

        // Log the transformation
        if self.transform_log.len() >= self.max_log_size {
            self.transform_log.remove(0);
        }
        self.transform_log.push(result.clone());

        result
    }
}

#[derive(Clone)]
pub struct TransformerImpl {
    core: Arc<parking_lot::Mutex<TransformerCore>>,
    analyzer: Option<Arc<dyn AnalyzerModule>>,
}

impl Default for TransformerImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl TransformerImpl {
    pub fn new() -> Self {
        Self {
            core: Arc::new(parking_lot::Mutex::new(TransformerCore::new())),
            analyzer: None,
        }
    }

    pub fn with_analyzer(mut self, analyzer: Arc<dyn AnalyzerModule>) -> Self {
        self.analyzer = Some(analyzer);
        self
    }
}

#[async_trait]
impl TransformerModule for TransformerImpl {
    async fn transform(&self, code: &str) -> Result<TransformResult, ModuleError> {
        // Can use analyzer if available
        if let Some(analyzer) = &self.analyzer {
            let metrics = analyzer.analyze(code).await?;
            if metrics.complexity > 10 {
                // High complexity - might need different transformation
            }
        }

        Ok(self.core.lock().apply_transform(code))
    }

    async fn refactor(&self, code: &str, rules: &[RefactorRule]) -> Result<String, ModuleError> {
        let mut result = code.to_string();

        for rule in rules {
            result = result.replace(&rule.pattern, &rule.replacement);
        }

        Ok(result)
    }
}

#[async_trait]
impl PmatModule for TransformerImpl {
    type Input = String;
    type Output = TransformResult;

    fn name(&self) -> &'static str {
        "TransformerModule"
    }

    async fn initialize(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    async fn process(&self, input: Self::Input) -> Result<Self::Output, ModuleError> {
        self.transform(&input).await
    }

    async fn shutdown(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transformer_module() {
        let transformer = TransformerImpl::new();

        let code = "fn test() {}";
        let result = transformer.transform(code).await.unwrap();

        assert_eq!(result.transformed, "pub fn test() {}");
        assert_eq!(result.changes.len(), 1);
    }

    #[tokio::test]
    async fn test_refactor() {
        let transformer = TransformerImpl::new();

        let rules = vec![RefactorRule {
            pattern: "let x".to_string(),
            replacement: "let mut x".to_string(),
            description: "Make variable mutable".to_string(),
        }];

        let code = "let x = 42;";
        let result = transformer.refactor(code, &rules).await.unwrap();

        assert_eq!(result, "let mut x = 42;");
    }
}
