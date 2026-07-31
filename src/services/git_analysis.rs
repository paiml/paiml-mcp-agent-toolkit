#![cfg_attr(coverage_nightly, coverage(off))]
//! Git repository analysis and code churn metrics.
//!
//! This module provides analysis of git repositories to extract code churn
//! metrics, contribution patterns, and file activity over time. Code churn
//! is a key indicator of technical debt and maintenance burden.
//!
//! # Metrics Calculated
//!
//! - **File Churn**: Additions, deletions, and modifications per file
//! - **Author Activity**: Contributions by developer over time
//! - **Hotspots**: Files with high change frequency
//! - **Stability**: Files that haven't changed recently
//!
//! # Example
//!
//! ```ignore
//! use pmat::services::git_analysis::GitAnalysisService;
//! use std::path::Path;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Analyze code churn for the last 30 days
//! let project_path = Path::new(".");
//! let analysis = GitAnalysisService::analyze_code_churn(project_path, 30)?;
//!
//! println!("Total commits: {}", analysis.summary.total_commits);
//! println!("Files changed: {}", analysis.summary.files_changed);
//! println!("Active authors: {}", analysis.summary.active_authors);
//!
//! // Find hotspots (files with high churn)
//! for file in &analysis.files {
//!     if file.commits > 10 {
//!         println!("Hotspot: {} ({} commits)", file.file_path.display(), file.commits);
//!     }
//! }
//! # Ok(())
//! # }
//! ```ignore

use crate::models::churn::{ChurnSummary, CodeChurnAnalysis, FileChurnMetrics};
use crate::models::error::TemplateError;
use chrono::{DateTime, Duration, Utc};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;

/// Git analysis service.
pub struct GitAnalysisService;

impl GitAnalysisService {
    #[inline]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    /// Analyze code churn.
    pub fn analyze_code_churn(
        project_path: &Path,
        period_days: u32,
    ) -> Result<CodeChurnAnalysis, TemplateError> {
        if !project_path.join(".git").exists() {
            return Err(TemplateError::NotFound(format!(
                "No git repository found at {project_path:?}"
            )));
        }

        // Checked arithmetic: `Utc::now() - Duration::days(...)` aborts the
        // process with SIGABRT ("`DateTime - TimeDelta` overflowed") past
        // roughly 10^8 days, reachable from the CLI as `-d 2147483647`. A
        // user-supplied integer must never reach unchecked calendar arithmetic.
        // See contracts/pmat-no-fabrication-v1.yaml, `bounded_time_arithmetic`.
        //
        // An out-of-range lookback means "all of history", and is expressed to
        // git by OMITTING --since rather than by naming a date.
        //
        // Clamping to DateTime::MIN_UTC was wrong in a way that is worse than
        // the SIGABRT it replaced: the resulting year is one `git log --since`
        // parses erratically, so results became non-monotonic and silently
        // empty. On a 2-commit repository, -d 400000 found 1 file, -d 730000
        // found ZERO ("Analyzed 0 files with changes" for a live repo), -d
        // 800000 found 1 again. A confidently wrong zero is a fabrication;
        // asking for more history must never return less.
        // The floor is the Unix epoch, not the limit of chrono's range. Checked
        // arithmetic alone was not enough: 400_000 days is ~year 931, which
        // chrono represents happily and `git log --since` then parses
        // erratically. On a 2-commit repo that produced a NON-MONOTONIC series
        // -- -d 400000 found 1 file, -d 730000 found ZERO ("Analyzed 0 files
        // with changes" on a live repo), -d 800000 found 1 again. Asking for
        // more history must never return less, and a confident zero is a
        // fabrication.
        //
        // Any window reaching past 1970 means "all of history", which is
        // expressed to git by OMITTING --since rather than naming a date git
        // will mis-parse.
        let epoch = DateTime::from_timestamp(0, 0).unwrap_or_else(Utc::now);
        let since_str = Duration::try_days(i64::from(period_days))
            .and_then(|d| Utc::now().checked_sub_signed(d))
            .filter(|since| *since >= epoch)
            .map(|since| since.format("%Y-%m-%d").to_string());

        info!("Analyzing code churn for last {} days", period_days);

        let (file_metrics, total_commits) =
            Self::get_file_metrics(project_path, since_str.as_deref())?;
        let summary = Self::generate_summary(&file_metrics, total_commits);

        Ok(CodeChurnAnalysis {
            generated_at: Utc::now(),
            period_days,
            repository_root: project_path.to_path_buf(),
            files: file_metrics,
            summary,
        })
    }

