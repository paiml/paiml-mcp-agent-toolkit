// Compute and git helper methods for IncrementalChurnAnalyzer
// Included from incremental_churn.rs - do NOT add `use` imports or `#!` attributes here.

impl IncrementalChurnAnalyzer {
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
                "Failed to get git log for file: {file_path:?}"
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
        let mut author_contributions: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
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

        // Calculate churn score statistics
        let (mean_churn_score, variance_churn_score, stddev_churn_score) =
            calculate_churn_statistics(files);

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
