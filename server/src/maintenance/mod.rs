//! Maintenance system for managing roadmaps and tickets.
//!
//! # Sprint 17 - Maintenance Engine
//!
//! This module provides tools for:
//! - Parsing and validating ROADMAP.md files
//! - Managing tickets and their linkage to roadmaps
//! - Calculating project health scores
//! - Auto-updating roadmaps and tickets

pub mod roadmap;
pub mod ticket;
pub mod validator;
pub mod git;
pub mod updater;

pub use roadmap::{Roadmap, Sprint, Ticket, SprintStatus, RoadmapError};
pub use ticket::{TicketFile, TicketStatus, Priority, TicketError, list_tickets, ticket_exists};
pub use validator::{ValidationReport, validate_project, format_report, ValidatorError};
pub use git::{CommitInfo, extract_ticket_ids, get_current_commit, ticket_file_updated, GitError};
pub use updater::{update_roadmap_ticket, write_roadmap, update_roadmap_from_commit};
