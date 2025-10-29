//! GREEN Phase Implementation for PMAT-070-002: JSON Parser
//!
//! Parses cargo-mutants JSON output and converts to PMAT mutation report format.
//!
//! ## Outcome Mapping
//!
//! cargo-mutants → PMAT MutantStatus:
//! - `caught` → `Killed` (test suite detected the mutant)
//! - `missed` → `Survived` (test suite missed the mutant)
//! - `timeout` → `Timeout` (test suite timed out)
//! - `unviable` → `CompileError` (mutant failed to compile)
//!
//! ## Example
//!
//! ```rust
//! use pmat::services::mutation::json_parser::CargoMutantsReport;
//!
//! let json = r#"{"mutants": [{"outcome": "caught", "file": "src/lib.rs", "line": 10}]}"#;
//! let report = CargoMutantsReport::from_json(json)?;
//! let pmat_report = report.to_pmat_report();
//! assert_eq!(pmat_report.len(), 1);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use super::types::{Mutant, MutantStatus, MutationOperatorType, SourceLocation};

/// Type alias for Result with boxed error
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// cargo-mutants JSON report structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CargoMutantsReport {
    /// List of mutants from cargo-mutants
    pub mutants: Vec<CargoMutant>,
}

/// Individual mutant from cargo-mutants
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CargoMutant {
    /// Mutation outcome (caught, missed, timeout, unviable)
    pub outcome: MutantOutcome,

    /// Source file path
    pub file: String,

    /// Function name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<String>,

    /// Line number where mutation occurred
    pub line: usize,

    /// Replacement text (optional - cargo-mutants may include this)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement: Option<String>,
}

/// Mutation outcome from cargo-mutants
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MutantOutcome {
    /// Test suite detected the mutant (good!)
    Caught,

    /// Test suite missed the mutant (test gap!)
    Missed,

    /// Test suite timed out
    Timeout,

    /// Mutant failed to compile
    Unviable,
}

impl CargoMutantsReport {
    /// Parse cargo-mutants JSON output
    ///
    /// # Arguments
    /// * `json` - JSON string from cargo-mutants output
    ///
    /// # Returns
    /// * `Ok(CargoMutantsReport)` - Successfully parsed report
    /// * `Err(...)` - Parse error with description
    ///
    /// # Example
    /// ```rust
    /// let json = r#"{"mutants": []}"#;
    /// let report = CargoMutantsReport::from_json(json)?;
    /// assert_eq!(report.mutants.len(), 0);
    /// ```
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| {
            format!("Failed to parse cargo-mutants JSON: {}", e).into()
        })
    }

    /// Convert cargo-mutants report to PMAT mutation report
    ///
    /// Maps cargo-mutants outcomes to PMAT MutantStatus:
    /// - caught → Killed
    /// - missed → Survived
    /// - timeout → Timeout
    /// - unviable → CompileError
    ///
    /// # Returns
    /// Vector of PMAT Mutant structs
    ///
    /// # Example
    /// ```rust
    /// let json = r#"{"mutants": [{"outcome": "caught", "file": "src/lib.rs", "line": 10}]}"#;
    /// let report = CargoMutantsReport::from_json(json)?;
    /// let pmat_report = report.to_pmat_report();
    /// assert_eq!(pmat_report[0].status, MutantStatus::Killed);
    /// ```
    pub fn to_pmat_report(&self) -> Vec<Mutant> {
        self.mutants
            .iter()
            .enumerate()
            .map(|(idx, cargo_mutant)| {
                // Map cargo-mutants outcome to PMAT status
                let status = match cargo_mutant.outcome {
                    MutantOutcome::Caught => MutantStatus::Killed,
                    MutantOutcome::Missed => MutantStatus::Survived,
                    MutantOutcome::Timeout => MutantStatus::Timeout,
                    MutantOutcome::Unviable => MutantStatus::CompileError,
                };

                // Generate unique ID
                let id = format!(
                    "cargo-mutants-{}-{}-{}",
                    cargo_mutant.file.replace('/', "_").replace(".rs", ""),
                    cargo_mutant.line,
                    idx
                );

                // Create source location
                let location = SourceLocation {
                    line: cargo_mutant.line,
                    column: 0, // cargo-mutants doesn't provide column
                    end_line: cargo_mutant.line,
                    end_column: 0,
                };

                // Generate mutated source description
                let mutated_source = if let Some(ref replacement) = cargo_mutant.replacement {
                    format!("Replacement: {}", replacement)
                } else {
                    "Unknown mutation".to_string()
                };

                // Generate hash using DefaultHasher
                let mut hasher = DefaultHasher::new();
                cargo_mutant.file.hash(&mut hasher);
                cargo_mutant.line.hash(&mut hasher);
                format!("{:?}", cargo_mutant.outcome).hash(&mut hasher);
                let hash = format!("{:x}", hasher.finish());

                Mutant {
                    id,
                    original_file: PathBuf::from(&cargo_mutant.file),
                    mutated_source,
                    location,
                    operator: MutationOperatorType::Custom("cargo-mutants".to_string()),
                    hash,
                    status,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_json() {
        let json = r#"{"mutants": []}"#;
        let report = CargoMutantsReport::from_json(json).unwrap();
        assert_eq!(report.mutants.len(), 0);
    }

    #[test]
    fn test_outcome_deserialization() {
        let json = r#"{"mutants": [
            {"outcome": "caught", "file": "src/lib.rs", "line": 10},
            {"outcome": "missed", "file": "src/lib.rs", "line": 15},
            {"outcome": "timeout", "file": "src/lib.rs", "line": 20},
            {"outcome": "unviable", "file": "src/lib.rs", "line": 25}
        ]}"#;

        let report = CargoMutantsReport::from_json(json).unwrap();
        assert_eq!(report.mutants.len(), 4);
        assert_eq!(report.mutants[0].outcome, MutantOutcome::Caught);
        assert_eq!(report.mutants[1].outcome, MutantOutcome::Missed);
        assert_eq!(report.mutants[2].outcome, MutantOutcome::Timeout);
        assert_eq!(report.mutants[3].outcome, MutantOutcome::Unviable);
    }

    #[test]
    fn test_to_pmat_conversion() {
        let json = r#"{"mutants": [
            {"outcome": "caught", "file": "src/lib.rs", "line": 10}
        ]}"#;

        let report = CargoMutantsReport::from_json(json).unwrap();
        let pmat_report = report.to_pmat_report();

        assert_eq!(pmat_report.len(), 1);
        assert_eq!(pmat_report[0].status, MutantStatus::Killed);
        assert_eq!(pmat_report[0].original_file, PathBuf::from("src/lib.rs"));
        assert_eq!(pmat_report[0].location.line, 10);
    }
}
