#![cfg_attr(coverage_nightly, coverage(off))]
//! Category D: Statistical Rigor (15 points)
//!
//! Sound methodology separates science from pseudoscience.
//!
//! ## Sub-categories
//!
//! | ID | Name | Points | Description |
//! |----|------|--------|-------------|
//! | D1 | Sample Size Justification | 5 | Benchmarks with N runs |
//! | D2 | Error Reporting | 5 | Standard deviation/CI in results |
//! | D3 | Effect Size Documentation | 5 | Meaningful difference thresholds |
//!
//! ## Academic Foundation
//!
//! - Cohen (1988): Statistical Power Analysis [18]
//! - Georges et al. (2007): Statistical Rigor in Benchmarks [19]
//! - Kalibera & Jones (2020): Statistical Benchmarking [20]

use crate::services::popper_score::models::{PopperCategoryScore, PopperFinding, PopperSubScore};
use crate::services::popper_score::scorer::{PopperScorer, PopperScorerResult};
use regex::Regex;
use std::path::Path;

/// Scorer for Category D: Statistical Rigor (15 points)
pub struct StatisticalRigorScorer;

impl StatisticalRigorScorer {
    /// Create a new statistical rigor scorer
    pub fn new() -> Self {
        Self
    }

    /// D1: Sample Size Justification (5 points)
    ///
    /// Checks for:
    /// - Criterion.rs with multiple samples (2 points)
    /// - Documented sample size in README (2 points)
    /// - Sample size > 30 (power analysis) (1 point)
    fn score_sample_size_justification(&self, project_path: &Path) -> PopperSubScore {
        let mut earned: f64 = 0.0;
        let max: f64 = 5.0;
        let mut description = Vec::new();

        // Check benches directory for sample configuration
        let benches_dir = project_path.join("benches");
        if benches_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&benches_dir) {
                for entry in entries.flatten() {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        // Check for sample size configuration in Criterion
                        if content.contains("sample_size")
                            || content.contains("measurement_time")
                            || content.contains("warm_up_time")
                        {
                            earned += 2.0;
                            description.push("benchmark sample config found");
                            break;
                        }
                    }
                }
            }
        }

        // Check README for sample size documentation
        let readme_path = project_path.join("README.md");
        if readme_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&readme_path) {
                let content_lower = content.to_lowercase();

                // Look for sample size mentions
                if content_lower.contains("sample")
                    || content_lower.contains("iteration")
                    || content_lower.contains("runs")
                    || content_lower.contains("trials")
                {
                    earned += 2.0;
                    description.push("sample documentation found");
                }

                // Check for n=30+ (power analysis standard)
                let sample_regex = Regex::new(r"(?i)n\s*[=:]\s*(\d+)").expect("internal error");
                if let Some(caps) = sample_regex.captures(&content) {
                    if let Ok(n) = caps.get(1).expect("internal error").as_str().parse::<u32>() {
                        if n >= 30 {
                            earned += 1.0;
                            description.push("adequate sample size (n≥30)");
                        }
                    }
                }
            }
        }

        if description.is_empty() {
            description.push("no sample size justification");
        }

        PopperSubScore::new(
            "D1",
            "Sample Size Justification",
            earned.min(max),
            max,
            &description.join(", "),
        )
    }

    /// D2: Error Reporting (5 points)
    ///
    /// Checks for:
    /// - Standard deviation in results (2 points)
    /// - Confidence intervals (2 points)
    /// - Error bars in documentation (1 point)
    fn score_error_reporting(&self, project_path: &Path) -> PopperSubScore {
        let mut earned: f64 = 0.0;
        let max: f64 = 5.0;
        let mut description = Vec::new();

        // Check README for statistical reporting
        let readme_path = project_path.join("README.md");
        if readme_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&readme_path) {
                let content_lower = content.to_lowercase();

                // Check for standard deviation
                if content.contains("σ")
                    || content_lower.contains("std dev")
                    || content_lower.contains("standard deviation")
                    || content.contains("±")
                {
                    earned += 2.0;
                    description.push("standard deviation reported");
                }

                // Check for confidence intervals
                if content_lower.contains("95%")
                    || content_lower.contains("99%")
                    || content_lower.contains("confidence interval")
                    || content_lower.contains("ci")
                    || content.contains("[")
                        && content.contains("]")
                        && content.contains(",")
                        && content.contains(".")
                {
                    earned += 2.0;
                    description.push("confidence intervals found");
                }

                // Check for error bars mention
                if content_lower.contains("error bar")
                    || content_lower.contains("margin of error")
                    || content_lower.contains("variance")
                {
                    earned += 1.0;
                    description.push("error metrics documented");
                }
            }
        }

        // Check for Criterion output files (which include statistical data)
        let criterion_dir = project_path.join("target/criterion");
        if criterion_dir.exists() {
            earned = (earned + 1.0).min(max);
            description.push("Criterion benchmark data exists");
        }

        if description.is_empty() {
            description.push("no error reporting found");
        }

        PopperSubScore::new(
            "D2",
            "Error Reporting",
            earned.min(max),
            max,
            &description.join(", "),
        )
    }

    /// D3: Effect Size Documentation (5 points)
    ///
    /// Checks for:
    /// - Documented performance claims with thresholds (2 points)
    /// - Comparison baselines (2 points)
    /// - Effect size interpretation (Cohen's d, etc.) (1 point)
    fn score_effect_size_documentation(&self, project_path: &Path) -> PopperSubScore {
        let mut earned: f64 = 0.0;
        let max: f64 = 5.0;
        let mut description = Vec::new();

        let readme_path = project_path.join("README.md");
        if readme_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&readme_path) {
                let content_lower = content.to_lowercase();

                // Check for performance thresholds (2 points)
                let threshold_regex = Regex::new(
                    r"(?i)(\d+x\s*faster|\d+%\s*(faster|slower|improvement)|\d+ms|\d+μs)",
                )
                .expect("internal error");
                if threshold_regex.is_match(&content) {
                    earned += 2.0;
                    description.push("performance thresholds documented");
                }

                // Check for comparison baselines (2 points)
                if content_lower.contains("baseline")
                    || content_lower.contains("compared to")
                    || content_lower.contains("versus")
                    || content_lower.contains("vs.")
                    || content_lower.contains("benchmark")
                {
                    earned += 2.0;
                    description.push("comparison baselines found");
                }

                // Check for effect size interpretation (1 point)
                if content_lower.contains("cohen")
                    || content_lower.contains("effect size")
                    || content_lower.contains("practical significance")
                    || content_lower.contains("meaningful difference")
                {
                    earned += 1.0;
                    description.push("effect size interpretation");
                }
            }
        }

        if description.is_empty() {
            description.push("no effect size documentation");
        }

        PopperSubScore::new(
            "D3",
            "Effect Size Documentation",
            earned.min(max),
            max,
            &description.join(", "),
        )
    }
}

