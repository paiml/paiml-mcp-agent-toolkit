//! Rust Project Score v2.0 (Phase 2: CI/CD Integration)
//!
//! Comprehensive Rust project quality scoring extending repo-score
//! with evidence-based refinements from 15 peer-reviewed papers (2022-2025).
//!
//! ## Scoring System
//!
//! Total: 155 points across 6 categories:
//! - **Rust Tooling & CI/CD (74pts)**: Clippy, rustfmt, cargo-audit, cargo-deny, workspace lints, **CI/CD integration (NEW)**
//! - Code Quality (26pts): Complexity 3pts, Unsafe 9pts, Mutation 8pts, Build time 4pts
//! - Testing Excellence (20pts): Coverage, integration, doc tests, mutation
//! - Documentation (15pts): Rustdoc, README, changelog
//! - Performance & Benchmarking (10pts): Criterion, profiling
//! - Dependency Health (12pts): Count, feature flags, tree pruning
//!
//! ### v2.0 Phase 1: Workspace-Level Lints (+12pts)
//!
//! Based on "Learn from Rust Giants" TPS-reviewed specification:
//! - Workspace-level lints configured: 5pts
//! - High-value lint categories (correctness, suspicious, perf): 4pts
//! - .clippy.toml with disallowed-methods: 3pts
//!
//! **Academic Foundation**:
//! - Johnson et al. 2013 ICSE: Quality over quantity (avoid warning blindness)
//! - Bacchelli & Bird 2013 ICSE: Automated style enforcement reduces review waste
//!
//! ### v2.0 Phase 2: CI/CD Integration (+37pts)
//!
//! Based on "Learn from Rust Giants" TPS-reviewed specification:
//!
//! **Multi-Platform CI (13pts)**:
//! - Linux + Windows + Mac testing: 6pts
//! - Feature matrix testing (minimal, default, full): 4pts
//! - Separate workflows (stress, loom, audit): 3pts
//!
//! **CI Workflow Diversity (15pts)**:
//! - ≥3 separate GitHub Actions workflows: 6pts
//! - Dedicated security audit workflow: 4pts
//! - Dedicated benchmark workflow: 3pts
//! - Dedicated lint/spell-check workflow: 2pts
//!
//! **Build Automation (9pts)**:
//! - justfile or cargo-xtask (Rust-native): 5pts
//! - Makefile (Windows-problematic, downgraded): 3pts
//! - Common targets (build, test, lint, bench): 3pts
//!
//! **Academic Foundation**:
//! - Hilton et al. 2016 ASE: CI adoption correlates with faster releases
//! - Memon et al. 2017 ICSE-SEIP: Flaky tests reduce productivity by 16%
//! - McIntosh et al. 2015 ICSE: Build system maintenance overhead
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

pub mod code_quality_scorer;
pub mod command_runner;
pub mod dependency_scorer;
pub mod documentation_scorer;
pub mod formal_verification_scorer;
pub mod models;
pub mod orchestrator;
pub mod performance_scorer;
pub mod rust_tooling_scorer;
pub mod scorer;
pub mod testing_scorer;

pub use code_quality_scorer::*;
pub use dependency_scorer::*;
pub use documentation_scorer::*;
pub use formal_verification_scorer::*;
pub use models::*;
pub use orchestrator::*;
pub use performance_scorer::*;
pub use rust_tooling_scorer::*;
pub use scorer::*;
pub use testing_scorer::*;
