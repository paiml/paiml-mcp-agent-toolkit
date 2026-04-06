/// Progress reporter for sprints
pub struct ProgressReporter;

impl ProgressReporter {
    /// Generate a progress report for a sprint
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn generate_report(sprint: &Sprint) -> Result<String> {
        let mut report = String::new();

        report.push_str(&format!("# Sprint {} Progress Report\n\n", sprint.version));
        report.push_str(&format!("## {}\n\n", sprint.title));

        // Task summary
        report.push_str("### Tasks\n");
        let completed = sprint
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();
        let in_progress = sprint
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::InProgress)
            .count();
        let planned = sprint
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Planned)
            .count();

        report.push_str(&format!("- Completed: {completed}\n"));
        report.push_str(&format!("- In Progress: {in_progress}\n"));
        report.push_str(&format!("- Planned: {planned}\n\n"));

        // Definition of Done
        report.push_str("### Definition of Done\n");
        for item in &sprint.definition_of_done {
            report.push_str(&format!("- {item}\n"));
        }
        report.push('\n');

        // Quality Gates
        report.push_str("### Quality Gates\n");
        for gate in &sprint.quality_gates {
            report.push_str(&format!("- {gate}\n"));
        }

        Ok(report)
    }
}

/// Dashboard generator for roadmap progress
pub struct RoadmapDashboard;

impl RoadmapDashboard {
    /// Generate markdown dashboard
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn generate(sprint_id: &str, roadmap: &Roadmap) -> Result<String> {
        debug_assert!(!sprint_id.is_empty(), "sprint_id must not be empty");
        let mut output = String::new();

        let sprint = roadmap
            .get_sprint(sprint_id)
            .ok_or_else(|| anyhow::anyhow!("Sprint {sprint_id} not found"))?;

        // Header
        output.push_str(&format!("# Sprint {sprint_id} Dashboard\n\n"));
        output.push_str(&format!("**{}**\n\n", sprint.title));
        output.push_str(&format!(
            "Duration: {} to {}\n\n",
            sprint.start_date.format("%Y-%m-%d"),
            sprint.end_date.format("%Y-%m-%d")
        ));

        // Progress
        let completed = sprint
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();
        let in_progress = sprint
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::InProgress)
            .count();
        let total = sprint.tasks.len();

        output.push_str("## Progress\n\n");
        output.push_str(&format!(
            "- **Completed**: {}/{} ({:.0}%)\n",
            completed,
            total,
            (completed as f64 / total as f64) * 100.0
        ));
        output.push_str(&format!("- **In Progress**: {in_progress}\n"));
        output.push_str(&format!(
            "- **Remaining**: {}\n\n",
            total - completed - in_progress
        ));

        // Progress bar
        output.push_str("```\n");
        let progress_width = 50;
        let completed_width = (completed as f64 / total as f64 * progress_width as f64) as usize;
        output.push('[');
        for i in 0..progress_width {
            if i < completed_width {
                output.push('█');
            } else {
                output.push('░');
            }
        }
        output.push_str(&format!(
            "] {:.0}%\n",
            (completed as f64 / total as f64) * 100.0
        ));
        output.push_str("```\n\n");

        // Tasks by status
        output.push_str("## Tasks\n\n");

        output.push_str("### ✅ Completed\n");
        for task in sprint
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
        {
            output.push_str(&format!("- {} - {}\n", task.id, task.description));
        }
        output.push('\n');

        output.push_str("### 🚧 In Progress\n");
        for task in sprint
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::InProgress)
        {
            output.push_str(&format!("- {} - {}\n", task.id, task.description));
        }
        output.push('\n');

        output.push_str("### 📋 Planned\n");
        for task in sprint
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Planned)
        {
            output.push_str(&format!("- {} - {}\n", task.id, task.description));
        }
        output.push('\n');

        // Velocity metrics if available
        if let Ok(tracker) = VelocityTracker::load(sprint_id) {
            output.push_str("## Metrics\n\n");
            output.push_str(&format!(
                "- **Average Cycle Time**: {} hours\n",
                tracker.average_cycle_time.as_secs() / 3600
            ));
            output.push_str(&format!(
                "- **Velocity**: {:.1} tasks/day\n",
                tracker.velocity()
            ));
            output.push_str(&format!(
                "- **Quality Score**: {:.1}%\n\n",
                tracker.average_quality_score() * 100.0
            ));

            // Burndown chart
            if !tracker.burndown_data.is_empty() {
                output.push_str("## Burndown Chart\n\n");
                output.push_str("```mermaid\n");
                output.push_str("graph LR\n");
                for point in &tracker.burndown_data {
                    output.push_str(&format!(
                        "  Day{} --> Tasks{}\n",
                        point.day, point.remaining_tasks
                    ));
                }
                output.push_str("```\n\n");
            }
        }

        // Definition of Done
        output.push_str("## Definition of Done\n\n");
        for item in &sprint.definition_of_done {
            let checked = if completed == total { "x" } else { " " };
            output.push_str(&format!("- [{checked}] {item}\n"));
        }
        output.push('\n');

        // Quality Gates
        output.push_str("## Quality Gates\n\n");
        for gate in &sprint.quality_gates {
            output.push_str(&format!("- [ ] {gate}\n"));
        }

        Ok(output)
    }
}
