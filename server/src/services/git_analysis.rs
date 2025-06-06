use crate::models::churn::{ChurnSummary, CodeChurnAnalysis, FileChurnMetrics};
use crate::models::error::TemplateError;
use chrono::{DateTime, Duration, Utc};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;

pub struct GitAnalysisService;

impl GitAnalysisService {
    pub fn analyze_code_churn(
        project_path: &Path,
        period_days: u32,
    ) -> Result<CodeChurnAnalysis, TemplateError> {
        if !project_path.join(".git").exists() {
            return Err(TemplateError::NotFound(format!(
                "No git repository found at {project_path:?}"
            )));
        }

        let since_date = Utc::now() - Duration::days(period_days as i64);
        let since_str = since_date.format("%Y-%m-%d").to_string();

        info!("Analyzing code churn for last {} days", period_days);

        let file_metrics = Self::get_file_metrics(project_path, &since_str)?;
        let summary = Self::generate_summary(&file_metrics);

        Ok(CodeChurnAnalysis {
            generated_at: Utc::now(),
            period_days,
            repository_root: project_path.to_path_buf(),
            files: file_metrics,
            summary,
        })
    }

    fn get_file_metrics(
        project_path: &Path,
        since_date: &str,
    ) -> Result<Vec<FileChurnMetrics>, TemplateError> {
        let output = Command::new("git")
            .arg("log")
            .arg("--since")
            .arg(since_date)
            .arg("--pretty=format:%H|%an|%aI")
            .arg("--numstat")
            .current_dir(project_path)
            .output()
            .map_err(TemplateError::Io)?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            // Handle empty repository case
            if error_msg.contains("does not have any commits yet") {
                return Ok(Vec::new());
            }
            return Err(TemplateError::NotFound(format!(
                "Git log failed: {error_msg}"
            )));
        }

        let log_output = String::from_utf8_lossy(&output.stdout);
        let mut file_stats: HashMap<PathBuf, FileStats> = HashMap::new();
        let mut current_commit: Option<CommitInfo> = None;

        for line in log_output.lines() {
            if line.is_empty() {
                continue;
            }

            if let Some((hash, author, date)) = Self::parse_commit_line(line) {
                current_commit = Some(CommitInfo { hash, author, date });
            } else if let Some(ref commit) = current_commit {
                if let Some((additions, deletions, file_path)) = Self::parse_numstat_line(line) {
                    let path = PathBuf::from(&file_path);
                    let stats = file_stats.entry(path.clone()).or_insert_with(|| FileStats {
                        commits: Vec::new(),
                        authors: HashSet::new(),
                        total_additions: 0,
                        total_deletions: 0,
                        first_seen: commit.date.clone(),
                        last_modified: commit.date.clone(),
                    });

                    stats.commits.push(commit.hash.clone());
                    stats.authors.insert(commit.author.clone());
                    stats.total_additions += additions;
                    stats.total_deletions += deletions;

                    if commit.date > stats.last_modified {
                        stats.last_modified = commit.date.clone();
                    }
                    if commit.date < stats.first_seen {
                        stats.first_seen = commit.date.clone();
                    }
                }
            }
        }

        let max_commits = file_stats
            .values()
            .map(|s| s.commits.len())
            .max()
            .unwrap_or(1);
        let max_changes = file_stats
            .values()
            .map(|s| s.total_additions + s.total_deletions)
            .max()
            .unwrap_or(1);

        let mut metrics: Vec<FileChurnMetrics> = file_stats
            .into_iter()
            .map(|(path, stats)| {
                let mut metric = FileChurnMetrics {
                    path: project_path.join(&path),
                    relative_path: path.to_string_lossy().to_string(),
                    commit_count: stats.commits.len(),
                    unique_authors: stats.authors.into_iter().collect(),
                    additions: stats.total_additions,
                    deletions: stats.total_deletions,
                    churn_score: 0.0,
                    last_modified: DateTime::parse_from_rfc3339(&stats.last_modified)
                        .unwrap_or_else(|_| Utc::now().into())
                        .with_timezone(&Utc),
                    first_seen: DateTime::parse_from_rfc3339(&stats.first_seen)
                        .unwrap_or_else(|_| Utc::now().into())
                        .with_timezone(&Utc),
                };
                metric.calculate_churn_score(max_commits, max_changes);
                metric
            })
            .collect();

        metrics.sort_by(|a, b| b.churn_score.partial_cmp(&a.churn_score).unwrap());

        Ok(metrics)
    }

    fn parse_commit_line(line: &str) -> Option<(String, String, String)> {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() == 3 {
            Some((
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
            ))
        } else {
            None
        }
    }

    fn parse_numstat_line(line: &str) -> Option<(usize, usize, String)> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let additions = parts[0].parse::<usize>().ok()?;
            let deletions = parts[1].parse::<usize>().ok()?;
            let file_path = parts[2..].join(" ");
            Some((additions, deletions, file_path))
        } else {
            None
        }
    }

    fn generate_summary(files: &[FileChurnMetrics]) -> ChurnSummary {
        let mut author_contributions: HashMap<String, usize> = HashMap::new();
        let mut total_commits = 0;

        for file in files {
            total_commits += file.commit_count;
            for author in &file.unique_authors {
                *author_contributions.entry(author.clone()).or_insert(0) += 1;
            }
        }

        let hotspot_files: Vec<PathBuf> = files
            .iter()
            .take(10)
            .filter(|f| f.churn_score > 0.5)
            .map(|f| f.path.clone())
            .collect();

        let stable_files: Vec<PathBuf> = files
            .iter()
            .rev()
            .take(10)
            .filter(|f| f.churn_score < 0.1 && f.commit_count > 0)
            .map(|f| f.path.clone())
            .collect();

        ChurnSummary {
            total_commits,
            total_files_changed: files.len(),
            hotspot_files,
            stable_files,
            author_contributions,
        }
    }
}

struct FileStats {
    commits: Vec<String>,
    authors: HashSet<String>,
    total_additions: usize,
    total_deletions: usize,
    first_seen: String,
    last_modified: String,
}

struct CommitInfo {
    hash: String,
    author: String,
    date: String,
}
