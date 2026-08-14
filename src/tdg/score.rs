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
        //
        // Note what this condition does NOT consult: `critical_defects_suppressed`.
        // It did, briefly, and that inverted the whole design — the #279 waiver
        // skipped the penalty instead of the gate, so an uncommitted file with
        // five `.unwrap()` calls scored 100.0/A+ and `check-quality --min-grade A`
        // exited 0 on it, while the byte-identical committed file scored 9.1/F.
        // That is the very defect #919 was filed for (a score that changes with
        // git status), recreated one release after fixing it. A file's quality
        // does not change when you `git add` it: the penalty is a property of the
        // code, the waiver is a property of the gate, and they must not share a
        // condition.
        // `calculate_total` is called more than once on some paths, so any
        // attribution this function owns is rebuilt from scratch rather than
        // appended to — otherwise a second call would double-report the penalty.
        self.penalties_applied
            .retain(|p| p.source_metric != MetricCategory::CriticalDefect);

        if self.has_critical_defects {
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

            let before = self.total;
            let extra = self.critical_defects_count.saturating_sub(1).min(64) as i32;
            self.total = self.total.min(CEILING) * RETAINED_PER_DEFECT.powi(extra);

            // DISCLOSURE: this is by far the largest term in the score — up to
            // ~91 points — and it used to appear in `penalties_applied` nowhere,
            // so a consumer summing that list to explain a grade concluded
            // nothing had been penalised. It is attributed like every other
            // penalty, with the amount actually deducted.
            let count = self.critical_defects_count;
            self.penalties_applied.push(PenaltyAttribution {
                source_metric: MetricCategory::CriticalDefect,
                amount: before - self.total,
                applied_to: std::collections::HashSet::new(),
                issue: format!(
                    "{count} critical defect(s): score capped at {CEILING:.1} and \
                     multiplied by {RETAINED_PER_DEFECT} per additional defect"
                ),
            });
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
            // Not a component: it exists only to attribute the penalty
            // `calculate_total` applies, and no scorer produces it. There is no
            // field to write, and inventing one would put the penalty into the
            // component sum it is subtracted FROM.
            MetricCategory::CriticalDefect => {}
        }
    }
}

#[cfg(test)]
mod critical_defect_attribution_tests {
    use super::*;

    fn with_defects(count: usize) -> TdgScore {
        let mut score = TdgScore {
            has_critical_defects: count > 0,
            critical_defects_count: count,
            ..Default::default()
        };
        score.calculate_total();
        score
    }

    fn critical_penalty(score: &TdgScore) -> Option<&PenaltyAttribution> {
        score
            .penalties_applied
            .iter()
            .find(|p| p.source_metric == MetricCategory::CriticalDefect)
    }

    /// The dominant term in the score was undisclosed: `penalties_applied` was
    /// `[]` (or listed only the small component penalties) on a file whose score
    /// the critical-defect rule had just cut by up to 91 points.
    #[test]
    fn the_critical_defect_penalty_is_attributed() {
        let score = with_defects(5);
        let penalty = critical_penalty(&score).unwrap_or_else(|| {
            panic!(
                "no CriticalDefect attribution: {:?}",
                score.penalties_applied
            )
        });
        assert!(
            penalty.amount > 0.0,
            "the attribution must carry the amount actually deducted, got {}",
            penalty.amount
        );
        assert!(
            penalty.issue.contains('5'),
            "issue must name the count: {}",
            penalty.issue
        );
    }

    /// The attributed amount must be the drop the rule actually caused, so a
    /// consumer can reconstruct the total from the pre-penalty score minus the
    /// attributions. Off by the entire penalty before this existed.
    #[test]
    fn the_attributed_amount_equals_the_score_drop() {
        let score = with_defects(3);
        let pre_penalty = score.structural_complexity
            + score.semantic_complexity
            + score.duplication_ratio
            + score.coupling_score
            + score.doc_coverage
            + score.consistency_score
            + score.entropy_score;
        let penalty = critical_penalty(&score).expect("attribution").amount;
        assert!(
            (pre_penalty - penalty - score.total).abs() < 0.05,
            "pre-penalty {pre_penalty} - attributed {penalty} != total {}",
            score.total
        );
    }

    /// A clean file must not grow a phantom penalty.
    #[test]
    fn a_file_without_critical_defects_gets_no_attribution() {
        assert!(critical_penalty(&with_defects(0)).is_none());
    }

    /// `calculate_total` is called more than once on some paths; the
    /// attribution must not accumulate.
    #[test]
    fn recalculating_does_not_duplicate_the_attribution() {
        let mut score = with_defects(2);
        let first = score.total;
        score.calculate_total();
        assert_eq!(
            score
                .penalties_applied
                .iter()
                .filter(|p| p.source_metric == MetricCategory::CriticalDefect)
                .count(),
            1
        );
        assert!(
            (score.total - first).abs() < f32::EPSILON,
            "recalculation must be idempotent: {first} then {}",
            score.total
        );
    }
}
