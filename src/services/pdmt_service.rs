#![cfg_attr(coverage_nightly, coverage(off))]
use crate::models::pdmt::{
    ImplementationSpecs, PdmtQualityConfig, PdmtTodo, PdmtTodoList, TodoPriority, TodoQualityGates,
    TodoStatus, ValidationCommands,
};
use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

/// Service for generating deterministic, quality-enforced todo lists
pub struct PdmtService {
    deterministic_seed: u64,
}

impl PdmtService {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            deterministic_seed: 42, // Fixed seed for determinism
        }
    }

    /// Generate a deterministic todo list from requirements
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn generate_todos(
        &self,
        requirements: Vec<String>,
        project_name: Option<String>,
        granularity: &str,
        quality_config: PdmtQualityConfig,
    ) -> Result<PdmtTodoList> {
        info!(
            "Generating deterministic todos for {} requirements",
            requirements.len()
        );

        let project_name = project_name.unwrap_or_else(|| "project".to_string());
        let mut todos = Vec::new();
        let mut dependency_map = HashMap::new();

        // Process each requirement into actionable todos
        for (idx, requirement) in requirements.iter().enumerate() {
            let requirement_todos = self.requirement_to_todos(
                requirement,
                granularity,
                &quality_config,
                idx,
                &mut dependency_map,
            )?;
            todos.extend(requirement_todos);
        }

        // Set dependencies based on logical ordering
        self.set_dependencies(&mut todos, &dependency_map);

        let todo_list = PdmtTodoList {
            project_name,
            todos,
            quality_config,
            generated_at: Utc::now().to_rfc3339(),
            deterministic_seed: self.deterministic_seed,
        };

        info!(
            "Generated {} deterministic todos with quality enforcement",
            todo_list.todos.len()
        );

        Ok(todo_list)
    }
}

impl Default for PdmtService {
    fn default() -> Self {
        Self::new()
    }
}

// --- Include files for method implementations and tests ---
include!("pdmt_service_generation.rs");
include!("pdmt_service_tests.rs");
