#![cfg_attr(coverage_nightly, coverage(off))]
//! Roadmap maintenance and validation handlers
//!
//! This module provides functionality for maintaining project roadmaps,
//! including validation, health reporting, and auto-fixing checkbox status.

use crate::cli::OutputFormat;
use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration for roadmap maintenance operations
#[derive(Debug, Clone)]
pub struct RoadmapMaintenanceConfig {
    pub validate: bool,
    pub health: bool,
    pub fix: bool,
    pub generate_tickets: bool,
    pub dry_run: bool,
}

impl RoadmapMaintenanceConfig {
    /// Create config from individual flags
    pub fn new(
        validate: bool,
        health: bool,
        fix: bool,
        generate_tickets: bool,
        dry_run: bool,
    ) -> Self {
        Self {
            validate,
            health,
            fix,
            generate_tickets,
            dry_run,
        }
    }

    /// Check if any action flags are set
    pub fn has_actions(&self) -> bool {
        self.validate || self.health || self.fix || self.generate_tickets
    }
}

/// Roadmap validation result
#[derive(Debug)]
pub struct RoadmapValidation {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Sprint information
#[derive(Debug, Serialize)]
pub struct SprintInfo {
    pub name: String,
    pub total_tickets: usize,
    pub completed_tickets: usize,
    pub status: SprintStatus,
}

#[derive(Debug, PartialEq, Serialize)]
pub enum SprintStatus {
    NotStarted,
    InProgress,
    Complete,
}

/// Ticket status from ticket file
#[derive(Debug, PartialEq)]
pub enum TicketStatus {
    Red,
    Green,
    Yellow,
    Unknown,
}

/// Ticket generation result (TICKET-PMAT-6021)
#[derive(Debug, Serialize)]
pub struct TicketGenerationResult {
    pub generated: Vec<String>,
    pub skipped: Vec<String>,
}

// Command handlers: handle_maintain_roadmap, validate_roadmap_internal, validate_roadmap,
// fix_roadmap_status, generate_tickets_internal, generate_missing_ticket_files
include!("roadmap_handler_commands.rs");

// Display/formatting: show_health_report, print_health_console, print_health_yaml
include!("roadmap_handler_display.rs");

// Parsing utilities: parse_roadmap_tickets, get_ticket_status, parse_sprint_info,
// extract_sprint_for_ticket, generate_ticket_template, build_checkbox_pattern, apply_roadmap_changes
include!("roadmap_handler_parsing.rs");

// Unit tests and property tests
include!("roadmap_handler_tests.rs");
