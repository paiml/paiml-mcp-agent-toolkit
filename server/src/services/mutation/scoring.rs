//! Mutation scoring and analysis

use super::types::*;
use std::collections::HashMap;
use std::path::PathBuf;

/// Mutation scorer for analyzing test suite quality
pub struct MutationScorer {
    results: Vec<MutationResult>,
}

impl MutationScorer {
    /// Create new scorer from results
    pub fn new(results: Vec<MutationResult>) -> Self {
        Self { results }
    }

    /// Calculate mutation score
    pub fn calculate_score(&self) -> MutationScore {
        MutationScore::from_results(&self.results)
    }

    /// Identify weak spots in test coverage
    pub fn weak_spots(&self) -> Vec<WeakSpot> {
        let mut file_survivors: HashMap<PathBuf, Vec<&MutationResult>> = HashMap::new();

        // Group survived mutants by file
        for result in &self.results {
            if result.status == MutantStatus::Survived {
                file_survivors
                    .entry(result.mutant.original_file.clone())
                    .or_default()
                    .push(result);
            }
        }

        // Create weak spots for files with survived mutants
        let mut weak_spots = Vec::new();

        for (file, survivors) in file_survivors {
            if survivors.is_empty() {
                continue;
            }

            // Find line range
            let min_line = survivors
                .iter()
                .map(|r| r.mutant.location.line)
                .min()
                .unwrap_or(0);
            let max_line = survivors
                .iter()
                .map(|r| r.mutant.location.end_line)
                .max()
                .unwrap_or(0);

            // Generate suggestions
            let suggestions = generate_suggestions(&file, survivors.len());

            weak_spots.push(WeakSpot {
                file,
                line_range: (min_line, max_line),
                survived_mutants: survivors.len(),
                suggestions,
            });
        }

        // Sort by survived mutants (most critical first)
        weak_spots.sort_by(|a, b| b.survived_mutants.cmp(&a.survived_mutants));

        weak_spots
    }

    /// Get summary statistics
    pub fn summary(&self) -> MutationSummary {
        let score = self.calculate_score();
        let weak_spots = self.weak_spots();

        MutationSummary {
            total_mutants: score.total,
            killed: score.killed,
            survived: score.survived,
            compile_errors: score.compile_errors,
            timeouts: score.timeouts,
            equivalent: score.equivalent,
            mutation_score: score.score,
            weak_spots,
        }
    }
}

/// Mutation testing summary
#[derive(Debug, Clone)]
pub struct MutationSummary {
    pub total_mutants: usize,
    pub killed: usize,
    pub survived: usize,
    pub compile_errors: usize,
    pub timeouts: usize,
    pub equivalent: usize,
    pub mutation_score: f64,
    pub weak_spots: Vec<WeakSpot>,
}

