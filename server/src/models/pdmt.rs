use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// PDMT todo item with comprehensive quality specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdmtTodo {
    pub id: String,
    pub content: String,
    pub status: TodoStatus,
    pub priority: TodoPriority,
    pub estimated_hours: f32,
    pub dependencies: Vec<String>,
    pub quality_gates: TodoQualityGates,
    pub validation_commands: ValidationCommands,
    pub success_criteria: Vec<String>,
    pub implementation_specs: ImplementationSpecs,
}

impl PdmtTodo {
    pub fn new(content: String, priority: TodoPriority) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content,
            status: TodoStatus::Pending,
            priority,
            estimated_hours: 4.0,
            dependencies: Vec::new(),
            quality_gates: TodoQualityGates::default(),
            validation_commands: ValidationCommands::default(),
            success_criteria: vec![
                "Unit tests pass with >80% coverage".to_string(),
                "All doctests execute successfully".to_string(),
                "Quality proxy approves all changes".to_string(),
                "Zero SATD comments present".to_string(),
            ],
            implementation_specs: ImplementationSpecs::default(),
        }
    }

    pub fn get_primary_file_path(&self) -> String {
        self.implementation_specs
            .primary_files
            .first()
            .cloned()
            .unwrap_or_else(|| "src/lib.rs".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoQualityGates {
    pub coverage_requirement: f32,
    pub doctest_requirement: bool,
    pub property_test_requirement: bool,
    pub example_requirement: bool,
    pub complexity_limit: u32,
    pub satd_tolerance: bool,
}

impl Default for TodoQualityGates {
    fn default() -> Self {
        Self {
            coverage_requirement: 80.0,
            doctest_requirement: true,
            property_test_requirement: true,
            example_requirement: true,
            complexity_limit: 8,
            satd_tolerance: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCommands {
    pub unit_tests: String,
    pub doctests: String,
    pub property_tests: String,
    pub examples: Vec<String>,
    pub coverage_check: String,
    pub quality_proxy: String,
}

impl Default for ValidationCommands {
    fn default() -> Self {
        Self {
            unit_tests: "cargo test".to_string(),
            doctests: "cargo test --doc".to_string(),
            property_tests: "cargo test --features property-tests".to_string(),
            examples: vec!["cargo run --example demo".to_string()],
            coverage_check: "cargo tarpaulin --min 80".to_string(),
            quality_proxy: "pmat quality-gate --file".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationSpecs {
    pub primary_files: Vec<String>,
    pub test_files: Vec<String>,
    pub doc_files: Vec<String>,
    pub example_files: Vec<String>,
}

impl Default for ImplementationSpecs {
    fn default() -> Self {
        Self {
            primary_files: vec!["src/lib.rs".to_string()],
            test_files: vec!["tests/integration.rs".to_string()],
            doc_files: vec!["README.md".to_string()],
            example_files: vec!["examples/demo.rs".to_string()],
        }
    }
}

/// PDMT todo list with quality configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdmtTodoList {
    pub project_name: String,
    pub todos: Vec<PdmtTodo>,
    pub quality_config: PdmtQualityConfig,
    pub generated_at: String,
    pub deterministic_seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdmtQualityConfig {
    pub enforcement_mode: EnforcementMode,
    pub coverage_threshold: f32,
    pub max_complexity: u32,
    pub require_doctests: bool,
    pub require_property_tests: bool,
    pub require_examples: bool,
    pub zero_satd_tolerance: bool,
}

impl Default for PdmtQualityConfig {
    fn default() -> Self {
        Self {
            enforcement_mode: EnforcementMode::Strict,
            coverage_threshold: 80.0,
            max_complexity: 8,
            require_doctests: true,
            require_property_tests: true,
            require_examples: true,
            zero_satd_tolerance: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementMode {
    Strict,
    Advisory,
    AutoFix,
}

/// Quality validation result for PDMT-generated todos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityValidationResult {
    pub overall_passed: bool,
    pub detailed_results: QualityResults,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityResults {
    pub structure_result: ValidationOutcome,
    pub coverage_result: ValidationOutcome,
    pub doctest_result: ValidationOutcome,
    pub property_result: ValidationOutcome,
    pub example_result: ValidationOutcome,
    pub satd_result: ValidationOutcome,
    pub proxy_result: ValidationOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationOutcome {
    pub passed: bool,
    pub message: String,
    pub violations: Vec<String>,
}

impl ValidationOutcome {
    pub fn success(message: String) -> Self {
        Self {
            passed: true,
            message,
            violations: Vec::new(),
        }
    }

    pub fn failure(message: String, violations: Vec<String>) -> Self {
        Self {
            passed: false,
            message,
            violations,
        }
    }
}
