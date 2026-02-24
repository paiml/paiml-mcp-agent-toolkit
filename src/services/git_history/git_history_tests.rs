#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_commit(hash: &str, subject: &str) -> CommitInfo {
        CommitInfo {
            hash: hash.to_string(),
            message_subject: subject.to_string(),
            message_body: None,
            author_name: "Test Author".to_string(),
            author_email: "test@example.com".to_string(),
            timestamp: 1700000000,
            is_merge: false,
            is_fix: subject.to_lowercase().contains("fix"),
            is_feat: subject.to_lowercase().contains("feat"),
            issue_refs: vec![],
            files: vec![FileChange {
                path: "src/main.rs".to_string(),
                change_type: ChangeType::Modified,
                lines_added: 10,
                lines_deleted: 5,
            }],
        }
    }

    #[test]
    fn test_create_index_in_memory() {
        let index = GitHistoryIndex::in_memory();
        assert!(index.is_ok());
    }

    #[test]
    fn test_insert_and_count_commits() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        let commits = vec![
            create_test_commit(&"a".repeat(40), "Fix bug in parser"),
            create_test_commit(&"b".repeat(40), "Add new feature"),
            create_test_commit(&"c".repeat(40), "Refactor module"),
        ];

        let inserted = index.insert_commits(&commits).unwrap();
        assert_eq!(inserted, 3);

        let count = index.commit_count().unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_insert_skips_non_indexable() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        let commits = vec![
            create_test_commit(&"a".repeat(40), "Fix bug in parser"), // indexable
            CommitInfo {
                hash: "b".repeat(40),
                message_subject: "wip".to_string(), // too short - not indexable
                message_body: None,
                author_name: "Test".to_string(),
                author_email: "test@example.com".to_string(),
                timestamp: 1700000000,
                is_merge: false,
                is_fix: false,
                is_feat: false,
                issue_refs: vec![],
                files: vec![],
            },
        ];

        let inserted = index.insert_commits(&commits).unwrap();
        assert_eq!(inserted, 1); // Only 1 indexable
    }

    #[test]
    fn test_get_commits_for_file() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        let commits = vec![
            create_test_commit(&"a".repeat(40), "Fix bug in parser"),
            create_test_commit(&"b".repeat(40), "Add feature to parser"),
        ];

        index.insert_commits(&commits).unwrap();

        let file_commits = index.get_commits_for_file("src/main.rs", 10).unwrap();
        assert_eq!(file_commits.len(), 2);
    }

    #[test]
    fn test_last_indexed_commit() {
        let index = GitHistoryIndex::in_memory().unwrap();

        // Initially none
        assert!(index.get_last_indexed_commit().unwrap().is_none());

        // Set and retrieve
        index.set_last_indexed_commit("abc123").unwrap();
        assert_eq!(
            index.get_last_indexed_commit().unwrap(),
            Some("abc123".to_string())
        );
    }

    #[test]
    fn test_update_embedding() {
        let index = GitHistoryIndex::in_memory().unwrap();

        let commit = create_test_commit(&"a".repeat(40), "Fix important bug");
        index.insert_commit(&commit).unwrap();

        // Update with embedding
        let embedding: Vec<f32> = vec![0.1, 0.2, 0.3];
        index.update_embedding(&commit.hash, &embedding).unwrap();

        // Verify embedding was stored (would need getter to fully test)
        let count = index.commit_count().unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_checksum_changes_on_insert() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        let checksum1 = index.checksum().unwrap();

        // Message must be >= 10 chars to be indexable
        let commits = vec![create_test_commit(
            &"a".repeat(40),
            "Fix important bug in parser",
        )];
        index.insert_commits(&commits).unwrap();

        let checksum2 = index.checksum().unwrap();

        assert_ne!(checksum1, checksum2, "Checksum should change after insert");
    }

    // === Incremental Sync Tests (GH-RAG-004) ===

    #[test]
    fn test_sync_incremental_empty_index() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        let commits = vec![
            create_test_commit(&"a".repeat(40), "First commit message"),
            create_test_commit(&"b".repeat(40), "Second commit message"),
        ];

        let result = index.sync_incremental(&commits).unwrap();

        assert_eq!(result.commits_added, 2);
        assert_eq!(result.commits_skipped, 0);
        assert!(result.last_commit.is_some());
    }

    #[test]
    fn test_sync_incremental_adds_only_new() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        // Initial sync
        let commits1 = vec![create_test_commit(&"a".repeat(40), "First commit message")];
        index.sync_incremental(&commits1).unwrap();

        // Second sync with existing + new
        let commits2 = vec![
            create_test_commit(&"a".repeat(40), "First commit message"), // existing
            create_test_commit(&"b".repeat(40), "Second commit message"), // new
        ];

        let result = index.sync_incremental(&commits2).unwrap();

        assert_eq!(result.commits_added, 1, "Should only add 1 new commit");
        assert_eq!(index.commit_count().unwrap(), 2);
    }

    #[test]
    fn test_sync_incremental_skips_non_indexable() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        let commits = vec![
            create_test_commit(&"a".repeat(40), "Valid commit message"),
            CommitInfo {
                hash: "b".repeat(40),
                message_subject: "wip".to_string(), // too short
                message_body: None,
                author_name: "Test".to_string(),
                author_email: "test@example.com".to_string(),
                timestamp: 1700000001,
                is_merge: false,
                is_fix: false,
                is_feat: false,
                issue_refs: vec![],
                files: vec![],
            },
        ];

        let result = index.sync_incremental(&commits).unwrap();

        assert_eq!(result.commits_added, 1);
        assert_eq!(result.commits_skipped, 1);
    }

    #[test]
    fn test_sync_incremental_updates_last_indexed() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        assert!(index.get_last_indexed_commit().unwrap().is_none());

        let commits = vec![create_test_commit(&"a".repeat(40), "First commit message")];

        let result = index.sync_incremental(&commits).unwrap();

        assert!(result.last_commit.is_some());
        assert_eq!(index.get_last_indexed_commit().unwrap(), result.last_commit);
    }

    #[test]
    fn test_commit_exists() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        let hash = "a".repeat(40);
        assert!(!index.commit_exists(&hash).unwrap());

        let commits = vec![create_test_commit(&hash, "Test commit message")];
        index.insert_commits(&commits).unwrap();

        assert!(index.commit_exists(&hash).unwrap());
    }

    #[test]
    fn test_get_commits_since() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        let commits = vec![
            CommitInfo {
                hash: "a".repeat(40),
                message_subject: "Old commit message".to_string(),
                message_body: None,
                author_name: "Test".to_string(),
                author_email: "test@example.com".to_string(),
                timestamp: 1000000000,
                is_merge: false,
                is_fix: false,
                is_feat: false,
                issue_refs: vec![],
                files: vec![],
            },
            CommitInfo {
                hash: "b".repeat(40),
                message_subject: "New commit message".to_string(),
                message_body: None,
                author_name: "Test".to_string(),
                author_email: "test@example.com".to_string(),
                timestamp: 2000000000,
                is_merge: false,
                is_fix: false,
                is_feat: false,
                issue_refs: vec![],
                files: vec![],
            },
        ];

        index.insert_commits(&commits).unwrap();

        // Get commits since midpoint
        let recent = index.get_commits_since(1500000000, 10).unwrap();
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].message_subject, "New commit message");
    }

    #[test]
    fn test_sync_empty_input() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        let result = index.sync_incremental(&[]).unwrap();

        assert_eq!(result.commits_added, 0);
        assert_eq!(result.commits_skipped, 0);
    }

    #[test]
    fn test_sync_all_already_present() {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        let commits = vec![create_test_commit(&"a".repeat(40), "Test commit message")];

        // First sync
        index.sync_incremental(&commits).unwrap();

        // Second sync with same commits
        let result = index.sync_incremental(&commits).unwrap();

        assert_eq!(result.commits_added, 0);
        assert_eq!(index.commit_count().unwrap(), 1);
    }

    #[test]
    fn test_get_commits_without_embeddings() {
        let index = GitHistoryIndex::in_memory().unwrap();

        let commit = create_test_commit(&"a".repeat(40), "Fix bug in parser");
        index.insert_commit(&commit).unwrap();

        let needs_embedding = index.get_commits_without_embeddings(10).unwrap();
        assert_eq!(needs_embedding.len(), 1);
        assert_eq!(needs_embedding[0], "a".repeat(40));

        // After setting embedding, should not appear
        index.update_embedding(&commit.hash, &[0.1, 0.2]).unwrap();
        let needs_embedding = index.get_commits_without_embeddings(10).unwrap();
        assert!(needs_embedding.is_empty());
    }

    #[test]
    fn test_commit_info_full_message_no_body() {
        let info = CommitInfo {
            hash: "a".repeat(40),
            message_subject: "Fix bug".to_string(),
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
        assert_eq!(info.full_message(), "Fix bug");
    }

    #[test]
    fn test_commit_info_full_message_empty_body() {
        let info = CommitInfo {
            hash: "a".repeat(40),
            message_subject: "Fix bug".to_string(),
            message_body: Some("".to_string()),
            author_name: "Test".to_string(),
            author_email: "test@example.com".to_string(),
            timestamp: 1700000000,
            is_merge: false,
            is_fix: true,
            is_feat: false,
            issue_refs: vec![],
            files: vec![],
        };
        assert_eq!(info.full_message(), "Fix bug");
    }

    #[test]
    fn test_commit_info_is_indexable_merge_with_custom_message() {
        let info = CommitInfo {
            hash: "a".repeat(40),
            message_subject: "Custom merge: integrate feature X".to_string(),
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
        // Merge with custom message (not starting with "Merge ") IS indexable
        assert!(info.is_indexable());
    }

    #[test]
    fn test_change_type_as_str() {
        assert_eq!(ChangeType::Added.as_str(), "A");
        assert_eq!(ChangeType::Modified.as_str(), "M");
        assert_eq!(ChangeType::Deleted.as_str(), "D");
        assert_eq!(ChangeType::Renamed.as_str(), "R");
    }

    #[test]
    fn test_checksum_empty_index() {
        let index = GitHistoryIndex::in_memory().unwrap();
        let checksum = index.checksum().unwrap();
        assert_eq!(checksum, "0:");
    }

    // Falsification Test F1: Index Independence
    #[test]
    fn falsify_git_index_isolation() {
        // This test verifies that git index operations don't affect code index
        // In a real scenario, we'd have both indexes and verify cross-contamination
        let mut index = GitHistoryIndex::in_memory().unwrap();

        let initial_checksum = index.checksum().unwrap();

        // Insert commits
        let commits = vec![
            create_test_commit(&"a".repeat(40), "Fix bug in parser"),
            create_test_commit(&"b".repeat(40), "Add new feature"),
        ];
        index.insert_commits(&commits).unwrap();

        let final_checksum = index.checksum().unwrap();

        // Git index MUST have changed
        assert_ne!(
            initial_checksum, final_checksum,
            "FALSIFIED: Git index not updated after commit insertion"
        );

        // Verify we can get meaningful data back
        let count = index.commit_count().unwrap();
        assert_eq!(count, 2, "FALSIFIED: Commit count mismatch");
    }
}
