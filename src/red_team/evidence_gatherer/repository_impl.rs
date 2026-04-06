impl RepositoryContext {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new_mock() -> Self {
        Self {
            subsequent_commits: Some(vec![]),
            test_results: Some((true, 0)),
            actual_coverage: None,
            coverage_error: None,
            broken_links_count: None,
            vulnerabilities_count: None,
            benchmark_results: None,
            issue_status: None,
            code_grep_results: None,
            latest_commit_timestamp: None,
            commit_timestamps: None,
            git_repo: None,
            test_files: vec![],
            coverage_path: None,
            test_results_path: None,
            repo_path: PathBuf::from("."),
        }
    }

    /// Build repository context from actual filesystem path
    ///
    /// GREEN Phase: Implementation for RED tests
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn from_path(path: &Path) -> Result<Self> {
        Self::from_path_with_config(path, false)
    }

    /// Build repository context with configuration options
    ///
    /// # Arguments
    /// * `path` - Repository path
    /// * `deep` - If true, fetch entire git history; if false, fetch recent commits only (last 30 days)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn from_path_with_config(path: &Path, deep: bool) -> Result<Self> {
        let repo_path = path.canonicalize().context("Failed to canonicalize path")?;

        // Detect git repository
        let git_repo = Self::find_git_repo(&repo_path);

        // Scan for test files
        let test_files = Self::scan_test_files(&repo_path)?;

        // Find coverage reports
        let coverage_path = Self::find_coverage_report(&repo_path);

        // Find test results
        let test_results_path = Self::find_test_results(&repo_path);

        // Fetch git history if repository detected
        let subsequent_commits = if git_repo.is_some() {
            Self::fetch_git_history(&repo_path, deep)
        } else {
            None
        };

        Ok(Self {
            subsequent_commits,
            test_results: None,
            actual_coverage: None,
            coverage_error: None,
            broken_links_count: None,
            vulnerabilities_count: None,
            benchmark_results: None,
            issue_status: None,
            code_grep_results: None,
            latest_commit_timestamp: None,
            commit_timestamps: None,
            git_repo,
            test_files,
            coverage_path,
            test_results_path,
            repo_path,
        })
    }

    /// Check if repository has git history
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn has_git_history(&self) -> bool {
        self.git_repo.is_some()
    }

    /// Get recent commits from git history
    #[cfg(feature = "git-lib")]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_recent_commits(&self, limit: usize) -> Vec<CommitInfo> {
        let Some(ref repo_path) = self.git_repo else {
            return vec![];
        };

        let Ok(repo) = git2::Repository::open(repo_path) else {
            return vec![];
        };

        let mut commits = Vec::new();
        let mut revwalk = match repo.revwalk() {
            Ok(w) => w,
            Err(_) => return vec![],
        };

        if revwalk.push_head().is_err() {
            return vec![];
        }

        for oid in revwalk.take(limit) {
            let Ok(oid) = oid else { continue };
            let Ok(commit) = repo.find_commit(oid) else {
                continue;
            };

            commits.push(CommitInfo {
                message: commit.message().unwrap_or("").to_string(),
                timestamp: commit.time().seconds(),
                author: commit.author().name().unwrap_or("Unknown").to_string(),
            });
        }

        commits
    }

    /// Get recent commits from git history (shell git fallback)
    #[cfg(not(feature = "git-lib"))]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_recent_commits(&self, limit: usize) -> Vec<CommitInfo> {
        use std::process::Command;

        let Some(ref repo_path) = self.git_repo else {
            return vec![];
        };

        // Use shell git log with a format that gives us message, timestamp, and author
        let output = Command::new("git")
            .args([
                "log",
                &format!("-{limit}"),
                "--format=%s|%ct|%an",
            ])
            .current_dir(repo_path)
            .output()
            .ok();

        let Some(output) = output else {
            return vec![];
        };

        if !output.status.success() {
            return vec![];
        }

        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(3, '|').collect();
                if parts.len() == 3 {
                    Some(CommitInfo {
                        message: parts[0].to_string(),
                        timestamp: parts[1].parse().unwrap_or(0),
                        author: parts[2].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get test files found in repository
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn get_test_files(&self) -> Vec<PathBuf> {
        self.test_files.clone()
    }

    /// Check if coverage report exists
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn has_coverage_report(&self) -> bool {
        self.coverage_path.is_some()
    }

    /// Get coverage percentage from report
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_coverage_percentage(&self) -> f64 {
        let Some(ref coverage_path) = self.coverage_path else {
            return 0.0;
        };

        Self::parse_coverage_report(coverage_path).unwrap_or(0.0)
    }

    /// Get test execution information
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_test_execution_info(&self) -> TestExecutionInfo {
        let Some(ref test_results_path) = self.test_results_path else {
            return TestExecutionInfo::default();
        };

        Self::parse_test_results(test_results_path).unwrap_or_default()
    }

    /// Search codebase for pattern using grep
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn grep_codebase(&self, pattern: &str) -> Vec<PathBuf> {
        Self::grep_directory(&self.repo_path, pattern).unwrap_or_default()
    }

    // Helper methods

    #[cfg(feature = "git-lib")]
    fn find_git_repo(path: &Path) -> Option<PathBuf> {
        // Use git2's discover() to properly handle worktrees, gitlinks, etc.
        match git2::Repository::discover(path) {
            Ok(repo) => {
                // Get the repository's work directory (not the .git directory)
                repo.workdir().map(|p| p.to_path_buf())
            }
            Err(_) => None,
        }
    }

    #[cfg(not(feature = "git-lib"))]
    fn find_git_repo(path: &Path) -> Option<PathBuf> {
        use std::process::Command;

        // Use shell git rev-parse to find repo root
        let output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(path)
            .output()
            .ok()?;

        if output.status.success() {
            let repo_path = String::from_utf8_lossy(&output.stdout);
            Some(PathBuf::from(repo_path.trim()))
        } else {
            None
        }
    }

    /// Fetch git history commit messages
    ///
    /// # Arguments
    /// * `repo_path` - Path to the repository
    /// * `deep` - If true, fetch entire history; if false, fetch last 30 days only
    ///
    /// # Returns
    /// `Some(Vec<String>)` if git repository detected, `None` otherwise
    fn fetch_git_history(repo_path: &Path, deep: bool) -> Option<Vec<String>> {
        // PMAT-REDTEAM-001: Default to recent commits (fast), use --deep for full history
        let git_command = if deep {
            // Deep mode: Get all commit messages from entire history
            "git log --all --pretty=format:%s"
        } else {
            // Fast mode: Get recent commit messages only (last 30 days)
            "git log --since='30 days ago' --pretty=format:%s"
        };

        // CRITICAL: Use shell to execute git command
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(git_command)
            .current_dir(repo_path)
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let commits: Vec<String> = stdout
                    .lines()
                    .map(|line| line.trim().to_string())
                    .filter(|line| !line.is_empty())
                    .collect();

                if commits.is_empty() {
                    None
                } else {
                    Some(commits)
                }
            }
            _ => None,
        }
    }

    fn scan_test_files(path: &Path) -> Result<Vec<PathBuf>> {
        use walkdir::WalkDir;

        let mut test_files = Vec::new();

        for entry in WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Match test file patterns
            let is_test = path.components().any(|c| c.as_os_str() == "tests")
                || file_name.starts_with("test_")
                || file_name.ends_with("_test.rs")
                || file_name.ends_with("_tests.rs");

            if is_test && path.is_file() {
                test_files.push(path.to_path_buf());
            }
        }

        Ok(test_files)
    }

    fn find_coverage_report(path: &Path) -> Option<PathBuf> {
        // Common coverage report locations
        let candidates = vec![
            path.join("target/coverage/lcov.info"),
            path.join("target/llvm-cov/lcov.info"),
            path.join("coverage/lcov.info"),
            path.join("lcov.info"),
        ];

        candidates.into_iter().find(|p| p.exists())
    }

    fn find_test_results(path: &Path) -> Option<PathBuf> {
        // Common test result locations
        let candidates = vec![
            path.join("target/test-results/output.txt"),
            path.join("test-results/output.txt"),
        ];

        candidates.into_iter().find(|p| p.exists())
    }

    fn parse_coverage_report(path: &Path) -> Result<f64> {
        let content = std::fs::read_to_string(path).context("Failed to read coverage report")?;

        let mut lines_found = 0;
        let mut lines_hit = 0;

        for line in content.lines() {
            if line.starts_with("LF:") {
                if let Some(num) = line.strip_prefix("LF:") {
                    lines_found += num.parse::<usize>().unwrap_or(0);
                }
            } else if line.starts_with("LH:") {
                if let Some(num) = line.strip_prefix("LH:") {
                    lines_hit += num.parse::<usize>().unwrap_or(0);
                }
            }
        }

        if lines_found > 0 {
            Ok((lines_hit as f64 / lines_found as f64) * 100.0)
        } else {
            Ok(0.0)
        }
    }

    fn parse_test_results(path: &Path) -> Result<TestExecutionInfo> {
        let content = std::fs::read_to_string(path).context("Failed to read test results")?;

        // Parse format: "test result: ok. 10 passed; 2 failed; 3 ignored"
        let mut info = TestExecutionInfo {
            has_results: true,
            ..Default::default()
        };

        for line in content.lines() {
            if line.contains("test result:") {
                // Extract numbers using regex-like pattern matching
                if let Some(passed_str) = line
                    .split("passed")
                    .next()
                    .and_then(|s| s.split_whitespace().last())
                {
                    info.passed_count = passed_str.parse().unwrap_or(0);
                }

                if let Some(failed_str) = line.split("failed").next().and_then(|s| {
                    s.split(';')
                        .next_back()
                        .and_then(|part| part.split_whitespace().last())
                }) {
                    info.failed_count = failed_str.parse().unwrap_or(0);
                }

                if let Some(ignored_str) = line.split("ignored").next().and_then(|s| {
                    s.split(';')
                        .next_back()
                        .and_then(|part| part.split_whitespace().last())
                }) {
                    info.ignored_count = ignored_str.parse().unwrap_or(0);
                }

                break;
            }
        }

        Ok(info)
    }

    fn grep_directory(path: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
        use walkdir::WalkDir;

        let mut matches = Vec::new();

        for entry in WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();
            if !entry_path.is_file() {
                continue;
            }

            // Skip binary files and directories
            if let Some(ext) = entry_path.extension() {
                let ext_str = ext.to_string_lossy();
                if ext_str == "so" || ext_str == "a" || ext_str == "o" {
                    continue;
                }
            }

            // Read file and search for pattern
            if let Ok(content) = std::fs::read_to_string(entry_path) {
                if content.contains(pattern) {
                    matches.push(entry_path.to_path_buf());
                }
            }
        }

        Ok(matches)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_coverage(mut self, coverage: f64) -> Self {
        self.actual_coverage = Some(coverage);
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_subsequent_commits(mut self, commits: Vec<String>) -> Self {
        self.subsequent_commits = Some(commits);
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_test_results(mut self, passing: bool, ignored: usize) -> Self {
        self.test_results = Some((passing, ignored));
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_broken_links(mut self, count: usize) -> Self {
        self.broken_links_count = Some(count);
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_vulnerabilities(mut self, count: usize) -> Self {
        self.vulnerabilities_count = Some(count);
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_benchmarks(mut self, data: Option<String>) -> Self {
        self.benchmark_results = data;
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_issue_status(mut self, _issue_num: u32, status: &str) -> Self {
        self.issue_status = Some(status.to_string());
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_code_grep_results(mut self, search_term: &str, count: usize) -> Self {
        self.code_grep_results = Some((search_term.to_string(), count));
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_commit_timestamps(mut self, timestamps: Vec<i64>) -> Self {
        self.commit_timestamps = Some(timestamps.clone());
        self.latest_commit_timestamp = timestamps.last().copied();
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_coverage_error(mut self, error: &str) -> Self {
        self.coverage_error = Some(error.to_string());
        self
    }
}
