use crate::models::pdmt::{
    EnforcementMode, PdmtTodo, PdmtTodoList, QualityResults, QualityValidationResult,
    ValidationOutcome,
};
use crate::models::proxy::{ProxyMode, ProxyOperation, ProxyRequest, ProxyResponse};
use crate::services::quality_proxy::QualityProxyService;
use anyhow::{Context, Result};
use tracing::{debug, info, warn};

/// Quality enforcement pipeline for PDMT-generated todos
pub struct PdmtQualityEnforcer {
    _quality_proxy: QualityProxyService,
}

impl PdmtQualityEnforcer {
    pub fn new() -> Self {
        Self {
            _quality_proxy: QualityProxyService::new(),
        }
    }

    /// Execute full quality enforcement pipeline for a todo list
    pub async fn enforce_quality_standards(
        &self,
        todo_list: &PdmtTodoList,
    ) -> Result<QualityValidationResult> {
        info!(
            "Starting quality enforcement for {} todos",
            todo_list.todos.len()
        );

        let mut all_results = Vec::new();
        let mut recommendations = Vec::new();

        for todo in &todo_list.todos {
            let validation_result = self.validate_single_todo(todo, todo_list).await?;

            if !validation_result.overall_passed {
                recommendations.push(format!(
                    "Todo '{}' failed quality validation. Review violations and fix.",
                    todo.content
                ));
            }

            all_results.push(validation_result);
        }

        // Aggregate all results
        let overall_passed = all_results.iter().all(|r| r.overall_passed);

        // Combine detailed results from all todos
        let detailed_results = self.aggregate_results(&all_results);

        Ok(QualityValidationResult {
            overall_passed,
            detailed_results,
            recommendations,
        })
    }

    /// Validate a single todo against quality standards
    async fn validate_single_todo(
        &self,
        todo: &PdmtTodo,
        todo_list: &PdmtTodoList,
    ) -> Result<QualityValidationResult> {
        debug!("Validating todo: {}", todo.content);

        // Phase 1: Structure validation
        let structure_result = self.validate_todo_structure(todo)?;

        // Phase 2: Coverage requirements
        let coverage_result = self.validate_coverage_requirements(todo)?;

        // Phase 3: Doctest requirements
        let doctest_result = self.validate_doctest_requirements(todo)?;

        // Phase 4: Property test requirements
        let property_result = self.validate_property_test_requirements(todo)?;

        // Phase 5: Example requirements
        let example_result = self.validate_example_requirements(todo)?;

        // Phase 6: SATD detection
        let satd_result = self.validate_satd_compliance(todo)?;

        // Phase 7: Quality proxy validation
        let proxy_result = self
            .run_quality_proxy_validation(todo, &todo_list.quality_config.enforcement_mode)
            .await?;

        let overall_passed = [
            &structure_result,
            &coverage_result,
            &doctest_result,
            &property_result,
            &example_result,
            &satd_result,
            &proxy_result,
        ]
        .iter()
        .all(|r| r.passed);

        Ok(QualityValidationResult {
            overall_passed,
            detailed_results: QualityResults {
                structure_result,
                coverage_result,
                doctest_result,
                property_result,
                example_result,
                satd_result,
                proxy_result,
            },
            recommendations: self.generate_recommendations(todo, overall_passed),
        })
    }