/// Generate test improvement suggestions
fn generate_suggestions(file: &PathBuf, survived_count: usize) -> Vec<String> {
    let mut suggestions = Vec::new();

    suggestions.push(format!(
        "Add {} test(s) to cover mutations in {}",
        survived_count,
        file.display()
    ));

    if survived_count > 5 {
        suggestions.push(
            "Consider adding property-based tests to catch edge cases".to_string(),
        );
    }

    suggestions.push(
        "Review boundary conditions and error handling in this file".to_string(),
    );

    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_result(status: MutantStatus, file: &str, line: usize) -> MutationResult {
        MutationResult {
            mutant: Mutant {
                id: "test".to_string(),
                original_file: PathBuf::from(file),
                mutated_source: String::new(),
                location: SourceLocation {
                    line,
                    column: 1,
                    end_line: line,
                    end_column: 10,
                },
                operator: MutationOperatorType::ArithmeticReplacement,
                hash: "hash".to_string(),
                status: status.clone(),
            },
            status,
            test_failures: vec![],
            execution_time_ms: 100,
            error_message: None,
        }
    }

    #[test]
    fn test_mutation_scorer_calculate_score() {
        let results = vec![
            create_result(MutantStatus::Killed, "foo.rs", 10),
            create_result(MutantStatus::Killed, "foo.rs", 20),
            create_result(MutantStatus::Survived, "foo.rs", 30),
        ];

        let scorer = MutationScorer::new(results);
        let score = scorer.calculate_score();

        assert_eq!(score.total, 3);
        assert_eq!(score.killed, 2);
        assert_eq!(score.survived, 1);
        assert!((score.score - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_mutation_scorer_weak_spots() {
        let results = vec![
            create_result(MutantStatus::Survived, "foo.rs", 10),
            create_result(MutantStatus::Survived, "foo.rs", 15),
            create_result(MutantStatus::Survived, "bar.rs", 5),
            create_result(MutantStatus::Killed, "baz.rs", 1),
        ];

        let scorer = MutationScorer::new(results);
        let weak_spots = scorer.weak_spots();

        assert_eq!(weak_spots.len(), 2);

        // foo.rs should be first (2 survivors)
        assert_eq!(weak_spots[0].file, PathBuf::from("foo.rs"));
        assert_eq!(weak_spots[0].survived_mutants, 2);
        assert_eq!(weak_spots[0].line_range, (10, 15));

        // bar.rs should be second (1 survivor)
        assert_eq!(weak_spots[1].file, PathBuf::from("bar.rs"));
        assert_eq!(weak_spots[1].survived_mutants, 1);
    }

    #[test]
    fn test_mutation_scorer_summary() {
        let results = vec![
            create_result(MutantStatus::Killed, "foo.rs", 10),
            create_result(MutantStatus::Survived, "foo.rs", 20),
            create_result(MutantStatus::CompileError, "bar.rs", 5),
        ];

        let scorer = MutationScorer::new(results);
        let summary = scorer.summary();

        assert_eq!(summary.total_mutants, 3);
        assert_eq!(summary.killed, 1);
        assert_eq!(summary.survived, 1);
        assert_eq!(summary.compile_errors, 1);
        assert_eq!(summary.weak_spots.len(), 1);
    }

    // Sprint 25: Dogfooding - Additional edge case tests

    #[test]
    fn test_weak_spots_empty_results() {
        let results = vec![];
        let scorer = MutationScorer::new(results);
        let weak_spots = scorer.weak_spots();

        assert_eq!(weak_spots.len(), 0, "Empty results should have no weak spots");
    }

    #[test]
    fn test_weak_spots_no_survivors() {
        let results = vec![
            create_result(MutantStatus::Killed, "foo.rs", 10),
            create_result(MutantStatus::Killed, "bar.rs", 20),
            create_result(MutantStatus::Equivalent, "baz.rs", 30),
        ];

        let scorer = MutationScorer::new(results);
        let weak_spots = scorer.weak_spots();

        assert_eq!(weak_spots.len(), 0, "No survivors should mean no weak spots");
    }

    #[test]
    fn test_weak_spots_single_survivor() {
        let results = vec![
            create_result(MutantStatus::Killed, "foo.rs", 10),
            create_result(MutantStatus::Survived, "bar.rs", 20),
        ];

        let scorer = MutationScorer::new(results);
        let weak_spots = scorer.weak_spots();

        assert_eq!(weak_spots.len(), 1);
        assert_eq!(weak_spots[0].file, PathBuf::from("bar.rs"));
        assert_eq!(weak_spots[0].survived_mutants, 1);
        assert_eq!(weak_spots[0].line_range, (20, 20));
    }

    #[test]
    fn test_weak_spots_all_survived() {
        let results = vec![
            create_result(MutantStatus::Survived, "foo.rs", 10),
            create_result(MutantStatus::Survived, "foo.rs", 20),
            create_result(MutantStatus::Survived, "foo.rs", 30),
        ];

        let scorer = MutationScorer::new(results);
        let weak_spots = scorer.weak_spots();

        assert_eq!(weak_spots.len(), 1);
        assert_eq!(weak_spots[0].file, PathBuf::from("foo.rs"));
        assert_eq!(weak_spots[0].survived_mutants, 3);
        assert_eq!(weak_spots[0].line_range, (10, 30));
    }

    #[test]
    fn test_weak_spots_sorting_by_survivor_count() {
        let results = vec![
            create_result(MutantStatus::Survived, "low.rs", 10),
            create_result(MutantStatus::Survived, "high.rs", 20),
            create_result(MutantStatus::Survived, "high.rs", 25),
            create_result(MutantStatus::Survived, "high.rs", 30),
            create_result(MutantStatus::Survived, "medium.rs", 40),
            create_result(MutantStatus::Survived, "medium.rs", 45),
        ];

        let scorer = MutationScorer::new(results);
        let weak_spots = scorer.weak_spots();

        assert_eq!(weak_spots.len(), 3);
        // Should be sorted by survivor count (descending)
        assert_eq!(weak_spots[0].file, PathBuf::from("high.rs"));
        assert_eq!(weak_spots[0].survived_mutants, 3);
        assert_eq!(weak_spots[1].file, PathBuf::from("medium.rs"));
        assert_eq!(weak_spots[1].survived_mutants, 2);
        assert_eq!(weak_spots[2].file, PathBuf::from("low.rs"));
        assert_eq!(weak_spots[2].survived_mutants, 1);
    }

    #[test]
    fn test_generate_suggestions_basic() {
        let file = PathBuf::from("test.rs");
        let suggestions = generate_suggestions(&file, 3);

        assert_eq!(suggestions.len(), 2);
        assert!(suggestions[0].contains("3 test(s)"));
        assert!(suggestions[0].contains("test.rs"));
        assert!(suggestions[1].contains("boundary conditions"));
    }

    #[test]
    fn test_generate_suggestions_many_survivors() {
        let file = PathBuf::from("test.rs");
        let suggestions = generate_suggestions(&file, 10);

        assert_eq!(suggestions.len(), 3);
        assert!(suggestions[0].contains("10 test(s)"));
        assert!(suggestions[1].contains("property-based tests"));
        assert!(suggestions[2].contains("boundary conditions"));
    }

    #[test]
    fn test_generate_suggestions_boundary_five() {
        let file = PathBuf::from("test.rs");

        // Exactly 5 should NOT include property-based test suggestion
        let suggestions_five = generate_suggestions(&file, 5);
        assert_eq!(suggestions_five.len(), 2);
        assert!(!suggestions_five.iter().any(|s| s.contains("property-based")));

        // 6 or more SHOULD include property-based test suggestion
        let suggestions_six = generate_suggestions(&file, 6);
        assert_eq!(suggestions_six.len(), 3);
        assert!(suggestions_six.iter().any(|s| s.contains("property-based")));
    }

    #[test]
    fn test_summary_empty_results() {
        let results = vec![];
        let scorer = MutationScorer::new(results);
        let summary = scorer.summary();

        assert_eq!(summary.total_mutants, 0);
        assert_eq!(summary.killed, 0);
        assert_eq!(summary.survived, 0);
        assert_eq!(summary.mutation_score, 0.0);
        assert_eq!(summary.weak_spots.len(), 0);
    }

    #[test]
    fn test_summary_all_killed() {
        let results = vec![
            create_result(MutantStatus::Killed, "foo.rs", 10),
            create_result(MutantStatus::Killed, "bar.rs", 20),
            create_result(MutantStatus::Killed, "baz.rs", 30),
        ];

        let scorer = MutationScorer::new(results);
        let summary = scorer.summary();

        assert_eq!(summary.total_mutants, 3);
        assert_eq!(summary.killed, 3);
        assert_eq!(summary.survived, 0);
        assert_eq!(summary.mutation_score, 1.0);
        assert_eq!(summary.weak_spots.len(), 0, "Perfect score should have no weak spots");
    }

    #[test]
    fn test_summary_mixed_with_multiple_weak_spots() {
        let results = vec![
            create_result(MutantStatus::Killed, "foo.rs", 10),
            create_result(MutantStatus::Survived, "weak1.rs", 20),
            create_result(MutantStatus::Survived, "weak2.rs", 30),
            create_result(MutantStatus::Survived, "weak2.rs", 35),
            create_result(MutantStatus::CompileError, "error.rs", 40),
        ];

        let scorer = MutationScorer::new(results);
        let summary = scorer.summary();

        assert_eq!(summary.total_mutants, 5);
        assert_eq!(summary.killed, 1);
        assert_eq!(summary.survived, 3);
        assert_eq!(summary.compile_errors, 1);
        assert_eq!(summary.weak_spots.len(), 2);
        // weak2.rs should be first (2 survivors)
        assert_eq!(summary.weak_spots[0].survived_mutants, 2);
        // weak1.rs should be second (1 survivor)
        assert_eq!(summary.weak_spots[1].survived_mutants, 1);
    }
}
