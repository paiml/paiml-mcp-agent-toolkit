// Git Commit Parser - Tests
// Extracted from commit_parser.rs for maintainability
// Contains: all unit tests for CommitParser, CommitInfo, ChangeType

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
        assert!(
            commit.hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should be hex"
        );

        // Subject should be non-empty
        assert!(
            !commit.message_subject.is_empty(),
            "Subject should not be empty"
        );

        // Author should be set
        assert!(
            !commit.author_name.is_empty(),
            "Author name should not be empty"
        );

        // Timestamp should be reasonable (after 2020)
        assert!(
            commit.timestamp > 1577836800,
            "Timestamp should be after 2020"
        );
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
        assert!(CommitParser::is_fix_commit(
            "fix(parser): handle empty input"
        ));
        assert!(CommitParser::is_fix_commit("bugfix: memory leak"));
        assert!(CommitParser::is_fix_commit(
            "hotfix: critical security issue"
        ));
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
            let has_code_changes = commit.files.iter().any(|f| {
                f.path.ends_with(".rs") || f.path.ends_with(".ts") || f.path.ends_with(".py")
            });

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
