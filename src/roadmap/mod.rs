#![cfg_attr(coverage_nightly, coverage(off))]
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
    #[must_use]
    pub fn to_emoji(&self) -> &'static str {
        match self {
            Self::Planned => "\u{1f4cb}",
            Self::InProgress => "\u{1f6a7}",
            Self::Completed => "\u{2705}",
            Self::Blocked => "\u{1f6ab}",
            Self::Deferred => "\u{23f8}\u{fe0f}",
        }
    }

    #[must_use]
    pub fn from_emoji(emoji: &str) -> Option<Self> {
        match emoji {
            "\u{1f4cb}" => Some(Self::Planned),
            "\u{1f6a7}" => Some(Self::InProgress),
            "\u{2705}" => Some(Self::Completed),
            "\u{1f6ab}" => Some(Self::Blocked),
            "\u{23f8}\u{fe0f}" => Some(Self::Deferred),
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
    #[must_use]
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
    #[must_use]
    pub fn seed(&self) -> u64 {
        // Extract number from PMAT-XXXX format
        if let Some(captures) = Regex::new(r"PMAT-(\d+)")
            .expect("internal error")
            .captures(&self.id)
        {
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

// Roadmap impl methods: file I/O, sprint/task lookup, and status updates
include!("roadmap_methods.rs");

// Configuration structs: RoadmapConfig, QualityGateConfig, GitConfig, TrackingConfig
include!("roadmap_config.rs");

// Unit tests and property tests
include!("roadmap_tests.rs");
