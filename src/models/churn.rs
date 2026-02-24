#![cfg_attr(coverage_nightly, coverage(off))]
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    include!("churn_tests.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    #[allow(unused_imports)]
    use std::str::FromStr;
    include!("churn_coverage_tests.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests_analysis {
    use super::*;
    #[allow(unused_imports)]
    use chrono::Utc;
    use std::str::FromStr;
    include!("churn_coverage_tests_analysis.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use std::str::FromStr;
    include!("churn_property_tests.rs");
}
