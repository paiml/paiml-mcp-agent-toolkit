// Tests for roadmap_service: unit tests, property-based tests, and edge cases.
// Included by roadmap_service.rs - shares parent module scope.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::roadmap::ItemStatus;
    use tempfile::TempDir;

    fn setup_temp_service() -> (TempDir, RoadmapService) {
        let temp_dir = TempDir::new().unwrap();
        let roadmap_path = temp_dir.path().join("roadmap.yaml");
        let service = RoadmapService::new(&roadmap_path);
        (temp_dir, service)
    }

    #[test]
    fn test_load_nonexistent_returns_empty() {
        let (_temp, service) = setup_temp_service();
        let roadmap = service.load().unwrap();
        assert!(roadmap.roadmap.is_empty());
    }

    #[test]
    fn test_save_and_load() {
        let (_temp, service) = setup_temp_service();

        let mut roadmap = Roadmap::new(Some("paiml/pmat".to_string()));
        let item = RoadmapItem::from_github_issue(42, "Test issue".to_string());
        roadmap.upsert_item(item);

        service.save(&roadmap).unwrap();
        assert!(service.exists());

        let loaded = service.load().unwrap();
        assert_eq!(loaded.roadmap.len(), 1);
        assert_eq!(loaded.roadmap[0].id, "GH-42");
        assert_eq!(loaded.github_repo, Some("paiml/pmat".to_string()));
    }

    #[test]
    fn test_upsert_item() {
        let (_temp, service) = setup_temp_service();

        let item1 = RoadmapItem::new("TEST-001".to_string(), "Task 1".to_string());
        service.upsert_item(item1.clone()).unwrap();

        let roadmap = service.load().unwrap();
        assert_eq!(roadmap.roadmap.len(), 1);
        assert_eq!(roadmap.roadmap[0].title, "Task 1");

        // Update
        let mut item1_updated = item1.clone();
        item1_updated.title = "Task 1 Updated".to_string();
        item1_updated.status = ItemStatus::Completed;
        service.upsert_item(item1_updated).unwrap();

        let roadmap = service.load().unwrap();
        assert_eq!(roadmap.roadmap.len(), 1);
        assert_eq!(roadmap.roadmap[0].title, "Task 1 Updated");
        assert_eq!(roadmap.roadmap[0].status, ItemStatus::Completed);
    }

    #[test]
    fn test_remove_item() {
        let (_temp, service) = setup_temp_service();

        let item = RoadmapItem::new("TEST-001".to_string(), "Task 1".to_string());
        service.upsert_item(item).unwrap();

        let removed = service.remove_item("TEST-001").unwrap();
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, "TEST-001");

        let roadmap = service.load().unwrap();
        assert_eq!(roadmap.roadmap.len(), 0);
    }

    #[test]
    fn test_find_item() {
        let (_temp, service) = setup_temp_service();

        let item = RoadmapItem::from_github_issue(42, "Test".to_string());
        service.upsert_item(item).unwrap();

        let found = service.find_item("GH-42").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Test");

        let not_found = service.find_item("NONEXISTENT").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_find_by_github_issue() {
        let (_temp, service) = setup_temp_service();

        let item = RoadmapItem::from_github_issue(42, "GitHub issue".to_string());
        service.upsert_item(item).unwrap();

        let found = service.find_item_by_github_issue(42).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "GH-42");

        let not_found = service.find_item_by_github_issue(999).unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_initialize() {
        let (_temp, service) = setup_temp_service();

        service.initialize(Some("paiml/pmat".to_string())).unwrap();
        assert!(service.exists());

        let roadmap = service.load().unwrap();
        assert_eq!(roadmap.github_repo, Some("paiml/pmat".to_string()));
        assert!(roadmap.github_enabled);
        assert!(roadmap.roadmap.is_empty());
    }

    #[test]
    fn test_yaml_format() {
        let (_temp, service) = setup_temp_service();

        let mut roadmap = Roadmap::new(Some("paiml/pmat".to_string()));
        let item = RoadmapItem::from_github_issue(42, "Test issue".to_string());
        roadmap.upsert_item(item);

        service.save(&roadmap).unwrap();

        // Verify YAML format is human-readable
        let contents = fs::read_to_string(service.path()).unwrap();
        assert!(contents.contains("roadmap_version:"));
        assert!(contents.contains("github_enabled:"));
        assert!(contents.contains("github_repo:"));
        assert!(contents.contains("- id: GH-42"));
        assert!(contents.contains("title: Test issue"));
    }

    // Property-based tests using proptest
    mod property_tests {
        use super::*;
        use crate::models::roadmap::{ItemStatus, Priority};
        use proptest::prelude::*;

        // Custom strategy for ItemStatus (proptest doesn't auto-derive for our enums)
        fn arb_item_status() -> impl Strategy<Value = ItemStatus> {
            prop_oneof![
                Just(ItemStatus::Planned),
                Just(ItemStatus::InProgress),
                Just(ItemStatus::Blocked),
                Just(ItemStatus::Review),
                Just(ItemStatus::Completed),
                Just(ItemStatus::Cancelled),
            ]
        }

        // Custom strategy for Priority
        fn arb_priority() -> impl Strategy<Value = Priority> {
            prop_oneof![
                Just(Priority::Low),
                Just(Priority::Medium),
                Just(Priority::High),
                Just(Priority::Critical),
            ]
        }

        // Generate arbitrary roadmap items for testing
        fn arb_roadmap_item() -> impl Strategy<Value = RoadmapItem> {
            (
                "[A-Z]{2,4}-[0-9]{1,4}",          // id
                proptest::option::of(1u64..1000), // github_issue
                any::<String>(),                  // title (any valid string)
                arb_item_status(),                // status
                arb_priority(),                   // priority
            )
                .prop_map(|(id, github_issue, title, status, priority)| {
                    let mut item = RoadmapItem::new(id, title);
                    item.github_issue = github_issue;
                    item.status = status;
                    item.priority = priority;
                    item
                })
        }

        proptest! {
            #[test]
            fn prop_roundtrip_serialization(item in arb_roadmap_item()) {
                let (_temp, service) = setup_temp_service();

                // Save item
                service.upsert_item(item.clone()).unwrap();

                // Load and verify
                let loaded = service.find_item(&item.id).unwrap().unwrap();
                assert_eq!(loaded.id, item.id);
                assert_eq!(loaded.title, item.title);
                assert_eq!(loaded.status, item.status);
                assert_eq!(loaded.priority, item.priority);
                assert_eq!(loaded.github_issue, item.github_issue);
            }

            #[test]
            fn prop_multiple_items_preserved(items in prop::collection::vec(arb_roadmap_item(), 1..20)) {
                let (_temp, service) = setup_temp_service();

                // Ensure unique IDs
                let mut unique_items = Vec::new();
                let mut seen_ids = std::collections::HashSet::new();
                for item in items {
                    if seen_ids.insert(item.id.clone()) {
                        unique_items.push(item);
                    }
                }

                // Save all items
                for item in &unique_items {
                    service.upsert_item(item.clone()).unwrap();
                }

                // Load and verify count
                let roadmap = service.load().unwrap();
                assert_eq!(roadmap.roadmap.len(), unique_items.len());

                // Verify each item can be found
                for item in &unique_items {
                    let found = service.find_item(&item.id).unwrap();
                    assert!(found.is_some());
                }
            }

            #[test]
            fn prop_upsert_is_idempotent(item in arb_roadmap_item()) {
                let (_temp, service) = setup_temp_service();

                // Insert multiple times
                for _ in 0..5 {
                    service.upsert_item(item.clone()).unwrap();
                }

                // Should only have one item
                let roadmap = service.load().unwrap();
                assert_eq!(roadmap.roadmap.len(), 1);
            }

            #[test]
            fn prop_remove_actually_removes(item in arb_roadmap_item()) {
                let (_temp, service) = setup_temp_service();

                // Add item
                service.upsert_item(item.clone()).unwrap();
                assert!(service.find_item(&item.id).unwrap().is_some());

                // Remove item
                let removed = service.remove_item(&item.id).unwrap();
                assert!(removed.is_some());

                // Verify it's gone
                assert!(service.find_item(&item.id).unwrap().is_none());
            }

            #[test]
            fn prop_yaml_always_valid(items in prop::collection::vec(arb_roadmap_item(), 0..10)) {
                let (_temp, service) = setup_temp_service();

                let mut roadmap = Roadmap::new(Some("test/test".to_string()));
                for item in items {
                    roadmap.upsert_item(item);
                }

                // Save should never fail for valid items
                service.save(&roadmap).unwrap();

                // Load should always succeed
                let loaded = service.load().unwrap();
                assert_eq!(loaded.roadmap.len(), roadmap.roadmap.len());

                // YAML should be parseable by external parser
                let yaml_content = fs::read_to_string(service.path()).unwrap();
                let _: serde_yaml_ng::Value = serde_yaml_ng::from_str(&yaml_content).unwrap();
            }
        }
    }

    // Edge case tests
    mod edge_cases {
        use super::*;

        #[test]
        fn test_empty_title_allowed() {
            let (_temp, service) = setup_temp_service();
            let item = RoadmapItem::new("TEST-001".to_string(), "".to_string());
            service.upsert_item(item.clone()).unwrap();

            let loaded = service.find_item("TEST-001").unwrap().unwrap();
            assert_eq!(loaded.title, "");
        }

        #[test]
        fn test_special_characters_in_title() {
            let (_temp, service) = setup_temp_service();
            let special_title = "Test: \"quotes\", 'apostrophes', <html>, & ampersands, 日本語";
            let item = RoadmapItem::new("TEST-001".to_string(), special_title.to_string());
            service.upsert_item(item.clone()).unwrap();

            let loaded = service.find_item("TEST-001").unwrap().unwrap();
            assert_eq!(loaded.title, special_title);
        }

        #[test]
        fn test_multiline_title() {
            let (_temp, service) = setup_temp_service();
            let multiline = "Line 1\nLine 2\nLine 3";
            let item = RoadmapItem::new("TEST-001".to_string(), multiline.to_string());
            service.upsert_item(item.clone()).unwrap();

            let loaded = service.find_item("TEST-001").unwrap().unwrap();
            assert_eq!(loaded.title, multiline);
        }

        #[test]
        fn test_very_long_title() {
            let (_temp, service) = setup_temp_service();
            let long_title = "A".repeat(10000);
            let item = RoadmapItem::new("TEST-001".to_string(), long_title.clone());
            service.upsert_item(item.clone()).unwrap();

            let loaded = service.find_item("TEST-001").unwrap().unwrap();
            assert_eq!(loaded.title, long_title);
        }

        #[test]
        fn test_concurrent_operations() {
            use std::sync::Arc;
            use std::thread;

            let temp_dir = TempDir::new().unwrap();
            let roadmap_path = temp_dir.path().join("roadmap.yaml");
            let service = Arc::new(RoadmapService::new(&roadmap_path));

            // Initialize empty roadmap
            service.initialize(None).unwrap();

            // Test concurrent WRITES (should be safe with file locking)
            let mut handles = vec![];
            for i in 0..10 {
                let svc = Arc::clone(&service);
                let handle = thread::spawn(move || {
                    let item = RoadmapItem::new(
                        format!("THREAD-{:03}", i),
                        format!("Item from thread {}", i),
                    );
                    // This will acquire an exclusive lock, preventing race conditions
                    svc.upsert_item(item).unwrap();
                });
                handles.push(handle);
            }

            // Wait for all threads to complete
            for handle in handles {
                handle.join().unwrap();
            }

            // Verify ALL 10 items were added (proves no race conditions)
            let roadmap = service.load().unwrap();
            assert_eq!(
                roadmap.roadmap.len(),
                10,
                "All 10 concurrent writes should succeed with file locking"
            );

            // Verify each item exists
            for i in 0..10 {
                let item_id = format!("THREAD-{:03}", i);
                let found = service.find_item(&item_id).unwrap();
                assert!(
                    found.is_some(),
                    "Item {} should exist after concurrent write",
                    item_id
                );
            }
        }

        #[test]
        fn test_malformed_yaml_recovery() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_path = temp_dir.path().join("roadmap.yaml");
            let service = RoadmapService::new(&roadmap_path);

            // Write malformed YAML
            let malformed = "roadmap_version: '1.0'\ninvalid: yaml: structure:";
            fs::write(&roadmap_path, malformed).unwrap();

            // Load should fail gracefully with enhanced error message
            let result = service.load();
            assert!(result.is_err());
            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains("Failed to parse roadmap YAML"));
            // Enhanced error should include troubleshooting steps
            assert!(error_msg.contains("Troubleshooting steps"));
        }

        #[test]
        fn test_invalid_item_type_error() {
            let temp_dir = TempDir::new().unwrap();
            let roadmap_path = temp_dir.path().join("roadmap.yaml");
            let service = RoadmapService::new(&roadmap_path);

            // Invalid item_type 'bugfix' (should be 'bug')
            let invalid_format = r#"roadmap_version: '1.0'
roadmap:
  - id: TASK-001
    title: Example task
    item_type: bugfix
    status: completed
"#;
            fs::write(&roadmap_path, invalid_format).unwrap();

            // Load should fail with helpful error message
            let result = service.load();
            assert!(result.is_err());
            let error_msg = result.unwrap_err().to_string();

            // Should show parse error with location
            assert!(error_msg.contains("Parse error") || error_msg.contains("unknown variant"));

            // Should mention the invalid field
            assert!(error_msg.contains("bugfix") || error_msg.contains("item_type"));

            // Should suggest migration path
            assert!(error_msg.contains("pmat work init"));

            // Should list common issues or expected values
            assert!(error_msg.contains("Common issues") || error_msg.contains("expected"));
        }

        #[test]
        fn test_missing_parent_directory() {
            let temp_dir = TempDir::new().unwrap();
            let nested_path = temp_dir
                .path()
                .join("a")
                .join("b")
                .join("c")
                .join("roadmap.yaml");
            let service = RoadmapService::new(&nested_path);

            // Should create parent directories
            let roadmap = Roadmap::new(None);
            service.save(&roadmap).unwrap();

            assert!(nested_path.exists());
        }

        #[test]
        fn test_github_issue_search() {
            let (_temp, service) = setup_temp_service();

            // Add multiple items with GitHub issues
            service
                .upsert_item(RoadmapItem::from_github_issue(42, "Issue 42".to_string()))
                .unwrap();
            service
                .upsert_item(RoadmapItem::from_github_issue(100, "Issue 100".to_string()))
                .unwrap();
            service
                .upsert_item(RoadmapItem::new(
                    "YAML-001".to_string(),
                    "No GitHub".to_string(),
                ))
                .unwrap();

            // Search by GitHub issue
            assert!(service.find_item_by_github_issue(42).unwrap().is_some());
            assert!(service.find_item_by_github_issue(100).unwrap().is_some());
            assert!(service.find_item_by_github_issue(999).unwrap().is_none());

            // Verify roadmap has 3 items
            let roadmap = service.load().unwrap();
            assert_eq!(roadmap.roadmap.len(), 3);
        }
    }
}
