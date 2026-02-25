//! Project health score calculation for TICKET-PMAT-5014
//!
//! Provides quantitative metrics for project health based on roadmap and ticket data.

#![cfg_attr(coverage_nightly, coverage(off))]
use super::roadmap::Roadmap;
use super::ticket::TicketFile;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Project health metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthScore {
    /// Overall health score (0-100)
    pub overall_score: f64,
    /// Individual metric scores
    pub metrics: HealthMetrics,
    /// Timestamp of calculation
    pub timestamp: String,
    /// Project version
    pub version: String,
}

/// Individual health metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// Sprint velocity score (0-100)
    pub velocity_score: f64,
    /// Ticket aging score (0-100)
    pub aging_score: f64,
    /// Dependency health score (0-100)
    pub dependency_score: f64,
    /// Quality compliance score (0-100)
    pub quality_score: f64,
}

/// Sprint velocity metrics
#[derive(Debug, Clone, PartialEq)]
pub struct VelocityMetrics {
    /// Total tickets in roadmap
    pub total_tickets: usize,
    /// Completed tickets
    pub completed_tickets: usize,
    /// Completion percentage
    pub completion_rate: f64,
}

/// Ticket aging metrics
#[derive(Debug, Clone, PartialEq)]
pub struct AgingMetrics {
    /// Tickets in RED status
    pub red_tickets: Vec<String>,
    /// Tickets aged >7 days in RED (simplified: all RED tickets)
    pub stale_tickets: Vec<String>,
}

/// Dependency graph metrics
#[derive(Debug, Clone, PartialEq)]
pub struct DependencyMetrics {
    /// Total dependencies
    pub total_dependencies: usize,
    /// Broken dependencies
    pub broken_dependencies: usize,
}

/// Health calculation errors
#[derive(Debug, thiserror::Error)]
pub enum HealthError {
    #[error("Roadmap error: {0}")]
    RoadmapError(#[from] super::roadmap::RoadmapError),

    #[error("Ticket error: {0}")]
    TicketError(#[from] super::ticket::TicketError),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, HealthError>;

// Velocity, aging, dependency analysis, overall scoring, and report formatting
include!("health_calculations.rs");

// Unit tests for all health score calculations
include!("health_tests.rs");
