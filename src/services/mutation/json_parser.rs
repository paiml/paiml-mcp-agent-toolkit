#![cfg_attr(coverage_nightly, coverage(off))]
//! FIXED Phase Implementation for PMAT-070-002: JSON Parser
//!
//! Parses cargo-mutants JSON output and converts to PMAT mutation report format.
//!
//! ## Actual cargo-mutants v25.3.1 Format
//!
//! cargo-mutants writes to `mutants.out/` directory:
//! - `mutants.json`: List of mutant definitions (no outcomes)
//! - `outcomes.json`: Execution results with outcomes
//!
//! ## Outcome Mapping
//!
//! cargo-mutants summary → PMAT MutantStatus:
//! - `CaughtMutant` → `Killed` (test suite detected the mutant)
//! - `MissedMutant` → `Survived` (test suite missed the mutant)
//! - `Timeout` → `Timeout` (test suite timed out)
//! - `Unviable` → `CompileError` (mutant failed to compile)
//!
//! ## Example
//!
//! ```rust
//! use pmat::services::mutation::json_parser::CargoMutantsReport;
//! use std::path::PathBuf;
//!
//! let output_dir = PathBuf::from("mutants.out");
//! let report = CargoMutantsReport::from_output_dir(&output_dir)?;
//! let pmat_report = report.to_pmat_report();
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

// ============================================================================
// Actual cargo-mutants v25.3.1 JSON Format Structs
// ============================================================================

/// Actual outcomes.json structure from cargo-mutants
#[derive(Debug, Deserialize)]
struct OutcomesFile {
    outcomes: Vec<Outcome>,
}

/// Individual outcome entry
#[derive(Debug, Deserialize)]
struct Outcome {
    scenario: ScenarioType,
    summary: String,
}

/// Scenario can be either "Baseline" or {"Mutant": {...}}
#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[allow(dead_code)]
enum ScenarioType {
    Baseline(String),
    Mutant {
        #[serde(rename = "Mutant")]
        mutant: MutantDefinition,
    },
}

/// Mutant definition from outcomes.json (embedded in scenario)
#[derive(Debug, Deserialize)]
struct MutantDefinition {
    #[allow(dead_code)]
    package: String,
    file: String,
    function: FunctionInfo,
    span: SpanInfo,
    replacement: String,
    #[allow(dead_code)]
    genre: String,
}

/// Function information
#[derive(Debug, Deserialize)]
struct FunctionInfo {
    function_name: String,
}

/// Span information (line/column range)
#[derive(Debug, Deserialize)]
struct SpanInfo {
    start: Position,
}

/// Position in file
#[derive(Debug, Deserialize)]
struct Position {
    line: usize,
}

impl CargoMutantsReport {
    /// Parse cargo-mutants output from directory (ACTUAL v25.3.1 format)
    ///
    /// Reads `outcomes.json` from the cargo-mutants output directory and
    /// converts it to PMAT mutation report format.
    ///
    /// # Arguments
    /// * `dir` - Path to `mutants.out/` directory
    ///
    /// # Returns
    /// * `Ok(CargoMutantsReport)` - Successfully parsed report
    /// * `Err(...)` - Parse error with description
    ///
    /// # Example
    /// ```rust
    /// use std::path::PathBuf;
    /// let output_dir = PathBuf::from("mutants.out");
    /// let report = CargoMutantsReport::from_output_dir(&output_dir)?;
    /// ```
    pub fn from_output_dir(dir: &std::path::Path) -> Result<Self> {
        debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
        // Read outcomes.json
        let outcomes_path = dir.join("outcomes.json");
        let outcomes_json = std::fs::read_to_string(&outcomes_path)
            .map_err(|e| format!("Failed to read outcomes.json: {}", e))?;

        // Parse outcomes file
        let outcomes_file: OutcomesFile = serde_json::from_str(&outcomes_json)
            .map_err(|e| format!("Failed to parse outcomes.json: {}", e))?;

        // Extract mutants from outcomes
        let mut mutants = Vec::new();
        for outcome in outcomes_file.outcomes {
            // Skip baseline scenario
            if let ScenarioType::Mutant { mutant } = outcome.scenario {
                // Map cargo-mutants summary to our outcome enum
                let outcome_type = match outcome.summary.as_str() {
                    "CaughtMutant" => MutantOutcome::Caught,
                    "MissedMutant" => MutantOutcome::Missed,
                    "Timeout" => MutantOutcome::Timeout,
                    _ => MutantOutcome::Unviable, // Includes "Unviable" and unknown
                };

                mutants.push(CargoMutant {
                    outcome: outcome_type,
                    file: mutant.file,
                    function: Some(mutant.function.function_name),
                    line: mutant.span.start.line,
                    replacement: Some(mutant.replacement),
                });
            }
        }

        Ok(CargoMutantsReport { mutants })
    }

