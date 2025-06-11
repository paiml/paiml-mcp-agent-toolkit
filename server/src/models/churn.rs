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
