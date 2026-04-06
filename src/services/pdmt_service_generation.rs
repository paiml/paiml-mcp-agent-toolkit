// pdmt_service_generation.rs — requirement-to-todo conversion and helper methods
// Included by pdmt_service.rs — shares parent module scope (no `use` imports here)

impl PdmtService {
    /// Convert a single requirement into detailed todos
    fn requirement_to_todos(
        &self,
        requirement: &str,
        granularity: &str,
        quality_config: &PdmtQualityConfig,
        _requirement_idx: usize,
        dependency_map: &mut HashMap<String, Vec<String>>,
    ) -> Result<Vec<PdmtTodo>> {
        debug_assert!(!requirement.is_empty(), "requirement must not be empty");
        debug_assert!(!granularity.is_empty(), "granularity must not be empty");
        let mut todos = Vec::new();

        // Determine task breakdown based on granularity
        let task_count = match granularity {
            "low" => 1,
            "medium" => 2,
            "high" => 3,
            _ => 2,
        };

        // Analyze requirement to determine appropriate tasks
        let requirement_lower = requirement.to_lowercase();
        let is_feature = requirement_lower.contains("implement")
            || requirement_lower.contains("add")
            || requirement_lower.contains("create");
        let is_fix = requirement_lower.contains("fix") || requirement_lower.contains("bug");
        let is_refactor = requirement_lower.contains("refactor")
            || requirement_lower.contains("improve")
            || requirement_lower.contains("optimize");
        let needs_tests = !requirement_lower.contains("test");

        // Create base implementation task
        let base_id = Uuid::new_v4().to_string();
        let base_todo = PdmtTodo {
            id: base_id.clone(),
            content: self.generate_action_content(requirement, is_feature, is_fix, is_refactor),
            status: TodoStatus::Pending,
            priority: self.determine_priority(&requirement_lower),
            estimated_hours: self.estimate_hours(requirement, task_count as f32),
            dependencies: Vec::new(),
            quality_gates: TodoQualityGates {
                coverage_requirement: quality_config.coverage_threshold,
                doctest_requirement: quality_config.require_doctests,
                property_test_requirement: quality_config.require_property_tests,
                example_requirement: quality_config.require_examples,
                complexity_limit: quality_config.max_complexity,
                satd_tolerance: false, // Always zero tolerance
            },
            validation_commands: self.generate_validation_commands(requirement),
            success_criteria: self.generate_success_criteria(quality_config),
            implementation_specs: self.generate_implementation_specs(requirement),
        };

        todos.push(base_todo.clone());

        // Include test task when granularity permits
        if needs_tests && task_count >= 2 {
            let test_id = Uuid::new_v4().to_string();
            dependency_map
                .entry(test_id.clone())
                .or_default()
                .push(base_id.clone());

            let test_todo = PdmtTodo {
                id: test_id,
                content: format!("Write comprehensive tests for: {requirement}"),
                status: TodoStatus::Pending,
                priority: base_todo.priority.clone(),
                estimated_hours: base_todo.estimated_hours * 0.5,
                dependencies: vec![base_id.clone()],
                quality_gates: base_todo.quality_gates.clone(),
                validation_commands: ValidationCommands {
                    unit_tests: "cargo test".to_string(),
                    doctests: "cargo test --doc".to_string(),
                    property_tests: "cargo test --features property-tests".to_string(),
                    examples: vec![],
                    coverage_check: format!(
                        "cargo llvm-cov --fail-under-lines {}",
                        quality_config.coverage_threshold
                    ),
                    quality_proxy: "pmat quality-gate --file".to_string(),
                },
                success_criteria: vec![
                    format!(
                        "Tests achieve >{}% coverage",
                        quality_config.coverage_threshold
                    ),
                    "All test cases pass".to_string(),
                    "Property tests validate invariants".to_string(),
                ],
                implementation_specs: ImplementationSpecs {
                    primary_files: vec![],
                    test_files: vec![format!("tests/{}_test.rs", base_id)],
                    doc_files: vec![],
                    example_files: vec![],
                },
            };
            todos.push(test_todo);
        }

        // Include documentation task for detailed granularity
        if task_count >= 3 {
            let doc_id = Uuid::new_v4().to_string();
            dependency_map
                .entry(doc_id.clone())
                .or_default()
                .push(base_id.clone());

            let doc_todo = PdmtTodo {
                id: doc_id,
                content: format!("Document and create examples for: {requirement}"),
                status: TodoStatus::Pending,
                priority: TodoPriority::Low,
                estimated_hours: 2.0,
                dependencies: vec![base_id],
                quality_gates: base_todo.quality_gates,
                validation_commands: ValidationCommands {
                    unit_tests: String::new(),
                    doctests: "cargo test --doc".to_string(),
                    property_tests: String::new(),
                    examples: vec!["cargo run --example demo".to_string()],
                    coverage_check: String::new(),
                    quality_proxy: "pmat quality-gate --file".to_string(),
                },
                success_criteria: vec![
                    "Documentation is comprehensive".to_string(),
                    "Examples run without errors".to_string(),
                    "Doctests pass".to_string(),
                ],
                implementation_specs: ImplementationSpecs {
                    primary_files: vec![],
                    test_files: vec![],
                    doc_files: vec!["README.md".to_string()],
                    example_files: vec!["examples/demo.rs".to_string()],
                },
            };
            todos.push(doc_todo);
        }

        Ok(todos)
    }

