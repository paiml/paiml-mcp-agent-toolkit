// Evidence Gatherer: Multi-source validation for hallucination detection
//
// Specification: Section 3.2 - Claim Categories
// Implements empirical evidence gathering for 8 claim categories

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::{Claim, ClaimCategory};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EvidenceSource {
    GitHistory,       // Subsequent commits contradicting claim
    TestExecution,    // Running tests to verify claim
    CoverageReport,   // Actual coverage vs claimed
    LinkValidation,   // Checking documentation links
    CargoAudit,       // Security audit results
    BenchmarkResults, // Performance measurements
    IssueTracker,     // GitHub issue status
    CodeGrep,         // Searching codebase for references
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceResult {
    pub source: EvidenceSource,
    pub supports_claim: bool,
    pub confidence: f64, // 0.0 to 1.0
    pub details: String,
    pub timestamp: Option<i64>,
}

pub struct EvidenceGatherer {
    // Configuration for evidence gathering (future use)
    #[allow(dead_code)]
    git_history_window_days: u32,
    #[allow(dead_code)]
    confidence_threshold: f64,
}

impl EvidenceGatherer {
    pub fn new() -> Self {
        Self {
            git_history_window_days: 30,
            confidence_threshold: 0.7,
        }
    }

    pub fn gather_evidence(
        &self,
        claim: &Claim,
        context: &RepositoryContext,
    ) -> Vec<EvidenceResult> {
        let mut evidence = Vec::new();

        match claim.category {
            ClaimCategory::TestStatus => {
                evidence.extend(self.gather_test_status_evidence(claim, context));
            }
            ClaimCategory::Documentation => {
                evidence.extend(self.gather_documentation_evidence(claim, context));
            }
            ClaimCategory::Coverage => {
                evidence.extend(self.gather_coverage_evidence(claim, context));
            }
            ClaimCategory::FeatureCompletion => {
                evidence.extend(self.gather_feature_completion_evidence(claim, context));
            }
            ClaimCategory::Migration => {
                evidence.extend(self.gather_migration_evidence(claim, context));
            }
            ClaimCategory::BugFix => {
                evidence.extend(self.gather_bugfix_evidence(claim, context));
            }
            ClaimCategory::Performance => {
                evidence.extend(self.gather_performance_evidence(claim, context));
            }
            ClaimCategory::Security => {
                evidence.extend(self.gather_security_evidence(claim, context));
            }
        }

        evidence
    }

    fn gather_test_status_evidence(
        &self,
        claim: &Claim,
        context: &RepositoryContext,
    ) -> Vec<EvidenceResult> {
        let mut evidence = Vec::new();

        // Evidence 1: Git history - check for subsequent test fixes
        if let Some(ref commits) = context.subsequent_commits {
            let test_fixes = commits
                .iter()
                .filter(|msg| {
                    let lower = msg.to_lowercase();
                    lower.contains("fix") && (lower.contains("test") || lower.contains("ignore"))
                })
                .count();

            let supports_claim = test_fixes == 0;
            let confidence = if test_fixes > 0 { 0.85 } else { 0.6 };

            evidence.push(EvidenceResult {
                source: EvidenceSource::GitHistory,
                supports_claim,
                confidence,
                details: if test_fixes > 0 {
                    format!("{} subsequent test fixes found", test_fixes)
                } else {
                    "No subsequent test fixes found".to_string()
                },
                timestamp: context.latest_commit_timestamp,
            });
        }

        // Evidence 2: Test execution results
        if let Some((passing, ignored)) = context.test_results {
            let all_passing = passing && ignored == 0;
            let supports_claim = if claim.is_absolute {
                all_passing // Absolute claim requires all passing
            } else {
                passing // Qualified claim just needs passing (some ignored OK)
            };

            evidence.push(EvidenceResult {
                source: EvidenceSource::TestExecution,
                supports_claim,
                confidence: 0.9, // High confidence in test execution
                details: if all_passing {
                    "All tests passing".to_string()
                } else if ignored > 0 {
                    format!("{} tests ignored", ignored)
                } else {
                    "Tests failing".to_string()
                },
                timestamp: None,
            });
        }

        evidence
    }

    fn gather_documentation_evidence(
        &self,
        _claim: &Claim,
        context: &RepositoryContext,
    ) -> Vec<EvidenceResult> {
        let mut evidence = Vec::new();

        // Evidence 1: Git history - check for subsequent doc fixes
        if let Some(ref commits) = context.subsequent_commits {
            let doc_fixes = commits
                .iter()
                .filter(|msg| {
                    let lower = msg.to_lowercase();
                    lower.contains("docs") || lower.contains("link") || lower.contains("404")
                })
                .count();

            let supports_claim = doc_fixes == 0;

            evidence.push(EvidenceResult {
                source: EvidenceSource::GitHistory,
                supports_claim,
                confidence: 0.75,
                details: if doc_fixes > 0 {
                    format!("{} subsequent documentation fixes found", doc_fixes)
                } else {
                    "No subsequent documentation fixes found".to_string()
                },
                timestamp: context.latest_commit_timestamp,
            });
        }

        // Evidence 2: Link validation
        if let Some(broken_links) = context.broken_links_count {
            let supports_claim = broken_links == 0;

            evidence.push(EvidenceResult {
                source: EvidenceSource::LinkValidation,
                supports_claim,
                confidence: 0.9, // High confidence in link validation
                details: if broken_links > 0 {
                    format!("{} broken links found", broken_links)
                } else {
                    "All links valid".to_string()
                },
                timestamp: None,
            });
        }

        evidence
    }

    fn gather_coverage_evidence(
        &self,
        claim: &Claim,
        context: &RepositoryContext,
    ) -> Vec<EvidenceResult> {
        let mut evidence = Vec::new();

        // Evidence 1: Git history - check for subsequent coverage fixes
        if let Some(ref commits) = context.subsequent_commits {
            let coverage_fixes = commits
                .iter()
                .filter(|msg| {
                    let lower = msg.to_lowercase();
                    lower.contains("coverage")
                        && (lower.contains("fix") || lower.contains("regress"))
                })
                .count();

            let supports_claim = coverage_fixes == 0;

            evidence.push(EvidenceResult {
                source: EvidenceSource::GitHistory,
                supports_claim,
                confidence: 0.75,
                details: if coverage_fixes > 0 {
                    format!("{} subsequent coverage fixes found", coverage_fixes)
                } else {
                    "No subsequent coverage fixes found".to_string()
                },
                timestamp: context.latest_commit_timestamp,
            });
        }

        // Evidence 2: Coverage report comparison
        if let Some(actual_coverage) = context.actual_coverage {
            if let Some(claimed_coverage) = claim.numeric_value {
                let diff = (actual_coverage - claimed_coverage).abs();
                let supports_claim = diff <= 2.0; // Within 2% tolerance

                evidence.push(EvidenceResult {
                    source: EvidenceSource::CoverageReport,
                    supports_claim,
                    confidence: 0.95, // Very high confidence in coverage reports
                    details: format!(
                        "Claimed: {:.1}%, Actual: {:.1}%",
                        claimed_coverage, actual_coverage
                    ),
                    timestamp: None,
                });
            }
        } else if let Some(ref error) = context.coverage_error {
            // Coverage tool failed
            evidence.push(EvidenceResult {
                source: EvidenceSource::CoverageReport,
                supports_claim: false, // Cannot verify
                confidence: 0.5,
                details: format!("Coverage tool error: {}", error),
                timestamp: None,
            });
        }

        evidence
    }

    fn gather_feature_completion_evidence(
        &self,
        _claim: &Claim,
        context: &RepositoryContext,
    ) -> Vec<EvidenceResult> {
        let mut evidence = Vec::new();

        // Evidence 1: Git history - check for subsequent fixes/reverts
        if let Some(ref commits) = context.subsequent_commits {
            let fixes = commits
                .iter()
                .filter(|msg| {
                    let lower = msg.to_lowercase();
                    lower.contains("fix") || lower.contains("bug") || lower.contains("revert")
                })
                .count();

            let supports_claim = fixes == 0;

            evidence.push(EvidenceResult {
                source: EvidenceSource::GitHistory,
                supports_claim,
                confidence: 0.8,
                details: if fixes > 0 {
                    format!("{} subsequent fixes found", fixes)
                } else {
                    "No subsequent fixes found".to_string()
                },
                timestamp: context.latest_commit_timestamp,
            });
        }

        evidence
    }

    fn gather_migration_evidence(
        &self,
        _claim: &Claim,
        context: &RepositoryContext,
    ) -> Vec<EvidenceResult> {
        let mut evidence = Vec::new();

        // Evidence 1: Git history - check for rollbacks
        if let Some(ref commits) = context.subsequent_commits {
            let rollbacks = commits
                .iter()
                .filter(|msg| {
                    let lower = msg.to_lowercase();
                    lower.contains("revert") || lower.contains("rollback")
                })
                .count();

            let supports_claim = rollbacks == 0;

            evidence.push(EvidenceResult {
                source: EvidenceSource::GitHistory,
                supports_claim,
                confidence: 0.85,
                details: if rollbacks > 0 {
                    format!("{} rollback commits found", rollbacks)
                } else {
                    "No rollbacks found".to_string()
                },
                timestamp: context.latest_commit_timestamp,
            });
        }

        // Evidence 2: Code grep - check if old system still referenced
        if let Some((ref old_system, count)) = context.code_grep_results {
            let supports_claim = count == 0;

            evidence.push(EvidenceResult {
                source: EvidenceSource::CodeGrep,
                supports_claim,
                confidence: 0.8,
                details: if count > 0 {
                    format!("{} files still reference '{}'", count, old_system)
                } else {
                    format!("No references to '{}' found", old_system)
                },
                timestamp: None,
            });
        }

        evidence
    }

    fn gather_bugfix_evidence(
        &self,
        claim: &Claim,
        context: &RepositoryContext,
    ) -> Vec<EvidenceResult> {
        let mut evidence = Vec::new();

        // Evidence 1: Issue tracker status
        if let Some(issue_num) = claim.issue_number {
            if let Some(ref status) = context.issue_status {
                let is_closed = status == "closed";
                let is_reopened = status == "reopened" || status == "open";

                evidence.push(EvidenceResult {
                    source: EvidenceSource::IssueTracker,
                    supports_claim: is_closed,
                    confidence: 0.85,
                    details: if is_reopened {
                        format!("Issue #{} was reopened", issue_num)
                    } else if is_closed {
                        format!("Issue #{} is closed", issue_num)
                    } else {
                        format!("Issue #{} status: {}", issue_num, status)
                    },
                    timestamp: None,
                });
            }
        }

        // Evidence 2: Git history - check for regressions
        if let Some(ref commits) = context.subsequent_commits {
            let regressions = commits
                .iter()
                .filter(|msg| {
                    let lower = msg.to_lowercase();
                    lower.contains("regression") || lower.contains("re-fix")
                })
                .count();

            let supports_claim = regressions == 0;

            evidence.push(EvidenceResult {
                source: EvidenceSource::GitHistory,
                supports_claim,
                confidence: 0.8,
                details: if regressions > 0 {
                    format!("{} regression commits found", regressions)
                } else {
                    "No regressions found".to_string()
                },
                timestamp: context.latest_commit_timestamp,
            });
        }

        evidence
    }

    fn gather_performance_evidence(
        &self,
        claim: &Claim,
        context: &RepositoryContext,
    ) -> Vec<EvidenceResult> {
        let mut evidence = Vec::new();

        // Evidence 1: Benchmark results
        if let Some(ref benchmark_data) = context.benchmark_results {
            evidence.push(EvidenceResult {
                source: EvidenceSource::BenchmarkResults,
                supports_claim: true, // Benchmark data exists
                confidence: 0.9,
                details: format!("Benchmark data: {}", benchmark_data),
                timestamp: None,
            });
        } else {
            // No benchmark data available
            evidence.push(EvidenceResult {
                source: EvidenceSource::BenchmarkResults,
                supports_claim: false, // Cannot verify without data
                confidence: 0.7,
                details: if claim.numeric_value.is_some() {
                    "No benchmark data found to support numeric claim".to_string()
                } else {
                    "No benchmark data available".to_string()
                },
                timestamp: None,
            });
        }

        // Evidence 2: Git history - check for performance regressions
        if let Some(ref commits) = context.subsequent_commits {
            let perf_regressions = commits
                .iter()
                .filter(|msg| {
                    let lower = msg.to_lowercase();
                    (lower.contains("perf") || lower.contains("performance"))
                        && (lower.contains("regress")
                            || lower.contains("slow")
                            || lower.contains("timeout"))
                })
                .count();

            let supports_claim = perf_regressions == 0;

            evidence.push(EvidenceResult {
                source: EvidenceSource::GitHistory,
                supports_claim,
                confidence: 0.75,
                details: if perf_regressions > 0 {
                    format!("{} performance regression commits found", perf_regressions)
                } else {
                    "No performance regressions found".to_string()
                },
                timestamp: context.latest_commit_timestamp,
            });
        }

        evidence
    }

    fn gather_security_evidence(
        &self,
        _claim: &Claim,
        context: &RepositoryContext,
    ) -> Vec<EvidenceResult> {
        let mut evidence = Vec::new();

        // Evidence 1: Cargo audit results
        if let Some(vuln_count) = context.vulnerabilities_count {
            let supports_claim = vuln_count == 0;

            evidence.push(EvidenceResult {
                source: EvidenceSource::CargoAudit,
                supports_claim,
                confidence: 0.95, // Very high confidence in cargo audit
                details: if vuln_count > 0 {
                    format!("{} vulnerabilities found", vuln_count)
                } else {
                    "No vulnerabilities found".to_string()
                },
                timestamp: None,
            });
        }

        // Evidence 2: Git history - check for subsequent security fixes
        if let Some(ref commits) = context.subsequent_commits {
            let security_fixes = commits
                .iter()
                .filter(|msg| {
                    let lower = msg.to_lowercase();
                    lower.contains("security") || lower.contains("vuln") || lower.contains("cve")
                })
                .count();

            let supports_claim = security_fixes == 0;

            evidence.push(EvidenceResult {
                source: EvidenceSource::GitHistory,
                supports_claim,
                confidence: 0.85,
                details: if security_fixes > 0 {
                    format!("{} subsequent security fixes found", security_fixes)
                } else {
                    "No subsequent security fixes found".to_string()
                },
                timestamp: context.latest_commit_timestamp,
            });
        }

        evidence
    }
}

impl Default for EvidenceGatherer {
    fn default() -> Self {
        Self::new()
    }
}

// Supporting types for repository context
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub message: String,
    pub timestamp: i64,
    pub author: String,
}

