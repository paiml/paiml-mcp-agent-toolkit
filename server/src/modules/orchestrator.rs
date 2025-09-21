use super::{ModuleError, PmatModule};
use super::analyzer::AnalyzerModule;
use super::transformer::TransformerModule;
use super::validator::{ValidatorModule, Thresholds, ValidationResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Orchestrator can depend on all other modules
#[async_trait]
pub trait OrchestratorModule: Send + Sync {
    async fn analyze_and_validate(&self, code: &str) -> Result<ValidationResult, ModuleError>;
    async fn analyze_transform_validate(&self, code: &str) -> Result<ProcessResult, ModuleError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessResult {
    pub original_metrics: super::analyzer::Metrics,
    pub transformed_code: String,
    pub final_metrics: super::analyzer::Metrics,
    pub validation: ValidationResult,
    pub improved: bool,
}

pub struct OrchestratorImpl {
    analyzer: Arc<dyn AnalyzerModule>,
    transformer: Arc<dyn TransformerModule>,
    validator: Arc<dyn ValidatorModule>,
    thresholds: Thresholds,
}

impl OrchestratorImpl {
    pub fn new(
        analyzer: Arc<dyn AnalyzerModule>,
        transformer: Arc<dyn TransformerModule>,
        validator: Arc<dyn ValidatorModule>,
    ) -> Self {
        Self {
            analyzer,
            transformer,
            validator,
            thresholds: Thresholds::default(),
        }
    }

    pub fn with_thresholds(mut self, thresholds: Thresholds) -> Self {
        self.thresholds = thresholds;
        self
    }
}

#[async_trait]
impl OrchestratorModule for OrchestratorImpl {
    async fn analyze_and_validate(&self, code: &str) -> Result<ValidationResult, ModuleError> {
        let metrics = self.analyzer.analyze(code).await?;
        Ok(self.validator.validate(&metrics, &self.thresholds).await)
    }

    async fn analyze_transform_validate(&self, code: &str) -> Result<ProcessResult, ModuleError> {
        // Step 1: Analyze original code
        let original_metrics = self.analyzer.analyze(code).await?;

        // Step 2: Transform the code
        let transform_result = self.transformer.transform(code).await?;

        // Step 3: Analyze transformed code
        let final_metrics = self.analyzer.analyze(&transform_result.transformed).await?;

        // Step 4: Validate final result
        let validation = self.validator.validate(&final_metrics, &self.thresholds).await;

        // Determine if transformation improved the code
        let improved = final_metrics.complexity < original_metrics.complexity;

        Ok(ProcessResult {
            original_metrics,
            transformed_code: transform_result.transformed,
            final_metrics,
            validation,
            improved,
        })
    }
}

#[async_trait]
impl PmatModule for OrchestratorImpl {
    type Input = String;
    type Output = ProcessResult;

    fn name(&self) -> &'static str {
        "OrchestratorModule"
    }

    async fn initialize(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    async fn process(&self, input: Self::Input) -> Result<Self::Output, ModuleError> {
        self.analyze_transform_validate(&input).await
    }

    async fn shutdown(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }
}

// Workflow builder for complex orchestrations
pub struct WorkflowBuilder {
    steps: Vec<WorkflowStep>,
}

#[derive(Clone)]
pub enum WorkflowStep {
    Analyze,
    Transform,
    Validate,
    Custom(String),
}

impl WorkflowBuilder {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn add_step(mut self, step: WorkflowStep) -> Self {
        self.steps.push(step);
        self
    }

    pub fn build(self) -> Workflow {
        Workflow { steps: self.steps }
    }
}

pub struct Workflow {
    steps: Vec<WorkflowStep>,
}

impl Workflow {
    pub async fn execute(
        &self,
        code: &str,
        orchestrator: &OrchestratorImpl,
    ) -> Result<WorkflowResult, ModuleError> {
        let mut results = Vec::new();
        let mut current_code = code.to_string();

        for step in &self.steps {
            match step {
                WorkflowStep::Analyze => {
                    let metrics = orchestrator.analyzer.analyze(&current_code).await?;
                    results.push(StepResult::Analysis(metrics));
                }
                WorkflowStep::Transform => {
                    let transform = orchestrator.transformer.transform(&current_code).await?;
                    current_code = transform.transformed.clone();
                    results.push(StepResult::Transform(transform));
                }
                WorkflowStep::Validate => {
                    let metrics = orchestrator.analyzer.analyze(&current_code).await?;
                    let validation = orchestrator
                        .validator
                        .validate(&metrics, &orchestrator.thresholds)
                        .await;
                    results.push(StepResult::Validation(validation));
                }
                WorkflowStep::Custom(name) => {
                    results.push(StepResult::Custom(name.clone()));
                }
            }
        }

        Ok(WorkflowResult {
            final_code: current_code,
            step_results: results,
        })
    }
}

#[derive(Debug)]
pub struct WorkflowResult {
    pub final_code: String,
    pub step_results: Vec<StepResult>,
}

#[derive(Debug)]
pub enum StepResult {
    Analysis(super::analyzer::Metrics),
    Transform(super::transformer::TransformResult),
    Validation(ValidationResult),
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::analyzer::AnalyzerImpl;
    use super::super::transformer::TransformerImpl;
    use super::super::validator::ValidatorImpl;

    #[tokio::test]
    async fn test_orchestrator() {
        let analyzer = Arc::new(AnalyzerImpl::new()) as Arc<dyn AnalyzerModule>;
        let transformer = Arc::new(TransformerImpl::new()) as Arc<dyn TransformerModule>;
        let validator = Arc::new(ValidatorImpl::new()) as Arc<dyn ValidatorModule>;

        let orchestrator = OrchestratorImpl::new(
            analyzer.clone(),
            transformer.clone(),
            validator.clone(),
        );

        let code = "fn test() { println!(\"test\"); }";
        let result = orchestrator.analyze_and_validate(code).await.unwrap();

        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_workflow() {
        let analyzer = Arc::new(AnalyzerImpl::new()) as Arc<dyn AnalyzerModule>;
        let transformer = Arc::new(TransformerImpl::new()) as Arc<dyn TransformerModule>;
        let validator = Arc::new(ValidatorImpl::new()) as Arc<dyn ValidatorModule>;

        let orchestrator = OrchestratorImpl::new(analyzer, transformer, validator);

        let workflow = WorkflowBuilder::new()
            .add_step(WorkflowStep::Analyze)
            .add_step(WorkflowStep::Transform)
            .add_step(WorkflowStep::Validate)
            .build();

        let code = "fn test() {}";
        let result = workflow.execute(code, &orchestrator).await.unwrap();

        assert_eq!(result.final_code, "pub fn test() {}");
        assert_eq!(result.step_results.len(), 3);
    }
}