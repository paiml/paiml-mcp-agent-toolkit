//! Ticket file parsing and management - TICKET-PMAT-5011
//!
//! Parses ticket files from docs/tickets/ into structured data for validation.

#![cfg_attr(coverage_nightly, coverage(off))]
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Represents a parsed ticket file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TicketFile {
    /// Ticket ID (e.g., "TICKET-PMAT-5011")
    pub id: String,
    /// Ticket title
    pub title: String,
    /// Current status (RED, GREEN, REFACTOR, COMPLETE)
    pub status: TicketStatus,
    /// Priority level
    pub priority: Priority,
    /// Complexity estimate (1-10)
    pub complexity: u8,
    /// Estimated time
    pub estimated_time: String,
    /// Dependencies (other ticket IDs)
    pub dependencies: Vec<String>,
    /// Sprint number
    pub sprint: String,
    /// Objective section content
    pub objective: String,
    /// Success criteria checklist
    pub success_criteria: Vec<String>,
    /// File path
    pub file_path: PathBuf,
}

/// Ticket status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TicketStatus {
    Red,
    Green,
    Refactor,
    Complete,
}

/// Priority level
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Priority {
    P0,
    P1,
    P2,
}

/// Ticket management errors
#[derive(Debug, thiserror::Error)]
pub enum TicketError {
    #[error("Failed to read ticket file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid ticket format: {0}")]
    ParseError(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Ticket not found: {0}")]
    NotFound(String),

    #[error("Invalid ticket status: {0}")]
    InvalidStatus(String),

    #[error("Invalid priority: {0}")]
    InvalidPriority(String),

    #[error("Invalid complexity: {0}")]
    InvalidComplexity(u8),
}

pub type Result<T> = std::result::Result<T, TicketError>;

// --- Submodule includes ---
include!("ticket_impls.rs");
include!("ticket_parsing.rs");
include!("ticket_tests.rs");
