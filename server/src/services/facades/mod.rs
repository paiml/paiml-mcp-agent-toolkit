//! Service Facades for Simplified Access
//!
//! This module provides high-level facades for accessing analysis services.
//! Facades abstract away the complexity of service interaction and provide
//! a simplified API for common operations.

pub mod complexity_facade;
pub mod dead_code_facade;
pub mod satd_facade;
pub mod analysis_orchestrator;

pub use complexity_facade::ComplexityFacade;
pub use dead_code_facade::DeadCodeFacade;
pub use satd_facade::SatdFacade;
pub use analysis_orchestrator::AnalysisOrchestrator;