    /// Per-file churn metrics plus the number of DISTINCT commits in the window.
    ///
    /// The commit count is returned separately because it cannot be recovered
    /// from `FileChurnMetrics`: summing `commit_count` over files counts one
    /// commit once per file it touched. That sum was what "Total commits"
    /// reported (GH #660) — 756 for a window git puts at 40, and 2 for a
    /// fixture repo with a single commit touching two files.
    fn get_file_metrics(
        project_path: &Path,
        since_date: Option<&str>,
    ) -> Result<(Vec<FileChurnMetrics>, usize), TemplateError> {
        // Spawn git log with timeout to prevent runaway processes (#245)
        let (tx, rx) = std::sync::mpsc::channel();
        let project_dir = project_path.to_path_buf();
        let since = since_date.map(std::string::ToString::to_string);
        std::thread::spawn(move || {
            let mut cmd = Command::new("git");
            cmd.arg("log");
            // No --since at all means "all history", which is what an
            // out-of-range lookback asks for. Naming an extreme date instead
            // makes `git log` return erratic, sometimes empty, results.
            if let Some(ref since) = since {
                cmd.arg("--since").arg(since);
            }
            let result = cmd
                .arg("--pretty=format:%H|%an|%aI")
                .arg("--numstat")
                .current_dir(&project_dir)
                .output();
            let _ = tx.send(result);
        });

        let timeout = std::time::Duration::from_secs(60);
        let output = match rx.recv_timeout(timeout) {
            Ok(result) => result.map_err(TemplateError::Io)?,
            Err(_) => {
                tracing::warn!(
                    "git log --numstat timed out after {}s for {}",
                    timeout.as_secs(),
                    project_path.display()
                );
                return Ok((Vec::new(), 0));
            }
        };

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            // Handle empty repository case
            if error_msg.contains("does not have any commits yet") {
                return Ok((Vec::new(), 0));
            }
            return Err(TemplateError::NotFound(format!(
                "Git log failed: {error_msg}"
            )));
        }

        let log_output = String::from_utf8_lossy(&output.stdout);
        let mut file_stats: HashMap<PathBuf, FileStats> = HashMap::with_capacity(64);
        let mut current_commit: Option<CommitInfo> = None;
        // Distinct commits seen in the window. One header line per commit, so
        // this equals `git rev-list --count --since=<date>` — including merge
        // commits, which print a header but no --numstat lines.
        let mut commit_hashes: HashSet<String> = HashSet::with_capacity(64);

