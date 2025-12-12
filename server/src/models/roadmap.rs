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

/// Item status enumeration with alias support (Part A: YAML Parsing Resilience)
///
/// Supports multiple aliases for user-friendly YAML input:
/// - completed: "done", "finished", "closed"
/// - inprogress: "in_progress", "in-progress", "wip", "active", "started"
/// - planned: "todo", "open", "pending", "new"
/// - blocked: "stuck", "waiting", "on-hold"
/// - review: "reviewing", "pr", "pending-review"
/// - cancelled: "canceled", "dropped", "wontfix"
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ItemStatus {
    Planned,
    InProgress,
    Blocked,
    Review,
    Completed,
    Cancelled,
}

impl<'de> serde::Deserialize<'de> for ItemStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ItemStatus::from_string(&s).map_err(|e| serde::de::Error::custom(e))
    }
}

impl ItemStatus {
    /// Parse status from string with alias support
    ///
    /// Returns helpful error messages with suggestions for typos
    pub fn from_string(s: &str) -> Result<Self, String> {
        // Normalize: lowercase, remove hyphens/underscores, trim whitespace
        let normalized = s.to_lowercase().replace(['-', '_'], "").trim().to_string();

        match normalized.as_str() {
            // Completed aliases
            "completed" | "done" | "finished" | "closed" => Ok(Self::Completed),

            // InProgress aliases
            "inprogress" | "wip" | "active" | "started" | "working" => Ok(Self::InProgress),

            // Planned aliases
            "planned" | "todo" | "open" | "pending" | "new" => Ok(Self::Planned),

            // Blocked aliases
            "blocked" | "stuck" | "waiting" | "onhold" => Ok(Self::Blocked),

            // Review aliases
            "review" | "reviewing" | "pr" | "pendingreview" => Ok(Self::Review),

            // Cancelled aliases
            "cancelled" | "canceled" | "dropped" | "wontfix" => Ok(Self::Cancelled),

            _ => {
                // Provide helpful suggestion using Levenshtein distance
                let valid_statuses = ["completed", "done", "inprogress", "wip", "planned",
                                      "todo", "blocked", "stuck", "review", "cancelled"];
                let suggestion = valid_statuses
                    .iter()
                    .min_by_key(|v| levenshtein_distance(&normalized, v))
                    .map(|s| format!(" (did you mean '{}'?)", s))
                    .unwrap_or_default();

                Err(format!(
                    "unknown status '{}'{}\n\nValid values: completed, done, inprogress, wip, planned, todo, blocked, review, cancelled",
                    s, suggestion
                ))
            }
        }
    }

    /// Get all valid status strings for help text
    pub fn valid_values() -> &'static [&'static str] {
        &[
            "completed", "done", "finished", "closed",
            "inprogress", "in_progress", "wip", "active",
            "planned", "todo", "open", "pending",
            "blocked", "stuck", "waiting",
            "review", "reviewing", "pr",
            "cancelled", "canceled", "dropped",
        ]
    }
}

