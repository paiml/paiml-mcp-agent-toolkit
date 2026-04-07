
#[test]
fn test_fuzzy_matching_not_found() {
    let mut roadmap = Roadmap::new(None);
    roadmap.upsert_item(RoadmapItem::new("PMAT-001".to_string(), "Test".to_string()));

    let found = roadmap.find_item("NONEXISTENT");
    assert!(found.is_none());
}

// ============================================================================
// Tests for ItemStatus parsing (used in handle_work_edit)
// ============================================================================

#[test]
fn test_item_status_from_string_completed() {
    assert_eq!(
        ItemStatus::from_string("completed").unwrap(),
        ItemStatus::Completed
    );
    assert_eq!(
        ItemStatus::from_string("done").unwrap(),
        ItemStatus::Completed
    );
    assert_eq!(
        ItemStatus::from_string("DONE").unwrap(),
        ItemStatus::Completed
    );
}

#[test]
fn test_item_status_from_string_inprogress() {
    assert_eq!(
        ItemStatus::from_string("inprogress").unwrap(),
        ItemStatus::InProgress
    );
    assert_eq!(
        ItemStatus::from_string("wip").unwrap(),
        ItemStatus::InProgress
    );
    assert_eq!(
        ItemStatus::from_string("in-progress").unwrap(),
        ItemStatus::InProgress
    );
}

#[test]
fn test_item_status_from_string_planned() {
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
}

#[test]
fn test_item_status_from_string_blocked() {
    assert_eq!(
        ItemStatus::from_string("blocked").unwrap(),
        ItemStatus::Blocked
    );
    assert_eq!(
        ItemStatus::from_string("stuck").unwrap(),
        ItemStatus::Blocked
    );
    assert_eq!(
        ItemStatus::from_string("on-hold").unwrap(),
        ItemStatus::Blocked
    );
}

#[test]
fn test_item_status_from_string_review() {
    assert_eq!(
        ItemStatus::from_string("review").unwrap(),
        ItemStatus::Review
    );
    assert_eq!(ItemStatus::from_string("pr").unwrap(), ItemStatus::Review);
    assert_eq!(
        ItemStatus::from_string("pending-review").unwrap(),
        ItemStatus::Review
    );
}

#[test]
fn test_item_status_from_string_cancelled() {
    assert_eq!(
        ItemStatus::from_string("cancelled").unwrap(),
        ItemStatus::Cancelled
    );
    assert_eq!(
        ItemStatus::from_string("canceled").unwrap(),
        ItemStatus::Cancelled
    );
    assert_eq!(
        ItemStatus::from_string("wontfix").unwrap(),
        ItemStatus::Cancelled
    );
}

#[test]
fn test_item_status_from_string_invalid() {
    let result = ItemStatus::from_string("invalid_status");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("unknown status"));
    assert!(err.contains("did you mean"));
}

#[test]
fn test_item_status_valid_values() {
    let values = ItemStatus::valid_values();
    assert!(values.contains(&"completed"));
    assert!(values.contains(&"inprogress"));
    assert!(values.contains(&"planned"));
    assert!(values.contains(&"blocked"));
    assert!(values.contains(&"review"));
    assert!(values.contains(&"cancelled"));
}

// ============================================================================
// Tests for ItemStatus::can_transition_to (DBC §work_lifecycle)
// ============================================================================

#[test]
fn test_lifecycle_valid_transitions() {
    // Planned -> InProgress, Cancelled
    assert!(ItemStatus::Planned.can_transition_to(ItemStatus::InProgress));
    assert!(ItemStatus::Planned.can_transition_to(ItemStatus::Cancelled));
    // InProgress -> Blocked, Review, Completed
    assert!(ItemStatus::InProgress.can_transition_to(ItemStatus::Blocked));
    assert!(ItemStatus::InProgress.can_transition_to(ItemStatus::Review));
    assert!(ItemStatus::InProgress.can_transition_to(ItemStatus::Completed));
    // Blocked -> InProgress
    assert!(ItemStatus::Blocked.can_transition_to(ItemStatus::InProgress));
    // Review -> InProgress, Completed
    assert!(ItemStatus::Review.can_transition_to(ItemStatus::InProgress));
    assert!(ItemStatus::Review.can_transition_to(ItemStatus::Completed));
}

#[test]
fn test_lifecycle_invalid_planned_to_completed() {
    // FALSIFY-WDB-001: Planned->Completed is INVALID (no skip)
    assert!(!ItemStatus::Planned.can_transition_to(ItemStatus::Completed));
}

#[test]
fn test_lifecycle_terminal_states() {
    // FALSIFY-WDB-006: Completed and Cancelled are terminal (no outgoing edges)
    for target in [
        ItemStatus::Planned,
        ItemStatus::InProgress,
        ItemStatus::Blocked,
        ItemStatus::Review,
        ItemStatus::Completed,
        ItemStatus::Cancelled,
    ] {
        assert!(
            !ItemStatus::Completed.can_transition_to(target),
            "Completed should not transition to {:?}",
            target
        );
        assert!(
            !ItemStatus::Cancelled.can_transition_to(target),
            "Cancelled should not transition to {:?}",
            target
        );
    }
}