        for line in log_output.lines() {
            if line.is_empty() {
                continue;
            }

            if let Some((hash, author, date)) = Self::parse_commit_line(line) {
                commit_hashes.insert(hash.clone());
                current_commit = Some(CommitInfo { hash, author, date });
            } else if let Some(ref commit) = current_commit {
                if let Some((additions, deletions, file_path)) = Self::parse_numstat_line(line) {
                    let path = PathBuf::from(&file_path);
                    let stats = file_stats.entry(path.clone()).or_insert_with(|| FileStats {
                        commits: Vec::new(),
                        authors: HashSet::with_capacity(64),
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

        metrics.sort_by(|a, b| {
            b.churn_score
                .partial_cmp(&a.churn_score)
                .expect("internal error")
        });

        Ok((metrics, commit_hashes.len()))
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

    /// Build the summary. `total_commits` is the count of DISTINCT commits in
    /// the window, supplied by the caller.
    ///
    /// It used to be computed here as `sum(file.commit_count)`, i.e. the number
    /// of file revisions, and was still labelled "Total commits" (GH #660).
    fn generate_summary(files: &[FileChurnMetrics], total_commits: usize) -> ChurnSummary {
        let mut author_contributions: HashMap<String, usize> = HashMap::with_capacity(64);

        for file in files {
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

        // Calculate churn score statistics
        let (mean_churn_score, variance_churn_score, stddev_churn_score) = if files.is_empty() {
            (0.0, 0.0, 0.0)
        } else {
            let scores: Vec<f64> = files.iter().map(|f| f64::from(f.churn_score)).collect();
            let n = scores.len() as f64;
            let sum: f64 = scores.iter().sum();
            let mean = sum / n;
            let variance = scores.iter().map(|&s| (s - mean).powi(2)).sum::<f64>() / n;
            let stddev = variance.sqrt();
            (mean, variance, stddev)
        };

        ChurnSummary {
            total_commits,
            total_files_changed: files.len(),
            hotspot_files,
            stable_files,
            author_contributions,
            mean_churn_score,
            variance_churn_score,
            stddev_churn_score,
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_git_analysis_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }

    #[test]
    fn test_parse_commit_line_valid() {
        let line = "abc123|John Doe|2024-01-15T10:30:00Z";
        let result = GitAnalysisService::parse_commit_line(line);
        assert!(result.is_some());
        let (hash, author, date) = result.unwrap();
        assert_eq!(hash, "abc123");
        assert_eq!(author, "John Doe");
        assert_eq!(date, "2024-01-15T10:30:00Z");
    }

    #[test]
    fn test_parse_commit_line_invalid_too_few() {
        let line = "abc123|John Doe";
        let result = GitAnalysisService::parse_commit_line(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_commit_line_invalid_empty() {
        let line = "";
        let result = GitAnalysisService::parse_commit_line(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_numstat_line_valid() {
        let line = "10\t20\tsrc/main.rs";
        let result = GitAnalysisService::parse_numstat_line(line);
        assert!(result.is_some());
        let (additions, deletions, path) = result.unwrap();
        assert_eq!(additions, 10);
        assert_eq!(deletions, 20);
        assert_eq!(path, "src/main.rs");
    }

    #[test]
    fn test_parse_numstat_line_with_spaces() {
        let line = "5\t15\tpath/to file.rs";
        let result = GitAnalysisService::parse_numstat_line(line);
        assert!(result.is_some());
        let (additions, deletions, path) = result.unwrap();
        assert_eq!(additions, 5);
        assert_eq!(deletions, 15);
        assert_eq!(path, "path/to file.rs");
    }

    #[test]
    fn test_parse_numstat_line_invalid() {
        let line = "invalid";
        let result = GitAnalysisService::parse_numstat_line(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_numstat_line_binary() {
        let line = "-\t-\timage.png";
        let result = GitAnalysisService::parse_numstat_line(line);
        assert!(result.is_none()); // '-' can't parse as usize
    }

    #[test]
    fn test_generate_summary_empty() {
        let files: Vec<FileChurnMetrics> = vec![];
        let summary = GitAnalysisService::generate_summary(&files, 0);
        assert_eq!(summary.total_commits, 0);
        assert_eq!(summary.total_files_changed, 0);
        assert!(summary.hotspot_files.is_empty());
        assert!(summary.stable_files.is_empty());
        assert_eq!(summary.mean_churn_score, 0.0);
    }

    #[test]
    fn test_generate_summary_single_file() {
        let files = vec![FileChurnMetrics {
            path: PathBuf::from("src/main.rs"),
            relative_path: "src/main.rs".to_string(),
            commit_count: 5,
            unique_authors: vec!["Author1".to_string()],
            additions: 100,
            deletions: 50,
            churn_score: 0.7,
            last_modified: Utc::now(),
            first_seen: Utc::now(),
        }];
        // #660: the summary reports the DISTINCT commit count it is given, not
        // sum(commit_count). Here 5 revisions of one file came from 3 commits
        // (two of them touched it twice via amends/rebases in the window), so
        // "Total commits" must be 3 — the old code answered 5.
        let summary = GitAnalysisService::generate_summary(&files, 3);
        assert_eq!(summary.total_commits, 3);
        assert_eq!(summary.total_files_changed, 1);
        assert_eq!(summary.hotspot_files.len(), 1); // churn_score > 0.5
    }

    #[test]
    fn test_generate_summary_multiple_authors() {
        let files = vec![
            FileChurnMetrics {
                path: PathBuf::from("file1.rs"),
                relative_path: "file1.rs".to_string(),
                commit_count: 3,
                unique_authors: vec!["Author1".to_string(), "Author2".to_string()],
                additions: 50,
                deletions: 20,
                churn_score: 0.3,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            },
            FileChurnMetrics {
                path: PathBuf::from("file2.rs"),
                relative_path: "file2.rs".to_string(),
                commit_count: 2,
                unique_authors: vec!["Author2".to_string()],
                additions: 30,
                deletions: 10,
                churn_score: 0.05,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            },
        ];
        // #660: two files with 3 and 2 revisions can come from as few as 3
        // commits. The old code reported 5 "commits" for exactly this shape.
        let summary = GitAnalysisService::generate_summary(&files, 3);
        assert_eq!(summary.total_commits, 3);
        assert_eq!(summary.total_files_changed, 2);
        assert_eq!(summary.author_contributions.len(), 2);
    }

    #[test]
    fn test_analyze_code_churn_no_git() {
        let temp_dir = TempDir::new().unwrap();
        let result = GitAnalysisService::analyze_code_churn(temp_dir.path(), 30);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_summary_stable_files() {
        let files = vec![FileChurnMetrics {
            path: PathBuf::from("stable.rs"),
            relative_path: "stable.rs".to_string(),
            commit_count: 1,
            unique_authors: vec!["Author".to_string()],
            additions: 10,
            deletions: 5,
            churn_score: 0.05, // Low churn
            last_modified: Utc::now(),
            first_seen: Utc::now(),
        }];
        let summary = GitAnalysisService::generate_summary(&files, 1);
        assert_eq!(summary.stable_files.len(), 1);
    }

    #[test]
    fn test_churn_summary_statistics() {
        let files = vec![
            FileChurnMetrics {
                path: PathBuf::from("f1.rs"),
                relative_path: "f1.rs".to_string(),
                commit_count: 2,
                unique_authors: vec![],
                additions: 0,
                deletions: 0,
                churn_score: 0.2,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            },
            FileChurnMetrics {
                path: PathBuf::from("f2.rs"),
                relative_path: "f2.rs".to_string(),
                commit_count: 4,
                unique_authors: vec![],
                additions: 0,
                deletions: 0,
                churn_score: 0.8,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            },
        ];
        let summary = GitAnalysisService::generate_summary(&files, 4);
        // Mean of 0.2 and 0.8 = 0.5
        assert!((summary.mean_churn_score - 0.5).abs() < 0.01);
        // Variance and stddev should be calculated
        assert!(summary.variance_churn_score >= 0.0);
        assert!(summary.stddev_churn_score >= 0.0);
    }

    /// GH #660: "Total commits" must equal `git rev-list --count`, not the sum
    /// of per-file revision counts.
    ///
    /// One commit touching three files reported "Total commits: 3" before the
    /// fix. On the pmat repo the same bug turned 40 commits into 756.
    #[test]
    fn churn_total_commits_counts_commits_not_file_revisions() {
        use std::process::Command;

        let temp = TempDir::new().unwrap();
        let repo = temp.path();

        let git = |args: &[&str]| {
            Command::new("git")
                .args(args)
                .current_dir(repo)
                .output()
                .expect("git must be available")
        };

        if !git(&["init"]).status.success() {
            return; // no git in this environment; nothing to assert
        }
        let _ = git(&["config", "user.email", "t@example.com"]);
        let _ = git(&["config", "user.name", "T"]);

        for name in ["a.rs", "b.rs", "c.rs"] {
            std::fs::write(repo.join(name), "pub fn f() {}\n").unwrap();
        }
        let _ = git(&["add", "-A"]);
        let commit = git(&["commit", "-m", "one commit, three files", "--no-verify"]);
        if !commit.status.success() {
            return; // commit hooks / identity unavailable
        }

        let analysis = GitAnalysisService::analyze_code_churn(repo, 30).expect("churn analysis");

        assert_eq!(
            analysis.summary.total_files_changed, 3,
            "three files were touched"
        );
        assert_eq!(
            analysis.summary.total_commits, 1,
            "one commit touched three files; the pre-fix code reported 3"
        );
    }
}