/// Simple Levenshtein distance for typo suggestions
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 { return b_len; }
    if b_len == 0 { return a_len; }

    let mut matrix = vec![vec![0usize; b_len + 1]; a_len + 1];

    for i in 0..=a_len { matrix[i][0] = i; }
    for j in 0..=b_len { matrix[0][j] = j; }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[a_len][b_len]
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

    /// Find item by ID with fuzzy matching
    ///
    /// Matching strategy (in order):
    /// 1. Exact match (case-sensitive)
    /// 2. Case-insensitive match
    /// 3. Prefix match (starts_with, case-insensitive)
    /// 4. Contains match (partial, case-insensitive)
    ///
    /// This allows users to type:
    /// - Full ID: "Continue unwrap elimination: 27 more unwraps..."
    /// - Partial: "unwrap elimination"
    /// - Short: "unwrap"
    /// - Any case: "UNWRAP"
    pub fn find_item(&self, id: &str) -> Option<&RoadmapItem> {
        let id_lower = id.to_lowercase();

        // 1. Exact match (fastest, case-sensitive)
        if let Some(item) = self.roadmap.iter().find(|item| item.id == id) {
            return Some(item);
        }

        // 2. Case-insensitive exact match
        if let Some(item) = self
            .roadmap
            .iter()
            .find(|item| item.id.to_lowercase() == id_lower)
        {
            return Some(item);
        }

        // 3. Prefix match (starts_with)
        if let Some(item) = self
            .roadmap
            .iter()
            .find(|item| item.id.to_lowercase().starts_with(&id_lower))
        {
            return Some(item);
        }

        // 4. Contains match (last resort)
        self.roadmap
            .iter()
            .find(|item| item.id.to_lowercase().contains(&id_lower))
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
        let result: Result<Roadmap, _> = serde_yaml::from_str(yaml);

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

    // Part A: YAML Parsing Resilience - Status Alias Tests
    mod status_alias_tests {
        use super::*;

        #[test]
        fn test_completed_aliases() {
            assert_eq!(ItemStatus::from_string("completed").unwrap(), ItemStatus::Completed);
            assert_eq!(ItemStatus::from_string("done").unwrap(), ItemStatus::Completed);
            assert_eq!(ItemStatus::from_string("finished").unwrap(), ItemStatus::Completed);
            assert_eq!(ItemStatus::from_string("closed").unwrap(), ItemStatus::Completed);
            // Case insensitive
            assert_eq!(ItemStatus::from_string("DONE").unwrap(), ItemStatus::Completed);
            assert_eq!(ItemStatus::from_string("Done").unwrap(), ItemStatus::Completed);
        }

        #[test]
        fn test_inprogress_aliases() {
            assert_eq!(ItemStatus::from_string("inprogress").unwrap(), ItemStatus::InProgress);
            assert_eq!(ItemStatus::from_string("in_progress").unwrap(), ItemStatus::InProgress);
            assert_eq!(ItemStatus::from_string("in-progress").unwrap(), ItemStatus::InProgress);
            assert_eq!(ItemStatus::from_string("wip").unwrap(), ItemStatus::InProgress);
            assert_eq!(ItemStatus::from_string("active").unwrap(), ItemStatus::InProgress);
            assert_eq!(ItemStatus::from_string("started").unwrap(), ItemStatus::InProgress);
            assert_eq!(ItemStatus::from_string("WIP").unwrap(), ItemStatus::InProgress);
        }

        #[test]
        fn test_planned_aliases() {
            assert_eq!(ItemStatus::from_string("planned").unwrap(), ItemStatus::Planned);
            assert_eq!(ItemStatus::from_string("todo").unwrap(), ItemStatus::Planned);
            assert_eq!(ItemStatus::from_string("open").unwrap(), ItemStatus::Planned);
            assert_eq!(ItemStatus::from_string("pending").unwrap(), ItemStatus::Planned);
            assert_eq!(ItemStatus::from_string("new").unwrap(), ItemStatus::Planned);
        }

        #[test]
        fn test_blocked_aliases() {
            assert_eq!(ItemStatus::from_string("blocked").unwrap(), ItemStatus::Blocked);
            assert_eq!(ItemStatus::from_string("stuck").unwrap(), ItemStatus::Blocked);
            assert_eq!(ItemStatus::from_string("waiting").unwrap(), ItemStatus::Blocked);
            assert_eq!(ItemStatus::from_string("on-hold").unwrap(), ItemStatus::Blocked);
            assert_eq!(ItemStatus::from_string("on_hold").unwrap(), ItemStatus::Blocked);
        }

        #[test]
        fn test_review_aliases() {
            assert_eq!(ItemStatus::from_string("review").unwrap(), ItemStatus::Review);
            assert_eq!(ItemStatus::from_string("reviewing").unwrap(), ItemStatus::Review);
            assert_eq!(ItemStatus::from_string("pr").unwrap(), ItemStatus::Review);
            assert_eq!(ItemStatus::from_string("pending-review").unwrap(), ItemStatus::Review);
        }

        #[test]
        fn test_cancelled_aliases() {
            assert_eq!(ItemStatus::from_string("cancelled").unwrap(), ItemStatus::Cancelled);
            assert_eq!(ItemStatus::from_string("canceled").unwrap(), ItemStatus::Cancelled);
            assert_eq!(ItemStatus::from_string("dropped").unwrap(), ItemStatus::Cancelled);
            assert_eq!(ItemStatus::from_string("wontfix").unwrap(), ItemStatus::Cancelled);
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
            let roadmap: Roadmap = serde_yaml::from_str(yaml).expect("Should parse with aliases");
            assert_eq!(roadmap.roadmap.len(), 3);
            assert_eq!(roadmap.roadmap[0].status, ItemStatus::Completed);
            assert_eq!(roadmap.roadmap[1].status, ItemStatus::InProgress);
            assert_eq!(roadmap.roadmap[2].status, ItemStatus::Blocked);
        }
    }
}
