//! Incremental/Lazy Churn Analysis Service
//!
//! This module provides an incremental churn analysis implementation that
//! only analyzes files that have changed since the last analysis run.

use crate::models::churn::{ChurnSummary, CodeChurnAnalysis, FileChurnMetrics};
use crate::models::error::TemplateError;
use crate::services::git_analysis::GitAnalysisService;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Cache entry for file churn metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChurnCacheEntry {
    pub metrics: FileChurnMetrics,
    pub last_modified: DateTime<Utc>,
    pub git_commit_hash: String,
}

/// Incremental churn analyzer with lazy evaluation
pub struct IncrementalChurnAnalyzer {
    /// Cache of file metrics keyed by path
    cache: Arc<DashMap<PathBuf, ChurnCacheEntry>>,
    /// Project root path
    project_root: PathBuf,
}

impl IncrementalChurnAnalyzer {
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            project_root,
        }
    }

    /// Get churn metrics for a specific file (lazy evaluation)
    pub async fn get_file_churn(
        &self,
        file_path: &Path,
    ) -> Result<FileChurnMetrics, TemplateError> {
        // Check cache first
        let relative_path = file_path
            .strip_prefix(&self.project_root)
            .unwrap_or(file_path)
            .to_path_buf();

        if let Some(entry) = self.cache.get(&relative_path) {
            // Check if file has been modified since cache entry
            if self.is_cache_valid(&entry, file_path).await? {
                return Ok(entry.metrics.clone());
            }
        }

        // Compute churn for this specific file
        let metrics = self.compute_file_churn(file_path).await?;

        // Cache the result
        let commit_hash = self.get_current_commit_hash().await?;
        let entry = ChurnCacheEntry {
            metrics: metrics.clone(),
            last_modified: Utc::now(),
            git_commit_hash: commit_hash,
        };
        self.cache.insert(relative_path, entry);

        Ok(metrics)
    }

    /// Get churn analysis for multiple files (incremental)
    pub async fn analyze_incremental(
        &self,
        files: Vec<PathBuf>,
        period_days: u32,
    ) -> Result<CodeChurnAnalysis, TemplateError> {
        let mut file_metrics = Vec::new();
        let mut uncached_files = Vec::new();

        // Separate cached and uncached files
        for file in files {
            let relative_path = file
                .strip_prefix(&self.project_root)
                .unwrap_or(&file)
                .to_path_buf();

            if let Some(entry) = self.cache.get(&relative_path) {
                if self.is_cache_valid(&entry, &file).await? {
                    file_metrics.push(entry.metrics.clone());
                    continue;
                }
            }
            uncached_files.push(file);
        }

        // Batch analyze uncached files
        if !uncached_files.is_empty() {
            let new_metrics = self
                .batch_compute_churn(&uncached_files, period_days)
                .await?;

            // Cache new results
            let commit_hash = self.get_current_commit_hash().await?;
            for metrics in &new_metrics {
                let relative_path = metrics
                    .path
                    .strip_prefix(&self.project_root)
                    .unwrap_or(&metrics.path)
                    .to_path_buf();

                let entry = ChurnCacheEntry {
                    metrics: metrics.clone(),
                    last_modified: Utc::now(),
                    git_commit_hash: commit_hash.clone(),
                };
                self.cache.insert(relative_path, entry);
            }

            file_metrics.extend(new_metrics);
        }

        // Generate summary
        let summary = self.generate_summary(&file_metrics);

        Ok(CodeChurnAnalysis {
            generated_at: Utc::now(),
            period_days,
            repository_root: self.project_root.clone(),
            files: file_metrics,
            summary,
        })
    }

    /// Check if cache entry is still valid
    async fn is_cache_valid(
        &self,
        entry: &ChurnCacheEntry,
        file_path: &Path,
    ) -> Result<bool, TemplateError> {
        // Check if file has been modified in git since cache entry
        let current_hash = self.get_file_last_commit_hash(file_path).await?;
        Ok(current_hash == entry.git_commit_hash)
    }

    /// Compute churn for a single file
    async fn compute_file_churn(
        &self,
        file_path: &Path,
    ) -> Result<FileChurnMetrics, TemplateError> {
        // Use git log to get file-specific churn
        let output = std::process::Command::new("git")
            .arg("log")
            .arg("--follow")
            .arg("--numstat")
            .arg("--pretty=format:%H|%an|%aI")
            .arg("--")
            .arg(file_path)
            .current_dir(&self.project_root)
            .output()
            .map_err(TemplateError::Io)?;

        if !output.status.success() {
            return Err(TemplateError::NotFound(format!(
                "Failed to get git log for file: {:?}",
                file_path
            )));
        }

        // Parse git log output
        let log_output = String::from_utf8_lossy(&output.stdout);
        let mut commits = Vec::new();
        let mut authors = std::collections::HashSet::new();
        let mut total_additions = 0;
        let mut total_deletions = 0;
        let mut first_seen = None;
        let mut last_modified = None;

        let lines: Vec<&str> = log_output.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            if let Some((hash, author, date)) = Self::parse_commit_line(lines[i]) {
                commits.push(hash);
                authors.insert(author);

                let parsed_date = DateTime::parse_from_rfc3339(&date)
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc);

                if first_seen.is_none() {
                    first_seen = Some(parsed_date);
                }
                last_modified = Some(parsed_date);

                // Look for numstat on next line
                if i + 1 < lines.len() {
                    if let Some((additions, deletions, _)) = Self::parse_numstat_line(lines[i + 1])
                    {
                        total_additions += additions;
                        total_deletions += deletions;
                        i += 1; // Skip numstat line
                    }
                }
            }
            i += 1;
        }

        let mut metrics = FileChurnMetrics {
            path: file_path.to_path_buf(),
            relative_path: file_path
                .strip_prefix(&self.project_root)
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_string(),
            commit_count: commits.len(),
            unique_authors: authors.into_iter().collect(),
            additions: total_additions,
            deletions: total_deletions,
            churn_score: 0.0,
            last_modified: last_modified.unwrap_or_else(Utc::now),
            first_seen: first_seen.unwrap_or_else(Utc::now),
        };

        // Calculate churn score
        metrics.calculate_churn_score(100, 1000); // Default max values

        Ok(metrics)
    }

    /// Batch compute churn for multiple files
    async fn batch_compute_churn(
        &self,
        files: &[PathBuf],
        period_days: u32,
    ) -> Result<Vec<FileChurnMetrics>, TemplateError> {
        // Fall back to full analysis for batch
        let analysis = GitAnalysisService::analyze_code_churn(&self.project_root, period_days)?;

        // Filter to only requested files
        let requested_files: std::collections::HashSet<_> = files.iter().collect();
        let filtered_metrics: Vec<_> = analysis
            .files
            .into_iter()
            .filter(|m| requested_files.contains(&m.path))
            .collect();

        Ok(filtered_metrics)
    }

    /// Get current git commit hash
    async fn get_current_commit_hash(&self) -> Result<String, TemplateError> {
        let output = tokio::process::Command::new("git")
            .arg("rev-parse")
            .arg("HEAD")
            .current_dir(&self.project_root)
            .output()
            .await
            .map_err(TemplateError::Io)?;

        if !output.status.success() {
            return Err(TemplateError::NotFound(
                "Failed to get current commit hash".to_string(),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Get last commit hash for a specific file
    async fn get_file_last_commit_hash(&self, file_path: &Path) -> Result<String, TemplateError> {
        let output = tokio::process::Command::new("git")
            .arg("log")
            .arg("-1")
            .arg("--format=%H")
            .arg("--")
            .arg(file_path)
            .current_dir(&self.project_root)
            .output()
            .await
            .map_err(TemplateError::Io)?;

        if !output.status.success() {
            return Ok(String::new()); // File might be new
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Parse commit line from git log
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

    /// Parse numstat line from git log
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

    /// Generate summary from file metrics
    fn generate_summary(&self, files: &[FileChurnMetrics]) -> ChurnSummary {
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
            .filter(|f| f.churn_score > 0.5)
            .take(10)
            .map(|f| f.path.clone())
            .collect();

        let stable_files: Vec<PathBuf> = files
            .iter()
            .filter(|f| f.churn_score < 0.1 && f.commit_count > 0)
            .take(10)
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

    /// Clear the cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        let cache_size = self.cache.len();
        let cache_memory = cache_size * std::mem::size_of::<(PathBuf, ChurnCacheEntry)>();
        (cache_size, cache_memory)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_incremental_churn_cache() {
        let analyzer = IncrementalChurnAnalyzer::new(PathBuf::from("."));
        let (size, _) = analyzer.cache_stats();
        assert_eq!(size, 0);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let temp_dir = TempDir::new().unwrap();
        let analyzer = IncrementalChurnAnalyzer::new(temp_dir.path().to_path_buf());

        let (size, memory) = analyzer.cache_stats();
        assert_eq!(size, 0);
        assert_eq!(memory, 0);

        // Clear cache should work even when empty
        analyzer.clear_cache();
        let (size, _) = analyzer.cache_stats();
        assert_eq!(size, 0);
    }

    #[tokio::test]
    async fn test_parse_commit_line() {
        let line = "abc123|John Doe|2024-01-01T12:00:00Z";
        let result = IncrementalChurnAnalyzer::parse_commit_line(line);
        assert!(result.is_some());

        let (hash, author, date) = result.unwrap();
        assert_eq!(hash, "abc123");
        assert_eq!(author, "John Doe");
        assert_eq!(date, "2024-01-01T12:00:00Z");

        // Test invalid line
        let invalid = "invalid line";
        assert!(IncrementalChurnAnalyzer::parse_commit_line(invalid).is_none());
    }

    #[tokio::test]
    async fn test_parse_numstat_line() {
        let line = "10\t20\tsrc/main.rs";
        let result = IncrementalChurnAnalyzer::parse_numstat_line(line);
        assert!(result.is_some());

        let (additions, deletions, path) = result.unwrap();
        assert_eq!(additions, 10);
        assert_eq!(deletions, 20);
        assert_eq!(path, "src/main.rs");

        // Test with spaces in filename
        let line_with_spaces = "5\t15\tsrc/my file.rs";
        let result = IncrementalChurnAnalyzer::parse_numstat_line(line_with_spaces);
        assert!(result.is_some());
        let (_, _, path) = result.unwrap();
        assert_eq!(path, "src/my file.rs");

        // Test invalid line
        assert!(IncrementalChurnAnalyzer::parse_numstat_line("invalid").is_none());
    }

    #[tokio::test]
    async fn test_generate_summary() {
        let analyzer = IncrementalChurnAnalyzer::new(PathBuf::from("."));

        let mut metrics = vec![];

        // Add some test metrics
        for i in 0..5 {
            let mut m = FileChurnMetrics {
                path: PathBuf::from(format!("file{}.rs", i)),
                relative_path: format!("file{}.rs", i),
                commit_count: 10 - i * 2,
                unique_authors: vec![format!("Author{}", i)],
                additions: 100,
                deletions: 50,
                churn_score: (0.9 - (i as f64 * 0.2)) as f32,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            };
            m.calculate_churn_score(10, 150);
            metrics.push(m);
        }

        let summary = analyzer.generate_summary(&metrics);

        assert_eq!(summary.total_files_changed, 5);
        assert_eq!(summary.total_commits, 30); // 10 + 8 + 6 + 4 + 2
        assert_eq!(summary.author_contributions.len(), 5);
        assert!(summary.hotspot_files.len() <= 10);
        assert!(summary.stable_files.len() <= 10);
    }

    #[tokio::test]
    async fn test_get_file_churn_no_git() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "test content").unwrap();

        let analyzer = IncrementalChurnAnalyzer::new(temp_dir.path().to_path_buf());
        let result = analyzer.get_file_churn(&test_file).await;

        // Should fail because there's no git repo
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analyze_incremental_empty() {
        let temp_dir = TempDir::new().unwrap();
        let analyzer = IncrementalChurnAnalyzer::new(temp_dir.path().to_path_buf());

        let result = analyzer.analyze_incremental(vec![], 30).await;

        // Should succeed with empty results
        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert_eq!(analysis.files.len(), 0);
        assert_eq!(analysis.period_days, 30);
    }

    #[tokio::test]
    async fn test_batch_compute_churn_no_git() {
        let temp_dir = TempDir::new().unwrap();
        let analyzer = IncrementalChurnAnalyzer::new(temp_dir.path().to_path_buf());

        let files = vec![
            temp_dir.path().join("file1.rs"),
            temp_dir.path().join("file2.rs"),
        ];

        let result = analyzer.batch_compute_churn(&files, 30).await;

        // Should fail because there's no git repo
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_current_commit_hash_no_git() {
        let temp_dir = TempDir::new().unwrap();
        let analyzer = IncrementalChurnAnalyzer::new(temp_dir.path().to_path_buf());

        let result = analyzer.get_current_commit_hash().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_file_last_commit_hash_no_git() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "test content").unwrap();

        let analyzer = IncrementalChurnAnalyzer::new(temp_dir.path().to_path_buf());
        let result = analyzer.get_file_last_commit_hash(&test_file).await;

        // Should return empty string for non-git files
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[tokio::test]
    async fn test_is_cache_valid() {
        let temp_dir = TempDir::new().unwrap();
        let analyzer = IncrementalChurnAnalyzer::new(temp_dir.path().to_path_buf());

        let entry = ChurnCacheEntry {
            metrics: FileChurnMetrics {
                path: temp_dir.path().join("test.rs"),
                relative_path: "test.rs".to_string(),
                commit_count: 5,
                unique_authors: vec!["Test Author".to_string()],
                additions: 100,
                deletions: 50,
                churn_score: 0.5,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            },
            last_modified: Utc::now(),
            git_commit_hash: "abc123".to_string(),
        };

        // In a non-git directory, cache should be invalid
        let result = analyzer.is_cache_valid(&entry, &entry.metrics.path).await;
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Cache invalid because no git
    }

    #[tokio::test]
    async fn test_compute_file_churn_parsing() {
        // Test the parsing logic without actual git
        let output = r#"abc123|John Doe|2024-01-01T12:00:00Z
10	20	src/main.rs
def456|Jane Smith|2024-01-02T12:00:00Z
5	10	src/main.rs"#;

        // We can't test the full function without git, but we can test the parsing
        let lines: Vec<&str> = output.lines().collect();
        let mut commits = Vec::new();
        let mut authors = std::collections::HashSet::new();
        let mut total_additions = 0;
        let mut total_deletions = 0;

        let mut i = 0;
        while i < lines.len() {
            if let Some((hash, author, _date)) =
                IncrementalChurnAnalyzer::parse_commit_line(lines[i])
            {
                commits.push(hash);
                authors.insert(author);

                if i + 1 < lines.len() {
                    if let Some((additions, deletions, _)) =
                        IncrementalChurnAnalyzer::parse_numstat_line(lines[i + 1])
                    {
                        total_additions += additions;
                        total_deletions += deletions;
                        i += 1;
                    }
                }
            }
            i += 1;
        }

        assert_eq!(commits.len(), 2);
        assert_eq!(authors.len(), 2);
        assert_eq!(total_additions, 15);
        assert_eq!(total_deletions, 30);
    }
}
