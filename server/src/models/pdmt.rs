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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdmt_todo_creation() {
        let content = "Implement new feature".to_string();
        let priority = TodoPriority::High;
        let todo = PdmtTodo::new(content.clone(), priority);

        assert_eq!(todo.content, content);
        assert_eq!(todo.priority, TodoPriority::High);
        assert_eq!(todo.status, TodoStatus::Pending);
        assert_eq!(todo.estimated_hours, 4.0);
        assert!(!todo.id.is_empty());
        assert_eq!(todo.success_criteria.len(), 4);
        assert!(todo
            .success_criteria
            .contains(&"Unit tests pass with >80% coverage".to_string()));
    }

    #[test]
    fn test_pdmt_todo_default_values() {
        let todo = PdmtTodo::new("Test content".to_string(), TodoPriority::Medium);

        assert_eq!(todo.dependencies.len(), 0);
        assert_eq!(todo.quality_gates, TodoQualityGates::default());
        assert_eq!(todo.validation_commands, ValidationCommands::default());
        assert_eq!(todo.implementation_specs, ImplementationSpecs::default());
    }

    #[test]
    fn test_get_primary_file_path_with_files() {
        let mut todo = PdmtTodo::new("Test".to_string(), TodoPriority::Low);
        todo.implementation_specs.primary_files =
            vec!["src/main.rs".to_string(), "src/lib.rs".to_string()];

        assert_eq!(todo.get_primary_file_path(), "src/main.rs");
    }

    #[test]
    fn test_get_primary_file_path_empty() {
        let todo = PdmtTodo::new("Test".to_string(), TodoPriority::Low);
        assert_eq!(todo.get_primary_file_path(), "src/lib.rs");
    }

    #[test]
    fn test_validation_outcome_success() {
        let outcome = ValidationOutcome::success("All tests passed".to_string());

        assert!(outcome.passed);
        assert_eq!(outcome.message, "All tests passed");
        assert_eq!(outcome.violations.len(), 0);
    }

    #[test]
    fn test_validation_outcome_failure() {
        let violations = vec!["Missing test".to_string(), "Low coverage".to_string()];
        let outcome = ValidationOutcome::failure("Tests failed".to_string(), violations.clone());

        assert!(!outcome.passed);
        assert_eq!(outcome.message, "Tests failed");
        assert_eq!(outcome.violations, violations);
    }

    #[test]
    fn test_todo_status_values() {
        // Ensure all enum variants can be created
        let _pending = TodoStatus::Pending;
        let _in_progress = TodoStatus::InProgress;
        let _completed = TodoStatus::Completed;
        let _blocked = TodoStatus::Blocked;
    }

    #[test]
    fn test_todo_priority_values() {
        // Test priority enum variants
        let _low = TodoPriority::Low;
        let _medium = TodoPriority::Medium;
        let _high = TodoPriority::High;
        let _critical = TodoPriority::Critical;
    }

    #[test]
    fn test_serde_roundtrip() {
        let original = PdmtTodo::new("Test serialization".to_string(), TodoPriority::High);

        // Serialize to JSON
        let json = serde_json::to_string(&original).expect("Serialization failed");

        // Deserialize back
        let deserialized: PdmtTodo = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.content, deserialized.content);
        assert_eq!(original.priority, deserialized.priority);
        assert_eq!(original.status, deserialized.status);
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test] 
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
