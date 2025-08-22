//! Progress tracking and velocity metrics for roadmap

use super::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Tracks velocity and progress metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityTracker {
    pub sprint_id: String,
    pub started_at: DateTime<Utc>,
    pub tasks_completed: Vec<CompletedTask>,
    pub quality_scores: Vec<QualityScore>,
    pub average_cycle_time: Duration,
    pub burndown_data: Vec<BurndownPoint>,
}

/// A completed task with metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedTask {
    pub task_id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub complexity: Complexity,
    pub quality_score: f64,
    pub rework_count: u32,
}

/// Quality score for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScore {
    pub task_id: String,
    pub score: f64,
    pub timestamp: DateTime<Utc>,
}

/// Point in a burndown chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurndownPoint {
    pub day: u32,
    pub remaining_tasks: u32,
    pub timestamp: DateTime<Utc>,
}

impl VelocityTracker {
    pub fn new(sprint_id: &str) -> Self {
        Self {
            sprint_id: sprint_id.to_string(),
            started_at: Utc::now(),
            tasks_completed: Vec::new(),
            quality_scores: Vec::new(),
            average_cycle_time: Duration::from_secs(0),
            burndown_data: Vec::new(),
        }
    }
    
    /// Load tracker from file
    pub fn load(sprint_id: &str) -> Result<Self> {
        let path = format!("docs/execution/velocity_{}.json", sprint_id);
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    }
    
    /// Save tracker to file
    pub fn save(&self) -> Result<()> {
        let path = format!("docs/execution/velocity_{}.json", self.sprint_id);
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
    
    /// Add a completed task
    pub fn add_completed_task(&mut self, task: &Task, quality_score: f64) {
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
        if self.tasks_completed.is_empty() {
            return;
        }
        
        let total_duration: Duration = self.tasks_completed.iter()
            .map(|t| (t.completed_at - t.started_at).to_std().unwrap_or_default())
            .sum();
        
        self.average_cycle_time = total_duration / self.tasks_completed.len() as u32;
    }
    
    /// Add a burndown point
    pub fn add_burndown_point(&mut self, remaining_tasks: u32) {
        let day = (Utc::now() - self.started_at).num_days() as u32;
        self.burndown_data.push(BurndownPoint {
            day,
            remaining_tasks,
            timestamp: Utc::now(),
        });
    }
    
    /// Get average quality score
    pub fn average_quality_score(&self) -> f64 {
        if self.quality_scores.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.quality_scores.iter().map(|s| s.score).sum();
        sum / self.quality_scores.len() as f64
    }
    
    /// Get velocity (tasks per day)
    pub fn velocity(&self) -> f64 {
        let days_elapsed = (Utc::now() - self.started_at).num_days() as f64;
        if days_elapsed <= 0.0 {
            return 0.0;
        }
        
        self.tasks_completed.len() as f64 / days_elapsed
    }
}

/// Dashboard generator for roadmap progress
pub struct RoadmapDashboard;

impl RoadmapDashboard {
    /// Generate markdown dashboard
    pub async fn generate(sprint_id: &str, roadmap: &Roadmap) -> Result<String> {
        let mut output = String::new();
        
        let sprint = roadmap.get_sprint(sprint_id)
            .ok_or_else(|| anyhow::anyhow!("Sprint {} not found", sprint_id))?;
        
        // Header
        output.push_str(&format!("# Sprint {} Dashboard\n\n", sprint_id));
        output.push_str(&format!("**{}**\n\n", sprint.title));
        output.push_str(&format!("Duration: {} to {}\n\n", 
                                sprint.start_date.format("%Y-%m-%d"),
                                sprint.end_date.format("%Y-%m-%d")));
        
        // Progress
        let completed = sprint.tasks.iter().filter(|t| t.status == TaskStatus::Completed).count();
        let in_progress = sprint.tasks.iter().filter(|t| t.status == TaskStatus::InProgress).count();
        let total = sprint.tasks.len();
        
        output.push_str("## Progress\n\n");
        output.push_str(&format!("- **Completed**: {}/{} ({:.0}%)\n", 
                                completed, total, 
                                (completed as f64 / total as f64) * 100.0));
        output.push_str(&format!("- **In Progress**: {}\n", in_progress));
        output.push_str(&format!("- **Remaining**: {}\n\n", total - completed - in_progress));
        
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
        output.push_str(&format!("] {:.0}%\n", (completed as f64 / total as f64) * 100.0));
        output.push_str("```\n\n");
        
        // Tasks by status
        output.push_str("## Tasks\n\n");
        
        output.push_str("### ✅ Completed\n");
        for task in sprint.tasks.iter().filter(|t| t.status == TaskStatus::Completed) {
            output.push_str(&format!("- {} - {}\n", task.id, task.description));
        }
        output.push('\n');
        
        output.push_str("### 🚧 In Progress\n");
        for task in sprint.tasks.iter().filter(|t| t.status == TaskStatus::InProgress) {
            output.push_str(&format!("- {} - {}\n", task.id, task.description));
        }
        output.push('\n');
        
        output.push_str("### 📋 Planned\n");
        for task in sprint.tasks.iter().filter(|t| t.status == TaskStatus::Planned) {
            output.push_str(&format!("- {} - {}\n", task.id, task.description));
        }
        output.push('\n');
        
        // Velocity metrics if available
        if let Ok(tracker) = VelocityTracker::load(sprint_id) {
            output.push_str("## Metrics\n\n");
            output.push_str(&format!("- **Average Cycle Time**: {} hours\n", 
                                    tracker.average_cycle_time.as_secs() / 3600));
            output.push_str(&format!("- **Velocity**: {:.1} tasks/day\n", tracker.velocity()));
            output.push_str(&format!("- **Quality Score**: {:.1}%\n\n", 
                                    tracker.average_quality_score() * 100.0));
            
            // Burndown chart
            if !tracker.burndown_data.is_empty() {
                output.push_str("## Burndown Chart\n\n");
                output.push_str("```mermaid\n");
                output.push_str("graph LR\n");
                for point in &tracker.burndown_data {
                    output.push_str(&format!("  Day{} --> Tasks{}\n", point.day, point.remaining_tasks));
                }
                output.push_str("```\n\n");
            }
        }
        
        // Definition of Done
        output.push_str("## Definition of Done\n\n");
        for item in &sprint.definition_of_done {
            let checked = if completed == total { "x" } else { " " };
            output.push_str(&format!("- [{}] {}\n", checked, item));
        }
        output.push('\n');
        
        // Quality Gates
        output.push_str("## Quality Gates\n\n");
        for gate in &sprint.quality_gates {
            output.push_str(&format!("- [ ] {}\n", gate));
        }
        
        Ok(output)
    }
}