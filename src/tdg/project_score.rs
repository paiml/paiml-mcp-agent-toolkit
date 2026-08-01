#![cfg_attr(coverage_nightly, coverage(off))]

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::grade::Grade;
use super::language_simple::Language;
use super::score::TdgScore;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Project score.
pub struct ProjectScore {
    pub files: Vec<TdgScore>,
    pub average_score: f32,
    #[serde(default)]
    pub average_grade: Grade,
    /// Number of files ANALYSED. Independent of how many entries `files`
    /// carries — see `files_reported`/`files_truncated`.
    pub total_files: usize,
    // BTreeMap, not HashMap: these two maps are serialized straight into JSON,
    // where HashMap iteration made the key order differ on every run for
    // byte-identical input (6 runs -> 6 different grade_distribution orders).
    pub language_distribution: BTreeMap<Language, usize>,
    /// Grade distribution: count of files per grade (A+, A, ..., F).
    /// Always covers every analysed file, even when `files` is truncated.
    #[serde(default)]
    pub grade_distribution: BTreeMap<Grade, usize>,
    /// Count of F-grade files (critical quality issues)
    #[serde(default)]
    pub f_grade_count: usize,
    /// Whether grade was capped due to F-grade files
    #[serde(default)]
    pub grade_capped: bool,
    /// Number of entries actually present in `files`. Equals `total_files`
    /// unless `--top-files` truncated the list.
    #[serde(default)]
    pub files_reported: usize,
    /// True when `files` holds only the worst `--top-files` entries. A capped
    /// list is never presented as the whole population.
    #[serde(default)]
    pub files_truncated: bool,
}

impl ProjectScore {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Aggregate.
    pub fn aggregate(scores: Vec<TdgScore>) -> Self {
        let total_files = scores.len();
        let average_score = if total_files > 0 {
            scores.iter().map(|s| s.total).sum::<f32>() / total_files as f32
        } else {
            0.0
        };

        let mut language_distribution = BTreeMap::new();
        let mut grade_distribution = BTreeMap::new();
        let mut f_grade_count = 0;

        for score in &scores {
            *language_distribution.entry(score.language).or_insert(0) += 1;
            *grade_distribution.entry(score.grade).or_insert(0) += 1;
            if score.grade == Grade::F {
                f_grade_count += 1;
            }
        }

        // F-GRADE CAPPING: Any F-grade file caps the project grade at B
        // This prevents hiding critical quality issues in averaging
        let (average_grade, grade_capped) = if f_grade_count > 0 {
            let uncapped_grade = Grade::from_score(average_score);
            // Cap at B (score 79.9 equivalent) if any F-grades exist
            if uncapped_grade < Grade::B {
                (Grade::B, true)
            } else {
                (uncapped_grade, false)
            }
        } else {
            (Grade::from_score(average_score), false)
        };

        Self {
            files: scores,
            average_score,
            average_grade,
            total_files,
            language_distribution,
            grade_distribution,
            f_grade_count,
            grade_capped,
            files_reported: total_files,
            files_truncated: false,
        }
    }

    /// Keep only the `limit` WORST-scoring files in `files` (`limit == 0`
    /// keeps every file), always in a deterministic worst-first order.
    ///
    /// This is what `analyze tdg -n/--top-files` means. The flag used to be a
    /// complete no-op: `-n 5`, `-n 10` and `-n 100000` returned identical
    /// output, and `-n 3` on a 1593-file tree still emitted all 1593 entries.
    ///
    /// Truncation NEVER touches `total_files`, `grade_distribution`,
    /// `language_distribution`, `average_score` or `f_grade_count` — those stay
    /// whole-project — and it records `files_reported` + `files_truncated` so a
    /// capped list can never be mistaken for the full population.
    pub fn limit_to_worst_files(&mut self, limit: usize) {
        // Deterministic order regardless of filesystem walk order: worst score
        // first, ties broken by path.
        self.files.sort_by(|a, b| {
            a.total
                .partial_cmp(&b.total)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.file_path.cmp(&b.file_path))
        });

