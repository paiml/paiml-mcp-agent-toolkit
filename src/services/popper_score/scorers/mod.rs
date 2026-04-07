//! Individual category scorers for Popper Falsifiability Score v1.1
//!
//! Each scorer analyzes a specific category of the 100-point scoring system:
//!
//! | Module | Category | Points | Gate Status |
//! |--------|----------|--------|-------------|
//! | `falsifiability` | A. Falsifiability & Testability | 25 | **GATEWAY** |
//! | `reproducibility` | B. Reproducibility Infrastructure | 25 | Standard |
//! | `transparency` | C. Transparency & Openness | 20 | Standard |
//! | `statistical_rigor` | D. Statistical Rigor | 15 | Standard |
//! | `historical_integrity` | E. Historical Integrity | 10 | Standard |
//! | `ml_reproducibility` | F. ML/AI Reproducibility | 5 | Conditional (N/A) |
//!
//! ## Usage
//!
//! ```rust,ignore
//! use popper_score::scorers::*;
//!
//! let scorers: Vec<Box<dyn PopperScorer>> = vec![
//!     Box::new(FalsifiabilityScorer::new()),
//!     Box::new(ReproducibilityScorer::new()),
//!     Box::new(TransparencyScorer::new()),
//!     Box::new(StatisticalRigorScorer::new()),
//!     Box::new(HistoricalIntegrityScorer::new()),
//!     Box::new(MLReproducibilityScorer::new()),
//! ];
//! ```

pub mod falsifiability;
pub mod historical_integrity;
pub mod ml_reproducibility;
pub mod reproducibility;
pub mod statistical_rigor;
pub mod transparency;

pub use falsifiability::FalsifiabilityScorer;
pub use historical_integrity::HistoricalIntegrityScorer;
pub use ml_reproducibility::MLReproducibilityScorer;
pub use reproducibility::ReproducibilityScorer;
pub use statistical_rigor::StatisticalRigorScorer;
pub use transparency::TransparencyScorer;

/// Create all scorers in order
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn all_scorers() -> Vec<Box<dyn super::PopperScorer>> {
    vec![
        Box::new(FalsifiabilityScorer::new()),
        Box::new(ReproducibilityScorer::new()),
        Box::new(TransparencyScorer::new()),
        Box::new(StatisticalRigorScorer::new()),
        Box::new(HistoricalIntegrityScorer::new()),
        Box::new(MLReproducibilityScorer::new()),
    ]
}
