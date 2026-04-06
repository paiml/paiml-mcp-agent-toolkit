// Roadmap and RoadmapItem method implementations
//
// Included by roadmap.rs — shares parent scope, no `use` imports needed.

impl Roadmap {
    /// Create a new empty roadmap
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn find_item(&self, id: &str) -> Option<&RoadmapItem> {
        debug_assert!(!id.is_empty(), "id must not be empty");
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn find_item_by_github_issue(&self, issue: u64) -> Option<&RoadmapItem> {
        self.roadmap
            .iter()
            .find(|item| item.github_issue == Some(issue))
    }

    /// Find item by ID (mutable)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn find_item_mut(&mut self, id: &str) -> Option<&mut RoadmapItem> {
        debug_assert!(!id.is_empty(), "id must not be empty");
        self.roadmap.iter_mut().find(|item| item.id == id)
    }

    /// Add or update item
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn upsert_item(&mut self, item: RoadmapItem) {
        if let Some(existing) = self.find_item_mut(&item.id) {
            *existing = item;
        } else {
            self.roadmap.push(item);
        }
    }

    /// Remove item by ID
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn remove_item(&mut self, id: &str) -> Option<RoadmapItem> {
        debug_assert!(!id.is_empty(), "id must not be empty");
        if let Some(pos) = self.roadmap.iter().position(|item| item.id == id) {
            Some(self.roadmap.remove(pos))
        } else {
            None
        }
    }

    /// Get items without GitHub sync
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn yaml_only_items(&self) -> Vec<&RoadmapItem> {
        self.roadmap
            .iter()
            .filter(|item| item.github_issue.is_none())
            .collect()
    }

    /// Get epic items
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn epic_items(&self) -> Vec<&RoadmapItem> {
        self.roadmap
            .iter()
            .filter(|item| item.item_type == ItemType::Epic)
            .collect()
    }
}

impl RoadmapItem {
    /// Create a new roadmap item
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn from_github_issue(issue_number: u64, title: String) -> Self {
        let id = format!("GH-{}", issue_number);
        let mut item = Self::new(id, title);
        item.github_issue = Some(issue_number);
        item
    }

    /// Calculate overall completion percentage
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_github_synced(&self) -> bool {
        self.github_issue.is_some()
    }
}
