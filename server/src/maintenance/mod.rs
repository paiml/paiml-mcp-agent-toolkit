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

pub use roadmap::{Roadmap, Sprint, Ticket, SprintStatus, RoadmapError};
pub use ticket::{TicketFile, TicketStatus, Priority, TicketError, list_tickets, ticket_exists};
pub use validator::{ValidationReport, validate_project, format_report, ValidatorError};
