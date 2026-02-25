// Core implementation methods for RoadmapTodoGenerator
// Included by generator.rs - shares parent module scope

impl RoadmapTodoGenerator {
    /// Create new generator with quality configuration
    #[must_use]
    pub fn new(quality_config: QualityGateConfig) -> Self {
        Self {
            pdmt_service: PdmtService::new(),
            quality_config,
        }
    }

    /// Generate todos from a single task
    pub fn generate_todos_from_task(
        &mut self,
        task: &RoadmapTask,
    ) -> Result<Vec<QualityEnforcedTodo>> {
        let mut todos = Vec::new();

        // Create main implementation todo
        let todo = self.create_quality_enforced_todo(task, 1);
        todos.push(todo);

        // Create additional testing tasks for complex work
        if task.estimated_hours > 4.0 {
            let test_todo = QualityEnforcedTodo {
                id: format!("{}-test", task.id),
                task_id: task.id.clone(),
                description: format!("Comprehensive testing for {}", task.title),
                implementation_spec: "Add comprehensive test coverage".to_string(),
                quality_requirements: QualityRequirements::default(),
                validation_commands: vec!["cargo test".to_string()],
                success_criteria: vec!["Test coverage >80%".to_string()],
                estimated_time: Duration::from_secs_f64(task.estimated_hours * 0.5 * 3600.0),
                dependencies: vec![task.id.clone()],
            };
            todos.push(test_todo);
        }

        Ok(todos)
    }

    /// Generate todos from entire sprint
    pub fn generate_todos_from_sprint(
        &mut self,
        sprint: &Sprint,
    ) -> Result<Vec<QualityEnforcedTodo>> {
        let mut all_todos = Vec::new();

        for task in &sprint.tasks {
            // Convert Task to RoadmapTask
            let roadmap_task = RoadmapTask {
                id: task.id.clone(),
                title: task.description.clone(),
                description: task.description.clone(),
                status: task.status,
                priority: task.priority,
                estimated_hours: match task.complexity {
                    Complexity::Low => 2.0,
                    Complexity::Medium => 4.0,
                    Complexity::High => 8.0,
                },
                assigned_to: task.assignee.clone(),
                dependencies: Vec::new(),
                tags: Vec::new(),
            };

            let task_todos = self.generate_todos_from_task(&roadmap_task)?;
            all_todos.extend(task_todos);
        }

        Ok(all_todos)
    }

    /// Create quality-enforced todo from task
    #[must_use]
    pub fn create_quality_enforced_todo(
        &self,
        task: &RoadmapTask,
        sequence: usize,
    ) -> QualityEnforcedTodo {
        QualityEnforcedTodo {
            id: format!("{}-{}", task.id, sequence),
            task_id: task.id.clone(),
            description: task.description.clone(),
            implementation_spec: format!("Implement {} with TDD methodology", task.title),
            quality_requirements: QualityRequirements::default(),
            validation_commands: self.generate_validation_commands(task),
            success_criteria: self.generate_success_criteria(task),
            estimated_time: Duration::from_secs_f64(task.estimated_hours * 3600.0),
            dependencies: task.dependencies.clone(),
        }
    }

    /// Generate validation commands for task
    #[must_use]
    pub fn generate_validation_commands(&self, task: &RoadmapTask) -> Vec<String> {
        let mut commands = vec![
            "cargo test".to_string(),
            "cargo clippy -- -D warnings".to_string(),
            "cargo fmt --check".to_string(),
        ];

        if task.tags.contains(&"backend".to_string()) {
            commands.push("cargo doc --no-deps".to_string());
        }

        if task.tags.contains(&"api".to_string()) {
            commands.push("cargo test --test integration".to_string());
        }

        commands
    }

    /// Generate success criteria for task
    #[must_use]
    pub fn generate_success_criteria(&self, task: &RoadmapTask) -> Vec<String> {
        let mut criteria = vec![
            "All tests pass".to_string(),
            "Code coverage >80%".to_string(),
            "Zero SATD violations".to_string(),
            "Lint checks pass".to_string(),
        ];

        if task.priority == Priority::P0 {
            criteria.push("Performance benchmarks pass".to_string());
        }

        if task.estimated_hours > 8.0 {
            criteria.push("Documentation updated".to_string());
            criteria.push("Integration tests pass".to_string());
        }

        criteria
    }
}
