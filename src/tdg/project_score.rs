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
    pub total_files: usize,
    /// Keyed by a `BTreeMap`, not a `HashMap`: serde emits a map in iteration
    /// order, and `HashMap`'s is randomised per process. See the note on
    /// `grade_distribution` — the same defect, same fix.
    pub language_distribution: BTreeMap<Language, usize>,
    /// Grade distribution: count of files per grade (A+, A, ..., F)
    ///
    /// DETERMINISM (round-3 sweep): this was a `HashMap<Grade, usize>`, so
    /// `analyze tdg --format json` emitted the SAME counts under a different
    /// KEY ORDER on every run — 6 runs over an unchanged tree gave 6 orders
    /// (`["B","BMinus","BPlus","A","APlus","AMinus"]`,
    /// `["BMinus","BPlus","B","APlus","AMinus","A"]`, …) with a stable sum of
    /// 495. A byte-diff of two identical runs was therefore never empty, which
    /// is what makes a JSON baseline useless. `BTreeMap` emits in `Grade`'s
    /// declaration order (best grade first), which is also the order a reader
    /// expects.
    #[serde(default)]
    pub grade_distribution: BTreeMap<Grade, usize>,
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

        // GH #680: the project grade and the per-file grades must come from the
        // SAME mapping, and that mapping is `Grade::from_score` — the score and
        // nothing else. The first attempt at #680 unified them the other way
        // round, onto `TdgScore::grade_for(score, has_contract_coverage)`, which
        // capped at A-: a project averaging 100.0 reported average_grade
        // `AMinus`. Agreement on a wrong grade is still a wrong grade.
        //
        // F-GRADE CAPPING: any F-grade file caps the project grade at B. That
        // cap IS derived from measured input (a file actually graded F) and is
        // reported through `grade_capped`, so it stays.
        let uncapped_grade = Grade::from_score(average_score);
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

        // Reported, not applied: contract coverage no longer rewrites a grade
        // (see `Grade::from_score`), but the aggregate should still say whether
        // every file it covers was contract-covered.
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

        // The project has exactly ONE score, and it is `average_score` — the
        // mean of the per-file totals that `aggregate` already computed and
        // that `analyze tdg --format json` serialises.
        //
        // `calculate_total()` above re-derives a total from the *component*
        // means instead, and the two disagree whenever any file hit the
        // >100 → /1.1 rescale branch: on a two-file fixture the components
        // said 87.75 while `average_score` said 82.75, so `pmat tdg --format
        // json` printed 87.75 and the SARIF/JSON project properties printed
        // 82.75 for the same run. The components stay as the breakdown; the
        // headline number and its grade come from the aggregate.
        avg.total = self.average_score;
        avg.grade = self.average_grade;
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
    /// average_grade "APlus" for the same score — two grades for one number in
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

    /// GH #680, second round. This test previously asserted the OPPOSITE —
    /// `assert_eq!(project.average_grade, Grade::AMinus)` for two files each
    /// totalling 100.0 — i.e. it pinned the defect. Contract coverage is
    /// unmeasured for any project without `contracts/binding.yaml`, so that cap
    /// made `APlus`/`A` unreachable at any score. Rewritten to assert the
    /// corrected contract: a perfect project grades A+ whether or not contract
    /// coverage was measured, in both the file and the project position.
    #[test]
    fn perfect_project_reaches_the_top_grade_regardless_of_contract_coverage() {
        for coverage in [false, true] {
            let files = vec![
                file_score(100.0, Language::Rust, coverage),
                file_score(100.0, Language::Rust, coverage),
            ];
            let project = ProjectScore::aggregate(files);
            assert_eq!(
                project.average_grade,
                Grade::APlus,
                "100.0 must grade A+ (has_contract_coverage={coverage})"
            );
            assert_eq!(project.files[0].grade, Grade::APlus);
            assert_eq!(project.average().grade, Grade::APlus);
        }
    }

    /// The grade is a function of the score and of nothing else: flipping an
    /// unrelated input must not move it.
    #[test]
    fn grade_depends_only_on_the_score() {
        for total in [100.0, 97.0, 92.0, 87.0, 81.0, 60.0, 10.0] {
            let with = file_score(total, Language::Rust, true);
            let without = file_score(total, Language::Rust, false);
            assert_eq!(
                with.grade, without.grade,
                "contract coverage must not change the grade of {total}"
            );
            assert_eq!(with.grade, Grade::from_score(with.total));
        }
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

    /// DETERMINISM (round-3 sweep): `analyze tdg --format json` emitted
    /// `grade_distribution` in a different KEY ORDER on every run — 6 runs over
    /// an unchanged tree gave 6 orders (`["B","BMinus","BPlus","A","APlus",
    /// "AMinus"]`, `["BMinus","BPlus","B","APlus","AMinus","A"]`, …) while the
    /// counts summed to a stable 495 every time. Serde emits a map in iteration
    /// order and `HashMap`'s is randomised per process.
    ///
    /// Each iteration aggregates a FRESH set of scores, so each builds a fresh
    /// map: the in-process stand-in for "run the binary again".
    #[test]
    fn distribution_key_order_is_stable_across_fresh_aggregations() {
        fn serialized() -> (String, String) {
            let files = vec![
                file_score(99.0, Language::Rust, false),
                file_score(96.0, Language::Python, false),
                file_score(92.0, Language::Rust, false),
                file_score(88.0, Language::Markdown, false),
                file_score(84.0, Language::TypeScript, false),
                file_score(81.0, Language::Go, false),
                file_score(75.0, Language::Rust, false),
            ];
            let project = ProjectScore::aggregate(files);
            (
                serde_json::to_string(&project.grade_distribution).unwrap(),
                serde_json::to_string(&project.language_distribution).unwrap(),
            )
        }

        let (grades, languages) = serialized();
        // More than one key, or the test would pass vacuously.
        assert!(grades.matches(':').count() > 1, "grades: {grades}");
        assert!(languages.matches(':').count() > 1, "languages: {languages}");

        for i in 0..10 {
            assert_eq!(
                serialized(),
                (grades.clone(), languages.clone()),
                "iteration {i}: identical input must serialize byte-identically"
            );
        }

        // Ordered by grade, best first — a reader's order, not a hash seed's.
        let order: Vec<&str> = grades
            .split('"')
            .filter(|piece| {
                !piece.is_empty() && piece.chars().next().is_some_and(char::is_alphabetic)
            })
            .collect();
        let all_grades_best_first = [
            "APlus", "A", "AMinus", "BPlus", "B", "BMinus", "CPlus", "C", "CMinus", "D", "F",
        ];
        let mut expected = order.clone();
        expected.sort_by_key(|name| {
            all_grades_best_first
                .iter()
                .position(|g| g == name)
                .expect("known grade")
        });
        assert_eq!(order, expected, "grade keys must be in grade order");
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