#[derive(Debug, Clone, Default)]
pub struct TestExecutionInfo {
    pub has_results: bool,
    pub passed_count: usize,
    pub failed_count: usize,
    pub ignored_count: usize,
}

// RepositoryContext: Mock-friendly context for evidence gathering
#[derive(Debug, Clone)]
pub struct RepositoryContext {
    pub subsequent_commits: Option<Vec<String>>,
    pub test_results: Option<(bool, usize)>, // (passing, ignored_count)
    pub actual_coverage: Option<f64>,
    pub coverage_error: Option<String>,
    pub broken_links_count: Option<usize>,
    pub vulnerabilities_count: Option<usize>,
    pub benchmark_results: Option<String>,
    pub issue_status: Option<String>,
    pub code_grep_results: Option<(String, usize)>, // (search_term, count)
    pub latest_commit_timestamp: Option<i64>,
    pub commit_timestamps: Option<Vec<i64>>,

    // Real repository data (populated by from_path)
    git_repo: Option<PathBuf>,
    test_files: Vec<PathBuf>,
    coverage_path: Option<PathBuf>,
    test_results_path: Option<PathBuf>,
    repo_path: PathBuf, // Original path passed to from_path
}

impl RepositoryContext {
    /// Create a mock context for testing
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
    pub fn from_path(path: &Path) -> Result<Self> {
        Self::from_path_with_config(path, false)
    }

