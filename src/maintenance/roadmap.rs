//! Roadmap parsing and validation - TICKET-PMAT-5010
//!
//! Parses ROADMAP.md files into structured data for analysis and validation.

#![cfg_attr(coverage_nightly, coverage(off))]
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Represents a parsed roadmap
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Roadmap {
    /// Project version from roadmap title
    pub version: String,
    /// List of sprints in the roadmap
    pub sprints: Vec<Sprint>,
}

/// Represents a sprint in the roadmap
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sprint {
    /// Sprint number (e.g., 16, 17)
    pub number: u32,
    /// Sprint name (e.g., "Scaffolding Foundation")
    pub name: String,
    /// Sprint focus area
    pub focus: String,
    /// Sprint status (Complete, In Progress, Planned)
    pub status: SprintStatus,
    /// Estimated duration
    pub duration: String,
    /// Tickets in this sprint
    pub tickets: Vec<Ticket>,
    /// Quality gates for this sprint
    pub quality_gates: Vec<String>,
}

/// Sprint completion status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SprintStatus {
    Complete,
    InProgress,
    Planned,
}

/// Represents a ticket in a sprint
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ticket {
    /// Ticket ID (e.g., "TICKET-PMAT-5001")
    pub id: String,
    /// Ticket description
    pub description: String,
    /// Completion status
    pub completed: bool,
    /// Git commit reference (if completed)
    pub commit: Option<String>,
}

/// Roadmap parsing errors
#[derive(Debug, thiserror::Error)]
pub enum RoadmapError {
    #[error("Failed to read roadmap file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid roadmap format: {0}")]
    ParseError(String),

    #[error("Invalid ticket ID format: {0}")]
    InvalidTicketId(String),

    #[error("Sprint {0} has no tickets")]
    EmptySprint(u32),
}

pub type Result<T> = std::result::Result<T, RoadmapError>;

// --- Implementation split into include files ---

include!("roadmap_impl.rs");
include!("roadmap_parsing.rs");
include!("roadmap_tests.rs");
