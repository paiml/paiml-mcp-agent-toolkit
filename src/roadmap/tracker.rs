#![cfg_attr(coverage_nightly, coverage(off))]
//! Progress tracking and velocity metrics for roadmap

use super::{Complexity, DateTime, Roadmap, Sprint, Task, TaskStatus, Utc};
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

/// Cycle time statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleTimeStats {
    pub min_cycle_time: Duration,
    pub max_cycle_time: Duration,
    pub avg_cycle_time: Duration,
    pub task_count: usize,
}

// VelocityTracker implementation: new, add, load, save, metrics
include!("tracker_core.rs");

// ProgressReporter and RoadmapDashboard implementations
include!("tracker_reporting.rs");

// Unit tests and property tests
include!("tracker_tests.rs");
