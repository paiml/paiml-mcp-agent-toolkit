//! Maintenance system for managing roadmaps and tickets.
//!
//! # Sprint 17 - Maintenance Engine
//!
//! This module provides tools for:
//! - Parsing and validating ROADMAP.md files
//! - Managing tickets and their linkage to roadmaps
//! - Calculating project health scores
//! - Auto-updating roadmaps and tickets

pub mod git;
pub mod health;
pub mod roadmap;
pub mod ticket;
pub mod updater;
pub mod validator;

pub use git::{extract_ticket_ids, get_current_commit, ticket_file_updated, CommitInfo, GitError};
pub use health::{
    calculate_health_score, format_health_report, HealthError, HealthMetrics, HealthScore,
};
pub use roadmap::{Roadmap, RoadmapError, Sprint, SprintStatus, Ticket};
pub use ticket::{list_tickets, ticket_exists, Priority, TicketError, TicketFile, TicketStatus};
pub use updater::{update_roadmap_from_commit, update_roadmap_ticket, write_roadmap};
pub use validator::{format_report, validate_project, ValidationReport, ValidatorError};
