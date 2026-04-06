// Roadmap impl methods: file I/O, sprint/task lookup, and status updates.

impl Roadmap {
    /// Load roadmap from a markdown file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read roadmap from {}", path.display()))?;
        parser::parse_roadmap(&content)
    }

    /// Save roadmap to a markdown file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn to_file(&self, path: &Path) -> Result<()> {
        let content = parser::roadmap_to_markdown(self)?;
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write roadmap to {}", path.display()))?;
        Ok(())
    }

    /// Get a specific sprint
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_sprint(&self, sprint_id: &str) -> Option<&Sprint> {
        self.sprints.get(sprint_id)
    }

    /// Get a specific task across all sprints
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_task(&self, task_id: &str) -> Option<&Task> {
        for sprint in self.sprints.values() {
            if let Some(task) = sprint.tasks.iter().find(|t| t.id == task_id) {
                return Some(task);
            }
        }
        self.backlog.iter().find(|t| t.id == task_id)
    }

    /// Update task status
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn update_task_status(&mut self, task_id: &str, status: TaskStatus) -> Result<()> {
        // Update in sprints
        for sprint in self.sprints.values_mut() {
            if let Some(task) = sprint.tasks.iter_mut().find(|t| t.id == task_id) {
                task.status = status;

                // Update timestamps
                match status {
                    TaskStatus::InProgress if task.started_at.is_none() => {
                        task.started_at = Some(Utc::now());
                    }
                    TaskStatus::Completed => {
                        task.completed_at = Some(Utc::now());
                    }
                    _ => {}
                }

                return Ok(());
            }
        }

        // Update in backlog
        if let Some(task) = self.backlog.iter_mut().find(|t| t.id == task_id) {
            task.status = status;
            return Ok(());
        }

        anyhow::bail!("Task {task_id} not found in roadmap")
    }
}