impl Default for StatisticalRigorScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl PopperScorer for StatisticalRigorScorer {
    fn name(&self) -> &str {
        "Statistical Rigor"
    }

    fn category_id(&self) -> char {
        'D'
    }

    fn max_points(&self) -> f64 {
        15.0
    }

    fn score(&self, project_path: &Path) -> PopperScorerResult<PopperCategoryScore> {
        let mut category = PopperCategoryScore::new(self.name(), 0.0, self.max_points());

        // Score each sub-category
        let d1 = self.score_sample_size_justification(project_path);
        let d2 = self.score_error_reporting(project_path);
        let d3 = self.score_effect_size_documentation(project_path);

        // Add findings based on scores
        if d1.earned < 3.0 {
            category.add_finding(PopperFinding::warning(
                "Sample size justification missing - document benchmark sample sizes",
                5.0 - d1.earned,
            ));
        }

        if d2.earned < 3.0 {
            category.add_finding(PopperFinding::warning(
                "Error reporting incomplete - add confidence intervals to benchmark results",
                5.0 - d2.earned,
            ));
        }

        if d3.earned < 3.0 {
            category.add_finding(PopperFinding::warning(
                "Effect size documentation missing - clarify what performance differences are meaningful",
                5.0 - d3.earned,
            ));
        }

        if d1.earned + d2.earned + d3.earned >= 12.0 {
            category.add_finding(PopperFinding::positive(
                "Excellent statistical rigor in methodology",
            ));
        }

        // Add sub-scores
        category.add_sub_score(d1);
        category.add_sub_score(d2);
        category.add_sub_score(d3);

        Ok(category)
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_statistical_rigor_scorer_basics() {
        let scorer = StatisticalRigorScorer::new();
        assert_eq!(scorer.name(), "Statistical Rigor");
        assert_eq!(scorer.category_id(), 'D');
        assert_eq!(scorer.max_points(), 15.0);
        assert!(!scorer.is_gateway());
    }

    #[test]
    fn test_project_with_criterion_benchmarks() {
        let temp_dir = tempdir().expect("internal error");

        // Create benches with sample size config
        fs::create_dir_all(temp_dir.path().join("benches")).expect("internal error");
        fs::write(
            temp_dir.path().join("benches/bench.rs"),
            r#"
            use criterion::*;

            fn bench(c: &mut Criterion) {
                c.bench_function("test", |b| b.iter(|| 42))
                    .sample_size(100)
                    .measurement_time(Duration::from_secs(10));
            }
            "#,
        )
        .expect("internal error");

        let scorer = StatisticalRigorScorer::new();
        let result = scorer.score(temp_dir.path()).expect("internal error");

        // Should have sample size points
        let d1 = result
            .sub_scores
            .iter()
            .find(|s| s.id == "D1")
            .expect("internal error");
        assert!(d1.earned >= 2.0);
    }

    #[test]
    fn test_project_with_statistical_readme() {
        let temp_dir = tempdir().expect("internal error");

        // Create README with statistical reporting
        fs::write(
            temp_dir.path().join("README.md"),
            r#"# Performance

## Benchmarks

Results (n=100 runs):
- Operation A: 15.2ms ± 1.3ms (95% confidence interval)
- Operation B: 8.7ms ± 0.8ms

This represents a 2x faster improvement compared to the baseline implementation.
"#,
        )
        .expect("internal error");

        let scorer = StatisticalRigorScorer::new();
        let result = scorer.score(temp_dir.path()).expect("internal error");

        // Should have multiple points
        assert!(result.earned > 5.0);
    }

    #[test]
    fn test_empty_project_low_score() {
        let temp_dir = tempdir().expect("internal error");

        let scorer = StatisticalRigorScorer::new();
        let result = scorer.score(temp_dir.path()).expect("internal error");

        // Should have low/zero score
        assert!(result.earned < 5.0);
    }
}
