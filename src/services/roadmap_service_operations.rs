// Roadmap service CRUD operations: upsert, remove, find, initialize.
// Included by roadmap_service.rs - shares parent module scope.

impl RoadmapService {
    /// Add or update an item in the roadmap (atomic read-modify-write with exclusive lock)
    pub fn upsert_item(&self, item: RoadmapItem) -> Result<()> {
        // Acquire exclusive lock for entire read-modify-write operation
        let _lock = self.acquire_write_lock()?;

        // Load roadmap (no lock needed - we already have exclusive lock)
        let mut roadmap = if self.roadmap_path.exists() {
            let contents = fs::read_to_string(&self.roadmap_path)
                .with_context(|| format!("Failed to read roadmap file: {:?}", self.roadmap_path))?;
            self.parse_roadmap_yaml(&contents)?
        } else {
            Roadmap::default()
        };

        // Modify
        roadmap.upsert_item(item);

        // Save (no lock needed - we already have exclusive lock)
        if let Some(parent) = self.roadmap_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }
        let yaml = serde_yaml_ng::to_string(&roadmap)
            .with_context(|| "Failed to serialize roadmap to YAML")?;
        fs::write(&self.roadmap_path, yaml)
            .with_context(|| format!("Failed to write roadmap file: {:?}", self.roadmap_path))?;

        Ok(())
        // Lock released automatically
    }

    /// Remove an item from the roadmap (atomic read-modify-write with exclusive lock)
    pub fn remove_item(&self, id: &str) -> Result<Option<RoadmapItem>> {
        // Acquire exclusive lock for entire read-modify-write operation
        let _lock = self.acquire_write_lock()?;

        // Load roadmap (no lock needed - we already have exclusive lock)
        let mut roadmap = if self.roadmap_path.exists() {
            let contents = fs::read_to_string(&self.roadmap_path)
                .with_context(|| format!("Failed to read roadmap file: {:?}", self.roadmap_path))?;
            self.parse_roadmap_yaml(&contents)?
        } else {
            Roadmap::default()
        };

        // Modify
        let removed = roadmap.remove_item(id);

        // Save (no lock needed - we already have exclusive lock)
        if let Some(parent) = self.roadmap_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }
        let yaml = serde_yaml_ng::to_string(&roadmap)
            .with_context(|| "Failed to serialize roadmap to YAML")?;
        fs::write(&self.roadmap_path, yaml)
            .with_context(|| format!("Failed to write roadmap file: {:?}", self.roadmap_path))?;

        Ok(removed)
        // Lock released automatically
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