    /// Parse cargo-mutants JSON output (DEPRECATED - for backward compatibility)
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
    #[deprecated(
        note = "Use from_output_dir() instead - matches actual cargo-mutants v25.3.1 format"
    )]
    pub fn from_json(json: &str) -> Result<Self> {
        debug_assert!(!json.is_empty(), "json must not be empty");
        serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse cargo-mutants JSON: {}", e).into())
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
            .map(|(idx, cargo_mutant)| Self::convert_mutant(cargo_mutant, idx))
            .collect()
    }

    /// Calculate mutation score (percentage of caught mutants)
    ///
    /// # Returns
    /// Mutation score as a percentage (0.0 - 100.0)
    ///
    /// # Example
    /// ```rust
    /// let json = r#"{"mutants": [
    ///     {"outcome": "caught", "file": "src/lib.rs", "line": 10},
    ///     {"outcome": "missed", "file": "src/lib.rs", "line": 20}
    /// ]}"#;
    /// let report = CargoMutantsReport::from_json(json)?;
    /// assert_eq!(report.mutation_score(), 50.0);
    /// ```
    pub fn mutation_score(&self) -> f64 {
        if self.mutants.is_empty() {
            return 0.0;
        }

        let caught = self.count_by_outcome(MutantOutcome::Caught);
        (caught as f64 / self.mutants.len() as f64) * 100.0
    }

    /// Count mutants by outcome
    ///
    /// # Arguments
    /// * `outcome` - The outcome to count
    ///
    /// # Returns
    /// Number of mutants with the given outcome
    pub fn count_by_outcome(&self, outcome: MutantOutcome) -> usize {
        self.mutants.iter().filter(|m| m.outcome == outcome).count()
    }

    /// Convert a single cargo mutant to PMAT format
    ///
    /// # Arguments
    /// * `cargo_mutant` - The cargo-mutants mutant to convert
    /// * `idx` - Index for generating unique ID
    ///
    /// # Returns
    /// PMAT Mutant struct
    fn convert_mutant(cargo_mutant: &CargoMutant, idx: usize) -> Mutant {
        let status = Self::map_outcome(&cargo_mutant.outcome);
        let id = Self::generate_id(cargo_mutant, idx);
        let location = Self::create_location(cargo_mutant);
        let mutated_source = Self::format_mutated_source(cargo_mutant);
        let hash = Self::generate_hash(cargo_mutant);

        Mutant {
            id,
            original_file: PathBuf::from(&cargo_mutant.file),
            mutated_source,
            location,
            operator: MutationOperatorType::Custom("cargo-mutants".to_string()),
            hash,
            status,
        }
    }

    /// Map cargo-mutants outcome to PMAT status
    fn map_outcome(outcome: &MutantOutcome) -> MutantStatus {
        match outcome {
            MutantOutcome::Caught => MutantStatus::Killed,
            MutantOutcome::Missed => MutantStatus::Survived,
            MutantOutcome::Timeout => MutantStatus::Timeout,
            MutantOutcome::Unviable => MutantStatus::CompileError,
        }
    }

    /// Generate unique ID for mutant
    fn generate_id(cargo_mutant: &CargoMutant, idx: usize) -> String {
        format!(
            "cargo-mutants-{}-{}-{}",
            cargo_mutant.file.replace('/', "_").replace(".rs", ""),
            cargo_mutant.line,
            idx
        )
    }

    /// Create source location from cargo mutant
    fn create_location(cargo_mutant: &CargoMutant) -> SourceLocation {
        SourceLocation {
            line: cargo_mutant.line,
            column: 0, // cargo-mutants doesn't provide column
            end_line: cargo_mutant.line,
            end_column: 0,
        }
    }

    /// Format mutated source description
    fn format_mutated_source(cargo_mutant: &CargoMutant) -> String {
        if let Some(ref replacement) = cargo_mutant.replacement {
            format!("Replacement: {}", replacement)
        } else {
            "Unknown mutation".to_string()
        }
    }

    /// Generate hash for mutant
    fn generate_hash(cargo_mutant: &CargoMutant) -> String {
        let mut hasher = DefaultHasher::new();
        cargo_mutant.file.hash(&mut hasher);
        cargo_mutant.line.hash(&mut hasher);
        format!("{:?}", cargo_mutant.outcome).hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn test_parse_simple_json() {
        let json = r#"{"mutants": []}"#;
        let report = CargoMutantsReport::from_json(json).unwrap();
        assert_eq!(report.mutants.len(), 0);
    }

    #[test]
    #[allow(deprecated)]
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
    #[allow(deprecated)]
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
