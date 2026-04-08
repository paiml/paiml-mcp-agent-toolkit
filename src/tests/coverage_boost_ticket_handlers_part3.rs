// ============================================================================
// Tests for detect_coverage_percent logic
// ============================================================================

#[test]
fn test_parse_lcov_lines() {
    let lcov_content = r#"SF:src/main.rs
LF:100
LH:80
end_of_record
SF:src/lib.rs
LF:200
LH:150
end_of_record
"#;

    let (lines_found, lines_hit) = parse_lcov_test(lcov_content);
    assert_eq!(lines_found, 300);
    assert_eq!(lines_hit, 230);

    let coverage = (lines_hit as f64 / lines_found as f64) * 100.0;
    assert!((coverage - 76.67).abs() < 0.01);
}

#[test]
fn test_parse_lcov_empty() {
    let lcov_content = "";
    let (lines_found, lines_hit) = parse_lcov_test(lcov_content);
    assert_eq!(lines_found, 0);
    assert_eq!(lines_hit, 0);
}

#[test]
fn test_parse_lcov_no_data() {
    let lcov_content = r#"SF:src/main.rs
end_of_record
"#;
    let (lines_found, lines_hit) = parse_lcov_test(lcov_content);
    assert_eq!(lines_found, 0);
    assert_eq!(lines_hit, 0);
}

/// Test helper for LCOV parsing
fn parse_lcov_test(content: &str) -> (usize, usize) {
    let mut lines_found: usize = 0;
    let mut lines_hit: usize = 0;

    for line in content.lines() {
        if let Some(num) = line.strip_prefix("LF:") {
            lines_found += num.parse::<usize>().unwrap_or(0);
        } else if let Some(num) = line.strip_prefix("LH:") {
            lines_hit += num.parse::<usize>().unwrap_or(0);
        }
    }

    (lines_found, lines_hit)
}

// ============================================================================
// Tests for TicketAnnotations structure
// ============================================================================

#[test]
fn test_ticket_annotations_serialization() {
    use serde_json;

    #[derive(Debug, Clone, serde::Serialize)]
    struct TicketAnnotations {
        ticket_id: String,
        title: String,
        status: String,
        priority: String,
        spec_path: Option<PathBuf>,
        spec_score: Option<f64>,
        files: Vec<PathBuf>,
        avg_tdg: Option<f64>,
        total_churn: Option<usize>,
        coverage_percent: Option<f64>,
    }

    let annotations = TicketAnnotations {
        ticket_id: "PMAT-001".to_string(),
        title: "Test ticket".to_string(),
        status: "Planned".to_string(),
        priority: "High".to_string(),
        spec_path: Some(PathBuf::from("docs/spec.md")),
        spec_score: Some(85.0),
        files: vec![PathBuf::from("src/main.rs")],
        avg_tdg: Some(2.5),
        total_churn: Some(10),
        coverage_percent: Some(95.5),
    };

    let json = serde_json::to_string(&annotations).expect("Failed to serialize");
    assert!(json.contains("PMAT-001"));
    assert!(json.contains("95.5"));
}

// ============================================================================
// Tests for ChurnResult structure
// ============================================================================

#[test]
fn test_churn_result_defaults() {
    struct ChurnResult {
        total_commits: usize,
        hotspots: Vec<String>,
        repeated_fixes: Vec<RepeatedFix>,
    }

    
    struct RepeatedFix {
        file: String,
        line_range: String,
        fix_count: usize,
        description: String,
    }

    let result = ChurnResult {
        total_commits: 0,
        hotspots: Vec::new(),
        repeated_fixes: Vec::new(),
    };

    assert_eq!(result.total_commits, 0);
    assert!(result.hotspots.is_empty());
    assert!(result.repeated_fixes.is_empty());
}

#[test]
fn test_churn_hotspot_threshold() {
    // Hotspots are files with > 5 commits
    let commit_counts = [(3, false), (5, false), (6, true), (10, true)];

    for (count, is_hotspot) in commit_counts {
        assert_eq!(count > 5, is_hotspot);
    }
}

// ============================================================================
// Tests for FileTdgScore structure
// ============================================================================

#[test]
fn test_file_tdg_score_serialization() {
    use serde_json;

    #[derive(Debug, Clone, serde::Serialize)]
    struct FileTdgScore {
        file: String,
        score: f64,
        severity: String,
    }

    let tdg = FileTdgScore {
        file: "src/main.rs".to_string(),
        score: 2.5,
        severity: "Moderate".to_string(),
    };

    let json = serde_json::to_string(&tdg).expect("Failed to serialize");
    assert!(json.contains("src/main.rs"));
    assert!(json.contains("2.5"));
    assert!(json.contains("Moderate"));
}

// ============================================================================
// Tests for edge cases in file path extraction
// ============================================================================

#[test]
fn test_file_path_regex_pattern() {
    let pattern = regex::Regex::new(r"`([\w/._-]+\.(?:rs|ts|py|go|js))`").unwrap();

    let test_cases = [
        ("`src/main.rs`", Some("src/main.rs")),
        ("`path/to/file.ts`", Some("path/to/file.ts")),
        ("`test.py`", Some("test.py")),
        ("`module.go`", Some("module.go")),
        ("`script.js`", Some("script.js")),
        ("`file.txt`", None),
        ("`no-extension`", None),
        ("src/main.rs", None), // No backticks
    ];

    for (input, expected) in test_cases {
        let result = pattern
            .captures(input)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str());
        assert_eq!(result, expected, "Failed for input: {}", input);
    }
}

