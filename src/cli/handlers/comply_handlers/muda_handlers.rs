//! CB-300: Muda (Seven Wastes) Score for Code Quality
//!
//! Maps Toyota Production System's Seven Wastes to code quality metrics:
//!
//! | Toyota Waste     | Code Equivalent           | Detection                    |
//! |------------------|---------------------------|------------------------------|
//! | Overproduction   | Dead Code                 | `dead_code` analysis (CB-128)|
//! | Waiting          | Slow Tests / Builds       | Test time (CB-126/CB-127)    |
//! | Inventory        | Stale SATD markers        | SATD age > 90 days           |
//! | Transport        | Excessive cloning         | `.clone()` in hot paths      |
//! | Over-processing  | High complexity           | Cyclomatic > 15              |
//! | Motion           | Dependency sprawl         | Dep count / graph depth      |
//! | Defects          | Bugs / Test failures      | Panic count / stub count     |
//!
//! Score: 0-100 (lower is better, 0 = zero waste)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Muda Waste Report aggregating all seven wastes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MudaReport {
    /// Overproduction: dead code percentage (0-100)
    pub overproduction: f64,
    /// Waiting: slow test/build score (0-100)
    pub waiting: f64,
    /// Inventory: stale SATD/branch score (0-100)
    pub inventory: f64,
    /// Transport: excessive copying score (0-100)
    pub transport: f64,
    /// Over-processing: complexity waste (0-100)
    pub over_processing: f64,
    /// Motion: dependency sprawl (0-100)
    pub motion: f64,
    /// Defects: bug/panic indicators (0-100)
    pub defects: f64,
    /// Total aggregate score (0-100, lower is better)
    pub total_score: f64,
    /// Grade based on total score
    pub grade: MudaGrade,
    /// Maps each Muda category to its top contributing files (up to 5 per category)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub file_details: HashMap<String, Vec<String>>,
}

/// Muda grade classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MudaGrade {
    /// 0-20: Lean (minimal waste)
    Lean,
    /// 21-40: Efficient
    Efficient,
    /// 41-60: Moderate waste
    Moderate,
    /// 61-80: High waste
    High,
    /// 81-100: Critical waste
    Critical,
}

impl std::fmt::Display for MudaGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MudaGrade::Lean => write!(f, "Lean"),
            MudaGrade::Efficient => write!(f, "Efficient"),
            MudaGrade::Moderate => write!(f, "Moderate"),
            MudaGrade::High => write!(f, "High"),
            MudaGrade::Critical => write!(f, "Critical"),
        }
    }
}

impl MudaGrade {
    fn from_score(score: f64) -> Self {
        debug_assert!(score >= 0.0, "score must be non-negative");
        match score as u32 {
            0..=20 => MudaGrade::Lean,
            21..=40 => MudaGrade::Efficient,
            41..=60 => MudaGrade::Moderate,
            61..=80 => MudaGrade::High,
            _ => MudaGrade::Critical,
        }
    }
}

/// Calculate the Muda Waste Score for a project.
///
/// Weights: Defects (25%), Inventory (20%), Over-processing (15%),
///          Overproduction (15%), Waiting (15%), Motion (5%), Transport (5%)
///
/// Inventory (SATD) elevated to 20% — stale TODO/FIXME/HACK accumulation
/// is a primary signal of unmaintained code and must not be masked.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn calculate_muda_score(project_path: &Path) -> MudaReport {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let overproduction = measure_overproduction(project_path);
    let waiting = measure_waiting(project_path);
    let inventory = measure_inventory(project_path);
    let transport = measure_transport(project_path);
    let over_processing = measure_over_processing(project_path);
    let motion = measure_motion(project_path);
    let defects = measure_defects(project_path);

    // Collect file details for categories with concrete file mappings
    let mut file_details = HashMap::new();
    let overproduction_files = collect_overproduction_files(project_path);
    if !overproduction_files.is_empty() {
        file_details.insert("Overproduction".to_string(), overproduction_files);
    }
    let inventory_files = collect_inventory_files(project_path);
    if !inventory_files.is_empty() {
        file_details.insert("Inventory".to_string(), inventory_files);
    }
    let over_processing_files = collect_over_processing_files(project_path);
    if !over_processing_files.is_empty() {
        file_details.insert("Over-processing".to_string(), over_processing_files);
    }
    let defect_files = collect_defect_files(project_path);
    if !defect_files.is_empty() {
        file_details.insert("Defects".to_string(), defect_files);
    }

    // Weighted average (weights sum to 1.0)
    // Inventory elevated: stale SATD is a primary waste signal
    let total_score = (defects * 0.25)
        + (inventory * 0.20)
        + (over_processing * 0.15)
        + (overproduction * 0.15)
        + (waiting * 0.15)
        + (motion * 0.05)
        + (transport * 0.05);

    let total_score = total_score.clamp(0.0, 100.0);
    let grade = MudaGrade::from_score(total_score);

    MudaReport {
        overproduction,
        waiting,
        inventory,
        transport,
        over_processing,
        motion,
        defects,
        total_score,
        grade,
        file_details,
    }
}

// --- Include split submodules ---

// SATD measurement: overproduction, waiting, inventory, SATD counting helpers
include!("muda_handlers_measurement.rs");

// Project metrics: transport, over-processing, motion, defects
include!("muda_handlers_metrics.rs");

// Unit tests
include!("muda_handlers_tests.rs");
// #[requires(project_path.exists())]
// #[ensures(result.is_ok())]
