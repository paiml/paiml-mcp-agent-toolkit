//! SATD detection, scanning, and file processing logic.
//!
//! Split into submodules for maintainability:
//! - `detection_extraction.rs`: Constructors, content parsing, comment extraction, context hashing
//! - `detection_analysis.rs`: Project analysis, directory scanning, result aggregation
//! - `detection_file_discovery.rs`: Source file finding, filtering, test/vendor detection
//! - `detection_false_positives.rs`: False positive detection, documentation/metadata checks

use blake3::Hasher;
use std::path::{Path, PathBuf};

use crate::models::error::TemplateError;

use super::types::{
    AstContext, AstNodeType, DebtClassifier, ProjectAnalysisStats, SATDAnalysisResult,
    SATDDetector, SATDSummary, TechnicalDebt, TestBlockTracker,
};

include!("detection_extraction.rs");
include!("detection_analysis.rs");
include!("detection_file_discovery.rs");
include!("detection_false_positives.rs");
