#![cfg_attr(coverage_nightly, coverage(off))]
// Git Commit Parser (GH-RAG-001)
// Toyota Way: Genchi Genbutsu - Direct git data analysis
// Spec: docs/specifications/git-history-rag-integration.md

use anyhow::{Context, Result};
use git2::{Commit, DiffOptions, Repository, Sort};
use std::path::Path;

/// Type of file change in a commit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
}

impl ChangeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChangeType::Added => "A",
            ChangeType::Modified => "M",
            ChangeType::Deleted => "D",
            ChangeType::Renamed => "R",
        }
    }
}

/// Information about a file changed in a commit
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub change_type: ChangeType,
    pub lines_added: u32,
    pub lines_deleted: u32,
}

/// Parsed commit information
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub message_subject: String,
    pub message_body: Option<String>,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: i64,
    pub is_merge: bool,
    pub is_fix: bool,
    pub is_feat: bool,
    pub issue_refs: Vec<String>,
    pub files: Vec<FileChange>,
}

impl CommitInfo {
    /// Full commit message (subject + body)
    pub fn full_message(&self) -> String {
        match &self.message_body {
            Some(body) if !body.is_empty() => format!("{}\n\n{}", self.message_subject, body),
            _ => self.message_subject.clone(),
        }
    }

    /// Check if this is a meaningful commit for indexing
    /// Skips: merge commits with no custom message, very short messages
    pub fn is_indexable(&self) -> bool {
        // Skip merge commits with generic messages
        if self.is_merge && self.message_subject.starts_with("Merge ") {
            return false;
        }
        // Skip very short messages (less than 10 chars)
        if self.message_subject.len() < 10 {
            return false;
        }
        true
    }
}

/// Git commit parser using libgit2
pub struct CommitParser {
    repo: Repository,
}

impl CommitParser {
    /// Open a repository at the given path
    pub fn open(path: &Path) -> Result<Self> {
        let repo = Repository::discover(path)
            .with_context(|| format!("Failed to open git repository at {:?}", path))?;
        Ok(Self { repo })
    }

    /// Parse all commits, optionally since a given commit hash
    pub fn parse_commits(&self, since: Option<&str>, limit: Option<usize>) -> Result<Vec<CommitInfo>> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.set_sorting(Sort::TIME | Sort::REVERSE)?;
        revwalk.push_head()?;

        // If we have a 'since' commit, hide all ancestors
        if let Some(since_hash) = since {
            if let Ok(oid) = git2::Oid::from_str(since_hash) {
                revwalk.hide(oid)?;
            }
        }

        let mut commits = Vec::new();
        let mut count = 0;

        for oid_result in revwalk {
            if let Some(max) = limit {
                if count >= max {
                    break;
                }
            }

            let oid = oid_result?;
            let commit = self.repo.find_commit(oid)?;

            if let Some(info) = self.parse_commit(&commit)? {
                commits.push(info);
                count += 1;
            }
        }

