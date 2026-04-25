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

    #[test]
    fn test_yaml_only_items_empty_roadmap() {
        let roadmap = Roadmap::new(None);
        assert!(roadmap.yaml_only_items().is_empty());
    }

    #[test]
    fn test_yaml_only_items_filters_github_synced() {
        let mut roadmap = Roadmap::new(None);
        roadmap.upsert_item(RoadmapItem::new(
            "YAML-1".to_string(),
            "Pure yaml".to_string(),
        ));
        roadmap.upsert_item(RoadmapItem::from_github_issue(
            42,
            "Synced".to_string(),
        ));
        roadmap.upsert_item(RoadmapItem::new(
            "YAML-2".to_string(),
            "Another yaml".to_string(),
        ));

        let yaml_only = roadmap.yaml_only_items();
        assert_eq!(yaml_only.len(), 2);
        assert!(yaml_only.iter().all(|i| i.github_issue.is_none()));
        let ids: Vec<&str> = yaml_only.iter().map(|i| i.id.as_str()).collect();
        assert!(ids.contains(&"YAML-1"));
        assert!(ids.contains(&"YAML-2"));
    }

    #[test]
    fn test_epic_items_empty_roadmap() {
        let roadmap = Roadmap::new(None);
        assert!(roadmap.epic_items().is_empty());
    }

    #[test]
    fn test_epic_items_filters_by_type() {
        let mut roadmap = Roadmap::new(None);
        let mut epic = RoadmapItem::new("EPIC-1".to_string(), "Big effort".to_string());
        epic.item_type = ItemType::Epic;
        let task = RoadmapItem::new("TASK-1".to_string(), "Small task".to_string());
        let mut epic2 = RoadmapItem::new("EPIC-2".to_string(), "Another epic".to_string());
        epic2.item_type = ItemType::Epic;

        roadmap.upsert_item(epic);
        roadmap.upsert_item(task);
        roadmap.upsert_item(epic2);

        let epics = roadmap.epic_items();
        assert_eq!(epics.len(), 2);
        assert!(epics.iter().all(|i| i.item_type == ItemType::Epic));
    }

    #[test]
    fn test_epic_items_no_epics() {
        let mut roadmap = Roadmap::new(None);
        roadmap.upsert_item(RoadmapItem::new(
            "TASK-1".to_string(),
            "Just a task".to_string(),
        ));
        assert!(roadmap.epic_items().is_empty());
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

    /// roadmap_impl.rs:153 — subtasks branch wins over phases / criteria
    /// / status arms when populated. Tests the non-empty-subtasks code path
    /// (existing tests only cover the leaf-status branch).
    #[test]
    fn test_completion_percentage_subtasks_branch() {
        let mut item = RoadmapItem::new("EPIC-1".to_string(), "Epic".to_string());
        item.subtasks = vec![
            Subtask {
                id: "S1".to_string(),
                github_issue: None,
                title: "S1".to_string(),
                status: ItemStatus::Completed,
                completion: 100,
            },
            Subtask {
                id: "S2".to_string(),
                github_issue: None,
                title: "S2".to_string(),
                status: ItemStatus::InProgress,
                completion: 50,
            },
        ];
        // (100 + 50) / 2 = 75.
        assert_eq!(item.completion_percentage(), 75);
    }

    /// roadmap_impl.rs:157 — phases branch wins over criteria / status arms
    /// when subtasks empty but phases populated.
    #[test]
    fn test_completion_percentage_phases_branch() {
        let mut item = RoadmapItem::new("MULTI-1".to_string(), "Multi".to_string());
        item.phases = vec![
            Phase {
                name: "P1".to_string(),
                status: ItemStatus::Completed,
                estimated_effort: None,
                completion: 80,
            },
            Phase {
                name: "P2".to_string(),
                status: ItemStatus::InProgress,
                estimated_effort: None,
                completion: 20,
            },
        ];
        // (80 + 20) / 2 = 50.
        assert_eq!(item.completion_percentage(), 50);
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

    /// deserialize_bool_lenient rejects invalid string values. Exercises the
    /// `_ => Err(...)` arm inside visit_str — uncovered until now.
    #[test]
    fn test_github_enabled_invalid_string_rejected() {
        let yaml = "roadmap_version: '1.0'\ngithub_enabled: \"yes\"\nroadmap: []\n";
        let err = serde_yaml_ng::from_str::<Roadmap>(yaml)
            .expect_err("invalid bool string must fail");
        let msg = err.to_string();
        assert!(
            msg.contains("expected true/false") && msg.contains("yes"),
            "error should name the invalid value, got: {msg}"
        );
    }

    /// deserialize_bool_lenient rejects non-bool, non-string values (e.g. integer).
    /// Exercises the visitor's `expecting` message surfaced by serde when no
    /// visit_* method matches.
    #[test]
    fn test_github_enabled_integer_rejected() {
        let yaml = "roadmap_version: '1.0'\ngithub_enabled: 1\nroadmap: []\n";
        let err = serde_yaml_ng::from_str::<Roadmap>(yaml)
            .expect_err("integer must not satisfy lenient bool deserializer");
        let msg = err.to_string();
        assert!(
            msg.contains("boolean") || msg.contains("true/false"),
            "error should reference expected type, got: {msg}"
        );
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

        /// roadmap_status.rs:164-165 — `a_len == 0` early-return arm returns
        /// `b_len`. from_string normalizes input to lowercase-no-hyphens and
        /// feeds it as `a` to levenshtein_distance against each valid_status.
        /// An empty input string produces normalized == "", triggering the
        /// a_len-zero arm against every valid_status candidate.
        #[test]
        fn test_empty_status_string_triggers_levenshtein_empty_a_arm() {
            let err = ItemStatus::from_string("").unwrap_err();
            assert!(err.contains("unknown status"));
            // Suggestion is present: min_by_key picks one valid_status; the
            // a_len==0 arm returned b_len for every candidate, so all ties
            // collapse to the first — "completed" (longest list entry still
            // present). Just verify the suggestion scaffold rendered.
            assert!(
                err.contains("did you mean"),
                "empty input must still produce a suggestion, got {err:?}"
            );
        }

        /// roadmap_status.rs:167-168 — `b_len == 0` arm. from_string never
        /// passes an empty valid_status to levenshtein_distance, so this arm
        /// is unreachable via the public API. Call the private helper
        /// directly to cover the arm. Private fn is visible through the
        /// include! parent scope.
        #[test]
        fn test_levenshtein_distance_empty_b_returns_a_len() {
            assert_eq!(levenshtein_distance("hello", ""), 5);
            assert_eq!(levenshtein_distance("", ""), 0);
            assert_eq!(levenshtein_distance("", "world"), 5);
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

    /// roadmap_impl.rs:152 — `completion_percentage` leaf-status branches.
    /// Fires the ItemStatus match arms that run only when the item has no
    /// subtasks, no phases, and no acceptance_criteria (the "leaf" path).
    #[test]
    fn test_completion_percentage_leaf_status_arms() {
        let mut item = RoadmapItem::new("LEAF-001".to_string(), "leaf".to_string());
        // Freshly constructed — subtasks, phases, acceptance_criteria all empty.
        item.status = ItemStatus::Planned;
        assert_eq!(item.completion_percentage(), 0, "Planned → 0");
        item.status = ItemStatus::InProgress;
        assert_eq!(item.completion_percentage(), 50, "InProgress → 50");
        item.status = ItemStatus::Review;
        assert_eq!(item.completion_percentage(), 90, "Review → 90");
        item.status = ItemStatus::Completed;
        assert_eq!(item.completion_percentage(), 100, "Completed → 100");
        item.status = ItemStatus::Cancelled;
        assert_eq!(item.completion_percentage(), 0, "Cancelled → 0");
        item.status = ItemStatus::Blocked;
        assert_eq!(item.completion_percentage(), 0, "Blocked → 0");
    }

    /// roadmap_impl.rs:161 — acceptance_criteria non-empty, subtasks & phases
    /// empty. Falls through to the `0` placeholder branch.
    #[test]
    fn test_completion_percentage_acceptance_criteria_returns_zero() {
        let mut item = RoadmapItem::new("AC-001".to_string(), "ac".to_string());
        item.acceptance_criteria = vec!["criterion 1".to_string(), "criterion 2".to_string()];
        // Status shouldn't affect this branch — the AC branch returns 0 unconditionally.
        item.status = ItemStatus::Completed;
        assert_eq!(item.completion_percentage(), 0);
    }
}
