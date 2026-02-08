#![cfg_attr(coverage_nightly, coverage(off))]
//! Popper Falsifiability Score v1.1
//!
//! A 100-point scoring system evaluating whether software repositories
//! meet Karl Popper's scientific standards of falsifiability.
//!
//! ## Philosophical Foundation
//!
//! > "A theory is scientific if and only if it is falsifiable."
//! > — Karl Popper, *The Logic of Scientific Discovery* (1934)
//!
//! ## Scoring Categories (100 Total Points)
//!
//! | Category | Points | Popper Principle | Gate Status |
//! |----------|--------|------------------|-------------|
//! | A. Falsifiability & Testability | 25 | Core Criterion | **GATEWAY** |
//! | B. Reproducibility Infrastructure | 25 | Independent Verification | Standard |
//! | C. Transparency & Openness | 20 | Scrutiny Enablement | Standard |
//! | D. Statistical Rigor | 15 | Sound Methodology | Standard |
//! | E. Historical Integrity | 10 | Evolution Tracking | Standard |
//! | F. ML/AI Reproducibility | 5 | Modern Science Standards | Conditional (N/A) |
//!
//! ## Falsifiability Gateway (v1.1)
//!
//! Category A is a prerequisite gate. If a project scores below 15/25 (60%)
//! on Falsifiability & Testability, the total score is 0 regardless of
//! other categories. This implements Jidoka (stop the line) at the most
//! critical quality checkpoint.
//!
//! ## Score Normalization
//!
//! For categories with N/A values (e.g., non-ML projects), normalize:
//!
//! ```text
//! Normalized_Score = (Points_Earned / Points_Available) × 100
//! ```
//!
//! ## Toyota Way Integration
//!
//! - **Genchi Genbutsu**: Analyze actual artifacts, not claims
//! - **Jidoka**: Falsifiability gateway stops the line
//! - **Kaizen**: Track score velocity over time
//! - **Hansei**: Five Whys integration for root cause analysis
//!
//! ## Academic Foundation
//!
//! 31 peer-reviewed references including:
//! - Popper (1934): The Logic of Scientific Discovery
//! - ACM Artifact Badging (2020)
//! - NeurIPS ML Reproducibility Checklist
//! - FAIR Principles (Wilkinson et al., 2016)

pub mod models;
pub mod orchestrator;
pub mod scorer;
pub mod scorers;

pub use models::*;
pub use orchestrator::*;
pub use scorer::*;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests;