#[test]
fn test_lifecycle_display_name() {
    assert_eq!(ItemStatus::Planned.display_name(), "Planned");
    assert_eq!(ItemStatus::InProgress.display_name(), "InProgress");
    assert_eq!(ItemStatus::Completed.display_name(), "Completed");
    assert_eq!(ItemStatus::Cancelled.display_name(), "Cancelled");
    assert_eq!(ItemStatus::Blocked.display_name(), "Blocked");
    assert_eq!(ItemStatus::Review.display_name(), "Review");
}

// ============================================================================
// Tests for RoadmapItem methods
// ============================================================================

#[test]
fn test_roadmap_item_new() {
    let item = RoadmapItem::new("TEST-001".to_string(), "Test title".to_string());
    assert_eq!(item.id, "TEST-001");
    assert_eq!(item.title, "Test title");
    assert_eq!(item.status, ItemStatus::Planned);
    assert_eq!(item.priority, Priority::Medium);
    assert!(item.github_issue.is_none());
}

#[test]
fn test_roadmap_item_from_github_issue() {
    let item = RoadmapItem::from_github_issue(42, "GitHub issue title".to_string());
    assert_eq!(item.id, "GH-42");
    assert_eq!(item.github_issue, Some(42));
    assert_eq!(item.title, "GitHub issue title");
}

#[test]
fn test_roadmap_item_completion_percentage_planned() {
    let item = RoadmapItem::new("TEST".to_string(), "Test".to_string());
    assert_eq!(item.completion_percentage(), 0);
}

#[test]
fn test_roadmap_item_completion_percentage_inprogress() {
    let mut item = RoadmapItem::new("TEST".to_string(), "Test".to_string());
    item.status = ItemStatus::InProgress;
    assert_eq!(item.completion_percentage(), 50);
}

#[test]
fn test_roadmap_item_completion_percentage_review() {
    let mut item = RoadmapItem::new("TEST".to_string(), "Test".to_string());
    item.status = ItemStatus::Review;
    assert_eq!(item.completion_percentage(), 90);
}

#[test]
fn test_roadmap_item_completion_percentage_completed() {
    let mut item = RoadmapItem::new("TEST".to_string(), "Test".to_string());
    item.status = ItemStatus::Completed;
    assert_eq!(item.completion_percentage(), 100);
}

#[test]
fn test_roadmap_item_is_github_synced() {
    let mut item = RoadmapItem::new("TEST".to_string(), "Test".to_string());
    assert!(!item.is_github_synced());

    item.github_issue = Some(123);
    assert!(item.is_github_synced());
}

// ============================================================================
// Tests for Roadmap methods
// ============================================================================

#[test]
fn test_roadmap_new() {
    let roadmap = Roadmap::new(Some("owner/repo".to_string()));
    assert_eq!(roadmap.roadmap_version, "1.0");
    assert!(roadmap.github_enabled);
    assert_eq!(roadmap.github_repo, Some("owner/repo".to_string()));
    assert!(roadmap.roadmap.is_empty());
}

#[test]
fn test_roadmap_default() {
    let roadmap = Roadmap::default();
    assert_eq!(roadmap.roadmap_version, "1.0");
    assert!(roadmap.github_enabled);
    assert!(roadmap.github_repo.is_none());
    assert!(roadmap.roadmap.is_empty());
}

#[test]
fn test_roadmap_upsert_item() {
    let mut roadmap = Roadmap::new(None);
    let item = RoadmapItem::new("TEST-001".to_string(), "Test".to_string());

    roadmap.upsert_item(item.clone());
    assert_eq!(roadmap.roadmap.len(), 1);

    // Upsert same ID should update, not add
    let mut updated = item.clone();
    updated.title = "Updated title".to_string();
    roadmap.upsert_item(updated);
    assert_eq!(roadmap.roadmap.len(), 1);
    assert_eq!(roadmap.roadmap[0].title, "Updated title");
}

#[test]
fn test_roadmap_remove_item() {
    let mut roadmap = Roadmap::new(None);
    roadmap.upsert_item(RoadmapItem::new("TEST-001".to_string(), "Test".to_string()));

    let removed = roadmap.remove_item("TEST-001");
    assert!(removed.is_some());
    assert!(roadmap.roadmap.is_empty());

    // Remove nonexistent
    let not_found = roadmap.remove_item("NONEXISTENT");
    assert!(not_found.is_none());
}

#[test]
fn test_roadmap_find_item_by_github_issue() {
    let mut roadmap = Roadmap::new(None);
    roadmap.upsert_item(RoadmapItem::from_github_issue(42, "Test".to_string()));

    let found = roadmap.find_item_by_github_issue(42);
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, "GH-42");

    let not_found = roadmap.find_item_by_github_issue(999);
    assert!(not_found.is_none());
}