    /// Validate todo structure and content
    fn validate_todo_structure(&self, todo: &PdmtTodo) -> Result<ValidationOutcome> {
        let mut violations = Vec::new();

        // Check content length
        if todo.content.len() < 15 {
            violations.push("Todo content too short (min 15 chars)".to_string());
        }
        if todo.content.len() > 80 {
            violations.push("Todo content too long (max 80 chars)".to_string());
        }

        // Check for action verb
        let action_verbs = [
            "implement",
            "create",
            "build",
            "fix",
            "refactor",
            "add",
            "update",
            "write",
            "document",
        ];
        let has_action_verb = action_verbs
            .iter()
            .any(|verb| todo.content.to_lowercase().starts_with(verb));

        if !has_action_verb {
            violations.push("Todo must start with an action verb".to_string());
        }

        // Check time estimate
        if todo.estimated_hours < 0.5 || todo.estimated_hours > 8.0 {
            violations.push("Time estimate must be between 0.5 and 8.0 hours".to_string());
        }

        if violations.is_empty() {
            Ok(ValidationOutcome::success(
                "Todo structure validation passed".to_string(),
            ))
        } else {
            Ok(ValidationOutcome::failure(
                "Todo structure validation failed".to_string(),
                violations,
            ))
        }
    }

    /// Validate coverage requirements
    fn validate_coverage_requirements(&self, todo: &PdmtTodo) -> Result<ValidationOutcome> {
        let coverage_req = todo.quality_gates.coverage_requirement;

        if coverage_req < 80.0 {
            Ok(ValidationOutcome::failure(
                "Coverage requirement too low".to_string(),
                vec![format!(
                    "Coverage requirement {}% is below minimum 80%",
                    coverage_req
                )],
            ))
        } else {
            Ok(ValidationOutcome::success(format!(
                "Coverage requirement {}% meets standards",
                coverage_req
            )))
        }
    }

    /// Validate doctest requirements
    fn validate_doctest_requirements(&self, todo: &PdmtTodo) -> Result<ValidationOutcome> {
        if !todo.quality_gates.doctest_requirement {
            Ok(ValidationOutcome::failure(
                "Doctests are mandatory".to_string(),
                vec!["Doctest requirement must be enabled".to_string()],
            ))
        } else {
            Ok(ValidationOutcome::success(
                "Doctest requirement enabled".to_string(),
            ))
        }
    }

    /// Validate property test requirements
    fn validate_property_test_requirements(&self, todo: &PdmtTodo) -> Result<ValidationOutcome> {
        if !todo.quality_gates.property_test_requirement {
            warn!("Property tests not required for todo: {}", todo.content);
        }
        Ok(ValidationOutcome::success(
            "Property test requirements validated".to_string(),
        ))
    }

    /// Validate example requirements
    fn validate_example_requirements(&self, todo: &PdmtTodo) -> Result<ValidationOutcome> {
        if !todo.quality_gates.example_requirement {
            warn!("Examples not required for todo: {}", todo.content);
        }
        Ok(ValidationOutcome::success(
            "Example requirements validated".to_string(),
        ))
    }

    /// Validate SATD compliance (zero tolerance)
    fn validate_satd_compliance(&self, todo: &PdmtTodo) -> Result<ValidationOutcome> {
        if todo.quality_gates.satd_tolerance {
            Ok(ValidationOutcome::failure(
                "SATD tolerance must be zero".to_string(),
                vec!["Project enforces zero SATD tolerance".to_string()],
            ))
        } else {
            Ok(ValidationOutcome::success(
                "Zero SATD tolerance enforced".to_string(),
            ))
        }
    }

    /// Run quality proxy validation for generated code
    async fn run_quality_proxy_validation(
        &self,
        todo: &PdmtTodo,
        enforcement_mode: &EnforcementMode,
    ) -> Result<ValidationOutcome> {
        // For now, we simulate proxy validation since we don't have actual generated code
        // In a real implementation, this would validate actual generated code
        let proxy_mode = match enforcement_mode {
            EnforcementMode::Strict => ProxyMode::Strict,
            EnforcementMode::Advisory => ProxyMode::Advisory,
            EnforcementMode::AutoFix => ProxyMode::AutoFix,
        };

        debug!(
            "Running quality proxy validation for todo {} in {:?} mode",
            todo.id, proxy_mode
        );

        // Simulate proxy validation based on quality gates
        let mut violations = Vec::new();

        if todo.quality_gates.complexity_limit > 20 {
            violations.push(format!(
                "Complexity limit {} exceeds maximum allowed 20",
                todo.quality_gates.complexity_limit
            ));
        }

        if violations.is_empty() {
            Ok(ValidationOutcome::success(format!(
                "Quality proxy validation passed in {:?} mode",
                proxy_mode
            )))
        } else {
            Ok(ValidationOutcome::failure(
                "Quality proxy validation failed".to_string(),
                violations,
            ))
        }
    }

