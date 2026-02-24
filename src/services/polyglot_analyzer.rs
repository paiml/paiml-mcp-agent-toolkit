#![cfg_attr(coverage_nightly, coverage(off))]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// Type definitions: structs, enums, and internal types
include!("polyglot_analyzer_types.rs");

// Language detection, scanning, framework detection, and stats
include!("polyglot_analyzer_detection.rs");

// Cross-language dependency analysis
include!("polyglot_analyzer_dependencies.rs");

// Architecture detection, confidence scoring, and insights
include!("polyglot_analyzer_architecture.rs");

/// Toyota Way: Extract Method - Check if directory should be skipped (complexity <= 3)
fn should_skip_directory(path: &Path) -> bool {
    if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
        matches!(
            dir_name,
            "node_modules" | "target" | "build" | ".git" | "__pycache__" | ".venv" | "venv"
        )
    } else {
        false
    }
}

impl Default for PolyglotAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

// Tests extracted to polyglot_analyzer_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "polyglot_analyzer_tests.rs"]
mod tests;
