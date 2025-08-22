//! PDMT todo generation from roadmap tasks

use super::*;
use crate::services::pdmt_service::PdmtService;
use crate::models::pdmt::{PdmtTodo, PdmtQualityConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Quality-enforced todo with validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityEnforcedTodo {
    pub id: String,
    pub task_id: String,  // PMAT-XXXX
    pub description: String,
    pub implementation_spec: String,
    pub quality_requirements: QualityRequirements,
    pub validation_commands: Vec<String>,
    pub success_criteria: Vec<String>,
    pub estimated_time: std::time::Duration,
    pub dependencies: Vec<String>,
}

/// Quality requirements for a todo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRequirements {
    pub max_complexity: u32,
    pub min_test_coverage: u8,
    pub required_docs: bool,
    pub satd_allowed: u32,  // Always 0
    pub lint_compliance: bool,
}

impl Default for QualityRequirements {
    fn default() -> Self {
        Self {
            max_complexity: 20,
            min_test_coverage: 80,
            required_docs: true,
            satd_allowed: 0,
            lint_compliance: true,
        }
    }
}

/// Generates PDMT todos from roadmap tasks
pub struct RoadmapTodoGenerator {
    pdmt_service: PdmtService,
    quality_config: QualityGateConfig,
}

impl RoadmapTodoGenerator {
    pub fn new(quality_config: QualityGateConfig) -> Self {
        Self {
            pdmt_service: PdmtService::new(),
            quality_config,
        }
    }
    
    /// Generate todos for an entire sprint
    pub async fn generate_sprint_todos(&self, sprint: &Sprint) -> Result<Vec<QualityEnforcedTodo>> {
        let mut todos = Vec::new();
        
        for task in &sprint.tasks {
            if task.status != TaskStatus::Completed {
                let task_todos = self.generate_task_todos(task).await?;
                todos.extend(task_todos);
            }
        }
        
        Ok(todos)
    }
    
    /// Generate todos for a specific task
    pub async fn generate_task_todos(&self, task: &Task) -> Result<Vec<QualityEnforcedTodo>> {
        // Generate PDMT todos with deterministic seed
        let quality_config = PdmtQualityConfig {
            enforcement_mode: crate::models::pdmt::EnforcementMode::Strict,
            coverage_threshold: self.quality_config.coverage_min as f32,
            max_complexity: self.quality_config.complexity_max,
            require_doctests: self.quality_config.documentation_required,
            require_property_tests: true,
            require_examples: false,
            zero_satd_tolerance: self.quality_config.satd_tolerance == 0,
        };
        
        let todo_list = self.pdmt_service.generate_todos(
            vec![task.description.clone()],
            Some(task.id.clone()),
            &task.complexity.to_string().to_lowercase(),
            quality_config,
        )?;
        
        let pdmt_todos = todo_list.todos;
        
        // Enhance with quality requirements
        let mut quality_todos = Vec::new();
        for (i, pdmt_todo) in pdmt_todos.into_iter().enumerate() {
            let quality_todo = self.enhance_with_quality(task, &pdmt_todo, i)?;
            quality_todos.push(quality_todo);
        }
        
        Ok(quality_todos)
    }
    
