// Roadmap unit tests
//
// Included by roadmap.rs — shares parent scope, no `use` imports needed.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roadmap_creation() {
        let roadmap = Roadmap::new(Some("paiml/pmat".to_string()));
        assert_eq!(roadmap.roadmap_version, "1.0");
        assert!(roadmap.github_enabled);
        assert_eq!(roadmap.github_repo, Some("paiml/pmat".to_string()));
        assert_eq!(roadmap.roadmap.len(), 0);
    }

    #[test]
    fn test_roadmap_item_creation() {
        let item = RoadmapItem::new("TEST-001".to_string(), "Test Item".to_string());
        assert_eq!(item.id, "TEST-001");
        assert_eq!(item.title, "Test Item");
        assert_eq!(item.status, ItemStatus::Planned);
        assert_eq!(item.priority, Priority::Medium);
        assert!(item.github_issue.is_none());
    }

    #[test]
    fn test_github_issue_creation() {
        let item = RoadmapItem::from_github_issue(42, "GitHub Issue".to_string());
        assert_eq!(item.id, "GH-42");
        assert_eq!(item.github_issue, Some(42));
        assert_eq!(item.title, "GitHub Issue");
    }

    #[test]
    fn test_upsert_item() {
        let mut roadmap = Roadmap::new(None);
        let item = RoadmapItem::new("TEST-001".to_string(), "Test".to_string());

        roadmap.upsert_item(item.clone());
        assert_eq!(roadmap.roadmap.len(), 1);

        // Update existing
        let mut updated = item.clone();
        updated.status = ItemStatus::Completed;
        roadmap.upsert_item(updated);
        assert_eq!(roadmap.roadmap.len(), 1);
        assert_eq!(roadmap.roadmap[0].status, ItemStatus::Completed);
    }

    /// Test fuzzy ID matching for improved UX
    #[test]
    fn test_fuzzy_id_matching() {
        let mut roadmap = Roadmap::new(None);

        // Add test items
        roadmap.upsert_item(RoadmapItem::new(
            "Continue unwrap elimination: 27 more unwraps to reach 60-unwrap milestone (EXTREME TDD)".to_string(),
            "Unwrap work".to_string(),
        ));
        roadmap.upsert_item(RoadmapItem::new(
            "Fix critical bugs in parser".to_string(),
            "Parser fixes".to_string(),
        ));

        // Test 1: Exact match (case-sensitive)
        assert!(roadmap
            .find_item("Continue unwrap elimination: 27 more unwraps to reach 60-unwrap milestone (EXTREME TDD)")
            .is_some());

        // Test 2: Case-insensitive exact match
        assert!(roadmap
            .find_item("continue unwrap elimination: 27 more unwraps to reach 60-unwrap milestone (extreme tdd)")
            .is_some());

        // Test 3: Partial match (prefix)
        let found = roadmap.find_item("Continue unwrap");
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Unwrap work");

        // Test 4: Contains match (not at start)
        let found = roadmap.find_item("unwrap elimination");
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Unwrap work");

        // Test 5: Single word match
        let found = roadmap.find_item("unwrap");
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Unwrap work");

        // Test 6: Case-insensitive partial
        let found = roadmap.find_item("UNWRAP");
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Unwrap work");

        // Test 7: Different item
        let found = roadmap.find_item("parser");
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Parser fixes");

        // Test 8: No match
        assert!(roadmap.find_item("nonexistent").is_none());
    }

    #[test]
    fn test_trueno_db_yaml_format_with_extra_fields() {
        // This test verifies the fix for issue #84
        // trueno-db's roadmap has extra fields: description, phase, implementation, references
        // These should be silently ignored to support backward compatibility
        let yaml = r#"
roadmap_version: '1.0'
github_enabled: true
github_repo: paiml/trueno-db
roadmap:
  - id: CORE-001
    title: "Arrow storage backend with morsel-based paging"
    description: |
      Implement Arrow/Parquet storage with 128MB morsel-based paging.
    status: completed
    priority: high
    phase: 1
    labels: [storage, poka-yoke, phase-1]
    acceptance_criteria:
      - Parquet reader with Arrow columnar format
      - 128MB morsel chunks
    implementation:
      - StorageEngine::load_parquet() with Arrow/Parquet integration
      - MORSEL_SIZE_BYTES = 128MB
    references:
      - "Funke et al. (2018): GPU paging for out-of-core workloads"
"#;

        // After removing #[serde(deny_unknown_fields)], parsing should succeed
        // Extra fields (description, phase, implementation, references) are silently ignored
        let result: Result<Roadmap, _> = serde_yaml_ng::from_str(yaml);

        assert!(
            result.is_ok(),
            "Expected parsing to succeed with extra fields silently ignored"
        );

        let roadmap = result.unwrap();
        assert_eq!(roadmap.github_repo, Some("paiml/trueno-db".to_string()));
        assert_eq!(roadmap.roadmap.len(), 1);

        let item = &roadmap.roadmap[0];
        assert_eq!(item.id, "CORE-001");
        assert_eq!(item.title, "Arrow storage backend with morsel-based paging");
        assert_eq!(item.status, ItemStatus::Completed);
        assert_eq!(item.priority, Priority::High);
        assert_eq!(item.labels, vec!["storage", "poka-yoke", "phase-1"]);
        assert_eq!(item.acceptance_criteria.len(), 2);
    }

    #[test]
    fn test_completion_percentage() {
        let mut item = RoadmapItem::new("TEST-001".to_string(), "Test".to_string());

        // Planned status
        assert_eq!(item.completion_percentage(), 0);

        // In progress
        item.status = ItemStatus::InProgress;
        assert_eq!(item.completion_percentage(), 50);

        // Review
        item.status = ItemStatus::Review;
        assert_eq!(item.completion_percentage(), 90);

        // Completed
        item.status = ItemStatus::Completed;
        assert_eq!(item.completion_percentage(), 100);
    }

    #[test]
    fn test_find_item() {
        let mut roadmap = Roadmap::new(None);
        let item1 = RoadmapItem::new("TEST-001".to_string(), "Test 1".to_string());
        let item2 = RoadmapItem::new("TEST-002".to_string(), "Test 2".to_string());

        roadmap.upsert_item(item1);
        roadmap.upsert_item(item2);

        assert!(roadmap.find_item("TEST-001").is_some());
        assert!(roadmap.find_item("TEST-999").is_none());
    }

    #[test]
    fn test_find_by_github_issue() {
        let mut roadmap = Roadmap::new(None);
        let item = RoadmapItem::from_github_issue(42, "GitHub Issue".to_string());

        roadmap.upsert_item(item);

        assert!(roadmap.find_item_by_github_issue(42).is_some());
        assert!(roadmap.find_item_by_github_issue(999).is_none());
    }

    #[test]
    fn test_github_enabled_native_bool() {
        let yaml = "roadmap_version: '1.0'\ngithub_enabled: true\nroadmap: []\n";
        let roadmap: Roadmap = serde_yaml_ng::from_str(yaml).unwrap();
        assert!(roadmap.github_enabled);

        let yaml = "roadmap_version: '1.0'\ngithub_enabled: false\nroadmap: []\n";
        let roadmap: Roadmap = serde_yaml_ng::from_str(yaml).unwrap();
        assert!(!roadmap.github_enabled);
    }

    #[test]
    fn test_github_enabled_quoted_string() {
        let yaml = "roadmap_version: '1.0'\ngithub_enabled: \"true\"\nroadmap: []\n";
        let roadmap: Roadmap = serde_yaml_ng::from_str(yaml).unwrap();
        assert!(roadmap.github_enabled);

        let yaml = "roadmap_version: '1.0'\ngithub_enabled: \"false\"\nroadmap: []\n";
        let roadmap: Roadmap = serde_yaml_ng::from_str(yaml).unwrap();
        assert!(!roadmap.github_enabled);
    }

    #[test]
    fn test_github_enabled_missing_defaults_true() {
        let yaml = "roadmap_version: '1.0'\nroadmap: []\n";
        let roadmap: Roadmap = serde_yaml_ng::from_str(yaml).unwrap();
        assert!(roadmap.github_enabled);
    }

    // Part A: YAML Parsing Resilience - Status Alias Tests
    mod status_alias_tests {
        use super::*;

        #[test]
        fn test_completed_aliases() {
            assert_eq!(
                ItemStatus::from_string("completed").unwrap(),
                ItemStatus::Completed
            );
            assert_eq!(
                ItemStatus::from_string("done").unwrap(),
                ItemStatus::Completed
            );
            assert_eq!(
                ItemStatus::from_string("finished").unwrap(),
                ItemStatus::Completed
            );
            assert_eq!(
                ItemStatus::from_string("closed").unwrap(),
                ItemStatus::Completed
            );
            // Case insensitive
            assert_eq!(
                ItemStatus::from_string("DONE").unwrap(),
                ItemStatus::Completed
            );
            assert_eq!(
                ItemStatus::from_string("Done").unwrap(),
                ItemStatus::Completed
            );
        }

        #[test]
        fn test_inprogress_aliases() {
            assert_eq!(
                ItemStatus::from_string("inprogress").unwrap(),
                ItemStatus::InProgress
            );
            assert_eq!(
                ItemStatus::from_string("in_progress").unwrap(),
                ItemStatus::InProgress
            );
            assert_eq!(
                ItemStatus::from_string("in-progress").unwrap(),
                ItemStatus::InProgress
            );
            assert_eq!(
                ItemStatus::from_string("wip").unwrap(),
                ItemStatus::InProgress
            );
            assert_eq!(
                ItemStatus::from_string("active").unwrap(),
                ItemStatus::InProgress
            );
            assert_eq!(
                ItemStatus::from_string("started").unwrap(),
                ItemStatus::InProgress
            );
            assert_eq!(
                ItemStatus::from_string("WIP").unwrap(),
                ItemStatus::InProgress
            );
        }

        #[test]
        fn test_planned_aliases() {
            assert_eq!(
                ItemStatus::from_string("planned").unwrap(),
                ItemStatus::Planned
            );
            assert_eq!(
                ItemStatus::from_string("todo").unwrap(),
                ItemStatus::Planned
            );
            assert_eq!(
                ItemStatus::from_string("open").unwrap(),
                ItemStatus::Planned
            );
            assert_eq!(
                ItemStatus::from_string("pending").unwrap(),
                ItemStatus::Planned
            );
            assert_eq!(ItemStatus::from_string("new").unwrap(), ItemStatus::Planned);
        }

        #[test]
        fn test_blocked_aliases() {
            assert_eq!(
                ItemStatus::from_string("blocked").unwrap(),
                ItemStatus::Blocked
            );
            assert_eq!(
                ItemStatus::from_string("stuck").unwrap(),
                ItemStatus::Blocked
            );
            assert_eq!(
                ItemStatus::from_string("waiting").unwrap(),
                ItemStatus::Blocked
            );
            assert_eq!(
                ItemStatus::from_string("on-hold").unwrap(),
                ItemStatus::Blocked
            );
            assert_eq!(
                ItemStatus::from_string("on_hold").unwrap(),
                ItemStatus::Blocked
            );
        }

        #[test]
        fn test_review_aliases() {
            assert_eq!(
                ItemStatus::from_string("review").unwrap(),
                ItemStatus::Review
            );
            assert_eq!(
                ItemStatus::from_string("reviewing").unwrap(),
                ItemStatus::Review
            );
            assert_eq!(ItemStatus::from_string("pr").unwrap(), ItemStatus::Review);
            assert_eq!(
                ItemStatus::from_string("pending-review").unwrap(),
                ItemStatus::Review
            );
        }

        #[test]
        fn test_cancelled_aliases() {
            assert_eq!(
                ItemStatus::from_string("cancelled").unwrap(),
                ItemStatus::Cancelled
            );
            assert_eq!(
                ItemStatus::from_string("canceled").unwrap(),
                ItemStatus::Cancelled
            );
            assert_eq!(
                ItemStatus::from_string("dropped").unwrap(),
                ItemStatus::Cancelled
            );
            assert_eq!(
                ItemStatus::from_string("wontfix").unwrap(),
                ItemStatus::Cancelled
            );
        }

        #[test]
        fn test_invalid_status_with_suggestion() {
            let err = ItemStatus::from_string("compl").unwrap_err();
            assert!(err.contains("did you mean"));
            assert!(err.contains("completed"));

            let err = ItemStatus::from_string("progres").unwrap_err();
            assert!(err.contains("did you mean"));
        }

        #[test]
        fn test_yaml_parsing_with_aliases() {
            let yaml = r#"
roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: TEST-001
    title: "Test with done status"
    status: done
    priority: high
  - id: TEST-002
    title: "Test with wip status"
    status: wip
    priority: medium
  - id: TEST-003
    title: "Test with stuck status"
    status: stuck
    priority: low
"#;
            let roadmap: Roadmap = serde_yaml_ng::from_str(yaml).expect("Should parse with aliases");
            assert_eq!(roadmap.roadmap.len(), 3);
            assert_eq!(roadmap.roadmap[0].status, ItemStatus::Completed);
            assert_eq!(roadmap.roadmap[1].status, ItemStatus::InProgress);
            assert_eq!(roadmap.roadmap[2].status, ItemStatus::Blocked);
        }
    }
}