    /// Build repository context with configuration options
    ///
    /// # Arguments
    /// * `path` - Repository path
    /// * `deep` - If true, fetch entire git history; if false, fetch recent commits only (last 30 days)
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
    pub fn has_git_history(&self) -> bool {
        self.git_repo.is_some()
    }

    /// Get recent commits from git history
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

    /// Get test files found in repository
    pub fn get_test_files(&self) -> Vec<PathBuf> {
        self.test_files.clone()
    }

    /// Check if coverage report exists
    pub fn has_coverage_report(&self) -> bool {
        self.coverage_path.is_some()
    }

    /// Get coverage percentage from report
    pub fn get_coverage_percentage(&self) -> f64 {
        let Some(ref coverage_path) = self.coverage_path else {
            return 0.0;
        };

        Self::parse_coverage_report(coverage_path).unwrap_or(0.0)
    }

    /// Get test execution information
    pub fn get_test_execution_info(&self) -> TestExecutionInfo {
        let Some(ref test_results_path) = self.test_results_path else {
            return TestExecutionInfo::default();
        };

        Self::parse_test_results(test_results_path).unwrap_or_default()
    }

    /// Search codebase for pattern using grep
    pub fn grep_codebase(&self, pattern: &str) -> Vec<PathBuf> {
        Self::grep_directory(&self.repo_path, pattern).unwrap_or_default()
    }

