#![cfg_attr(coverage_nightly, coverage(off))]

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::grade::{Grade, MetricCategory, PenaltyAttribution};
use super::language_simple::Language;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    pub has_critical_defects: bool,    // Known Defects v2.1: Auto-fail flag
    pub has_contract_coverage: bool,   // CB-1400: Provable-contract coverage (caps A→A- if false)
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
            grade: Grade::APLus,
            confidence: 1.0,
            language: Language::Unknown,
            file_path: None,
            penalties_applied: Vec::new(),
            critical_defects_count: 0,
            has_critical_defects: false,
            has_contract_coverage: false,
        }
    }
}

impl TdgScore {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

        // Known Defects v2.1: Auto-fail if critical defects detected
        if self.has_critical_defects {
            self.total = 0.0;
            self.grade = Grade::F;
        } else {
            self.grade = Grade::from_score(self.total);

            // CB-1400: Provable-contract coverage required for A-tier grades.
            // A "perfect" file without contract coverage caps at A-.
            // This enforces contract-first design as a hard requirement for
            // the highest quality tier — you cannot be truly "excellent"
            // without provable guarantees.
            // Note: Grade enum ordering is APLus < A < AMinus (lower = better)
            if !self.has_contract_coverage && self.grade < Grade::AMinus {
                self.grade = Grade::AMinus;
            }
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
