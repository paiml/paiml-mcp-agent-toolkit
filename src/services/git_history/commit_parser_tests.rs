// Git Commit Parser - Tests
// Extracted from commit_parser.rs for maintainability
// Contains: all unit tests for CommitParser, CommitInfo, ChangeType

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{RepositoryInitOptions, Signature, Time};
    use std::path::PathBuf;
    use tempfile::TempDir;

    const SECONDS_PER_DAY: i64 = 86_400;
    /// 2023-11-14T22:13:20Z. Fixed so day bucketing and timestamp assertions are exact.
    const FIXTURE_BASE_TS: i64 = 1_700_000_000;
    const FIXTURE_AUTHOR: &str = "Fixture Author";
    const FIXTURE_EMAIL: &str = "fixture@example.com";

    /// A throwaway git repository with a known, fixed history.
    struct FixtureRepo {
        // Held only to keep the directory alive for the lifetime of the fixture.
        _dir: TempDir,
        path: PathBuf,
        /// Commit hashes, oldest first.
        hashes: Vec<String>,
    }

    /// Build a self-contained repository: three commits on three distinct days,
    /// exactly one of which touches a source file.
    ///
    /// These tests used to resolve the repository through `PathBuf::from(".")`, i.e.
    /// the process-wide CWD. Other tests in this binary (`src/scaffold/tests.rs`,
    /// `src/models/deep_context_config_tests.rs`, ...) `set_current_dir` into a
    /// `TempDir` and never restore it, so by the time these ran the CWD could point
    /// at a deleted directory and `Repository::discover(".")` failed with ENOENT.
    /// A fixture removes that race and also removes the dependency on whatever
    /// history the surrounding checkout happens to have.
    fn build_fixture_repo() -> FixtureRepo {
        let dir = TempDir::new().expect("create temp dir");
        let path = dir.path().to_path_buf();

        let mut init_opts = RepositoryInitOptions::new();
        init_opts.external_template(false).initial_head("main");
        let repo = Repository::init_opts(&path, &init_opts).expect("init fixture repo");

        // (path, subject, timestamp) - only the middle commit touches code.
        let planned: [(&str, &str, i64); 3] = [
            ("README.md", "docs: describe the fixture", FIXTURE_BASE_TS),
            (
                "src/lib.rs",
                "feat: add fixture library",
                FIXTURE_BASE_TS + SECONDS_PER_DAY,
            ),
            (
                "docs/notes.txt",
                "docs: add release notes",
                FIXTURE_BASE_TS + 2 * SECONDS_PER_DAY,
            ),
        ];

        let mut hashes = Vec::with_capacity(planned.len());
        for (rel_path, subject, timestamp) in planned {
            let file = path.join(rel_path);
            std::fs::create_dir_all(file.parent().expect("fixture file has a parent"))
                .expect("create fixture subdirectory");
            std::fs::write(&file, format!("{subject}\n")).expect("write fixture file");

            let mut index = repo.index().expect("open fixture index");
            index
                .add_path(Path::new(rel_path))
                .expect("stage fixture file");
            index.write().expect("write fixture index");
            let tree_id = index.write_tree().expect("write fixture tree");
            let tree = repo.find_tree(tree_id).expect("find fixture tree");

            // Explicit signature: never reads the ambient git config, so author and
            // timestamp are identical on every machine.
            let signature = Signature::new(FIXTURE_AUTHOR, FIXTURE_EMAIL, &Time::new(timestamp, 0))
                .expect("build fixture signature");

            let parents: Vec<Commit> = repo
                .head()
                .ok()
                .and_then(|head| head.peel_to_commit().ok())
                .into_iter()
                .collect();
            let parent_refs: Vec<&Commit> = parents.iter().collect();

            let oid = repo
                .commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    subject,
                    &tree,
                    &parent_refs,
                )
                .expect("create fixture commit");
            hashes.push(oid.to_string());
        }

        FixtureRepo {
            _dir: dir,
            path,
            hashes,
        }
    }

    #[test]
    fn test_commit_parser_opens_repo() {
        let fixture = build_fixture_repo();
        let parser = CommitParser::open(&fixture.path);
        assert!(parser.is_ok(), "Should open git repository");
    }

    #[test]
    fn test_parse_commits_returns_results() {
        let fixture = build_fixture_repo();
        let parser = CommitParser::open(&fixture.path).unwrap();

        // A limit above the history size must return the whole history, oldest first.
        let commits = parser.parse_commits(None, Some(10)).unwrap();
        let hashes: Vec<String> = commits.iter().map(|c| c.hash.clone()).collect();
        assert_eq!(
            hashes, fixture.hashes,
            "Should return every commit in the repository, oldest first"
        );

        // A limit below the history size must truncate, keeping the same order.
        let limited = parser.parse_commits(None, Some(2)).unwrap();
        let limited_hashes: Vec<String> = limited.iter().map(|c| c.hash.clone()).collect();
        assert_eq!(
            limited_hashes,
            fixture.hashes[..2].to_vec(),
            "Should respect limit"
        );
    }

    #[test]
    fn test_commit_info_fields_populated() {
        let fixture = build_fixture_repo();
        let parser = CommitParser::open(&fixture.path).unwrap();
        let commits = parser.parse_commits(None, Some(1)).unwrap();

        let commit = &commits[0];

        // Hash should be the oldest commit, as 40 hex chars
        assert_eq!(
            commit.hash, fixture.hashes[0],
            "Should start from the oldest commit"
        );
        assert_eq!(commit.hash.len(), 40, "Hash should be 40 characters");
        assert!(
            commit.hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should be hex"
        );

        assert_eq!(
            commit.message_subject, "docs: describe the fixture",
            "Subject should be the commit's first line"
        );
        assert_eq!(commit.message_body, None, "Fixture commit has no body");
        assert_eq!(commit.author_name, FIXTURE_AUTHOR);
        assert_eq!(commit.author_email, FIXTURE_EMAIL);
        assert_eq!(
            commit.timestamp, FIXTURE_BASE_TS,
            "Timestamp should be the author time of the commit"
        );
        assert!(!commit.is_merge, "Root commit is not a merge");

        // The root commit adds exactly the one file it introduced.
        let files: Vec<(&str, ChangeType)> = commit
            .files
            .iter()
            .map(|f| (f.path.as_str(), f.change_type))
            .collect();
        assert_eq!(files, vec![("README.md", ChangeType::Added)]);
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
        let fixture = build_fixture_repo();
        let parser = CommitParser::open(&fixture.path).unwrap();
        let hash = parser.head_commit_hash().unwrap();

        assert_eq!(hash.len(), 40, "HEAD commit hash should be 40 chars");
        assert_eq!(
            hash,
            *fixture.hashes.last().expect("fixture has commits"),
            "HEAD should be the newest commit, not the oldest"
        );
    }

    // Falsification Test F4: Verify update frequency difference
    #[test]
    fn falsify_update_frequency_difference() {
        let fixture = build_fixture_repo();
        let parser = CommitParser::open(&fixture.path).unwrap();

        let commits = parser.parse_commits(None, None).unwrap();
        assert_eq!(commits.len(), 3, "Fixture history should be fully walked");

        // Count unique days with commits
        let mut commit_days = std::collections::HashSet::new();
        let mut code_change_days = std::collections::HashSet::new();

        for commit in &commits {
            // All commits count for git history updates
            let day = commit.timestamp / SECONDS_PER_DAY;
            commit_days.insert(day);

            // Only commits with source file changes count for code index
            let has_code_changes = commit.files.iter().any(|f| {
                f.path.ends_with(".rs") || f.path.ends_with(".ts") || f.path.ends_with(".py")
            });

            if has_code_changes {
                code_change_days.insert(day);
            }
        }

        // The fixture commits on three distinct days and touches source on exactly
        // one of them, so the two counts must actually differ. Asserting `>=` against
        // an arbitrary repository could never fail: code_change_days is by
        // construction a subset of commit_days.
        assert_eq!(commit_days.len(), 3, "Fixture spans three distinct days");
        assert_eq!(
            code_change_days.len(),
            1,
            "Only one fixture commit touches a source file"
        );
        assert!(
            commit_days.len() > code_change_days.len(),
            "Git history should update more often than code changes"
        );
    }
}
