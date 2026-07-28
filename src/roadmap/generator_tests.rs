// Unit tests and property tests for generator module
// Included by generator.rs - shares parent module scope

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_quality_requirements_default() {
        let qr = QualityRequirements::default();

        assert_eq!(qr.max_complexity, 20);
        assert_eq!(qr.min_test_coverage, 80);
        assert!(qr.required_docs);
        assert_eq!(qr.satd_allowed, 0);
        assert!(qr.lint_compliance);
    }

    #[test]
    fn test_quality_enforced_todo_creation() {
        let todo = QualityEnforcedTodo {
            id: "todo-001".to_string(),
            task_id: "PMAT-1001".to_string(),
            description: "Implement feature X".to_string(),
            implementation_spec: "Create module with tests".to_string(),
            quality_requirements: QualityRequirements::default(),
            validation_commands: vec!["cargo test".to_string()],
            success_criteria: vec!["All tests pass".to_string()],
            estimated_time: Duration::from_secs(4 * 3600),
            dependencies: vec![],
        };

        assert_eq!(todo.id, "todo-001");
        assert_eq!(todo.task_id, "PMAT-1001");
        assert!(todo.description.contains("feature X"));
        assert_eq!(todo.validation_commands.len(), 1);
        assert_eq!(todo.success_criteria.len(), 1);
    }

    #[test]
    fn test_roadmap_todo_generator_new() {
        let config = QualityGateConfig::default();
        let generator = RoadmapTodoGenerator::new(config.clone());

        // Verify generator is created properly
        assert_eq!(generator.quality_config.coverage_min, config.coverage_min);
    }

    #[test]
    fn test_generate_todos_from_task() {
        let config = QualityGateConfig::default();
        let mut generator = RoadmapTodoGenerator::new(config);

        let task = RoadmapTask {
            id: "PMAT-2001".to_string(),
            title: "Test task".to_string(),
            description: "Test description".to_string(),
            status: TaskStatus::Planned,
            priority: Priority::P1,
            estimated_hours: 8.0,
            assigned_to: None,
            dependencies: vec![],
            tags: vec![],
        };

        let result = generator.generate_todos_from_task(&task);
        assert!(result.is_ok());

        let todos = result.unwrap();
        assert!(!todos.is_empty());
        assert!(todos[0].task_id.contains("PMAT-2001"));
    }

    #[test]
    fn test_generate_todos_from_sprint() {
        let config = QualityGateConfig::default();
        let mut generator = RoadmapTodoGenerator::new(config);

        let sprint = Sprint {
            version: "v1.0.0".to_string(),
            title: "Test Sprint".to_string(),
            start_date: chrono::Utc::now(),
            end_date: chrono::Utc::now() + chrono::Duration::days(14),
            priority: Priority::P0,
            tasks: vec![Task {
                id: "PMAT-3001".to_string(),
                description: "Task 1: Description 1".to_string(),
                status: TaskStatus::Planned,
                complexity: Complexity::Medium,
                priority: Priority::P1,
                assignee: None,
                started_at: None,
                completed_at: None,
            }],
            definition_of_done: vec!["All tests pass".to_string()],
            quality_gates: vec!["Coverage >80%".to_string()],
        };

        let result = generator.generate_todos_from_sprint(&sprint);
        assert!(result.is_ok());

        let todos = result.unwrap();
        assert!(!todos.is_empty());
    }

    #[test]
    fn test_create_quality_enforced_todo() {
        let config = QualityGateConfig::default();
        let generator = RoadmapTodoGenerator::new(config);

        let task = RoadmapTask {
            id: "PMAT-4001".to_string(),
            title: "Complex task".to_string(),
            description: "A complex implementation task".to_string(),
            status: TaskStatus::Planned,
            priority: Priority::P0,
            estimated_hours: 12.0,
            assigned_to: Some("developer".to_string()),
            dependencies: vec!["PMAT-4000".to_string()],
            tags: vec!["feature".to_string(), "backend".to_string()],
        };

        let todo = generator.create_quality_enforced_todo(&task, 1);

        assert_eq!(todo.task_id, "PMAT-4001");
        assert!(todo.description.contains("A complex implementation"));
        assert!(todo.quality_requirements.min_test_coverage >= 80);
        assert_eq!(todo.quality_requirements.satd_allowed, 0);
        assert!(todo.quality_requirements.lint_compliance);
        assert!(!todo.dependencies.is_empty());
    }

    #[test]
    fn test_generate_validation_commands() {
        let config = QualityGateConfig::default();
        let generator = RoadmapTodoGenerator::new(config);

        let task = RoadmapTask {
            id: "PMAT-5001".to_string(),
            title: "Test validation".to_string(),
            description: "Test validation commands generation".to_string(),
            status: TaskStatus::Planned,
            priority: Priority::P2,
            estimated_hours: 2.0,
            assigned_to: None,
            dependencies: vec![],
            tags: vec!["testing".to_string()],
        };

        let commands = generator.generate_validation_commands(&task);

        assert!(!commands.is_empty());
        assert!(commands.iter().any(|cmd| cmd.contains("cargo test")));
        assert!(commands.iter().any(|cmd| cmd.contains("cargo clippy")));
    }

    #[test]
    fn test_generate_success_criteria() {
        let config = QualityGateConfig::default();
        let generator = RoadmapTodoGenerator::new(config);

        let task = RoadmapTask {
            id: "PMAT-6001".to_string(),
            title: "Success criteria test".to_string(),
            description: "Test success criteria generation".to_string(),
            status: TaskStatus::Planned,
            priority: Priority::P1,
            estimated_hours: 6.0,
            assigned_to: None,
            dependencies: vec![],
            tags: vec![],
        };

        let criteria = generator.generate_success_criteria(&task);

        assert!(!criteria.is_empty());
        assert!(criteria.iter().any(|c| c.contains("test")));
        assert!(criteria.iter().any(|c| c.contains("coverage")));
        assert!(criteria.iter().any(|c| c.contains("SATD")));
    }

    #[test]
    fn test_format_todos_as_markdown() {
        let config = QualityGateConfig::default();
        let generator = RoadmapTodoGenerator::new(config);

        let todos = vec![QualityEnforcedTodo {
            id: "todo-001".to_string(),
            task_id: "PMAT-7001".to_string(),
            description: "Test todo 1".to_string(),
            implementation_spec: "Implement with TDD".to_string(),
            quality_requirements: QualityRequirements::default(),
            validation_commands: vec!["cargo test".to_string()],
            success_criteria: vec!["Tests pass".to_string()],
            estimated_time: Duration::from_secs(2 * 3600),
            dependencies: vec![],
        }];

        let markdown = generator.format_todos_as_markdown(&todos);

        assert!(markdown.contains("PMAT-7001"));
        assert!(markdown.contains("Test todo 1"));
        assert!(markdown.contains("cargo test"));
        assert!(markdown.contains("Tests pass"));
    }

    #[test]
    fn test_quality_requirements_serialization() {
        let qr = QualityRequirements {
            max_complexity: 15,
            min_test_coverage: 90,
            required_docs: false,
            satd_allowed: 0,
            lint_compliance: true,
        };

        let json = serde_json::to_string(&qr).unwrap();
        let deserialized: QualityRequirements = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.max_complexity, 15);
        assert_eq!(deserialized.min_test_coverage, 90);
        assert!(!deserialized.required_docs);
        assert_eq!(deserialized.satd_allowed, 0);
        assert!(deserialized.lint_compliance);
    }
}

