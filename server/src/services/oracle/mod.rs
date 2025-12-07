//! PMAT Oracle - Unified PDCA Quality Improvement System
//!
//! Implements Plan-Do-Check-Act (PDCA) with Compiler-In-The-Loop (CITL)
//! learning to converge Rust projects toward "perfect" quality state.
//!
//! # Toyota Way Principles
//! - **Jidoka**: Auto-apply with confidence threshold; halt on regression
//! - **Kaizen**: Pattern capture from successful fixes; weight updates
//! - **Genchi Genbutsu**: Evidence from actual compiler output, not heuristics
//! - **Andon**: Stop-the-line on critical regression
//!
//! # Scientific Foundation
//! Based on 15 peer-reviewed publications (see specification).

pub mod types;
pub mod signal_collector;
pub mod pdca_loop;
pub mod convergence;

#[cfg(test)]
mod tests;

pub use types::*;
pub use signal_collector::*;
pub use pdca_loop::*;
pub use convergence::*;
