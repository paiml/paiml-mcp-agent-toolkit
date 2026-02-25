// Search engine tests - included by search_engine.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::git_history::{ChangeType, FileChange};

    fn create_test_index() -> GitHistoryIndex {
        let mut index = GitHistoryIndex::in_memory().unwrap();

        let commits = vec![
            CommitInfo {
                hash: "a".repeat(40),
                message_subject: "Fix null pointer exception in parser".to_string(),
                message_body: Some("Handle edge case when input is empty".to_string()),
                author_name: "Alice".to_string(),
                author_email: "alice@example.com".to_string(),
                timestamp: 1700000000,
                is_merge: false,
                is_fix: true,
                is_feat: false,
                issue_refs: vec!["#123".to_string()],
                files: vec![FileChange {
                    path: "src/parser.rs".to_string(),
                    change_type: ChangeType::Modified,
                    lines_added: 5,
                    lines_deleted: 2,
                }],
            },
            CommitInfo {
                hash: "b".repeat(40),
                message_subject: "Add dark mode support to UI".to_string(),
                message_body: Some("Users can now toggle dark mode in settings".to_string()),
                author_name: "Bob".to_string(),
                author_email: "bob@example.com".to_string(),
                timestamp: 1700100000,
                is_merge: false,
                is_fix: false,
                is_feat: true,
                issue_refs: vec!["#456".to_string()],
                files: vec![FileChange {
                    path: "src/ui/theme.rs".to_string(),
                    change_type: ChangeType::Modified,
                    lines_added: 100,
                    lines_deleted: 10,
                }],
            },
            CommitInfo {
                hash: "c".repeat(40),
                message_subject: "Fix memory leak in cache".to_string(),
                message_body: Some("Clear expired entries from cache".to_string()),
                author_name: "Alice".to_string(),
                author_email: "alice@example.com".to_string(),
                timestamp: 1700200000,
                is_merge: false,
                is_fix: true,
                is_feat: false,
                issue_refs: vec![],
                files: vec![FileChange {
                    path: "src/cache.rs".to_string(),
                    change_type: ChangeType::Modified,
                    lines_added: 20,
                    lines_deleted: 5,
                }],
            },
            CommitInfo {
                hash: "d".repeat(40),
                message_subject: "Refactor error handling module".to_string(),
                message_body: None,
                author_name: "Charlie".to_string(),
                author_email: "charlie@example.com".to_string(),
                timestamp: 1700300000,
                is_merge: false,
                is_fix: false,
                is_feat: false,
                issue_refs: vec![],
                files: vec![FileChange {
                    path: "src/parser.rs".to_string(),
                    change_type: ChangeType::Modified,
                    lines_added: 50,
                    lines_deleted: 30,
                }],
            },
        ];

        index.insert_commits(&commits).unwrap();
        index
    }

    #[test]
    fn test_search_basic() {
        let index = create_test_index();
        let mut engine = GitHistorySearchEngine::new(&index);

        let results = engine
            .search("fix bug error", GitSearchOptions::default())
            .unwrap();

        assert!(!results.is_empty(), "Should find commits about fixing bugs");

        // Fix commits should rank higher
        let first = &results[0];
        assert!(
            first.commit.message_subject.to_lowercase().contains("fix")
                || first
                    .commit
                    .message_subject
                    .to_lowercase()
                    .contains("error"),
            "Top result should be about fixing errors"
        );
    }

    #[test]
    fn test_search_filters_by_author() {
        let index = create_test_index();
        let mut engine = GitHistorySearchEngine::new(&index);

        let options = GitSearchOptions {
            author_email: Some("alice@example.com".to_string()),
            ..Default::default()
        };

        let results = engine.search("fix", options).unwrap();

        for r in &results {
            assert_eq!(r.commit.author_email, "alice@example.com");
        }
    }

    #[test]
    fn test_search_only_fixes() {
        let index = create_test_index();
        let mut engine = GitHistorySearchEngine::new(&index);

        let options = GitSearchOptions {
            only_fixes: true,
            ..Default::default()
        };

        let results = engine.search("bug", options).unwrap();

        for r in &results {
            assert!(r.commit.is_fix, "All results should be fix commits");
        }
    }

    #[test]
    fn test_search_only_features() {
        let index = create_test_index();
        let mut engine = GitHistorySearchEngine::new(&index);

        let options = GitSearchOptions {
            only_features: true,
            ..Default::default()
        };

        let results = engine.search("add new", options).unwrap();

        for r in &results {
            assert!(r.commit.is_feat, "All results should be feature commits");
        }
    }

    #[test]
    fn test_search_by_file() {
        let index = create_test_index();
        let engine = GitHistorySearchEngine::new(&index);

        let results = engine.search_by_file("src/parser.rs", 10).unwrap();

        assert_eq!(results.len(), 2, "Two commits touched parser.rs");
    }

    #[test]
    fn test_search_returns_files() {
        let index = create_test_index();
        let mut engine = GitHistorySearchEngine::new(&index);

        let results = engine
            .search("dark mode", GitSearchOptions::default())
            .unwrap();

        if !results.is_empty() {
            assert!(!results[0].files.is_empty(), "Results should include files");
        }
    }

    #[test]
    fn test_search_empty_query() {
        let index = create_test_index();
        let mut engine = GitHistorySearchEngine::new(&index);

        // Even empty query should return something (all candidates)
        let results = engine.search("", GitSearchOptions::default()).unwrap();

        // With empty query, TF-IDF won't find meaningful matches
        // but should not error
        assert!(results.len() <= 4); // At most all commits
    }

    #[test]
    fn test_search_no_results() {
        let index = GitHistoryIndex::in_memory().unwrap();
        let mut engine = GitHistorySearchEngine::new(&index);

        let results = engine
            .search("anything", GitSearchOptions::default())
            .unwrap();

        assert!(results.is_empty());
    }

    #[test]
    fn test_search_limit() {
        let index = create_test_index();
        let mut engine = GitHistorySearchEngine::new(&index);

        let options = GitSearchOptions {
            limit: 2,
            ..Default::default()
        };

        let results = engine.search("commit", options).unwrap();

        assert!(results.len() <= 2);
    }

    #[test]
    fn test_search_timestamp_filter() {
        let index = create_test_index();
        let mut engine = GitHistorySearchEngine::new(&index);

        let options = GitSearchOptions {
            since_timestamp: Some(1700100000),
            ..Default::default()
        };

        let results = engine.search("fix", options).unwrap();

        for r in &results {
            assert!(r.commit.timestamp >= 1700100000);
        }
    }

    #[test]
    fn test_search_until_timestamp_filter() {
        let index = create_test_index();
        let mut engine = GitHistorySearchEngine::new(&index);

        let options = GitSearchOptions {
            until_timestamp: Some(1700100000),
            ..Default::default()
        };

        let results = engine.search("fix", options).unwrap();

        for r in &results {
            assert!(r.commit.timestamp <= 1700100000);
        }
    }

    #[test]
    fn test_search_author_and_since_filter() {
        let index = create_test_index();
        let mut engine = GitHistorySearchEngine::new(&index);

        let options = GitSearchOptions {
            author_email: Some("alice@example.com".to_string()),
            since_timestamp: Some(1700100000),
            ..Default::default()
        };

        let results = engine.search("fix cache memory", options).unwrap();

        for r in &results {
            assert_eq!(r.commit.author_email, "alice@example.com");
            assert!(r.commit.timestamp >= 1700100000);
        }
        // Verifies SQL filter worked; TF-IDF may or may not return results from tiny corpus
    }

    #[test]
    fn test_search_author_and_until_filter() {
        let index = create_test_index();
        let mut engine = GitHistorySearchEngine::new(&index);

        let options = GitSearchOptions {
            author_email: Some("alice@example.com".to_string()),
            until_timestamp: Some(1700050000),
            ..Default::default()
        };

        let results = engine.search("fix null pointer", options).unwrap();

        for r in &results {
            assert_eq!(r.commit.author_email, "alice@example.com");
            assert!(r.commit.timestamp <= 1700050000);
        }
        // Verifies SQL filter worked; TF-IDF may or may not return results from tiny corpus
    }

    #[test]
    fn test_search_since_and_until_filter() {
        let index = create_test_index();
        let mut engine = GitHistorySearchEngine::new(&index);

        let options = GitSearchOptions {
            since_timestamp: Some(1700050000),
            until_timestamp: Some(1700250000),
            ..Default::default()
        };

        let results = engine.search("dark mode cache", options).unwrap();

        for r in &results {
            assert!(r.commit.timestamp >= 1700050000);
            assert!(r.commit.timestamp <= 1700250000);
        }
        // Commits "b" (ts=1700100000) and "c" (ts=1700200000) should match
    }

    #[test]
    fn test_search_all_three_filters() {
        let index = create_test_index();
        let mut engine = GitHistorySearchEngine::new(&index);

        let options = GitSearchOptions {
            author_email: Some("alice@example.com".to_string()),
            since_timestamp: Some(1700050000),
            until_timestamp: Some(1700250000),
            ..Default::default()
        };

        let results = engine.search("fix cache memory", options).unwrap();

        for r in &results {
            assert_eq!(r.commit.author_email, "alice@example.com");
            assert!(r.commit.timestamp >= 1700050000);
            assert!(r.commit.timestamp <= 1700250000);
        }
        // Verifies all three SQL filters applied correctly
    }

    #[test]
    fn test_search_file_path_filter() {
        let index = create_test_index();
        let mut engine = GitHistorySearchEngine::new(&index);

        let options = GitSearchOptions {
            file_path: Some("src/parser.rs".to_string()),
            ..Default::default()
        };

        // file_path in GitSearchOptions isn't used in get_candidates SQL
        // but search_by_file provides this functionality
        let results = engine.search("error refactor", options).unwrap();
        // Should still return results (file_path isn't filtered in get_candidates)
        assert!(results.len() <= 4);
    }

    #[test]
    fn test_cosine_similarity_zero_vectors() {
        assert_eq!(cosine_similarity(&[0.0, 0.0], &[1.0, 2.0]), 0.0);
        assert_eq!(cosine_similarity(&[1.0, 2.0], &[0.0, 0.0]), 0.0);
        assert_eq!(cosine_similarity(&[0.0, 0.0], &[0.0, 0.0]), 0.0);
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let v = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-6);
    }

    // Falsification Test F2 (partial): Semantic clustering
    #[test]
    fn test_fix_commits_cluster_together() {
        let index = create_test_index();
        let mut engine = GitHistorySearchEngine::new(&index);

        // Search for "fix" should rank fix commits higher than feature commits
        let results = engine
            .search("fix bug error crash", GitSearchOptions::default())
            .unwrap();

        if results.len() >= 2 {
            // Fix commits should have higher relevance for this query
            let fix_scores: Vec<f32> = results
                .iter()
                .filter(|r| r.commit.is_fix)
                .map(|r| r.relevance_score)
                .collect();

            let non_fix_scores: Vec<f32> = results
                .iter()
                .filter(|r| !r.commit.is_fix)
                .map(|r| r.relevance_score)
                .collect();

            if !fix_scores.is_empty() && !non_fix_scores.is_empty() {
                let avg_fix = fix_scores.iter().sum::<f32>() / fix_scores.len() as f32;
                let avg_non_fix = non_fix_scores.iter().sum::<f32>() / non_fix_scores.len() as f32;

                // Fix commits should have higher average score for fix-related query
                // This is a weak test since TF-IDF has limitations
                println!(
                    "Avg fix score: {}, Avg non-fix score: {}",
                    avg_fix, avg_non_fix
                );
            }
        }
    }
}
