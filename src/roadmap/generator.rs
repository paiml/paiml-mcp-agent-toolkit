#![cfg_attr(coverage_nightly, coverage(off))]
//! PDMT todo generation from roadmap tasks

use super::{Complexity, HashMap, Priority, QualityGateConfig, Sprint, Task, TaskStatus};
use crate::models::pdmt::{PdmtQualityConfig, PdmtTodo};
use crate::services::pdmt_service::PdmtService;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Roadmap task for todo generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: Priority,
    pub estimated_hours: f64,
    pub assigned_to: Option<String>,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
}

/// Quality-enforced todo with validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityEnforcedTodo {
    pub id: String,
    pub task_id: String, // Task identifier in PMAT-XXXX format
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
    pub satd_allowed: u32, // Always 0
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

// Core methods: new, generate_todos_from_task, generate_todos_from_sprint,
// create_quality_enforced_todo, generate_validation_commands, generate_success_criteria
include!("generator_core.rs");

// Formatting and async methods: format_todos_as_markdown, generate_sprint_todos,
// generate_task_todos, enhance_with_quality, export_todos_markdown
include!("generator_formatting.rs");

// Unit tests and property tests
include!("generator_tests.rs");
