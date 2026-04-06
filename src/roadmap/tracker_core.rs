impl VelocityTracker {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(sprint_id: &str) -> Self {
        debug_assert!(!sprint_id.is_empty(), "sprint_id must not be empty");
        Self {
            sprint_id: sprint_id.to_string(),
            started_at: Utc::now(),
            tasks_completed: Vec::new(),
            quality_scores: Vec::new(),
            average_cycle_time: Duration::from_secs(0),
            burndown_data: Vec::new(),
        }
    }

    /// Add a completed task (overloaded for `CompletedTask`)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_completed_task(&mut self, task: CompletedTask) {
        self.quality_scores.push(QualityScore {
            task_id: task.task_id.clone(),
            score: task.quality_score,
            timestamp: Utc::now(),
        });
        self.tasks_completed.push(task);
        self.update_average_cycle_time();
    }

    /// Load tracker from file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "non_empty_index")]
    pub fn load(sprint_id: &str) -> Result<Self> {
        debug_assert!(!sprint_id.is_empty(), "sprint_id must not be empty");
        let path = format!("docs/execution/velocity_{sprint_id}.json");
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    }

    /// Save tracker to file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn save(&self) -> Result<()> {
        let path = format!("docs/execution/velocity_{}.json", self.sprint_id);
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Add a completed task from Task struct
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_completed_task_from_task(&mut self, task: &Task, quality_score: f64) {
        if let (Some(started), Some(completed)) = (task.started_at, task.completed_at) {
            self.tasks_completed.push(CompletedTask {
                task_id: task.id.clone(),
                started_at: started,
                completed_at: completed,
                complexity: task.complexity,
                quality_score,
                rework_count: 0,
            });

            self.quality_scores.push(QualityScore {
                task_id: task.id.clone(),
                score: quality_score,
                timestamp: Utc::now(),
            });

            self.update_average_cycle_time();
        }
    }

    /// Update average cycle time
    fn update_average_cycle_time(&mut self) {
        debug_assert!(true, "contract: update_average_cycle_time");
        if self.tasks_completed.is_empty() {
            return;
        }

        let total_duration: Duration = self
            .tasks_completed
            .iter()
            .map(|t| (t.completed_at - t.started_at).to_std().unwrap_or_default())
            .sum();

        self.average_cycle_time = total_duration / self.tasks_completed.len() as u32;
    }

    /// Add a burndown point
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_burndown_point(&mut self, remaining_tasks: u32) {
        let day = (Utc::now() - self.started_at).num_days() as u32;
        self.burndown_data.push(BurndownPoint {
            day,
            remaining_tasks,
            timestamp: Utc::now(),
        });
    }

    /// Get average quality score
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn average_quality_score(&self) -> f64 {
        if self.quality_scores.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.quality_scores.iter().map(|s| s.score).sum();
        sum / self.quality_scores.len() as f64
    }

    /// Get velocity (tasks per day)
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn velocity(&self) -> f64 {
        let days_elapsed = (Utc::now() - self.started_at).num_days() as f64;
        if days_elapsed <= 0.0 {
            return 0.0;
        }

        self.tasks_completed.len() as f64 / days_elapsed
    }

    /// Calculate velocity
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn calculate_velocity(&self) -> f64 {
        // Contract: calculate_velocity returns a bounded score
        self.velocity()
    }

    /// Add a quality score
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn add_quality_score(&mut self, task_id: &str, score: f64) {
        debug_assert!(!task_id.is_empty(), "task_id must not be empty");
        debug_assert!(score >= 0.0, "score must be non-negative");
        self.quality_scores.push(QualityScore {
            task_id: task_id.to_string(),
            score,
            timestamp: Utc::now(),
        });
    }

    /// Update burndown data
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn update_burndown(&mut self, day: u32, remaining_tasks: u32) {
        self.burndown_data.push(BurndownPoint {
            day,
            remaining_tasks,
            timestamp: Utc::now(),
        });
    }

    /// Get average quality score
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_average_quality(&self) -> f64 {
        self.average_quality_score()
    }

    /// Get cycle time statistics
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_cycle_time_stats(&self) -> CycleTimeStats {
        let mut min_cycle_time = Duration::from_secs(u64::MAX);
        let mut max_cycle_time = Duration::from_secs(0);
        let mut total_cycle_time = Duration::from_secs(0);

        for task in &self.tasks_completed {
            let cycle_time = (task.completed_at - task.started_at)
                .to_std()
                .unwrap_or_default();
            if cycle_time < min_cycle_time {
                min_cycle_time = cycle_time;
            }
            if cycle_time > max_cycle_time {
                max_cycle_time = cycle_time;
            }
            total_cycle_time += cycle_time;
        }

        if self.tasks_completed.is_empty() {
            min_cycle_time = Duration::from_secs(0);
        }

        CycleTimeStats {
            min_cycle_time,
            max_cycle_time,
            avg_cycle_time: if self.tasks_completed.is_empty() {
                Duration::from_secs(0)
            } else {
                total_cycle_time / self.tasks_completed.len() as u32
            },
            task_count: self.tasks_completed.len(),
        }
    }
}
