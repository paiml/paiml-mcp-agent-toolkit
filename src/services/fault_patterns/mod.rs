//! Native bug-hunter pattern detection (PMAT-613).
//!
//! Ported from the legacy `batuta` bug_hunter module so pmat can detect fault
//! patterns without spawning a subprocess. Scope is intentionally narrower
//! than batuta: this module owns pattern matching only. Specialized modes
//! (cargo-mutants orchestration, SBFL, fuzzing) are out of scope.

pub mod project_scan;
pub mod scanner;
pub mod taxonomy;
pub mod types;
pub mod writer;

pub use project_scan::{scan_and_cache, scan_project};
pub use types::{BugHunterCache, DefectCategory, Finding, FindingSeverity, PatternRule};
pub use writer::{cache_path, project_hash, write_cache};
