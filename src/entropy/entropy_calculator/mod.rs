#![cfg_attr(coverage_nightly, coverage(off))]
//! Entropy Calculation Module
//!
//! This module provides advanced entropy-based analysis for detecting repetitive patterns
//! in codebases. Unlike traditional character-based entropy, it focuses on AST-level patterns
//! that can be actionably refactored to reduce code duplication and improve maintainability.
//!
//! # Key Concepts
//!
//! - **Pattern Entropy**: Measures the diversity of AST patterns in code
//! - **Actionable Violations**: Repetitive patterns that can be extracted into functions
//! - **LOC Reduction**: Estimated lines of code that can be eliminated through refactoring
//! - **Pattern Diversity**: Shannon entropy of pattern distribution across the codebase
//!
//! # Pattern Types Analyzed
//!
//! 1. **`ErrorHandling`**: try/catch blocks, Result handling → Extract error handler functions
//! 2. **`DataValidation`**: Input validation patterns → Create validation traits/modules
//! 3. **`ResourceManagement`**: RAII patterns, lifecycle management → Implement guards
//! 4. **`ControlFlow`**: Complex if/else chains → Strategy patterns/polymorphism
//! 5. **`DataTransformation`**: map/filter/reduce chains → Data pipelines
//! 6. **`ApiCall`**: HTTP/RPC call patterns → API client abstractions
//!
//! # Example Usage
//!
//! ```rust
//! use pmat::entropy::entropy_calculator::{EntropyMetrics, EntropyReport};
//! use std::collections::HashMap;
//!
//! // Example metrics showing good pattern diversity
//! let metrics = EntropyMetrics {
//!     file_level_entropy: 0.85,
//!     module_level_entropy: 0.75,
//!     project_level_entropy: 0.70,
//!     pattern_diversity: 0.78,
//!     total_patterns: 42,
//!     total_instances: 156,
//!     total_loc: 2500,
//!     patterns_by_type: HashMap::new(),
//! };
//!
//! // High entropy indicates good pattern diversity (low duplication)
//! assert!(metrics.pattern_diversity > 0.7);
//!
//! // Pattern density calculation
//! let pattern_density = metrics.total_instances as f64 / metrics.total_loc as f64;
//! println!("Pattern density: {:.2} patterns per LOC", pattern_density);
//! ```

mod calculator;
mod report;
mod types;

#[cfg(test)]
mod tests;

pub use calculator::EntropyCalculator;
pub use types::{EntropyMetrics, EntropyReport};
