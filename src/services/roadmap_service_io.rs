// Roadmap service I/O operations: file locking, parsing, load, and save.
// Included by roadmap_service.rs - shares parent module scope.

impl RoadmapService {
    /// Get lock file path
    fn lock_file_path(&self) -> PathBuf {
        let mut lock_path = self.roadmap_path.clone();
        lock_path.set_extension("yaml.lock");
        lock_path
    }

    /// Acquire exclusive lock for writing
    fn acquire_write_lock(&self) -> Result<File> {
        let lock_path = self.lock_file_path();

        // Create parent directory if needed
        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create lock directory: {:?}", parent))?;
        }

        let lock_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&lock_path)
            .with_context(|| format!("Failed to open lock file: {:?}", lock_path))?;

        lock_file
            .lock_exclusive()
            .with_context(|| format!("Failed to acquire exclusive lock: {:?}", lock_path))?;

        Ok(lock_file)
    }

    /// Acquire shared lock for reading
    fn acquire_read_lock(&self) -> Result<File> {
        let lock_path = self.lock_file_path();

        // Create parent directory if needed
        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create lock directory: {:?}", parent))?;
        }

        let lock_file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true) // Need write permission to create file
            .truncate(false) // Don't truncate lock file
            .open(&lock_path)
            .with_context(|| format!("Failed to open lock file: {:?}", lock_path))?;

        #[allow(clippy::incompatible_msrv)] // lock_shared() available in Rust 1.89.0
        lock_file
            .lock_shared()
            .with_context(|| format!("Failed to acquire shared lock: {:?}", lock_path))?;

        Ok(lock_file)
    }

    /// Parse YAML with enhanced error reporting
    fn parse_roadmap_yaml(&self, contents: &str) -> Result<Roadmap> {
        serde_yaml_ng::from_str(contents).map_err(|e| {
            // Extract line/column info if available
            let location_info = if let Some(location) = e.location() {
                format!(" at line {}, column {}", location.line(), location.column())
            } else {
                String::new()
            };

            // Build enhanced error message
            let error_msg = format!(
                "Failed to parse roadmap YAML: {:?}\n\
                 Parse error: {}{}\n\
                 \n\
                 This roadmap may be from an older version of PMAT or have invalid syntax.\n\
                 \n\
                 Troubleshooting steps:\n\
                 1. Check YAML syntax: python3 -c \"import yaml; yaml.safe_load(open('{path}'))\"\n\
                 2. Validate against current schema (see docs/roadmap-schema.md)\n\
                 3. If migrating from old version, run: pmat work init (creates new format)\n\
                 \n\
                 Common issues:\n\
                 - Unknown fields (e.g., 'commit', 'completion' at phase level)\n\
                 - Missing required fields (e.g., 'roadmap_version')\n\
                 - Incorrect field types",
                self.roadmap_path.display(),
                e,
                location_info,
                path = self.roadmap_path.display()
            );

            anyhow::anyhow!(error_msg)
        })
    }

    /// Load roadmap from file (with shared lock)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn load(&self) -> Result<Roadmap> {
        // Acquire shared lock (allows multiple concurrent readers)
        let _lock = self.acquire_read_lock()?;

        if !self.roadmap_path.exists() {
            // Return empty roadmap if file doesn't exist
            return Ok(Roadmap::default());
        }

        let contents = fs::read_to_string(&self.roadmap_path)
            .with_context(|| format!("Failed to read roadmap file: {:?}", self.roadmap_path))?;

        self.parse_roadmap_yaml(&contents)
        // Lock released automatically when _lock goes out of scope
    }

    /// Save roadmap to file (with exclusive lock)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn save(&self, roadmap: &Roadmap) -> Result<()> {
        // Acquire exclusive lock (blocks all other readers and writers)
        let _lock = self.acquire_write_lock()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = self.roadmap_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        let yaml = serde_yaml_ng::to_string(roadmap)
            .with_context(|| "Failed to serialize roadmap to YAML")?;

        fs::write(&self.roadmap_path, yaml)
            .with_context(|| format!("Failed to write roadmap file: {:?}", self.roadmap_path))?;

        Ok(())
        // Lock released automatically when _lock goes out of scope
    }
}