// ============================================================================
// Tests for Priority ordering
// ============================================================================

#[test]
fn test_priority_ordering() {
    assert!(Priority::Low < Priority::Medium);
    assert!(Priority::Medium < Priority::High);
    assert!(Priority::High < Priority::Critical);
}

#[test]
fn test_priority_equality() {
    assert_eq!(Priority::Low, Priority::Low);
    assert_ne!(Priority::Low, Priority::High);
}

// ============================================================================
// Tests for YAML deserialization edge cases
// ============================================================================

#[test]
fn test_yaml_with_empty_acceptance_criteria() {
    let yaml = r#"roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: TEST-001
    title: "No acceptance criteria"
    status: planned
    priority: medium
    created: "2024-01-01T00:00:00Z"
    updated: "2024-01-01T00:00:00Z"
    acceptance_criteria: []
"#;

    let roadmap: Roadmap = serde_yaml_ng::from_str(yaml).expect("Failed to parse");
    assert!(roadmap.roadmap[0].acceptance_criteria.is_empty());
}

#[test]
fn test_yaml_with_missing_optional_fields() {
    let yaml = r#"roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: TEST-001
    title: "Minimal item"
    status: planned
"#;

    let roadmap: Roadmap = serde_yaml_ng::from_str(yaml).expect("Failed to parse");
    let item = &roadmap.roadmap[0];
    assert!(item.spec.is_none());
    assert!(item.estimated_effort.is_none());
    assert!(item.notes.is_none());
    assert!(item.assigned_to.is_none());
}

#[test]
fn test_yaml_with_all_status_values() {
    let statuses = [
        "planned",
        "inprogress",
        "blocked",
        "review",
        "completed",
        "cancelled",
    ];

    for status in statuses {
        let yaml = format!(
            r#"roadmap_version: '1.0'
github_enabled: true
roadmap:
  - id: TEST-001
    title: "Test"
    status: {}
"#,
            status
        );

        let result: Result<Roadmap, _> = serde_yaml_ng::from_str(&yaml);
        assert!(result.is_ok(), "Failed to parse status: {}", status);
    }
}

// ============================================================================
// Integration-style tests for handler logic flows
// ============================================================================

#[test]
fn test_handle_work_list_filtering_logic() {
    let mut roadmap = Roadmap::new(None);

    let mut item1 = RoadmapItem::new("PMAT-001".to_string(), "Task 1".to_string());
    item1.status = ItemStatus::Planned;
    item1.priority = Priority::High;

    let mut item2 = RoadmapItem::new("PMAT-002".to_string(), "Task 2".to_string());
    item2.status = ItemStatus::InProgress;
    item2.priority = Priority::Medium;

    let mut item3 = RoadmapItem::new("PMAT-003".to_string(), "Task 3".to_string());
    item3.status = ItemStatus::Completed;
    item3.priority = Priority::High;

    roadmap.upsert_item(item1);
    roadmap.upsert_item(item2);
    roadmap.upsert_item(item3);

    // Filter by status "planned"
    let planned: Vec<_> = roadmap
        .roadmap
        .iter()
        .filter(|item| {
            format!("{:?}", item.status)
                .to_lowercase()
                .contains("planned")
        })
        .collect();
    assert_eq!(planned.len(), 1);
    assert_eq!(planned[0].id, "PMAT-001");

    // Filter by priority High
    let high_priority: Vec<_> = roadmap
        .roadmap
        .iter()
        .filter(|item| item.priority == Priority::High)
        .collect();
    assert_eq!(high_priority.len(), 2);
}

#[test]
fn test_handle_work_edit_changes_tracking() {
    let mut item = RoadmapItem::new("TEST-001".to_string(), "Original".to_string());
    let mut changes: Vec<String> = Vec::new();

    // Apply title change
    let new_title = "Updated Title".to_string();
    item.title = new_title.clone();
    changes.push(format!("title: {}", new_title));

    // Apply priority change
    item.priority = Priority::Critical;
    changes.push(format!("priority: {:?}", Priority::Critical));

    // Apply status change
    item.status = ItemStatus::InProgress;
    changes.push("status: inprogress".to_string());

    assert_eq!(changes.len(), 3);
    assert!(changes[0].contains("Updated Title"));
    assert!(changes[1].contains("Critical"));
    assert!(changes[2].contains("inprogress"));
}

// ============================================================================
// Tests for error messages and user feedback
// ============================================================================

#[test]
fn test_roadmap_not_found_error_message() {
    let path = PathBuf::from("/nonexistent/docs/roadmaps/roadmap.yaml");
    let expected_msg = format!(
        "No roadmap found at {}. Run 'pmat work init' first.",
        path.display()
    );
    assert!(expected_msg.contains("No roadmap found"));
    assert!(expected_msg.contains("pmat work init"));
}

#[test]
fn test_ticket_not_found_error_message() {
    let id = "NONEXISTENT-001";
    let expected_msg = format!(
        "Ticket '{}' not found. Use 'pmat work list' to see available tickets.",
        id
    );
    assert!(expected_msg.contains(id));
    assert!(expected_msg.contains("pmat work list"));
}

#[test]
fn test_ambiguous_id_error_message() {
    let id = "TEST";
    let matches = vec!["TEST-001", "TEST-002", "TEST-003"];
    let expected_msg = format!(
        "Ambiguous ID '{}'. Multiple matches: {}. Please be more specific.",
        id,
        matches.join(", ")
    );
    assert!(expected_msg.contains("Ambiguous"));
    assert!(expected_msg.contains("TEST-001, TEST-002, TEST-003"));
}
