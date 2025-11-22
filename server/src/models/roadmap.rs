// Roadmap data models for unified GitHub/YAML workflow
//
// Supports both GitHub-first and YAML-first workflows with write-through synchronization.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Roadmap YAML schema version
pub const ROADMAP_VERSION: &str = "1.0";

/// Main roadmap structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Roadmap {
    /// Schema version
    pub roadmap_version: String,

    /// GitHub integration enabled
    #[serde(default = "default_github_enabled")]
    pub github_enabled: bool,

    /// GitHub repository (owner/repo)
    pub github_repo: Option<String>,

    /// List of roadmap items (tickets)
    #[serde(default)]
    pub roadmap: Vec<RoadmapItem>,
}

fn default_github_enabled() -> bool {
    true
}

fn default_timestamp() -> String {
    "1970-01-01T00:00:00Z".to_string()
}

impl Default for Roadmap {
    fn default() -> Self {
        Self {
            roadmap_version: ROADMAP_VERSION.to_string(),
            github_enabled: true,
            github_repo: None,
            roadmap: Vec::new(),
        }
    }
}

/// Individual roadmap item (ticket/issue)
///
/// Note: Extra fields in YAML (like description, implementation, references)
/// are silently ignored to support backward compatibility with older roadmap formats.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoadmapItem {
    /// Unique ID (e.g., "GH-8", "PERF-001", "EPIC-001")
    pub id: String,

    /// GitHub issue number (null if YAML-only)
    pub github_issue: Option<u64>,

    /// Item type (task, epic, bug, etc.)
    #[serde(default = "default_item_type")]
    pub item_type: ItemType,

    /// Title
    pub title: String,

    /// Current status
    pub status: ItemStatus,

    /// Priority level
    #[serde(default)]
    pub priority: Priority,

    /// Assigned to (GitHub username with @)
    pub assigned_to: Option<String>,

    /// Created timestamp (ISO 8601)
    #[serde(default = "default_timestamp")]
    pub created: String,

    /// Last updated timestamp (ISO 8601)
    #[serde(default = "default_timestamp")]
    pub updated: String,

    /// Path to specification file
    pub spec: Option<PathBuf>,

    /// Acceptance criteria (checklist)
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,

    /// Phases (for multi-phase work)
    #[serde(default)]
    pub phases: Vec<Phase>,

    /// Subtasks (for epic items)
    #[serde(default)]
    pub subtasks: Vec<Subtask>,

    /// Estimated effort (human-readable)
    pub estimated_effort: Option<String>,

    /// Labels/tags
    #[serde(default)]
    pub labels: Vec<String>,

    /// Additional notes/documentation (markdown)
    #[serde(default)]
    pub notes: Option<String>,
}

fn default_item_type() -> ItemType {
    ItemType::Task
}

/// Item type enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ItemType {
    Task,
    Epic,
    Bug,
    Feature,
    Enhancement,
    Documentation,
    Refactor,
}

/// Item status enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ItemStatus {
    Planned,
    InProgress,
    Blocked,
    Review,
    Completed,
    Cancelled,
}

/// Priority enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

/// Phase within a roadmap item
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Phase {
    /// Phase name
    pub name: String,

    /// Phase status
    pub status: ItemStatus,

    /// Estimated effort
    pub estimated_effort: Option<String>,

    /// Completion percentage (0-100)
    #[serde(default)]
    pub completion: u8,
}

/// Subtask within an epic
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Subtask {
    /// Subtask ID
    pub id: String,

    /// GitHub issue number (if synced)
    pub github_issue: Option<u64>,

    /// Subtask title
    pub title: String,

    /// Subtask status
    pub status: ItemStatus,

    /// Completion percentage (0-100)
    #[serde(default)]
    pub completion: u8,
}

impl Roadmap {
    /// Create a new empty roadmap
    pub fn new(github_repo: Option<String>) -> Self {
        Self {
            roadmap_version: ROADMAP_VERSION.to_string(),
            github_enabled: true,
            github_repo,
            roadmap: Vec::new(),
        }
    }

    /// Find item by ID
    pub fn find_item(&self, id: &str) -> Option<&RoadmapItem> {
        self.roadmap.iter().find(|item| item.id == id)
    }

    /// Find item by GitHub issue number
    pub fn find_item_by_github_issue(&self, issue: u64) -> Option<&RoadmapItem> {
        self.roadmap
            .iter()
            .find(|item| item.github_issue == Some(issue))
    }

    /// Find item by ID (mutable)
    pub fn find_item_mut(&mut self, id: &str) -> Option<&mut RoadmapItem> {
        self.roadmap.iter_mut().find(|item| item.id == id)
    }

    /// Add or update item
    pub fn upsert_item(&mut self, item: RoadmapItem) {
        if let Some(existing) = self.find_item_mut(&item.id) {
            *existing = item;
        } else {
            self.roadmap.push(item);
        }
    }

    /// Remove item by ID
    pub fn remove_item(&mut self, id: &str) -> Option<RoadmapItem> {
        if let Some(pos) = self.roadmap.iter().position(|item| item.id == id) {
            Some(self.roadmap.remove(pos))
        } else {
            None
        }
    }

    /// Get items without GitHub sync
    pub fn yaml_only_items(&self) -> Vec<&RoadmapItem> {
        self.roadmap
            .iter()
            .filter(|item| item.github_issue.is_none())
            .collect()
    }

    /// Get epic items
    pub fn epic_items(&self) -> Vec<&RoadmapItem> {
        self.roadmap
            .iter()
            .filter(|item| item.item_type == ItemType::Epic)
            .collect()
    }
}

impl RoadmapItem {
    /// Create a new roadmap item
    pub fn new(id: String, title: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id,
            github_issue: None,
            item_type: ItemType::Task,
            title,
            status: ItemStatus::Planned,
            priority: Priority::Medium,
            assigned_to: None,
            created: now.clone(),
            updated: now,
            spec: None,
            acceptance_criteria: Vec::new(),
            phases: Vec::new(),
            subtasks: Vec::new(),
            estimated_effort: None,
            labels: Vec::new(),
            notes: None,
        }
    }

    /// Create from GitHub issue
    pub fn from_github_issue(issue_number: u64, title: String) -> Self {
        let id = format!("GH-{}", issue_number);
        let mut item = Self::new(id, title);
        item.github_issue = Some(issue_number);
        item
    }

    /// Calculate overall completion percentage
    pub fn completion_percentage(&self) -> u8 {
        if !self.subtasks.is_empty() {
            // Epic: weighted average of subtasks
            let total: u16 = self.subtasks.iter().map(|st| st.completion as u16).sum();
            (total / self.subtasks.len() as u16) as u8
        } else if !self.phases.is_empty() {
            // Multi-phase: weighted average of phases
            let total: u16 = self.phases.iter().map(|p| p.completion as u16).sum();
            (total / self.phases.len() as u16) as u8
        } else if !self.acceptance_criteria.is_empty() {
            // Count completed criteria (basic heuristic)
            0 // TODO: Track individual criteria completion
        } else {
            match self.status {
                ItemStatus::Planned => 0,
                ItemStatus::InProgress => 50,
                ItemStatus::Review => 90,
                ItemStatus::Completed => 100,
                ItemStatus::Cancelled => 0,
                ItemStatus::Blocked => 0,
            }
        }
    }

    /// Check if item is synced with GitHub
    pub fn is_github_synced(&self) -> bool {
        self.github_issue.is_some()
    }
}

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
        let result: Result<Roadmap, _> = serde_yaml::from_str(yaml);

        assert!(result.is_ok(), "Expected parsing to succeed with extra fields silently ignored");

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
}
