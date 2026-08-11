//! Critical-defect gate: fails on any file carrying an unsuppressed critical
//! defect, independently of that file's score.
//!
//! The Known-Defects auto-fail used to be expressed by writing `0.0 / F` into
//! the score, which meant the gate and the measurement were the same number.
//! Two bad consequences followed from that coupling. The score became useless
//! as a gradient — every offending file read exactly 0.0 no matter what else
//! was true of it — and any change that softened the score would silently
//! disarm the enforcement, since `FGradeGate` only ever saw `grade == F` and
//! could not tell a critical defect from ordinary low quality.
//!
//! Splitting them fixes both. `TdgScore::calculate_total` applies a graduated
//! penalty an agent can climb; this gate reads `has_critical_defects` directly
//! and fails hard regardless of where the score landed.

use super::types::{GateResult, QualityGate, Severity, Violation, ViolationType};
use crate::tdg::TdgBaseline;
use anyhow::Result;

/// Fails when any file carries a critical defect that was not suppressed.
///
/// Files whose defects were waived under #279 (in a git repository, no commits
/// yet) are reported as waived rather than counted as violations — the waiver
/// is recorded on `TdgScore::critical_defects_suppressed` with its reason.
pub struct CriticalDefectGate {
    /// Maximum allowed files with unsuppressed critical defects (default: 0).
    max_files: usize,
}

impl CriticalDefectGate {
    /// Create a new instance.
    #[must_use]
    pub fn new(max_files: usize) -> Self {
        Self { max_files }
    }

    /// Create with strict defaults (zero critical defects allowed).
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(0)
    }
}

impl Default for CriticalDefectGate {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl QualityGate for CriticalDefectGate {
    fn name(&self) -> &str {
        "CriticalDefectGate"
    }

    fn check(&self, _baseline: &TdgBaseline, current: &TdgBaseline) -> Result<GateResult> {
        let mut violations = Vec::new();
        let mut waived = 0usize;

        for (path, entry) in &current.files {
            if !entry.score.has_critical_defects {
                continue;
            }
            if entry.score.critical_defects_suppressed.is_some() {
                waived += 1;
                continue;
            }
            violations.push(Violation {
                path: path.clone(),
                violation_type: ViolationType::BelowMinimum,
                severity: Severity::Critical,
                message: format!(
                    "{} critical defect(s) - requires immediate attention",
                    entry.score.critical_defects_count
                ),
                old_score: None,
                new_score: entry.score.total,
                old_grade: None,
                new_grade: entry.score.grade,
            });
        }

        let offending = violations.len();
        let passed = offending <= self.max_files;

        // Say how many files were waived. A gate that silently skips part of
        // its input reports a pass that means less than it appears to.
        let message = match (offending, waived) {
            (0, 0) => "No critical defects".to_string(),
            (0, w) => format!("No unsuppressed critical defects ({w} file(s) waived under #279)"),
            (n, 0) => format!("{n} file(s) with critical defects"),
            (n, w) => format!("{n} file(s) with critical defects ({w} waived under #279)"),
        };

        Ok(GateResult {
            gate_name: self.name().to_string(),
            passed,
            violations,
            message,
        })
    }
}

#[cfg(test)]
#[path = "critical_defect_tests.rs"]
mod critical_defect_tests;
