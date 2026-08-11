#![cfg_attr(coverage_nightly, coverage(off))]

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::grade::{Grade, MetricCategory, PenaltyAttribution};
use super::language_simple::Language;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Tdg score.
pub struct TdgScore {
    pub structural_complexity: f32,
    pub semantic_complexity: f32,
    pub duplication_ratio: f32,
    pub coupling_score: f32,
    pub doc_coverage: f32,
    pub consistency_score: f32,
    pub entropy_score: f32, // New: Pattern entropy analysis
    pub total: f32,
    pub grade: Grade,
    pub confidence: f32,
    pub language: Language,
    pub file_path: Option<PathBuf>,
    pub penalties_applied: Vec<PenaltyAttribution>,
    pub critical_defects_count: usize, // Known Defects v2.1: Count of critical defects
    /// Whether critical defects EXIST. Always `critical_defects_count > 0`.
    ///
    /// This used to double as the auto-fail switch, so the #279 exemption for
    /// untracked files cleared it while leaving the count set — a record saying
    /// "1 critical defect" and "no critical defects" at once (#919). Whether the
    /// gate FIRES is now `critical_defects_suppressed`; this field only reports
    /// what was found.
    pub has_critical_defects: bool,
    /// Why the critical-defect auto-fail did not fire despite defects existing.
    ///
    /// `None` means it fired (or there was nothing to fire on). `Some(reason)`
    /// records the #279 exemption — a file with no git history must not be
    /// auto-failed by a gate it cannot pass until it is committed — and makes
    /// that exemption visible to anyone reading the score or a persisted
    /// baseline, instead of leaving them to infer it from a contradiction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub critical_defects_suppressed: Option<String>,
    pub has_contract_coverage: bool, // CB-1400: Provable-contract coverage (caps A→A- if false)
}

impl Default for TdgScore {
    fn default() -> Self {
        Self {
            structural_complexity: 25.0,
            semantic_complexity: 20.0,
            duplication_ratio: 20.0,
            coupling_score: 15.0,
            doc_coverage: 10.0,
            consistency_score: 10.0,
            entropy_score: 0.0, // New: Start with 0, calculated during analysis
            total: 100.0,
            grade: Grade::APlus,
            confidence: 1.0,
            language: Language::Unknown,
            file_path: None,
            penalties_applied: Vec::new(),
            critical_defects_count: 0,
            has_critical_defects: false,
            critical_defects_suppressed: None,
            has_contract_coverage: false,
        }
    }
}

impl TdgScore {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Calculate total.
    pub fn calculate_total(&mut self) {
        // Clamp individual components to their expected weight ranges
        // This ensures components can never exceed their designated contribution
        self.structural_complexity = self.structural_complexity.clamp(0.0, 25.0);
        self.semantic_complexity = self.semantic_complexity.clamp(0.0, 20.0);
        self.duplication_ratio = self.duplication_ratio.clamp(0.0, 20.0);
        self.coupling_score = self.coupling_score.clamp(0.0, 15.0);
        self.doc_coverage = self.doc_coverage.clamp(0.0, 10.0);
        self.consistency_score = self.consistency_score.clamp(0.0, 10.0);

        // Entropy score should have a reasonable weight (max ~10 points)
        // to balance with other metrics without dominating
        self.entropy_score = self.entropy_score.clamp(0.0, 10.0);

        // Sum all clamped components
        let raw_total = self.structural_complexity
            + self.semantic_complexity
            + self.duplication_ratio
            + self.coupling_score
            + self.doc_coverage
            + self.consistency_score
            + self.entropy_score;

        // The total is already in 0-110 range after clamping individual components
        // Since the original weights sum to 100, and entropy adds up to 10 more,
        // we need to normalize back to 0-100 scale
        // Strategy: If raw_total <= 100, use it as-is for backward compatibility
        //           If raw_total > 100, scale it proportionally
        if raw_total <= 100.0 {
            self.total = raw_total.clamp(0.0, 100.0);
        } else {
            // Scale down proportionally when entropy pushes total above 100
            const THEORETICAL_MAX: f32 = 110.0; // 25+20+20+15+10+10+10
            self.total = (raw_total / THEORETICAL_MAX * 100.0).clamp(0.0, 100.0);
        }

        // Known Defects v2.1: critical defects degrade the score steeply, but
        // they no longer erase it.
        //
        // This used to assign `total = 0.0, grade = F` outright. That reads as
        // decisive and is in fact the least informative answer available: every
        // file with one `.unwrap()` collapsed to EXACTLY 0.0 regardless of its
        // documentation, complexity, size or test coverage, so a 61-function
        // fully documented module and a one-line disaster were indistinguishable,
        // and fixing nine of ten defects moved the number not at all. For the
        // agents and CI jobs that consume this score as a control signal, a
        // constant is not a signal — there is no gradient to climb.
        //
        // The penalty is now proportional and monotone in the defect count, and
        // is additionally capped below B- so that a file carrying unsuppressed
        // critical defects can never present as merely "good".
        //
        // Softening the SCORE must never soften the GATE. Whether a build fails
        // is `CriticalDefectGate`, which keys on `has_critical_defects` directly
        // and is unaffected by anything computed here — see
        // `crate::tdg::quality_gate::critical_defect`. Expressing the gate as a
        // magic score value is what coupled them in the first place.
        if self.has_critical_defects && self.critical_defects_suppressed.is_none() {
            /// Ceiling for any file still carrying critical defects: the top of
            /// the C+ band, so such a file cannot grade B- or better whatever
            /// else it does well. Applied BEFORE the decay, so one defect in a
            /// perfect file and one in a mediocre one are still distinguishable.
            const CEILING: f32 = 69.9;
            /// Each additional defect retains this fraction of the score.
            ///
            /// Geometric rather than linear so the result is strictly
            /// decreasing at every count and never reaches a floor where
            /// further defects stop mattering. A flat region anywhere is the
            /// same defect as the old constant 0.0, only further out.
            const RETAINED_PER_DEFECT: f32 = 0.6;

            let extra = self.critical_defects_count.saturating_sub(1).min(64) as i32;
            self.total = self.total.min(CEILING) * RETAINED_PER_DEFECT.powi(extra);
        }

        // GH #680, second round. Both the file path and the aggregate path now
        // use `Grade::from_score` and nothing else, so the grade is a function
        // of the score alone.
        //
        // What used to sit here was the CB-1400 override
        // `if !has_contract_coverage && grade < AMinus { grade = AMinus }`.
        // `has_contract_coverage` is false whenever contract coverage was never
        // *measured* (no `contracts/binding.yaml` ⇒ the field keeps its `false`
        // default), so the cap fired for essentially every project: a fixture
        // totalling 100.0 was graded `AMinus`, and `pmat tdg` printed the
        // self-contradicting `Overall Score: 100.0/100 (A-)`. An unmeasured
        // signal must never rewrite a measured one. Contract coverage is still
        // recorded on `has_contract_coverage` and still gated by `pmat comply`
        // (CB-1400).
        self.grade = Grade::from_score(self.total);
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Set metric.
    pub fn set_metric(&mut self, category: MetricCategory, value: f32) {
        match category {
            MetricCategory::StructuralComplexity => self.structural_complexity = value,
            MetricCategory::SemanticComplexity => self.semantic_complexity = value,
            MetricCategory::Duplication => self.duplication_ratio = value,
            MetricCategory::Coupling => self.coupling_score = value,
            MetricCategory::Documentation => self.doc_coverage = value,
            MetricCategory::Consistency => self.consistency_score = value,
        }
    }
}