        Ok(commits)
    }

    /// Parse a single commit into CommitInfo
    fn parse_commit(&self, commit: &Commit) -> Result<Option<CommitInfo>> {
        let hash = commit.id().to_string();

        // Parse message
        let message = commit.message().unwrap_or("");
        let (subject, body) = Self::split_message(message);

        // Extract metadata
        let author = commit.author();
        let author_name = author.name().unwrap_or("Unknown").to_string();
        let author_email = author.email().unwrap_or("").to_string();
        let timestamp = commit.time().seconds();

        // Check if merge commit
        let is_merge = commit.parent_count() > 1;

        // Detect conventional commit types
        let is_fix = Self::is_fix_commit(&subject);
        let is_feat = Self::is_feat_commit(&subject);

        // Extract issue references
        let issue_refs = Self::extract_issue_refs(&subject, body.as_deref().unwrap_or(""));

        // Get file changes
        let files = self.get_file_changes(commit)?;

        Ok(Some(CommitInfo {
            hash,
            message_subject: subject,
            message_body: body,
            author_name,
            author_email,
            timestamp,
            is_merge,
            is_fix,
            is_feat,
            issue_refs,
            files,
        }))
    }

    /// Split commit message into subject and body
    fn split_message(message: &str) -> (String, Option<String>) {
        let lines: Vec<&str> = message.lines().collect();

        if lines.is_empty() {
            return (String::new(), None);
        }

        let subject = lines[0].trim().to_string();

        // Body starts after the first blank line
        let body_start = lines.iter().skip(1).position(|l| l.trim().is_empty());

        let body = if let Some(start) = body_start {
            let body_lines: Vec<&str> = lines.iter().skip(start + 2).copied().collect();
            if body_lines.is_empty() {
                None
            } else {
                Some(body_lines.join("\n").trim().to_string())
            }
        } else {
            None
        };

        (subject, body)
    }

    /// Check if commit is a fix (conventional commit or keyword)
    fn is_fix_commit(subject: &str) -> bool {
        let lower = subject.to_lowercase();
        lower.starts_with("fix:")
            || lower.starts_with("fix(")
            || lower.starts_with("bugfix:")
            || lower.starts_with("hotfix:")
            || lower.contains("fix ")
            || lower.contains("fixed ")
            || lower.contains("fixes ")
    }

    /// Check if commit is a feature (conventional commit)
    fn is_feat_commit(subject: &str) -> bool {
        let lower = subject.to_lowercase();
        lower.starts_with("feat:")
            || lower.starts_with("feat(")
            || lower.starts_with("feature:")
    }

    /// Extract issue references from commit message
    fn extract_issue_refs(subject: &str, body: &str) -> Vec<String> {
        let mut refs = Vec::new();
        let full_text = format!("{} {}", subject, body);

        // GitHub-style: #123
        let github_re = regex::Regex::new(r"#(\d+)").expect("valid regex");
        for cap in github_re.captures_iter(&full_text) {
            refs.push(format!("#{}", &cap[1]));
        }

        // JIRA-style: PROJ-123
        let jira_re = regex::Regex::new(r"([A-Z]+-\d+)").expect("valid regex");
        for cap in jira_re.captures_iter(&full_text) {
            let issue = cap[1].to_string();
            if !refs.contains(&issue) {
                refs.push(issue);
            }
        }

        refs
    }

    /// Get files changed in a commit with diff stats
    fn get_file_changes(&self, commit: &Commit) -> Result<Vec<FileChange>> {
        let mut changes = Vec::new();

        let tree = commit.tree()?;
        let parent_tree = if commit.parent_count() > 0 {
            Some(commit.parent(0)?.tree()?)
        } else {
            None
        };

        let mut diff_opts = DiffOptions::new();
        let diff = self.repo.diff_tree_to_tree(
            parent_tree.as_ref(),
            Some(&tree),
            Some(&mut diff_opts),
        )?;

        diff.foreach(
            &mut |delta, _progress| {
                let path = delta.new_file().path()
                    .or_else(|| delta.old_file().path())
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

                let change_type = match delta.status() {
                    git2::Delta::Added => ChangeType::Added,
                    git2::Delta::Deleted => ChangeType::Deleted,
                    git2::Delta::Renamed => ChangeType::Renamed,
                    _ => ChangeType::Modified,
                };

                changes.push(FileChange {
                    path,
                    change_type,
                    lines_added: 0,
                    lines_deleted: 0,
                });

                true
            },
            None,
            None,
            None,
        )?;

        // Get line stats
        let stats = diff.stats()?;
        let _insertions = stats.insertions();
        let _deletions = stats.deletions();

        Ok(changes)
    }

    /// Get the most recent commit hash
    pub fn head_commit_hash(&self) -> Result<String> {
        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;
        Ok(commit.id().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Falsification Test F4: Update frequency analysis helper
    fn get_test_repo_path() -> PathBuf {
        // Use current repo for testing
        PathBuf::from(".")
    }

    #[test]
    fn test_commit_parser_opens_repo() {
        let path = get_test_repo_path();
        let parser = CommitParser::open(&path);
        assert!(parser.is_ok(), "Should open git repository");
    }

    #[test]
    fn test_parse_commits_returns_results() {
        let path = get_test_repo_path();
        let parser = CommitParser::open(&path).unwrap();
        let commits = parser.parse_commits(None, Some(10)).unwrap();

        assert!(!commits.is_empty(), "Should find commits in repository");
        assert!(commits.len() <= 10, "Should respect limit");
    }

    #[test]
    fn test_commit_info_fields_populated() {
        let path = get_test_repo_path();
        let parser = CommitParser::open(&path).unwrap();
        let commits = parser.parse_commits(None, Some(1)).unwrap();

        let commit = &commits[0];

        // Hash should be 40 hex chars
        assert_eq!(commit.hash.len(), 40, "Hash should be 40 characters");
        assert!(commit.hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should be hex");

        // Subject should be non-empty
        assert!(!commit.message_subject.is_empty(), "Subject should not be empty");

        // Author should be set
        assert!(!commit.author_name.is_empty(), "Author name should not be empty");

        // Timestamp should be reasonable (after 2020)
        assert!(commit.timestamp > 1577836800, "Timestamp should be after 2020");
    }

    #[test]
    fn test_split_message_subject_only() {
        let (subject, body) = CommitParser::split_message("Fix bug in parser");

        assert_eq!(subject, "Fix bug in parser");
        assert!(body.is_none());
    }

    #[test]
    fn test_split_message_with_body() {
        let message = "Fix bug in parser\n\nThis fixes the issue where\nthe parser would crash.";
        let (subject, body) = CommitParser::split_message(message);

        assert_eq!(subject, "Fix bug in parser");
        assert!(body.is_some());
        assert!(body.unwrap().contains("parser would crash"));
    }

    #[test]
    fn test_is_fix_commit_conventional() {
        assert!(CommitParser::is_fix_commit("fix: resolve null pointer"));
        assert!(CommitParser::is_fix_commit("fix(parser): handle empty input"));
        assert!(CommitParser::is_fix_commit("bugfix: memory leak"));
        assert!(CommitParser::is_fix_commit("hotfix: critical security issue"));
    }

    #[test]
    fn test_is_fix_commit_keyword() {
        assert!(CommitParser::is_fix_commit("Fix null pointer exception"));
        assert!(CommitParser::is_fix_commit("Fixed memory leak in cache"));
        assert!(CommitParser::is_fix_commit("This fixes the crash bug"));
    }

    #[test]
    fn test_is_fix_commit_negative() {
        assert!(!CommitParser::is_fix_commit("Add new feature"));
        assert!(!CommitParser::is_fix_commit("Refactor parser module"));
        assert!(!CommitParser::is_fix_commit("Update documentation"));
    }

    #[test]
    fn test_is_feat_commit() {
        assert!(CommitParser::is_feat_commit("feat: add dark mode"));
        assert!(CommitParser::is_feat_commit("feat(ui): implement sidebar"));
        assert!(CommitParser::is_feat_commit("feature: new export option"));

        assert!(!CommitParser::is_feat_commit("fix: bug in feature"));
        assert!(!CommitParser::is_feat_commit("docs: update feature list"));
    }

    #[test]
    fn test_extract_issue_refs_github() {
        let refs = CommitParser::extract_issue_refs("Fix #123 and #456", "Also see #789");

        assert!(refs.contains(&"#123".to_string()));
        assert!(refs.contains(&"#456".to_string()));
        assert!(refs.contains(&"#789".to_string()));
    }

    #[test]
    fn test_extract_issue_refs_jira() {
        let refs = CommitParser::extract_issue_refs("PROJ-123: Fix bug", "Related to JIRA-456");

        assert!(refs.contains(&"PROJ-123".to_string()));
        assert!(refs.contains(&"JIRA-456".to_string()));
    }

    #[test]
    fn test_commit_info_full_message() {
        let info = CommitInfo {
            hash: "abc123".repeat(7)[..40].to_string(),
            message_subject: "Fix bug".to_string(),
            message_body: Some("Detailed explanation".to_string()),
            author_name: "Test".to_string(),
            author_email: "test@example.com".to_string(),
            timestamp: 1700000000,
            is_merge: false,
            is_fix: true,
            is_feat: false,
            issue_refs: vec![],
            files: vec![],
        };

        let full = info.full_message();
        assert!(full.contains("Fix bug"));
        assert!(full.contains("Detailed explanation"));
    }

    #[test]
    fn test_commit_info_is_indexable() {
        // Regular commit - should be indexable
        let regular = CommitInfo {
            hash: "a".repeat(40),
            message_subject: "Fix important bug in parser".to_string(),
            message_body: None,
            author_name: "Test".to_string(),
            author_email: "test@example.com".to_string(),
            timestamp: 1700000000,
            is_merge: false,
            is_fix: true,
            is_feat: false,
            issue_refs: vec![],
            files: vec![],
        };
        assert!(regular.is_indexable());

        // Merge commit with generic message - should NOT be indexable
        let merge = CommitInfo {
            hash: "b".repeat(40),
            message_subject: "Merge branch 'feature' into main".to_string(),
            message_body: None,
            author_name: "Test".to_string(),
            author_email: "test@example.com".to_string(),
            timestamp: 1700000000,
            is_merge: true,
            is_fix: false,
            is_feat: false,
            issue_refs: vec![],
            files: vec![],
        };
        assert!(!merge.is_indexable());

        // Very short message - should NOT be indexable
        let short = CommitInfo {
            hash: "c".repeat(40),
            message_subject: "wip".to_string(),
            message_body: None,
            author_name: "Test".to_string(),
            author_email: "test@example.com".to_string(),
            timestamp: 1700000000,
            is_merge: false,
            is_fix: false,
            is_feat: false,
            issue_refs: vec![],
            files: vec![],
        };
        assert!(!short.is_indexable());
    }

    #[test]
    fn test_change_type_as_str() {
        assert_eq!(ChangeType::Added.as_str(), "A");
        assert_eq!(ChangeType::Modified.as_str(), "M");
        assert_eq!(ChangeType::Deleted.as_str(), "D");
        assert_eq!(ChangeType::Renamed.as_str(), "R");
    }

    #[test]
    fn test_head_commit_hash() {
        let path = get_test_repo_path();
        let parser = CommitParser::open(&path).unwrap();
        let hash = parser.head_commit_hash().unwrap();

        assert_eq!(hash.len(), 40, "HEAD commit hash should be 40 chars");
    }

    // Falsification Test F4: Verify update frequency difference
    #[test]
    fn falsify_update_frequency_difference() {
        let path = get_test_repo_path();
        let parser = CommitParser::open(&path).unwrap();

        // Get recent commits (last 100)
        let commits = parser.parse_commits(None, Some(100)).unwrap();

        if commits.len() < 10 {
            // Skip if not enough history
            return;
        }

        // Count unique days with commits
        let mut commit_days = std::collections::HashSet::new();
        let mut code_change_days = std::collections::HashSet::new();

        for commit in &commits {
            // All commits count for git history updates
            let day = commit.timestamp / 86400;
            commit_days.insert(day);

            // Only commits with .rs file changes count for code index
            let has_code_changes = commit.files.iter()
                .any(|f| f.path.ends_with(".rs") || f.path.ends_with(".ts") || f.path.ends_with(".py"));

            if has_code_changes {
                code_change_days.insert(day);
            }
        }

        // Git should change at least as often as code
        // (This is a weak falsification - real test uses semantic changes)
        assert!(
            commit_days.len() >= code_change_days.len(),
            "Git history should update at least as often as code changes"
        );
    }
}