    /// Enhance a PDMT todo with quality requirements
    fn enhance_with_quality(&self, task: &Task, pdmt_todo: &PdmtTodo, index: usize) -> Result<QualityEnforcedTodo> {
        let mut success_criteria = pdmt_todo.success_criteria.clone();
        
        // Create validation commands list
        let mut validation_commands = vec![
            pdmt_todo.validation_commands.unit_tests.clone(),
            pdmt_todo.validation_commands.doctests.clone(),
            pdmt_todo.validation_commands.coverage_check.clone(),
            pdmt_todo.validation_commands.quality_proxy.clone(),
        ];
        
        // Add quality gate validation
        validation_commands.push(format!("pmat roadmap quality-check --task-id {}", task.id));
        validation_commands.push("pmat quality-gate --file-path .".to_string());
        
        // Add quality success criteria
        success_criteria.push("All quality gates pass".to_string());
        success_criteria.push(format!("Complexity ≤ {}", self.quality_config.complexity_max));
        success_criteria.push(format!("Test coverage ≥ {}%", self.quality_config.coverage_min));
        success_criteria.push("Zero SATD violations".to_string());
        
        // Add documentation requirement
        if self.quality_config.documentation_required {
            validation_commands.push("pmat roadmap check-docs".to_string());
            success_criteria.push("Documentation updated".to_string());
        }
        
        // Add lint compliance
        if self.quality_config.lint_compliance {
            validation_commands.push("make lint".to_string());
            success_criteria.push("Zero lint warnings".to_string());
        }
        
        Ok(QualityEnforcedTodo {
            id: format!("{}-{:03}", task.id, index + 1),
            task_id: task.id.clone(),
            description: pdmt_todo.content.clone(),
            implementation_spec: pdmt_todo.implementation_specs.primary_files.first().cloned().unwrap_or_default(),
            quality_requirements: QualityRequirements {
                max_complexity: self.quality_config.complexity_max,
                min_test_coverage: self.quality_config.coverage_min,
                required_docs: self.quality_config.documentation_required,
                satd_allowed: self.quality_config.satd_tolerance,
                lint_compliance: self.quality_config.lint_compliance,
            },
            validation_commands,
            success_criteria,
            estimated_time: std::time::Duration::from_secs(3600), // Default 1 hour
            dependencies: pdmt_todo.dependencies.clone(),
        })
    }
    
    /// Export todos to markdown format
    pub fn export_todos_markdown(&self, todos: &[QualityEnforcedTodo]) -> String {
        let mut output = String::new();
        
        output.push_str("# Sprint Todos (PDMT Generated)\n\n");
        output.push_str("*Generated with deterministic seeds for reproducibility*\n\n");
        
        // Group by task
        let mut tasks_map: HashMap<String, Vec<&QualityEnforcedTodo>> = HashMap::new();
        for todo in todos {
            tasks_map.entry(todo.task_id.clone()).or_default().push(todo);
        }
        
        for (task_id, task_todos) in tasks_map {
            output.push_str(&format!("## Task {}\n\n", task_id));
            
            for todo in task_todos {
                output.push_str(&format!("### {} - {}\n\n", todo.id, todo.description));
                
                if !todo.implementation_spec.is_empty() {
                    output.push_str("**Implementation:**\n");
                    output.push_str(&format!("{}\n\n", todo.implementation_spec));
                }
                
                output.push_str("**Quality Requirements:**\n");
                output.push_str(&format!("- Max Complexity: {}\n", todo.quality_requirements.max_complexity));
                output.push_str(&format!("- Min Test Coverage: {}%\n", todo.quality_requirements.min_test_coverage));
                output.push_str(&format!("- Documentation Required: {}\n", todo.quality_requirements.required_docs));
                output.push_str(&format!("- SATD Allowed: {}\n", todo.quality_requirements.satd_allowed));
                output.push_str(&format!("- Lint Compliance: {}\n\n", todo.quality_requirements.lint_compliance));
                
                output.push_str("**Validation Commands:**\n```bash\n");
                for cmd in &todo.validation_commands {
                    output.push_str(&format!("{}\n", cmd));
                }
                output.push_str("```\n\n");
                
                output.push_str("**Success Criteria:**\n");
                for criterion in &todo.success_criteria {
                    output.push_str(&format!("- [ ] {}\n", criterion));
                }
                output.push('\n');
                
                if !todo.dependencies.is_empty() {
                    output.push_str("**Dependencies:**\n");
                    for dep in &todo.dependencies {
                        output.push_str(&format!("- {}\n", dep));
                    }
                    output.push('\n');
                }
                
                output.push_str("---\n\n");
            }
        }
        
        output
    }
}