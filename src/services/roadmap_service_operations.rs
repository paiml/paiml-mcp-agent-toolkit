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

    /// Upsert an item, refusing a roadmap `pmat work validate` would reject.
    ///
    /// PMAT-676 (#1199). `pmat work edit` saved through [`Self::upsert_item`],
    /// which has no text check at all, so it was the second way to launder a
    /// roadmap `validate` fails: the duplicate survived the round trip and
    /// every field the serde model drops was silently discarded with it. The
    /// check runs on the bytes just read, under the exclusive lock that will
    /// do the write, so nothing can slip in between.
    ///
    /// # Errors
    ///
    /// The strict parse error, or the validator's `duplicate id X at
    /// <path>:<line>, …`. In both cases the roadmap is untouched.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn upsert_item_checked(&self, item: RoadmapItem) -> Result<()> {
        let _lock = self.acquire_write_lock()?;

        let raw = if self.roadmap_path.exists() {
            fs::read_to_string(&self.roadmap_path)
                .with_context(|| format!("Failed to read roadmap file: {:?}", self.roadmap_path))?
        } else {
            String::new()
        };
        let mut roadmap = if raw.trim().is_empty() {
            Roadmap::default()
        } else {
            self.parse_roadmap_yaml(&raw)?
        };
        crate::services::roadmap_text::check_roadmap_text(&raw, &self.roadmap_path)?;

        roadmap.upsert_item(item);
        self.write_roadmap_unlocked(&roadmap)
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
    /// PMAT-676 (#1199). Refusing only an UNPARSEABLE roadmap was not enough:
    /// a roadmap `pmat work validate` rejects for a duplicated id parses, so
    /// `add` accepted it, minted from it and rewrote the file through the
    /// lossy serde model. `check_roadmap_text` — the same validator `validate`
    /// runs — is now applied to the same bytes, under the same lock, before
    /// any write.
    ///
    /// PMAT-680 (#1193). Both fixes above reasoned about ONE checkout, and a
    /// repository is not one checkout: the id came from this worktree's text
    /// and this worktree's sibling lock, so two worktrees minted the same id
    /// without ever contending, and an id spent on a branch this worktree has
    /// not checked out was minted twice. The lock and its high-water mark now
    /// live in the git common directory — shared by every worktree — and
    /// [`IdAuthority::max_id_across_refs`] adds what every ref has spent.
    ///
    /// # Errors
    ///
    /// Returns the strict parse error (which already names the file and the
    /// line) when the roadmap does not parse, or the validator's error
    /// (`duplicate id X at <path>:<line>, …`) when it parses but is invalid.
    /// In both cases NOTHING has been written: neither the roadmap nor the
    /// lock file's high-water mark is touched.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_item_with_next_id(
        &self,
        build: impl FnOnce(String) -> RoadmapItem,
    ) -> Result<String> {
        // PMAT-680: the authority is resolved ONCE — it supplies both the lock
        // to hold and the refs to read, and they must be the same repository's.
        let authority = self.id_authority();
        let mut lock = Self::acquire_write_lock_at(&authority.lock_path)?;

        // (a) the RAW text, which is also what the allocator scans.
        let raw = if self.roadmap_path.exists() {
            fs::read_to_string(&self.roadmap_path)
                .with_context(|| format!("Failed to read roadmap file: {:?}", self.roadmap_path))?
        } else {
            String::new()
        };

        // (b) strict parse FIRST: a broken roadmap must not be appended to,
        // and must not consume an id either. The parsed model is thrown away
        // on the append path — it exists to refuse, not to write (PMAT-679).
        let parsed = if raw.trim().is_empty() {
            None
        } else {
            Some(self.parse_roadmap_yaml(&raw)?)
        };

        // (c) PMAT-676: the same text `pmat work validate` judges, judged
        // here, before anything is written. A roadmap that declares an id
        // twice PARSES, so (b) waves it through; `add` used to mint from it
        // and rewrite the whole file from the lossy model.
        crate::services::roadmap_text::check_roadmap_text(&raw, &self.roadmap_path)?;

        // (d) every id line in the raw text, plus the persisted high-water
        // mark, beats the parsed model: subtask ids and rows the model drops
        // are ids in use too.
        //
        // PMAT-680: and every REF's roadmap beats both. The text and the mark
        // are what one checkout can see, and a repository is not one checkout:
        // an id spent on a branch this worktree has never checked out is spent
        // all the same, and without this term it is minted a second time. The
        // mark is now shared by every worktree (it lives in the git common
        // dir), so the three terms together are the whole repository's opinion.
        let next = crate::services::roadmap_text::next_id_number(
            &raw,
            roadmap_id_authority::high_water_mark(&mut lock),
        )
        .max(
            authority
                .max_id_across_refs()
                .map_or(1, |spent| spent.saturating_add(1)),
        );
        let id = format!("PMAT-{next:03}");

        // (e) append the row's TEXT — never upsert, and never re-serialise the
        // model over the file. The id is fresh by construction, so upsert
        // would only turn a collision into a silent overwrite; and a whole-file
        // rewrite is what made one added ticket move 2,532 lines and drop
        // every byte the model does not carry (PMAT-679). An empty file is the
        // one case with no bytes to preserve, and needs the header keys the
        // model supplies.
        let item = build(id.clone());
        match parsed {
            Some(_) => {
                let block = crate::services::roadmap_text::render_item_block(
                    &item,
                    crate::services::roadmap_text::row_indent(&raw),
                );
                let appended = crate::services::roadmap_text::append_item(&raw, &block);
                fs::write(&self.roadmap_path, appended).with_context(|| {
                    format!("Failed to write roadmap file: {:?}", self.roadmap_path)
                })?;
            }
            None => {
                let mut roadmap = Roadmap::default();
                roadmap.roadmap.push(item);
                self.write_roadmap_unlocked(&roadmap)?;
            }
        }
        roadmap_id_authority::write_high_water_mark(&mut lock, next)?;

        Ok(id)
        // Lock released automatically
    }

    /// Replace ONE row's raw text — the row declaring `id` — with `item`.
    ///
    /// PMAT-679 (#1193, #1169). `pmat work edit` saved through the whole model
    /// ([`Self::upsert_item_checked`]), so editing one title rewrote every
    /// line of the file: comments, unknown keys, flow-style rows and block
    /// scalars belonging to rows nobody edited were reformatted or dropped,
    /// and every concurrent branch conflicted on the roadmap. Only the edited
    /// row's span is rewritten now; every other byte is left as it was found.
    ///
    /// Read, check and write happen under one exclusive lock, on the same
    /// bytes, so nothing can slip in between.
    ///
    /// # Errors
    ///
    /// The validator's `duplicate id X at <path>:<line>, …` (the same refusal
    /// `pmat work validate` prints), or an error naming `id` when the roadmap
    /// holds no single row declaring it. In both cases the roadmap is
    /// untouched.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn replace_item_raw(&self, id: &str, item: &RoadmapItem) -> Result<()> {
        let _lock = self.acquire_write_lock()?;

        let raw = fs::read_to_string(&self.roadmap_path)
            .with_context(|| format!("Failed to read roadmap file: {:?}", self.roadmap_path))?;
        crate::services::roadmap_text::check_roadmap_text(&raw, &self.roadmap_path)?;

        let block = crate::services::roadmap_text::render_item_block(
            item,
            crate::services::roadmap_text::row_indent(&raw),
        );
        let updated = crate::services::roadmap_text::replace_item_block(&raw, id, &block)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No single row declares {id} in {:?}: it is absent, or declared more than once",
                    self.roadmap_path
                )
            })?;
        fs::write(&self.roadmap_path, updated)
            .with_context(|| format!("Failed to write roadmap file: {:?}", self.roadmap_path))
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

// PMAT-676: `next_id_number` and its `id_key_value` line scanner used to live
// here. They were the SECOND scanner over the roadmap's raw text — the first
// being `work validate`'s — and the two disagreed about what an id line is,
// which is how `add` came to accept a roadmap `validate` rejects. Both now live
// in `crate::services::roadmap_text`, next to the validator that reads the same
// lines; the tests that pinned them moved with them. The name is re-exported
// here for the PMAT-673 cases that address it at this path — there is no second
// implementation behind it any more, and nothing outside the tests reads it.
#[cfg(test)]
pub(crate) use crate::services::roadmap_text::next_id_number;

// PMAT-680: `read_high_water_mark` and `write_high_water_mark` moved to
// `crate::services::roadmap_id_authority`, beside the code that decides WHICH
// file they act on. The mark used to be a fact about this checkout's sibling
// lock; it is now a fact about the repository, and the two halves of that
// claim — the path and the number — must not live apart.