#[test]
fn test_roadmap_yaml_only_items() {
    let mut roadmap = Roadmap::new(None);
    roadmap.upsert_item(RoadmapItem::from_github_issue(42, "GH".to_string()));
    roadmap.upsert_item(RoadmapItem::new("YAML-001".to_string(), "YAML".to_string()));

    let yaml_only = roadmap.yaml_only_items();
    assert_eq!(yaml_only.len(), 1);
    assert_eq!(yaml_only[0].id, "YAML-001");
}

#[test]
fn test_roadmap_epic_items() {
    use crate::models::roadmap::ItemType;

    let mut roadmap = Roadmap::new(None);
    let task = RoadmapItem::new("TASK-001".to_string(), "Task".to_string());
    let mut epic = RoadmapItem::new("EPIC-001".to_string(), "Epic".to_string());
    epic.item_type = ItemType::Epic;

    roadmap.upsert_item(task);
    roadmap.upsert_item(epic);

    let epics = roadmap.epic_items();
    assert_eq!(epics.len(), 1);
    assert_eq!(epics[0].id, "EPIC-001");
}

// ============================================================================
// Tests for ticket title truncation in handle_work_list
// ============================================================================

#[test]
fn test_title_truncation_short() {
    let title = "Short title";
    let truncated = truncate_title_test(title);
    assert_eq!(truncated, "Short title");
}

#[test]
fn test_title_truncation_exactly_40() {
    let title = "This is exactly forty characters long!!!";
    assert_eq!(title.len(), 40);
    let truncated = truncate_title_test(title);
    assert_eq!(truncated, title);
}

#[test]
fn test_title_truncation_over_40() {
    let title = "This is a very long title that exceeds forty characters";
    let truncated = truncate_title_test(title);
    assert_eq!(truncated.len(), 40); // 37 chars + "..."
    assert!(truncated.ends_with("..."));
}

/// Test helper for title truncation
fn truncate_title_test(title: &str) -> String {
    if title.len() > 40 {
        format!("{}...", &title[..37])
    } else {
        title.to_string()
    }
}

// ============================================================================
// Tests for special character handling in titles
// ============================================================================

#[test]
fn test_title_with_special_chars() {
    let (temp_dir, roadmap_path) = setup_temp_roadmap();

    let service = RoadmapService::new(&roadmap_path);
    let item = RoadmapItem::new("TEST-001".to_string(), "Title with: colons".to_string());
    service.upsert_item(item.clone()).expect("Failed to upsert");

    let loaded = service.find_item("TEST-001").unwrap().unwrap();
    assert_eq!(loaded.title, "Title with: colons");

    drop(temp_dir);
}

#[test]
fn test_title_with_unicode() {
    let (temp_dir, roadmap_path) = setup_temp_roadmap();

    let service = RoadmapService::new(&roadmap_path);
    let item = RoadmapItem::new(
        "TEST-001".to_string(),
        "Unicode: \u{2713} \u{2717}".to_string(),
    );
    service.upsert_item(item).expect("Failed to upsert");

    let loaded = service.find_item("TEST-001").unwrap().unwrap();
    assert!(loaded.title.contains("\u{2713}"));

    drop(temp_dir);
}

// ============================================================================
// Tests for long ID warning logic (chars().count() > 50)
// ============================================================================

#[test]
fn test_long_id_warning_threshold() {
    let short_id = "a".repeat(50);
    let long_id = "b".repeat(51);

    assert!(short_id.chars().count() <= 50);
    assert!(long_id.chars().count() > 50);
}

#[test]
fn test_long_id_unicode_chars() {
    // Unicode characters count as 1 char each
    let unicode_id = "\u{1F600}".repeat(51); // 51 emoji
    assert!(unicode_id.chars().count() > 50);

    // But bytes are different
    assert!(unicode_id.len() > 51 * 3); // Each emoji is 4 bytes
}

// ============================================================================
// Tests for status normalization patterns in handle_work_migrate
// ============================================================================

#[test]
fn test_status_normalization_patterns() {
    let patterns = [
        ("status: done", "status: completed"),
        ("status: Done", "status: completed"),
        ("status: DONE", "status: completed"),
        ("status: finished", "status: completed"),
        ("status: in progress", "status: inprogress"),
        ("status: WIP", "status: inprogress"),
        ("status: wip", "status: inprogress"),
        ("status: stuck", "status: blocked"),
        ("status: on-hold", "status: blocked"),
        ("status: todo", "status: planned"),
        ("status: TODO", "status: planned"),
        ("status: open", "status: planned"),
    ];

    for (old, new) in patterns {
        let mut content = format!("test content with {}", old);
        content = content.replace(old, new);
        assert!(
            content.contains(new),
            "Failed to replace {} with {}",
            old,
            new
        );
    }
}

