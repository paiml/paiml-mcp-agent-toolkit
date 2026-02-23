#![cfg_attr(coverage_nightly, coverage(off))]
/// Diagnostic tools for Transactional Hashed TDG System
///
/// Provides comprehensive monitoring, profiling, and debugging capabilities
/// for the TDG system including storage, scheduling, and performance metrics.

mod property_tests;
mod tests;
mod tests_part2;
mod tool;
mod types;

pub use tool::DiagnosticTool;
pub use types::{
    AdaptiveDiagnostics, EnforcementStats, HealthStatus, PerformanceDiagnostics,
    ResourceDiagnostics, SchedulerDiagnostics, StorageDiagnostics, SystemDiagnostics,
};
