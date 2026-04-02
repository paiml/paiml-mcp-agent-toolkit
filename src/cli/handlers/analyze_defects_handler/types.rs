#![cfg_attr(coverage_nightly, coverage(off))]
//! Type definitions for the defect analysis handler

use crate::services::defect_detector::DefectPattern;
use serde::{Deserialize, Serialize};

pub use crate::contracts::OutputFormat;

#[derive(Debug, Serialize, Deserialize)]
pub struct DefectSummary {
    pub total_files_scanned: usize,
    pub files_with_defects: usize,
    pub total_defects: usize,
    pub by_severity: SeverityCount,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeverityCount {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefectReport {
    pub summary: DefectSummary,
    pub defects: Vec<DefectPattern>,
    pub exit_code: i32,
    pub has_critical_defects: bool,
}
