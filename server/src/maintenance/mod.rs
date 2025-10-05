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

pub use roadmap::{Roadmap, Sprint, Ticket, SprintStatus, RoadmapError};