    // Helper methods

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

    pub fn with_coverage(mut self, coverage: f64) -> Self {
        self.actual_coverage = Some(coverage);
        self
    }

    pub fn with_subsequent_commits(mut self, commits: Vec<String>) -> Self {
        self.subsequent_commits = Some(commits);
        self
    }

    pub fn with_test_results(mut self, passing: bool, ignored: usize) -> Self {
        self.test_results = Some((passing, ignored));
        self
    }

    pub fn with_broken_links(mut self, count: usize) -> Self {
        self.broken_links_count = Some(count);
        self
    }

    pub fn with_vulnerabilities(mut self, count: usize) -> Self {
        self.vulnerabilities_count = Some(count);
        self
    }

    pub fn with_benchmarks(mut self, data: Option<String>) -> Self {
        self.benchmark_results = data;
        self
    }

    pub fn with_issue_status(mut self, _issue_num: u32, status: &str) -> Self {
        self.issue_status = Some(status.to_string());
        self
    }

    pub fn with_code_grep_results(mut self, search_term: &str, count: usize) -> Self {
        self.code_grep_results = Some((search_term.to_string(), count));
        self
    }

    pub fn with_commit_timestamps(mut self, timestamps: Vec<i64>) -> Self {
        self.commit_timestamps = Some(timestamps.clone());
        self.latest_commit_timestamp = timestamps.last().copied();
        self
    }

    pub fn with_coverage_error(mut self, error: &str) -> Self {
        self.coverage_error = Some(error.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_gatherer_compiles() {
        let gatherer = EvidenceGatherer::new();
        assert!(gatherer.git_history_window_days == 30);
    }

    #[test]
    fn test_repository_context_builder() {
        let context = RepositoryContext::new_mock()
            .with_coverage(85.0)
            .with_vulnerabilities(0);

        assert_eq!(context.actual_coverage, Some(85.0));
        assert_eq!(context.vulnerabilities_count, Some(0));
    }
}