    /// Generate improvement recommendations
    fn generate_recommendations(&self, todo: &PdmtTodo, passed: bool) -> Vec<String> {
        let mut recommendations = Vec::new();

        if !passed {
            recommendations.push(format!(
                "Review and fix quality violations for: {}",
                todo.content
            ));

            if todo.quality_gates.coverage_requirement < 80.0 {
                recommendations.push("Increase coverage requirement to at least 80%".to_string());
            }

            if todo.quality_gates.complexity_limit > 8 {
                recommendations.push("Reduce complexity limit to 8 or lower".to_string());
            }
        }

        recommendations
    }

    /// Aggregate results from multiple todos
    fn aggregate_results(&self, results: &[QualityValidationResult]) -> QualityResults {
        // For simplicity, take the first result's detailed results
        // In a real implementation, this would properly aggregate all results
        if let Some(first) = results.first() {
            first.detailed_results.clone()
        } else {
            QualityResults {
                structure_result: ValidationOutcome::success("No todos to validate".to_string()),
                coverage_result: ValidationOutcome::success("No todos to validate".to_string()),
                doctest_result: ValidationOutcome::success("No todos to validate".to_string()),
                property_result: ValidationOutcome::success("No todos to validate".to_string()),
                example_result: ValidationOutcome::success("No todos to validate".to_string()),
                satd_result: ValidationOutcome::success("No todos to validate".to_string()),
                proxy_result: ValidationOutcome::success("No todos to validate".to_string()),
            }
        }
    }
}

impl Default for PdmtQualityEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

/// Integration with existing quality proxy for code validation
pub async fn validate_generated_code_with_proxy(
    proxy: &QualityProxyService,
    code: &str,
    file_path: &str,
    todo: &PdmtTodo,
) -> Result<ProxyResponse> {
    let request = ProxyRequest {
        operation: ProxyOperation::Write,
        file_path: file_path.to_string(),
        content: Some(code.to_string()),
        old_content: None,
        new_content: None,
        mode: ProxyMode::Strict,
        quality_config: crate::models::proxy::QualityConfig {
            max_complexity: todo.quality_gates.complexity_limit,
            allow_satd: todo.quality_gates.satd_tolerance,
            require_docs: todo.quality_gates.doctest_requirement,
            auto_format: true,
        },
    };

    proxy
        .proxy_operation(request)
        .await
        .context("Failed to validate generated code with quality proxy")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::pdmt::{PdmtQualityConfig, TodoPriority};

    #[tokio::test]
    async fn test_quality_enforcement_basic() {
        let enforcer = PdmtQualityEnforcer::new();
        let todo = PdmtTodo::new(
            "Implement user authentication".to_string(),
            TodoPriority::High,
        );
        let todo_list = PdmtTodoList {
            project_name: "test".to_string(),
            todos: vec![todo],
            quality_config: PdmtQualityConfig::default(),
            generated_at: "2024-01-01".to_string(),
            deterministic_seed: 42,
        };

        let result = enforcer
            .enforce_quality_standards(&todo_list)
            .await
            .unwrap();
        assert!(result.overall_passed);
    }

    #[test]
    fn test_structure_validation() {
        let enforcer = PdmtQualityEnforcer::new();
        let mut todo = PdmtTodo::new("Do something".to_string(), TodoPriority::Low);
        todo.content = "x".to_string(); // Too short

        let result = enforcer.validate_todo_structure(&todo).unwrap();
        assert!(!result.passed);
        assert!(!result.violations.is_empty());
    }
}
