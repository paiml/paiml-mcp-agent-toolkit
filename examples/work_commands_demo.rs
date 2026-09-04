//! Example demonstrating pmat work commands
//!
//! This example shows how to:
//! - Initialize a roadmap
//! - Create and manage work items
//! - Parse and validate YAML with status aliases (v2.211.0)
//! - Validate specifications with Popperian scoring (v2.211.0)
//! - Handle errors gracefully
//!
//! Run with: cargo run --example work_commands_demo

use anyhow::Result;
use pmat::models::roadmap::{ItemStatus, ItemType, Priority, Roadmap, RoadmapItem};
use pmat::services::roadmap_service::RoadmapService;
use pmat::services::spec_parser::SpecParser;
use std::path::PathBuf;
use tempfile::TempDir;

fn main() -> Result<()> {
    println!("🦀 PMAT Work Commands Demo\n");

    // Create temporary directory for testing
    let temp_dir = TempDir::new()?;
    let roadmap_path = temp_dir.path().join("roadmap.yaml");

    println!("📁 Using temporary roadmap: {:?}\n", roadmap_path);

    // === Example 1: Create a new roadmap ===
    println!("=== Example 1: Create New Roadmap ===");
    let service = RoadmapService::new(&roadmap_path);

    let mut roadmap = Roadmap {
        roadmap_version: "1.0".to_string(),
        github_enabled: true,
        github_repo: Some("paiml/pmat".to_string()),
        roadmap: vec![],
    };

    println!("✅ Created empty roadmap");
    println!("   GitHub: enabled");
    println!("   Repo: paiml/pmat\n");

    // === Example 2: Add work items ===
    println!("=== Example 2: Add Work Items ===");

    let item1 = RoadmapItem {
        id: "PERF-001".to_string(),
        github_issue: None,
        item_type: ItemType::Task,
        title: "Optimize YAML parsing performance".to_string(),
        status: ItemStatus::InProgress,
        priority: Priority::High,
        assigned_to: Some("@noah".to_string()),
        created: "2025-11-22T09:00:00Z".to_string(),
        updated: "2025-11-22T09:30:00Z".to_string(),
        spec: Some(PathBuf::from("docs/specifications/perf-001.md")),
        acceptance_criteria: vec![
            "Parse 1000 YAML files in <100ms".to_string(),
            "Zero memory leaks".to_string(),
            "Pass all property tests".to_string(),
        ],
        phases: vec![],
        subtasks: vec![],
        estimated_effort: Some("2 days".to_string()),
        labels: vec!["performance".to_string(), "optimization".to_string()],
        notes: None,
        links: Vec::new(),
    };

    roadmap.roadmap.push(item1.clone());
    println!("✅ Added item: {}", item1.id);
    println!("   Title: {}", item1.title);
    println!("   Status: {:?}", item1.status);
    println!("   Priority: {:?}", item1.priority);

    let item2 = RoadmapItem {
        id: "BUG-042".to_string(),
        github_issue: Some(42),
        item_type: ItemType::Bug,
        title: "Fix malformed YAML handling".to_string(),
        status: ItemStatus::Completed,
        priority: Priority::Critical,
        assigned_to: Some("@noah".to_string()),
        created: "2025-11-22T08:00:00Z".to_string(),
        updated: "2025-11-22T09:00:00Z".to_string(),
        spec: None,
        acceptance_criteria: vec![
            "Graceful error messages".to_string(),
            "No panics on invalid YAML".to_string(),
        ],
        phases: vec![],
        subtasks: vec![],
        estimated_effort: Some("1 day".to_string()),
        labels: vec!["bug".to_string(), "robustness".to_string()],
        notes: None,
        links: Vec::new(),
    };

    roadmap.roadmap.push(item2.clone());
    println!("\n✅ Added item: {}", item2.id);
    println!("   Title: {}", item2.title);
    println!("   GitHub: #{:?}", item2.github_issue);
    println!("   Status: {:?}", item2.status);

    // === Example 3: Save roadmap ===
    println!("\n=== Example 3: Save Roadmap ===");
    service.save(&roadmap)?;
    println!("✅ Saved roadmap to: {:?}", roadmap_path);

    // === Example 4: Load and verify ===
    println!("\n=== Example 4: Load and Verify ===");
    let loaded = service.load()?;
    println!("✅ Loaded roadmap successfully");
    println!("   Version: {}", loaded.roadmap_version);
    println!("   Items: {}", loaded.roadmap.len());

    for item in &loaded.roadmap {
        println!("\n   📋 {}", item.id);
        println!("      Title: {}", item.title);
        println!("      Status: {:?}", item.status);
        println!(
            "      Acceptance Criteria: {}",
            item.acceptance_criteria.len()
        );
    }

    // === Example 5: Find specific items ===
    println!("\n=== Example 5: Find Specific Items ===");

    if let Some(found) = service.find_item("PERF-001")? {
        println!("✅ Found by ID: {}", found.id);
        println!("   Title: {}", found.title);
    }

    if let Some(found) = service.find_item_by_github_issue(42)? {
        println!(
            "\n✅ Found by GitHub issue: #{}",
            found.github_issue.unwrap()
        );
        println!("   ID: {}", found.id);
        println!("   Title: {}", found.title);
    }

    // === Example 6: Update item status ===
    println!("\n=== Example 6: Update Item Status ===");

    let mut updated_item = item1.clone();
    updated_item.status = ItemStatus::Completed;
    updated_item.updated = "2025-11-22T10:00:00Z".to_string();

    service.upsert_item(updated_item)?;
    println!("✅ Updated PERF-001 status to Completed");

    let verified = service.find_item("PERF-001")?.unwrap();
    println!("   Verified status: {:?}", verified.status);

    // === Example 7: Remove item ===
    println!("\n=== Example 7: Remove Item ===");

    let removed = service.remove_item("BUG-042")?;
    if let Some(item) = removed {
        println!("✅ Removed item: {}", item.id);
        println!("   Title: {}", item.title);
    }

    let final_roadmap = service.load()?;
    println!("   Remaining items: {}", final_roadmap.roadmap.len());

    // === Example 8: Read raw YAML ===
    println!("\n=== Example 8: Raw YAML Output ===");
    let yaml_content = std::fs::read_to_string(&roadmap_path)?;
    println!("{}", yaml_content);

    // === Example 9: Status Alias Parsing (v2.211.0) ===
    println!("\n=== Example 9: Status Alias Parsing (v2.211.0) ===");

    let aliases = [
        ("done", "completed"),
        ("wip", "inprogress"),
        ("todo", "planned"),
        ("stuck", "blocked"),
        ("pr", "review"),
        ("wontfix", "cancelled"),
    ];

    println!("Status aliases are now supported:");
    for (alias, _canonical) in &aliases {
        match ItemStatus::from_string(alias) {
            Ok(status) => println!("   '{}' → {:?} ✅", alias, status),
            Err(e) => println!("   '{}' → Error: {} ❌", alias, e),
        }
    }

    // Demonstrate typo suggestion
    println!("\nTypo suggestions with Levenshtein distance:");
    match ItemStatus::from_string("inporgress") {
        Ok(_) => println!("   Unexpected success"),
        Err(e) => println!("   'inporgress' → {}", e.lines().next().unwrap_or(&e)),
    }

    // === Example 10: Spec Parser Demo (v2.211.0) ===
    println!("\n=== Example 10: Spec Parser Demo (v2.211.0) ===");

    let spec_content = r##"---
title: Feature Implementation
issue: "#123"
status: In Progress
---

# Feature Implementation Specification

## Summary

This feature MUST complete within 100ms response time.

## Requirements

- [ ] Implement core functionality
- [x] Write unit tests
- [ ] Add documentation

## Testing

Coverage MUST be at least 85%.
All property tests SHOULD pass.

## Integration

The system SHALL integrate with GitHub API.
"##;

    let parser = SpecParser::new();
    let spec = parser.parse_content(spec_content, std::path::Path::new("example-spec.md"))?;

    println!("Parsed specification:");
    println!("   Title: {}", spec.title);
    println!("   Issue refs: {:?}", spec.issue_refs);
    println!("   Claims: {}", spec.claims.len());
    println!("   Acceptance criteria: {}", spec.acceptance_criteria.len());
    println!("   Code examples: {}", spec.code_examples.len());

    println!("\nExtracted claims (Popperian validation):");
    for claim in spec.claims.iter().take(3) {
        println!("   [{:?}] {}", claim.category, claim.text);
    }

    println!("\n✅ Demo completed successfully!");
    println!("\n💡 Key Takeaways:");
    println!("   - Roadmap service handles all CRUD operations");
    println!("   - Items support GitHub integration");
    println!("   - Robust YAML serialization/deserialization");
    println!("   - Type-safe status and priority enums");
    println!("   - Status aliases with typo suggestions (v2.211.0)");
    println!("   - Spec parsing with Popperian validation (v2.211.0)");
    println!("   - Graceful error handling");

    Ok(())
}
