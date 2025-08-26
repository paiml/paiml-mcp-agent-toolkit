//! Roadmap management with PDMT todo generation and quality gate enforcement
//!
//! This module institutionalizes the development workflow by integrating:
//! - Roadmap parsing and management
//! - PDMT-based todo generation
//! - Quality gate enforcement
//! - Progress tracking and reporting

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod commands;
pub mod generator;
pub mod parser;
pub mod quality;
pub mod tracker;

/// Task status in the roadmap
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// 📋 - Not started
    Planned,
    /// 🚧 - Currently working
    InProgress,
    /// ✅ - Done
    Completed,
    /// 🚫 - Cannot proceed
    Blocked,
    /// ⏸️ - Postponed
    Deferred,
}

impl TaskStatus {
    pub fn to_emoji(&self) -> &'static str {
        match self {
            Self::Planned => "📋",
            Self::InProgress => "🚧",
            Self::Completed => "✅",
            Self::Blocked => "🚫",
            Self::Deferred => "⏸️",
        }
    }

    pub fn from_emoji(emoji: &str) -> Option<Self> {
        match emoji {
            "📋" => Some(Self::Planned),
            "🚧" => Some(Self::InProgress),
            "✅" => Some(Self::Completed),
            "🚫" => Some(Self::Blocked),
            "⏸️" => Some(Self::Deferred),
            _ => None,
        }
    }
}

/// Task complexity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Complexity {
    Low,
    Medium,
    High,
}

impl std::str::FromStr for Complexity {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            _ => Err(()),
        }
    }
}

impl Complexity {
    pub fn to_string(&self) -> &str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

/// Priority level for tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    P0, // Critical
    P1, // Important
    P2, // Nice to have
}

impl std::str::FromStr for Priority {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "P0" => Ok(Self::P0),
            "P1" => Ok(Self::P1),
            "P2" => Ok(Self::P2),
            _ => Err(()),
        }
    }
}

impl Priority {}

/// A single task in the roadmap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String, // PMAT-XXXX
    pub description: String,
    pub status: TaskStatus,
    pub complexity: Complexity,
    pub priority: Priority,
    pub assignee: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Task {
    /// Generate a deterministic seed from the task ID
    pub fn seed(&self) -> u64 {
        // Extract number from PMAT-XXXX format
        if let Some(captures) = Regex::new(r"PMAT-(\d+)").unwrap().captures(&self.id) {
            if let Some(num) = captures.get(1) {
                return num.as_str().parse().unwrap_or(42);
            }
        }
        42 // Default seed
    }
}

/// A sprint in the roadmap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprint {
    pub version: String,
    pub title: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub priority: Priority,
    pub tasks: Vec<Task>,
    pub definition_of_done: Vec<String>,
    pub quality_gates: Vec<String>,
}

/// The complete roadmap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Roadmap {
    pub current_sprint: Option<String>,
    pub sprints: HashMap<String, Sprint>,
    pub backlog: Vec<Task>,
    pub completed_sprints: Vec<String>,
}

impl Roadmap {
    /// Load roadmap from a markdown file
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read roadmap from {}", path.display()))?;
        parser::parse_roadmap(&content)
    }

    /// Save roadmap to a markdown file
    pub fn to_file(&self, path: &Path) -> Result<()> {
        let content = parser::roadmap_to_markdown(self)?;
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write roadmap to {}", path.display()))?;
        Ok(())
    }

    /// Get a specific sprint
    pub fn get_sprint(&self, sprint_id: &str) -> Option<&Sprint> {
        self.sprints.get(sprint_id)
    }

    /// Get a specific task across all sprints
    pub fn get_task(&self, task_id: &str) -> Option<&Task> {
        for sprint in self.sprints.values() {
            if let Some(task) = sprint.tasks.iter().find(|t| t.id == task_id) {
                return Some(task);
            }
        }
        self.backlog.iter().find(|t| t.id == task_id)
    }

    /// Update task status
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

        anyhow::bail!("Task {} not found in roadmap", task_id)
    }
}

/// Configuration for roadmap management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapConfig {
    pub enabled: bool,
    pub path: PathBuf,
    pub auto_generate_todos: bool,
    pub enforce_quality_gates: bool,
    pub require_task_ids: bool,
    pub task_id_pattern: String,
    pub quality_gates: QualityGateConfig,
    pub git: GitConfig,
    pub tracking: TrackingConfig,
}

impl Default for RoadmapConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: PathBuf::from("docs/execution/roadmap.md"),
            auto_generate_todos: true,
            enforce_quality_gates: true,
            require_task_ids: true,
            task_id_pattern: "PMAT-[0-9]{4}".to_string(),
            quality_gates: QualityGateConfig::default(),
            git: GitConfig::default(),
            tracking: TrackingConfig::default(),
        }
    }
}

/// Quality gate configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateConfig {
    pub complexity_max: u32,
    pub coverage_min: u8,
    pub documentation_required: bool,
    pub satd_tolerance: u32,
    pub lint_compliance: bool,
}

impl Default for QualityGateConfig {
    fn default() -> Self {
        Self {
            complexity_max: 20,
            coverage_min: 80,
            documentation_required: true,
            satd_tolerance: 0,
            lint_compliance: true,
        }
    }
}

/// Git integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub create_branches: bool,
    pub branch_pattern: String,
    pub commit_pattern: String,
    pub require_quality_check: bool,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            create_branches: true,
            branch_pattern: "feature/{task_id}".to_string(),
            commit_pattern: "{task_id}: {message}".to_string(),
            require_quality_check: true,
        }
    }
}

/// Tracking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingConfig {
    pub velocity_tracking: bool,
    pub burndown_charts: bool,
    pub quality_metrics: bool,
    pub export_format: String,
}

impl Default for TrackingConfig {
    fn default() -> Self {
        Self {
            velocity_tracking: true,
            burndown_charts: true,
            quality_metrics: true,
            export_format: "markdown".to_string(),
        }
    }
}
