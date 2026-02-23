//! Quality Gate System for TDG Enforcement (Sprint 66 Phase 2)
//!
//! This module provides quality gates that can enforce quality standards
//! by detecting regressions, enforcing minimum grades, and validating new files.

mod f_grade;
mod grade;
mod newfile;
mod regression;
mod types;

pub use f_grade::FGradeGate;
pub use grade::MinimumGradeGate;
pub use newfile::NewFileGate;
pub use regression::RegressionGate;
pub use types::{GateConfig, GateResult, QualityGate, Severity, Violation, ViolationType};

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_defaults_methods() {
        let regression = RegressionGate::with_defaults();
        assert_eq!(regression.name(), "RegressionGate");

        let min_grade = MinimumGradeGate::with_defaults();
        assert_eq!(min_grade.name(), "MinimumGradeGate");

        let new_file = NewFileGate::with_defaults();
        assert_eq!(new_file.name(), "NewFileGate");

        let f_grade = FGradeGate::with_defaults();
        assert_eq!(f_grade.name(), "FGradeGate");
    }
}