        if limit > 0 && limit < self.files.len() {
            self.files.truncate(limit);
            self.files_truncated = true;
        } else {
            self.files_truncated = false;
        }
        self.files_reported = self.files.len();
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Calculate the average.
    pub fn average(&self) -> TdgScore {
        if self.files.is_empty() {
            // No files analyzed — return zero score, not perfect score
            return TdgScore {
                total: 0.0,
                grade: crate::tdg::Grade::F,
                confidence: 0.0,
                ..TdgScore::default()
            };
        }

        let mut avg = TdgScore::default();
        let count = self.files.len() as f32;

        avg.structural_complexity = self
            .files
            .iter()
            .map(|s| s.structural_complexity)
            .sum::<f32>()
            / count;
        avg.semantic_complexity = self
            .files
            .iter()
            .map(|s| s.semantic_complexity)
            .sum::<f32>()
            / count;
        avg.duplication_ratio = self.files.iter().map(|s| s.duplication_ratio).sum::<f32>() / count;
        avg.coupling_score = self.files.iter().map(|s| s.coupling_score).sum::<f32>() / count;
        avg.doc_coverage = self.files.iter().map(|s| s.doc_coverage).sum::<f32>() / count;
        avg.consistency_score = self.files.iter().map(|s| s.consistency_score).sum::<f32>() / count;
        avg.entropy_score = self.files.iter().map(|s| s.entropy_score).sum::<f32>() / count;
        avg.confidence = self.files.iter().map(|s| s.confidence).sum::<f32>() / count;

        // Set language to the most common language in the project
        if let Some((&lang, _)) = self
            .language_distribution
            .iter()
            .max_by_key(|(_, &count)| count)
        {
            avg.language = lang;
        }

        avg.calculate_total();
        avg
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Comparison.
pub struct Comparison {
    pub source1: TdgScore,
    pub source2: TdgScore,
    pub delta: f32,
    pub improvement_percentage: f32,
    pub winner: String,
    pub improvements: Vec<String>,
    pub regressions: Vec<String>,
}

impl Comparison {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(source1: TdgScore, source2: TdgScore) -> Self {
        let delta = source2.total - source1.total;
        let improvement_percentage = if source1.total > 0.0 {
            (delta / source1.total) * 100.0
        } else {
            0.0
        };

        let winner = if source2.total > source1.total {
            source2
                .file_path
                .as_ref()
                .map_or_else(|| "source2".to_string(), |p| p.display().to_string())
        } else {
            source1
                .file_path
                .as_ref()
                .map_or_else(|| "source1".to_string(), |p| p.display().to_string())
        };

        let mut improvements = Vec::new();
        let mut regressions = Vec::new();

        if source2.structural_complexity > source1.structural_complexity {
            improvements.push(format!(
                "Structural complexity improved by {:.1}",
                source2.structural_complexity - source1.structural_complexity
            ));
        } else if source2.structural_complexity < source1.structural_complexity {
            regressions.push(format!(
                "Structural complexity degraded by {:.1}",
                source1.structural_complexity - source2.structural_complexity
            ));
        }

        if source2.semantic_complexity > source1.semantic_complexity {
            improvements.push(format!(
                "Semantic complexity improved by {:.1}",
                source2.semantic_complexity - source1.semantic_complexity
            ));
        } else if source2.semantic_complexity < source1.semantic_complexity {
            regressions.push(format!(
                "Semantic complexity degraded by {:.1}",
                source1.semantic_complexity - source2.semantic_complexity
            ));
        }

        if source2.duplication_ratio > source1.duplication_ratio {
            improvements.push(format!(
                "Code duplication reduced by {:.1}",
                source2.duplication_ratio - source1.duplication_ratio
            ));
        } else if source2.duplication_ratio < source1.duplication_ratio {
            regressions.push(format!(
                "Code duplication increased by {:.1}",
                source1.duplication_ratio - source2.duplication_ratio
            ));
        }

        if source2.doc_coverage > source1.doc_coverage {
            improvements.push(format!(
                "Documentation coverage improved by {:.1}",
                source2.doc_coverage - source1.doc_coverage
            ));
        } else if source2.doc_coverage < source1.doc_coverage {
            regressions.push(format!(
                "Documentation coverage decreased by {:.1}",
                source1.doc_coverage - source2.doc_coverage
            ));
        }

        Self {
            source1,
            source2,
            delta,
            improvement_percentage,
            winner,
            improvements,
            regressions,
        }
    }
}
