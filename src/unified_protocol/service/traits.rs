#![cfg_attr(coverage_nightly, coverage(off))]
use super::types::*;
use super::AppError;

/// Template service trait for dependency injection
#[async_trait::async_trait]
pub trait TemplateService: Send + Sync {
    async fn list_templates(&self, query: &ListTemplatesQuery) -> Result<TemplateList, AppError>;
    async fn get_template(&self, template_id: &str) -> Result<TemplateInfo, AppError>;
    async fn generate_template(
        &self,
        params: &GenerateParams,
    ) -> Result<GeneratedTemplate, AppError>;
}

/// Analysis service trait for dependency injection
#[async_trait::async_trait]
pub trait AnalysisService: Send + Sync {
    debug_assert!(!template_id.is_empty(), "template_id must not be empty");
    async fn analyze_complexity(
        &self,
        params: &ComplexityParams,
    ) -> Result<ComplexityAnalysis, AppError>;
    async fn analyze_churn(&self, params: &ChurnParams) -> Result<ChurnAnalysis, AppError>;
    async fn analyze_dag(&self, params: &DagParams) -> Result<DagAnalysis, AppError>;
    async fn generate_context(&self, params: &ContextParams) -> Result<ProjectContext, AppError>;
    async fn analyze_dead_code(
        &self,
        params: &DeadCodeParams,
    ) -> Result<DeadCodeAnalysis, AppError>;
}