    /// Generate specific action content for a todo
    fn generate_action_content(
        &self,
        requirement: &str,
        is_feature: bool,
        is_fix: bool,
        is_refactor: bool,
    ) -> String {
        debug_assert!(!requirement.is_empty(), "requirement must not be empty");
        let action_verb = if is_feature {
            "Implement"
        } else if is_fix {
            "Fix"
        } else if is_refactor {
            "Refactor"
        } else {
            "Create"
        };

        format!("{action_verb} {requirement}")
    }

    /// Determine priority based on requirement keywords
    fn determine_priority(&self, requirement: &str) -> TodoPriority {
        debug_assert!(!requirement.is_empty(), "requirement must not be empty");
        if requirement.contains("critical") || requirement.contains("urgent") {
            TodoPriority::Critical
        } else if requirement.contains("bug") || requirement.contains("fix") {
            TodoPriority::High
        } else if requirement.contains("refactor") || requirement.contains("improve") {
            TodoPriority::Medium
        } else {
            TodoPriority::Low
        }
    }

    /// Estimate hours based on requirement complexity
    fn estimate_hours(&self, requirement: &str, base_multiplier: f32) -> f32 {
        debug_assert!(!requirement.is_empty(), "requirement must not be empty");
        let complexity_score = if requirement.len() > 100 {
            3.0
        } else if requirement.len() > 50 {
            2.0
        } else {
            1.0
        };

        (complexity_score * base_multiplier * 2.0).clamp(0.5, 8.0)
    }

    /// Generate validation commands for a todo
    fn generate_validation_commands(&self, _requirement: &str) -> ValidationCommands {
        debug_assert!(!_requirement.is_empty(), "_requirement must not be empty");
        ValidationCommands {
            unit_tests: "cargo test".to_string(),
            doctests: "cargo test --doc".to_string(),
            property_tests: "cargo test --features property-tests".to_string(),
            examples: vec!["cargo run --example demo".to_string()],
            coverage_check: "cargo llvm-cov --fail-under-lines 80".to_string(),
            quality_proxy: "pmat quality-gate --file".to_string(),
        }
    }

    /// Generate success criteria based on quality config
    fn generate_success_criteria(&self, config: &PdmtQualityConfig) -> Vec<String> {
        let mut criteria = vec![
            format!(
                "Unit tests pass with >{}% coverage",
                config.coverage_threshold
            ),
            "Quality proxy approves all changes".to_string(),
            "Zero SATD comments present".to_string(),
            format!("Complexity stays under {} limit", config.max_complexity),
        ];

        if config.require_doctests {
            criteria.push("All doctests execute successfully".to_string());
        }
        if config.require_property_tests {
            criteria.push("Property tests validate invariants".to_string());
        }
        if config.require_examples {
            criteria.push("Examples run without errors".to_string());
        }

        criteria
    }

    /// Generate implementation specs for a requirement
    fn generate_implementation_specs(&self, requirement: &str) -> ImplementationSpecs {
        debug_assert!(!requirement.is_empty(), "requirement must not be empty");
        let base_name = requirement
            .split_whitespace()
            .take(2)
            .collect::<Vec<_>>()
            .join("_")
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric() && c != '_', "");

        ImplementationSpecs {
            primary_files: vec![format!("src/{}.rs", base_name)],
            test_files: vec![format!("tests/{}_test.rs", base_name)],
            doc_files: vec!["README.md".to_string()],
            example_files: vec![format!("examples/{}_demo.rs", base_name)],
        }
    }

    /// Set dependencies between todos based on logical ordering
    fn set_dependencies(
        &self,
        todos: &mut [PdmtTodo],
        dependency_map: &HashMap<String, Vec<String>>,
    ) {
        debug!("Setting dependencies for {} todos", todos.len());

        for todo in todos.iter_mut() {
            if let Some(deps) = dependency_map.get(&todo.id) {
                todo.dependencies = deps.clone();
            }
        }
    }
}
