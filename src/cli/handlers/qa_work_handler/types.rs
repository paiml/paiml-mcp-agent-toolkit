//! QA Work Handler - Types and Data Structures
//!
//! Part 1: Core types, enums, and structs for QA validation

#![cfg_attr(coverage_nightly, coverage(off))]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// QA Checklist with 25-point Toyota Way validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QaChecklist {
    pub task_id: String,
    pub task_type: String,
    pub generated: DateTime<Utc>,
    pub categories: ChecklistCategories,
}

/// Example script for QA validation (V2 feature)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleScript {
    pub name: String,
    pub content: String,
    pub description: String,
}

/// Epic QA summary status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EpicStatus {
    /// All tasks complete (100%)
    Complete,
    /// At least one task in progress
    InProgress,
    /// No tasks started
    Pending,
}

/// Epic QA summary aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpicSummary {
    pub epic_id: String,
    pub total_tasks: usize,
    pub total_checks: u32,
    pub passed_checks: u32,
    pub overall_score: f64,
    pub status: EpicStatus,
    pub task_scores: Vec<(String, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistCategories {
    pub safety_ethics: Vec<ChecklistItem>,
    pub code_quality: Vec<ChecklistItem>,
    pub testing: Vec<ChecklistItem>,
    pub documentation: Vec<ChecklistItem>,
    pub process: Vec<ChecklistItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: String,
    pub description: String,
    pub checked: bool,
    pub automated: bool,
    pub evidence: Option<String>,
}

/// QA Validation Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QaValidationResult {
    pub task_id: String,
    pub timestamp: DateTime<Utc>,
    pub categories: HashMap<String, CategoryResult>,
    pub overall_score: f64,
    pub passed: bool,
    pub manual_checks_required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryResult {
    pub name: String,
    pub passed: u32,
    pub total: u32,
    pub items: Vec<ValidationItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationItem {
    pub id: String,
    pub description: String,
    pub status: ValidationStatus,
    pub value: Option<String>,
    pub threshold: Option<String>,
    pub evidence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationStatus {
    Passed,
    Failed,
    Warning,
    Skipped,
    Manual,
}
