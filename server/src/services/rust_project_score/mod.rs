//! Rust Project Score v1.1
//!
//! Comprehensive Rust project quality scoring extending repo-score
//! with evidence-based refinements from 15 peer-reviewed papers (2022-2025).
//!
//! ## Scoring System
//!
//! Total: 106 points across 6 categories:
//! - Rust Tooling Compliance (25pts): Clippy, rustfmt, cargo-audit, cargo-deny
//! - Code Quality (26pts): Complexity 3pts, Unsafe 9pts, Mutation 8pts, Build time 4pts
//! - Testing Excellence (20pts): Coverage, integration, doc tests, mutation
//! - Documentation (15pts): Rustdoc, README, changelog
//! - Performance & Benchmarking (10pts): Criterion, profiling
//! - Dependency Health (12pts): Count, feature flags, tree pruning
//!
//! ## Evidence-Based Design
//!
//! Key refinements based on peer-reviewed research:
//! - Complexity weight reduced (8→3pts): Low bug correlation (arXiv 2024)
//! - Unsafe code weight increased (6→9pts): Memory safety critical
//! - Mutation testing weight increased (5→8pts): Test quality validated
//! - Tiered Clippy scoring: correctness > suspicious > pedantic
//! - Build time as first-class metric (4pts): Developer productivity
//!
//! ## v1.1 Innovation: Score Velocity Tracking (Kaizen)
//!
//! - Current vs Previous comparison
//! - Points/day velocity calculation
//! - Most improved category detection
//! - 90-day trend visualization
//! - Days to next grade projection
//!
//! ## Usage
//!
//! ```ignore
//! use pmat::services::rust_project_score::*;
//!
//! let score = RustProjectScore::new();
//! ```

pub mod models;
pub mod scorer;

pub use models::*;
pub use scorer::*;
