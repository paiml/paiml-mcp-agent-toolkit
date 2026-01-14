use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChurnAnalysis {
    pub generated_at: DateTime<Utc>,
    pub period_days: u32,
    pub repository_root: PathBuf,
    pub files: Vec<FileChurnMetrics>,
    pub summary: ChurnSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChurnMetrics {
    pub path: PathBuf,
    pub relative_path: String,
    pub commit_count: usize,
    pub unique_authors: Vec<String>,
    pub additions: usize,
    pub deletions: usize,
    pub churn_score: f32,
    pub last_modified: DateTime<Utc>,
    pub first_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChurnSummary {
    pub total_commits: usize,
    pub total_files_changed: usize,
    pub hotspot_files: Vec<PathBuf>,
    pub stable_files: Vec<PathBuf>,
    pub author_contributions: HashMap<String, usize>,
    /// Mean of churn scores across all files
    pub mean_churn_score: f64,
    /// Variance of churn scores (population variance)
    pub variance_churn_score: f64,
    /// Standard deviation of churn scores
    pub stddev_churn_score: f64,
}

impl FileChurnMetrics {
    /// Calculates a normalized churn score based on commits and changes
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::churn::FileChurnMetrics;
    /// use std::path::PathBuf;
    /// use chrono::Utc;
    ///
    /// let mut metrics = FileChurnMetrics {
    ///     path: PathBuf::from("src/main.rs"),
    ///     relative_path: "src/main.rs".to_string(),
    ///     commit_count: 10,
    ///     unique_authors: vec![],
    ///     additions: 200,
    ///     deletions: 100,
    ///     churn_score: 0.0,
    ///     last_modified: Utc::now(),
    ///     first_seen: Utc::now(),
    /// };
    ///
    /// metrics.calculate_churn_score(20, 600);
    /// assert!(metrics.churn_score > 0.0 && metrics.churn_score <= 1.0);
    /// ```
    pub fn calculate_churn_score(&mut self, max_commits: usize, max_changes: usize) {
        let commit_factor = if max_commits > 0 {
            self.commit_count as f32 / max_commits as f32
        } else {
            0.0
        };

        let change_factor = if max_changes > 0 {
            (self.additions + self.deletions) as f32 / max_changes as f32
        } else {
            0.0
        };

        self.churn_score = (commit_factor * 0.6 + change_factor * 0.4).min(1.0);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
pub enum ChurnOutputFormat {
    Json,
    Markdown,
    Csv,
    Summary,
}

impl std::str::FromStr for ChurnOutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(ChurnOutputFormat::Json),
            "markdown" => Ok(ChurnOutputFormat::Markdown),
            "csv" => Ok(ChurnOutputFormat::Csv),
            "summary" => Ok(ChurnOutputFormat::Summary),
            _ => Err(format!("Invalid output format: {s}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_file_churn_metrics_calculate_score() {
        let mut metrics = FileChurnMetrics {
            path: PathBuf::from("test.rs"),
            relative_path: "test.rs".to_string(),
            commit_count: 10,
            unique_authors: vec!["author1".to_string()],
            additions: 100,
            deletions: 50,
            churn_score: 0.0,
            last_modified: Utc::now(),
            first_seen: Utc::now(),
        };

        metrics.calculate_churn_score(20, 300);
        assert!(metrics.churn_score > 0.0);
        assert!(metrics.churn_score <= 1.0);

        // Test with max values
        metrics.commit_count = 20;
        metrics.additions = 150;
        metrics.deletions = 150;
        metrics.calculate_churn_score(20, 300);
        assert_eq!(metrics.churn_score, 1.0);
    }

    #[test]
    fn test_file_churn_metrics_zero_max() {
        let mut metrics = FileChurnMetrics {
            path: PathBuf::from("test.rs"),
            relative_path: "test.rs".to_string(),
            commit_count: 10,
            unique_authors: vec![],
            additions: 100,
            deletions: 50,
            churn_score: 0.0,
            last_modified: Utc::now(),
            first_seen: Utc::now(),
        };

        metrics.calculate_churn_score(0, 0);
        assert_eq!(metrics.churn_score, 0.0);
    }

    #[test]
    fn test_churn_output_format_from_str() {
        assert_eq!(
            ChurnOutputFormat::from_str("json").unwrap(),
            ChurnOutputFormat::Json
        );
        assert_eq!(
            ChurnOutputFormat::from_str("JSON").unwrap(),
            ChurnOutputFormat::Json
        );
        assert_eq!(
            ChurnOutputFormat::from_str("markdown").unwrap(),
            ChurnOutputFormat::Markdown
        );
        assert_eq!(
            ChurnOutputFormat::from_str("csv").unwrap(),
            ChurnOutputFormat::Csv
        );
        assert_eq!(
            ChurnOutputFormat::from_str("summary").unwrap(),
            ChurnOutputFormat::Summary
        );

        assert!(ChurnOutputFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_code_churn_analysis_creation() {
        let analysis = CodeChurnAnalysis {
            generated_at: Utc::now(),
            period_days: 30,
            repository_root: PathBuf::from("/test/repo"),
            files: vec![],
            summary: ChurnSummary {
                total_commits: 100,
                total_files_changed: 50,
                hotspot_files: vec![],
                stable_files: vec![],
                author_contributions: HashMap::new(),
                mean_churn_score: 0.0,
                variance_churn_score: 0.0,
                stddev_churn_score: 0.0,
            },
        };

        assert_eq!(analysis.period_days, 30);
        assert_eq!(analysis.summary.total_commits, 100);
        assert_eq!(analysis.summary.total_files_changed, 50);
    }

    #[test]
    fn test_churn_summary_with_data() {
        let mut author_contributions = HashMap::new();
        author_contributions.insert("author1".to_string(), 50);
        author_contributions.insert("author2".to_string(), 30);

        let summary = ChurnSummary {
            total_commits: 80,
            total_files_changed: 25,
            hotspot_files: vec![PathBuf::from("hot1.rs"), PathBuf::from("hot2.rs")],
            stable_files: vec![PathBuf::from("stable1.rs")],
            author_contributions,
            mean_churn_score: 0.0,
            variance_churn_score: 0.0,
            stddev_churn_score: 0.0,
        };

        assert_eq!(summary.total_commits, 80);
        assert_eq!(summary.hotspot_files.len(), 2);
        assert_eq!(summary.stable_files.len(), 1);
        assert_eq!(summary.author_contributions.get("author1"), Some(&50));
    }

    #[test]
    fn test_serialization() {
        let metrics = FileChurnMetrics {
            path: PathBuf::from("test.rs"),
            relative_path: "test.rs".to_string(),
            commit_count: 5,
            unique_authors: vec!["dev".to_string()],
            additions: 50,
            deletions: 20,
            churn_score: 0.5,
            last_modified: Utc::now(),
            first_seen: Utc::now(),
        };

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: FileChurnMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.commit_count, metrics.commit_count);
        assert_eq!(deserialized.churn_score, metrics.churn_score);
    }
}

#[cfg(test)]
mod coverage_tests {
    //! EXTREME TDD coverage tests for churn analysis module
    //!
    //! These tests exercise all edge cases, boundary conditions, and
    //! ensure comprehensive coverage of the churn analysis data structures.

    use super::*;
    use chrono::{TimeZone, Utc};
    use std::str::FromStr;

    // ============================================================================
    // FileChurnMetrics Tests
    // ============================================================================

    mod file_churn_metrics_tests {
        use super::*;

        fn create_default_metrics() -> FileChurnMetrics {
            FileChurnMetrics {
                path: PathBuf::from("src/lib.rs"),
                relative_path: "src/lib.rs".to_string(),
                commit_count: 0,
                unique_authors: vec![],
                additions: 0,
                deletions: 0,
                churn_score: 0.0,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            }
        }

        #[test]
        fn test_calculate_churn_score_zero_everything() {
            let mut metrics = create_default_metrics();
            metrics.calculate_churn_score(0, 0);
            assert_eq!(metrics.churn_score, 0.0);
        }

        #[test]
        fn test_calculate_churn_score_zero_max_commits_only() {
            let mut metrics = create_default_metrics();
            metrics.additions = 100;
            metrics.deletions = 50;
            metrics.calculate_churn_score(0, 300);
            // With zero max_commits, commit_factor is 0.0
            // change_factor = 150/300 = 0.5
            // score = 0.0 * 0.6 + 0.5 * 0.4 = 0.2
            assert!((metrics.churn_score - 0.2).abs() < 0.001);
        }

        #[test]
        fn test_calculate_churn_score_zero_max_changes_only() {
            let mut metrics = create_default_metrics();
            metrics.commit_count = 10;
            metrics.calculate_churn_score(20, 0);
            // commit_factor = 10/20 = 0.5
            // change_factor = 0.0 (max_changes is 0)
            // score = 0.5 * 0.6 + 0.0 * 0.4 = 0.3
            assert!((metrics.churn_score - 0.3).abs() < 0.001);
        }

        #[test]
        fn test_calculate_churn_score_exact_max() {
            let mut metrics = create_default_metrics();
            metrics.commit_count = 100;
            metrics.additions = 500;
            metrics.deletions = 500;
            metrics.calculate_churn_score(100, 1000);
            // commit_factor = 1.0, change_factor = 1.0
            // score = 1.0 * 0.6 + 1.0 * 0.4 = 1.0
            assert_eq!(metrics.churn_score, 1.0);
        }

        #[test]
        fn test_calculate_churn_score_exceeds_max_capped() {
            let mut metrics = create_default_metrics();
            metrics.commit_count = 200; // Exceeds max
            metrics.additions = 2000;
            metrics.deletions = 2000;
            metrics.calculate_churn_score(100, 1000);
            // commit_factor = 200/100 = 2.0
            // change_factor = 4000/1000 = 4.0
            // raw_score = 2.0 * 0.6 + 4.0 * 0.4 = 1.2 + 1.6 = 2.8
            // capped to 1.0
            assert_eq!(metrics.churn_score, 1.0);
        }

        #[test]
        fn test_calculate_churn_score_weighting_60_40() {
            let mut metrics = create_default_metrics();
            // Set up so commit_factor = 1.0 and change_factor = 0.0
            metrics.commit_count = 50;
            metrics.additions = 0;
            metrics.deletions = 0;
            metrics.calculate_churn_score(50, 100);
            // score = 1.0 * 0.6 + 0.0 * 0.4 = 0.6
            assert!((metrics.churn_score - 0.6).abs() < 0.001);

            // Now opposite: commit_factor = 0.0, change_factor = 1.0
            metrics.commit_count = 0;
            metrics.additions = 50;
            metrics.deletions = 50;
            metrics.calculate_churn_score(50, 100);
            // score = 0.0 * 0.6 + 1.0 * 0.4 = 0.4
            assert!((metrics.churn_score - 0.4).abs() < 0.001);
        }

        #[test]
        fn test_calculate_churn_score_fractional_values() {
            let mut metrics = create_default_metrics();
            metrics.commit_count = 7;
            metrics.additions = 33;
            metrics.deletions = 17;
            metrics.calculate_churn_score(14, 100);
            // commit_factor = 7/14 = 0.5
            // change_factor = 50/100 = 0.5
            // score = 0.5 * 0.6 + 0.5 * 0.4 = 0.3 + 0.2 = 0.5
            assert!((metrics.churn_score - 0.5).abs() < 0.001);
        }

        #[test]
        fn test_file_churn_metrics_with_multiple_authors() {
            let metrics = FileChurnMetrics {
                path: PathBuf::from("src/important.rs"),
                relative_path: "src/important.rs".to_string(),
                commit_count: 50,
                unique_authors: vec![
                    "alice".to_string(),
                    "bob".to_string(),
                    "charlie".to_string(),
                ],
                additions: 1000,
                deletions: 200,
                churn_score: 0.8,
                last_modified: Utc::now(),
                first_seen: Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
            };

            assert_eq!(metrics.unique_authors.len(), 3);
            assert!(metrics.unique_authors.contains(&"alice".to_string()));
        }

        #[test]
        fn test_file_churn_metrics_serialization_full() {
            let now = Utc::now();
            let metrics = FileChurnMetrics {
                path: PathBuf::from("complex/path/to/file.rs"),
                relative_path: "complex/path/to/file.rs".to_string(),
                commit_count: 42,
                unique_authors: vec!["dev1".to_string(), "dev2".to_string()],
                additions: 500,
                deletions: 250,
                churn_score: 0.75,
                last_modified: now,
                first_seen: now,
            };

            let json = serde_json::to_string(&metrics).unwrap();
            let deserialized: FileChurnMetrics = serde_json::from_str(&json).unwrap();

            assert_eq!(deserialized.path, metrics.path);
            assert_eq!(deserialized.relative_path, metrics.relative_path);
            assert_eq!(deserialized.commit_count, metrics.commit_count);
            assert_eq!(deserialized.unique_authors.len(), 2);
            assert_eq!(deserialized.additions, metrics.additions);
            assert_eq!(deserialized.deletions, metrics.deletions);
            assert!((deserialized.churn_score - metrics.churn_score).abs() < 0.0001);
        }

        #[test]
        fn test_file_churn_metrics_clone() {
            let metrics = create_default_metrics();
            let cloned = metrics.clone();

            assert_eq!(cloned.path, metrics.path);
            assert_eq!(cloned.relative_path, metrics.relative_path);
            assert_eq!(cloned.churn_score, metrics.churn_score);
        }
    }

    // ============================================================================
    // ChurnSummary Tests
    // ============================================================================

    mod churn_summary_tests {
        use super::*;

        #[test]
        fn test_churn_summary_empty() {
            let summary = ChurnSummary {
                total_commits: 0,
                total_files_changed: 0,
                hotspot_files: vec![],
                stable_files: vec![],
                author_contributions: HashMap::new(),
                mean_churn_score: 0.0,
                variance_churn_score: 0.0,
                stddev_churn_score: 0.0,
            };

            assert_eq!(summary.total_commits, 0);
            assert!(summary.hotspot_files.is_empty());
            assert!(summary.author_contributions.is_empty());
        }

        #[test]
        fn test_churn_summary_with_hotspots() {
            let summary = ChurnSummary {
                total_commits: 150,
                total_files_changed: 45,
                hotspot_files: vec![
                    PathBuf::from("src/main.rs"),
                    PathBuf::from("src/lib.rs"),
                    PathBuf::from("src/core.rs"),
                ],
                stable_files: vec![PathBuf::from("src/utils.rs")],
                author_contributions: HashMap::new(),
                mean_churn_score: 0.65,
                variance_churn_score: 0.04,
                stddev_churn_score: 0.2,
            };

            assert_eq!(summary.hotspot_files.len(), 3);
            assert_eq!(summary.stable_files.len(), 1);
            assert!((summary.mean_churn_score - 0.65).abs() < 0.001);
        }

        #[test]
        fn test_churn_summary_author_contributions() {
            let mut contributions = HashMap::new();
            contributions.insert("lead_dev".to_string(), 100);
            contributions.insert("junior_dev".to_string(), 25);
            contributions.insert("reviewer".to_string(), 10);

            let summary = ChurnSummary {
                total_commits: 135,
                total_files_changed: 80,
                hotspot_files: vec![],
                stable_files: vec![],
                author_contributions: contributions,
                mean_churn_score: 0.5,
                variance_churn_score: 0.1,
                stddev_churn_score: 0.316,
            };

            assert_eq!(summary.author_contributions.len(), 3);
            assert_eq!(summary.author_contributions.get("lead_dev"), Some(&100));
            assert_eq!(summary.author_contributions.get("junior_dev"), Some(&25));
        }

        #[test]
        fn test_churn_summary_statistics() {
            let summary = ChurnSummary {
                total_commits: 200,
                total_files_changed: 100,
                hotspot_files: vec![],
                stable_files: vec![],
                author_contributions: HashMap::new(),
                mean_churn_score: 0.45,
                variance_churn_score: 0.0225, // 0.15^2
                stddev_churn_score: 0.15,
            };

            // Verify statistical relationship
            let calculated_stddev = summary.variance_churn_score.sqrt();
            assert!((calculated_stddev - summary.stddev_churn_score).abs() < 0.001);
        }

        #[test]
        fn test_churn_summary_serialization() {
            let mut contributions = HashMap::new();
            contributions.insert("dev".to_string(), 50);

            let summary = ChurnSummary {
                total_commits: 100,
                total_files_changed: 30,
                hotspot_files: vec![PathBuf::from("hot.rs")],
                stable_files: vec![PathBuf::from("stable.rs")],
                author_contributions: contributions,
                mean_churn_score: 0.6,
                variance_churn_score: 0.04,
                stddev_churn_score: 0.2,
            };

            let json = serde_json::to_string(&summary).unwrap();
            let deserialized: ChurnSummary = serde_json::from_str(&json).unwrap();

            assert_eq!(deserialized.total_commits, summary.total_commits);
            assert_eq!(deserialized.hotspot_files.len(), 1);
            assert_eq!(deserialized.author_contributions.get("dev"), Some(&50));
        }
    }

    // ============================================================================
    // CodeChurnAnalysis Tests
    // ============================================================================

    mod code_churn_analysis_tests {
        use super::*;

        fn create_test_analysis() -> CodeChurnAnalysis {
            CodeChurnAnalysis {
                generated_at: Utc::now(),
                period_days: 30,
                repository_root: PathBuf::from("/project"),
                files: vec![],
                summary: ChurnSummary {
                    total_commits: 0,
                    total_files_changed: 0,
                    hotspot_files: vec![],
                    stable_files: vec![],
                    author_contributions: HashMap::new(),
                    mean_churn_score: 0.0,
                    variance_churn_score: 0.0,
                    stddev_churn_score: 0.0,
                },
            }
        }

        #[test]
        fn test_code_churn_analysis_period_days() {
            let mut analysis = create_test_analysis();

            // Test various period days
            analysis.period_days = 7;
            assert_eq!(analysis.period_days, 7);

            analysis.period_days = 90;
            assert_eq!(analysis.period_days, 90);

            analysis.period_days = 365;
            assert_eq!(analysis.period_days, 365);
        }

        #[test]
        fn test_code_churn_analysis_with_files() {
            let now = Utc::now();
            let files = vec![
                FileChurnMetrics {
                    path: PathBuf::from("src/main.rs"),
                    relative_path: "src/main.rs".to_string(),
                    commit_count: 10,
                    unique_authors: vec!["dev".to_string()],
                    additions: 100,
                    deletions: 50,
                    churn_score: 0.7,
                    last_modified: now,
                    first_seen: now,
                },
                FileChurnMetrics {
                    path: PathBuf::from("src/lib.rs"),
                    relative_path: "src/lib.rs".to_string(),
                    commit_count: 5,
                    unique_authors: vec!["dev".to_string()],
                    additions: 50,
                    deletions: 20,
                    churn_score: 0.4,
                    last_modified: now,
                    first_seen: now,
                },
            ];

            let analysis = CodeChurnAnalysis {
                generated_at: now,
                period_days: 30,
                repository_root: PathBuf::from("/project"),
                files,
                summary: ChurnSummary {
                    total_commits: 15,
                    total_files_changed: 2,
                    hotspot_files: vec![PathBuf::from("src/main.rs")],
                    stable_files: vec![],
                    author_contributions: HashMap::new(),
                    mean_churn_score: 0.55,
                    variance_churn_score: 0.0225,
                    stddev_churn_score: 0.15,
                },
            };

            assert_eq!(analysis.files.len(), 2);
            assert_eq!(analysis.summary.total_commits, 15);
        }

        #[test]
        fn test_code_churn_analysis_serialization() {
            let now = Utc::now();
            let mut contributions = HashMap::new();
            contributions.insert("alice".to_string(), 30);

            let analysis = CodeChurnAnalysis {
                generated_at: now,
                period_days: 14,
                repository_root: PathBuf::from("/test/project"),
                files: vec![FileChurnMetrics {
                    path: PathBuf::from("test.rs"),
                    relative_path: "test.rs".to_string(),
                    commit_count: 5,
                    unique_authors: vec!["alice".to_string()],
                    additions: 25,
                    deletions: 10,
                    churn_score: 0.35,
                    last_modified: now,
                    first_seen: now,
                }],
                summary: ChurnSummary {
                    total_commits: 5,
                    total_files_changed: 1,
                    hotspot_files: vec![],
                    stable_files: vec![PathBuf::from("test.rs")],
                    author_contributions: contributions,
                    mean_churn_score: 0.35,
                    variance_churn_score: 0.0,
                    stddev_churn_score: 0.0,
                },
            };

            let json = serde_json::to_string(&analysis).unwrap();
            let deserialized: CodeChurnAnalysis = serde_json::from_str(&json).unwrap();

            assert_eq!(deserialized.period_days, 14);
            assert_eq!(deserialized.files.len(), 1);
            assert_eq!(deserialized.summary.total_commits, 5);
        }
    }

    // ============================================================================
    // ChurnOutputFormat Tests
    // ============================================================================

    mod churn_output_format_tests {
        use super::*;

        #[test]
        fn test_from_str_lowercase() {
            assert_eq!(
                ChurnOutputFormat::from_str("json").unwrap(),
                ChurnOutputFormat::Json
            );
            assert_eq!(
                ChurnOutputFormat::from_str("markdown").unwrap(),
                ChurnOutputFormat::Markdown
            );
            assert_eq!(
                ChurnOutputFormat::from_str("csv").unwrap(),
                ChurnOutputFormat::Csv
            );
            assert_eq!(
                ChurnOutputFormat::from_str("summary").unwrap(),
                ChurnOutputFormat::Summary
            );
        }

        #[test]
        fn test_from_str_uppercase() {
            assert_eq!(
                ChurnOutputFormat::from_str("JSON").unwrap(),
                ChurnOutputFormat::Json
            );
            assert_eq!(
                ChurnOutputFormat::from_str("MARKDOWN").unwrap(),
                ChurnOutputFormat::Markdown
            );
            assert_eq!(
                ChurnOutputFormat::from_str("CSV").unwrap(),
                ChurnOutputFormat::Csv
            );
            assert_eq!(
                ChurnOutputFormat::from_str("SUMMARY").unwrap(),
                ChurnOutputFormat::Summary
            );
        }

        #[test]
        fn test_from_str_mixed_case() {
            assert_eq!(
                ChurnOutputFormat::from_str("Json").unwrap(),
                ChurnOutputFormat::Json
            );
            assert_eq!(
                ChurnOutputFormat::from_str("MarkDown").unwrap(),
                ChurnOutputFormat::Markdown
            );
            assert_eq!(
                ChurnOutputFormat::from_str("CsV").unwrap(),
                ChurnOutputFormat::Csv
            );
            assert_eq!(
                ChurnOutputFormat::from_str("SuMmArY").unwrap(),
                ChurnOutputFormat::Summary
            );
        }

        #[test]
        fn test_from_str_invalid() {
            let result = ChurnOutputFormat::from_str("invalid");
            assert!(result.is_err());

            let err = result.err().unwrap();
            assert!(err.contains("Invalid output format"));
            assert!(err.contains("invalid"));
        }

        #[test]
        fn test_from_str_empty() {
            let result = ChurnOutputFormat::from_str("");
            assert!(result.is_err());
        }

        #[test]
        fn test_from_str_whitespace() {
            // Should fail because we don't trim whitespace
            let result = ChurnOutputFormat::from_str(" json ");
            assert!(result.is_err());
        }

        #[test]
        fn test_churn_output_format_equality() {
            assert_eq!(ChurnOutputFormat::Json, ChurnOutputFormat::Json);
            assert_ne!(ChurnOutputFormat::Json, ChurnOutputFormat::Csv);
            assert_ne!(ChurnOutputFormat::Markdown, ChurnOutputFormat::Summary);
        }

        #[test]
        fn test_churn_output_format_clone() {
            let format = ChurnOutputFormat::Markdown;
            let cloned = format.clone();
            assert_eq!(format, cloned);
        }

        #[test]
        fn test_churn_output_format_debug() {
            let format = ChurnOutputFormat::Json;
            let debug_str = format!("{:?}", format);
            assert!(debug_str.contains("Json"));
        }
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::str::FromStr;

    proptest! {
        #[test]
        fn prop_churn_score_always_in_range(
            commit_count in 0usize..1000,
            additions in 0usize..10000,
            deletions in 0usize..10000,
            max_commits in 1usize..500,  // Avoid division by zero
            max_changes in 1usize..20000
        ) {
            let mut metrics = FileChurnMetrics {
                path: PathBuf::from("test.rs"),
                relative_path: "test.rs".to_string(),
                commit_count,
                unique_authors: vec![],
                additions,
                deletions,
                churn_score: 0.0,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            };

            metrics.calculate_churn_score(max_commits, max_changes);

            prop_assert!(metrics.churn_score >= 0.0, "Churn score should be >= 0.0");
            prop_assert!(metrics.churn_score <= 1.0, "Churn score should be <= 1.0");
        }

        #[test]
        fn prop_churn_score_monotonic_with_commits(
            base_commits in 0usize..100,
            extra_commits in 1usize..100,
            additions in 0usize..500,
            deletions in 0usize..500
        ) {
            let max_commits = 200usize;
            let max_changes = 1000usize;

            let mut metrics_low = FileChurnMetrics {
                path: PathBuf::from("test.rs"),
                relative_path: "test.rs".to_string(),
                commit_count: base_commits,
                unique_authors: vec![],
                additions,
                deletions,
                churn_score: 0.0,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            };

            let mut metrics_high = FileChurnMetrics {
                path: PathBuf::from("test.rs"),
                relative_path: "test.rs".to_string(),
                commit_count: base_commits + extra_commits,
                unique_authors: vec![],
                additions,
                deletions,
                churn_score: 0.0,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            };

            metrics_low.calculate_churn_score(max_commits, max_changes);
            metrics_high.calculate_churn_score(max_commits, max_changes);

            // More commits should mean higher or equal score (monotonic)
            prop_assert!(
                metrics_high.churn_score >= metrics_low.churn_score,
                "Score should increase with more commits: {} >= {}",
                metrics_high.churn_score,
                metrics_low.churn_score
            );
        }

        #[test]
        fn prop_file_churn_metrics_serialization_roundtrip(
            path_suffix in "[a-z]{1,10}",
            commit_count in 0usize..1000,
            additions in 0usize..10000,
            deletions in 0usize..10000
        ) {
            let path = format!("src/{}.rs", path_suffix);
            let metrics = FileChurnMetrics {
                path: PathBuf::from(&path),
                relative_path: path.clone(),
                commit_count,
                unique_authors: vec!["test_author".to_string()],
                additions,
                deletions,
                churn_score: 0.5,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            };

            let json = serde_json::to_string(&metrics).expect("Serialization failed");
            let deserialized: FileChurnMetrics = serde_json::from_str(&json).expect("Deserialization failed");

            prop_assert_eq!(deserialized.relative_path, path);
            prop_assert_eq!(deserialized.commit_count, commit_count);
            prop_assert_eq!(deserialized.additions, additions);
            prop_assert_eq!(deserialized.deletions, deletions);
        }

        #[test]
        fn prop_churn_output_format_case_insensitive(
            format_name in prop_oneof![
                Just("json"),
                Just("JSON"),
                Just("Json"),
                Just("jSoN"),
                Just("markdown"),
                Just("MARKDOWN"),
                Just("Markdown"),
                Just("csv"),
                Just("CSV"),
                Just("Csv"),
                Just("summary"),
                Just("SUMMARY"),
                Just("Summary")
            ]
        ) {
            let result = ChurnOutputFormat::from_str(&format_name);
            prop_assert!(result.is_ok(), "Format '{}' should parse successfully", format_name);
        }

        #[test]
        fn prop_author_contributions_preserved(
            author_count in 0usize..10,
            contribution in 1usize..100
        ) {
            let mut contributions = HashMap::new();
            for i in 0..author_count {
                contributions.insert(format!("author_{}", i), contribution + i);
            }

            let summary = ChurnSummary {
                total_commits: 100,
                total_files_changed: 50,
                hotspot_files: vec![],
                stable_files: vec![],
                author_contributions: contributions.clone(),
                mean_churn_score: 0.5,
                variance_churn_score: 0.1,
                stddev_churn_score: 0.316,
            };

            let json = serde_json::to_string(&summary).expect("Serialization failed");
            let deserialized: ChurnSummary = serde_json::from_str(&json).expect("Deserialization failed");

            prop_assert_eq!(deserialized.author_contributions.len(), author_count);

            for (author, count) in contributions.iter() {
                prop_assert_eq!(
                    deserialized.author_contributions.get(author),
                    Some(count),
                    "Author {} contribution should be preserved", author
                );
            }
        }

        #[test]
        fn prop_churn_summary_statistics_non_negative(
            mean in 0.0f64..1.0,
            variance in 0.0f64..1.0
        ) {
            let summary = ChurnSummary {
                total_commits: 100,
                total_files_changed: 50,
                hotspot_files: vec![],
                stable_files: vec![],
                author_contributions: HashMap::new(),
                mean_churn_score: mean,
                variance_churn_score: variance,
                stddev_churn_score: variance.sqrt(),
            };

            prop_assert!(summary.mean_churn_score >= 0.0);
            prop_assert!(summary.variance_churn_score >= 0.0);
            prop_assert!(summary.stddev_churn_score >= 0.0);
        }
    }
}
