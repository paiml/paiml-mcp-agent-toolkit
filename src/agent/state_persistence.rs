#![cfg_attr(coverage_nightly, coverage(off))]
//! State persistence layer for Claude Code Agent
//!
//! PMAT-7006: Provides persistent storage for monitoring state, project configurations,
//! and quality metrics history across agent restarts.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

// Data types: AgentState, ProjectState, QualityMetrics, QualityThresholds,
// QualitySnapshot, AgentStatistics, and their Default/inherent impls
include!("state_persistence_types.rs");

// StatePersistence manager struct and impl
include!("state_persistence_manager.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Default/Construction, Serialization, StatePersistence Core,
    // Remove Project, and Metrics Update tests
    include!("state_persistence_tests.rs");

    // Update Statistics, Persistence/Load, Clone/Debug, and Edge Case tests
    include!("state_persistence_tests_advanced.rs");
}
