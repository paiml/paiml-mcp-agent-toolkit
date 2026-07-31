#![cfg_attr(coverage_nightly, coverage(off))]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub total_files: usize,
    pub language_distribution: HashMap<Language, usize>,
    /// Grade distribution: count of files per grade (A+, A, ..., F)
    #[serde(default)]
    pub grade_distribution: HashMap<Grade, usize>,
    /// Count of F-grade files (critical quality issues)
    #[serde(default)]
    pub f_grade_count: usize,
    /// Whether grade was capped due to F-grade files
    #[serde(default)]
    pub grade_capped: bool,
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

        let mut language_distribution = HashMap::new();
        let mut grade_distribution = HashMap::new();
        let mut f_grade_count = 0;

        for score in &scores {
            *language_distribution.entry(score.language).or_insert(0) += 1;
            *grade_distribution.entry(score.grade).or_insert(0) += 1;
            if score.grade == Grade::F {
                f_grade_count += 1;
            }
        }

        // The project grade must come from the SAME mapping the per-file grades
        // came from. `Grade::from_score(average_score)` alone skipped the
        // CB-1400 contract-coverage cap that `TdgScore::calculate_total` applies
        // to every file, so a single-file project reported files[0].grade
        // "AMinus" and average_grade "APLus" for one and the same score of 99.7
        // (GH #680). A project is only eligible for the A-tier if every file it
        // contains was.
        let all_files_have_contract_coverage = scores.iter().all(|s| s.has_contract_coverage);

        // F-GRADE CAPPING: Any F-grade file caps the project grade at B
        // This prevents hiding critical quality issues in averaging
        let uncapped_grade = TdgScore::grade_for(average_score, all_files_have_contract_coverage);
        let (average_grade, grade_capped) = if f_grade_count > 0 && uncapped_grade < Grade::B {
            // Cap at B (score 79.9 equivalent) if any F-grades exist
            (Grade::B, true)
        } else {
            (uncapped_grade, false)
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
        }
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

        // Carried so `calculate_total()` below applies the same CB-1400 cap the
        // files were graded with (GH #680).
        avg.has_contract_coverage = self.files.iter().all(|s| s.has_contract_coverage);

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

        // Set language to the most common language in the project.
        //
        // The tiebreak is part of the answer, not a detail: `language_distribution`
        // is a HashMap, so `max_by_key(count)` alone returned whichever tied
        // language the randomised iteration order happened to visit last. Two
        // .rs and two .md files made `tdg <dir>` report "Rust" and "Markdown"
        // in alternate runs over unchanged input — 7 vs 5 out of 12 runs, with
        // a byte-identical score and confidence (GH #673). Ties now resolve to
        // the lowest `Language` discriminant, which orders source languages
        // ahead of YAML/Markdown.
        if let Some((&lang, _)) = self
            .language_distribution
            .iter()
            .max_by_key(|(&lang, &count)| (count, std::cmp::Reverse(lang as usize)))
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod no_contradiction_tests {
    use super::*;

    fn file_score(total: f32, language: Language, has_contract_coverage: bool) -> TdgScore {
        // Drive the real scoring path so `grade` is whatever pmat would print.
        let mut score = TdgScore {
            language,
            has_contract_coverage,
            structural_complexity: total * 0.25,
            semantic_complexity: total * 0.20,
            duplication_ratio: total * 0.20,
            coupling_score: total * 0.15,
            doc_coverage: total * 0.10,
            consistency_score: total * 0.10,
            entropy_score: 0.0,
            ..TdgScore::default()
        };
        score.calculate_total();
        score
    }

    /// GH #680: a single-file project reported files[0].grade "AMinus" and
    /// average_grade "APLus" for the same score — two grades for one number in
    /// one document.
    #[test]
    fn single_file_project_grade_equals_that_files_grade() {
        let file = file_score(99.7, Language::Rust, false);
        let expected = file.grade;
        let project = ProjectScore::aggregate(vec![file]);

        assert_eq!(project.total_files, 1);
        assert_eq!(
            project.average_grade, expected,
            "average_grade must equal the only file's grade"
        );
        assert_eq!(
            project.average_grade, project.files[0].grade,
            "the same score must map to the same grade in both places"
        );
    }

    /// The A-tier contract-coverage cap must apply to the project grade too,
    /// not only per file.
    #[test]
    fn project_grade_respects_contract_coverage_cap() {
        let files = vec![
            file_score(100.0, Language::Rust, false),
            file_score(100.0, Language::Rust, false),
        ];
        let project = ProjectScore::aggregate(files);
        assert_eq!(project.average_grade, Grade::AMinus);
    }

    /// GH #673: identical input flipped between Rust and Markdown run to run
    /// because the winner of a tie came out of HashMap iteration order.
    /// Deterministic over many freshly built maps, and source beats docs.
    #[test]
    fn tied_language_distribution_is_deterministic() {
        for _ in 0..64 {
            let files = vec![
                file_score(90.0, Language::Rust, true),
                file_score(90.0, Language::Markdown, true),
                file_score(90.0, Language::Rust, true),
                file_score(90.0, Language::Markdown, true),
            ];
            let project = ProjectScore::aggregate(files);
            assert_eq!(
                project.average().language,
                Language::Rust,
                "a 2-2 Rust/Markdown tie must always resolve the same way"
            );
        }
    }

    /// A clear majority still wins; the tiebreak only settles equal counts.
    #[test]
    fn language_majority_still_wins_over_the_tiebreak() {
        let files = vec![
            file_score(90.0, Language::Markdown, true),
            file_score(90.0, Language::Markdown, true),
            file_score(90.0, Language::Rust, true),
        ];
        let project = ProjectScore::aggregate(files);
        assert_eq!(project.average().language, Language::Markdown);
    }
}
