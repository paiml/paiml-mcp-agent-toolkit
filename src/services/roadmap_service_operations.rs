// Roadmap service CRUD operations: upsert, remove, find, initialize.
// Included by roadmap_service.rs - shares parent module scope.

impl RoadmapService {
    /// Add or update an item in the roadmap (atomic read-modify-write with exclusive lock)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

    /// Mint the next ticket id and add the item under ONE exclusive lock.
    ///
    /// PMAT-673 (#1193, #1169). `pmat work add` used to load the roadmap under
    /// a SHARED lock, compute `max(id) + 1` from the parsed items, and only
    /// then take the exclusive lock to write. Two processes both read `max =
    /// N`, both minted `N + 1`, and the second REPLACED the first ticket —
    /// `upsert_item` matches on id, so the loss was silent. Reading, minting
    /// and writing here under a single `acquire_write_lock` closes that
    /// window.
    ///
    /// `build` receives the minted id and returns the item to append. It is
    /// called only after the roadmap has parsed, so a refused add builds
    /// nothing.
    ///
    /// # Errors
    ///
    /// Returns the strict parse error (which already names the file and the
    /// line) when the roadmap does not parse, having written NOTHING: neither
    /// the roadmap nor the lock file's high-water mark is touched.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_item_with_next_id(
        &self,
        build: impl FnOnce(String) -> RoadmapItem,
    ) -> Result<String> {
        let mut lock = self.acquire_write_lock()?;

        // (a) the RAW text, which is also what the allocator scans.
        let raw = if self.roadmap_path.exists() {
            fs::read_to_string(&self.roadmap_path)
                .with_context(|| format!("Failed to read roadmap file: {:?}", self.roadmap_path))?
        } else {
            String::new()
        };

        // (b) strict parse FIRST: a broken roadmap must not be rewritten from
        // a lossy model, and must not consume an id either.
        let mut roadmap = if raw.trim().is_empty() {
            Roadmap::default()
        } else {
            self.parse_roadmap_yaml(&raw)?
        };

        // (c) every id line in the raw text, plus the persisted high-water
        // mark, beats the parsed model: subtask ids and rows the model drops
        // are ids in use too.
        let next = next_id_number(&raw, read_high_water_mark(&mut lock));
        let id = format!("PMAT-{next:03}");

        // (d) append — never upsert. The id is fresh by construction, and
        // upsert would turn a collision into a silent overwrite.
        roadmap.roadmap.push(build(id.clone()));
        self.write_roadmap_unlocked(&roadmap)?;
        write_high_water_mark(&mut lock, next)?;

        Ok(id)
        // Lock released automatically
    }

    /// Serialise and write the roadmap. The caller must already hold the lock.
    fn write_roadmap_unlocked(&self, roadmap: &Roadmap) -> Result<()> {
        if let Some(parent) = self.roadmap_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }
        let yaml = serde_yaml_ng::to_string(roadmap)
            .with_context(|| "Failed to serialize roadmap to YAML")?;
        fs::write(&self.roadmap_path, yaml)
            .with_context(|| format!("Failed to write roadmap file: {:?}", self.roadmap_path))
    }

    /// Remove an item from the roadmap (atomic read-modify-write with exclusive lock)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn find_item(&self, id: &str) -> Result<Option<RoadmapItem>> {
        let roadmap = self.load()?;
        Ok(roadmap.find_item(id).cloned())
    }

    /// Find an item by GitHub issue number
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn find_item_by_github_issue(&self, issue: u64) -> Result<Option<RoadmapItem>> {
        let roadmap = self.load()?;
        Ok(roadmap.find_item_by_github_issue(issue).cloned())
    }

    /// Get the roadmap file path
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn path(&self) -> &Path {
        &self.roadmap_path
    }

    /// Check if roadmap file exists
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn exists(&self) -> bool {
        self.roadmap_path.exists()
    }

    /// Initialize a new roadmap file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn initialize(&self, github_repo: Option<String>) -> Result<()> {
        let roadmap = Roadmap::new(github_repo);
        self.save(&roadmap)?;
        Ok(())
    }
}

/// The next ticket number: one past every id already spoken for.
///
/// PMAT-673. Pure, and deliberately reads the RAW roadmap text rather than the
/// parsed model:
///
/// * a `- id:` under `subtasks:` is an id in use, and the model's items do not
///   carry it at the top level;
/// * any prefix counts (`GH-7` and `PMAT-3` are both numbered), because the
///   number, not the prefix, is what collides;
/// * `lock_high_water` is the number persisted in the roadmap's lock file by
///   the last mint, so an id survives even if the ticket that used it is later
///   deleted from the roadmap.
///
/// A suffix that is not a `u32` is ignored — it cannot collide with a minted
/// `PMAT-NNN`.
#[must_use]
pub(crate) fn next_id_number(raw_text: &str, lock_high_water: Option<u32>) -> u32 {
    let mut max = lock_high_water.unwrap_or(0);
    for line in raw_text.lines() {
        let Some(value) = line.trim_start().strip_prefix("- id:") else {
            continue;
        };
        let Some(token) = value.split_whitespace().next() else {
            continue;
        };
        let bare = token.trim_matches(|c| c == '"' || c == '\'');
        if let Some(number) = bare.rsplit('-').next() {
            if let Ok(n) = number.parse::<u32>() {
                max = max.max(n);
            }
        }
    }
    max.saturating_add(1)
}

/// The id high-water mark persisted in an already-held lock file, if it holds
/// one. A fresh (or hand-emptied) lock file simply has no opinion.
fn read_high_water_mark(lock: &mut File) -> Option<u32> {
    lock.seek(SeekFrom::Start(0)).ok()?;
    let mut text = String::new();
    lock.read_to_string(&mut text).ok()?;
    text.trim().parse::<u32>().ok()
}

/// Record the id just minted in the already-held lock file.
fn write_high_water_mark(lock: &mut File, next: u32) -> Result<()> {
    lock.seek(SeekFrom::Start(0))
        .with_context(|| "Failed to rewind roadmap lock file")?;
    lock.set_len(0)
        .with_context(|| "Failed to clear roadmap lock file")?;
    write!(lock, "{next}").with_context(|| "Failed to write roadmap id high-water mark")?;
    lock.flush()
        .with_context(|| "Failed to flush roadmap lock file")?;
    Ok(())
}
