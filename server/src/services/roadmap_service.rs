// Roadmap service for reading/writing roadmap.yaml files
//
// Provides file I/O operations for the unified GitHub/YAML workflow.

use crate::models::roadmap::{Roadmap, RoadmapItem};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Default roadmap file location
pub const DEFAULT_ROADMAP_PATH: &str = "docs/roadmaps/roadmap.yaml";

/// Roadmap service for file operations
pub struct RoadmapService {
    roadmap_path: PathBuf,
}

impl RoadmapService {
    /// Create a new roadmap service with custom path
    pub fn new<P: AsRef<Path>>(roadmap_path: P) -> Self {
        Self {
            roadmap_path: roadmap_path.as_ref().to_path_buf(),
        }
    }

    /// Create a roadmap service with default path
    pub fn default_path() -> Self {
        Self::new(DEFAULT_ROADMAP_PATH)
    }

    /// Load roadmap from file
    pub fn load(&self) -> Result<Roadmap> {
        if !self.roadmap_path.exists() {
            // Return empty roadmap if file doesn't exist
            return Ok(Roadmap::default());
        }

        let contents = fs::read_to_string(&self.roadmap_path)
            .with_context(|| format!("Failed to read roadmap file: {:?}", self.roadmap_path))?;

        let roadmap: Roadmap = serde_yaml::from_str(&contents)
            .with_context(|| format!("Failed to parse roadmap YAML: {:?}", self.roadmap_path))?;

        Ok(roadmap)
    }

    /// Save roadmap to file
    pub fn save(&self, roadmap: &Roadmap) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.roadmap_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        let yaml = serde_yaml::to_string(roadmap)
            .with_context(|| "Failed to serialize roadmap to YAML")?;

        fs::write(&self.roadmap_path, yaml)
            .with_context(|| format!("Failed to write roadmap file: {:?}", self.roadmap_path))?;

        Ok(())
    }

    /// Add or update an item in the roadmap
    pub fn upsert_item(&self, item: RoadmapItem) -> Result<()> {
        let mut roadmap = self.load()?;
        roadmap.upsert_item(item);
        self.save(&roadmap)?;
        Ok(())
    }

    /// Remove an item from the roadmap
    pub fn remove_item(&self, id: &str) -> Result<Option<RoadmapItem>> {
        let mut roadmap = self.load()?;
        let removed = roadmap.remove_item(id);
        self.save(&roadmap)?;
        Ok(removed)
    }

    /// Find an item by ID
    pub fn find_item(&self, id: &str) -> Result<Option<RoadmapItem>> {
        let roadmap = self.load()?;
        Ok(roadmap.find_item(id).cloned())
    }

    /// Find an item by GitHub issue number
    pub fn find_item_by_github_issue(&self, issue: u64) -> Result<Option<RoadmapItem>> {
        let roadmap = self.load()?;
        Ok(roadmap.find_item_by_github_issue(issue).cloned())
    }

    /// Get the roadmap file path
    pub fn path(&self) -> &Path {
        &self.roadmap_path
    }

    /// Check if roadmap file exists
    pub fn exists(&self) -> bool {
        self.roadmap_path.exists()
    }

    /// Initialize a new roadmap file
    pub fn initialize(&self, github_repo: Option<String>) -> Result<()> {
        let roadmap = Roadmap::new(github_repo);
        self.save(&roadmap)?;
        Ok(())
    }
}

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
}
