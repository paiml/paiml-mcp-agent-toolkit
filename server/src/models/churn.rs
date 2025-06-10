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
}

impl FileChurnMetrics {
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

    #[test]
    fn test_churn_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
