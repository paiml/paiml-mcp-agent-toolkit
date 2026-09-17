// Roadmap service I/O operations: file locking, parsing, load, and save.
// Included by roadmap_service.rs - shares parent module scope.

impl RoadmapService {
    /// Where this roadmap's ids are minted from.
    ///
    /// PMAT-680: one authority per REPOSITORY. Inside git that is
    /// `<git-common-dir>/pmat/roadmap-id.lock`, which every worktree shares;
    /// outside git it is the sibling `<roadmap>.yaml.lock` of PMAT-673.
    fn id_authority(&self) -> IdAuthority {
        IdAuthority::discover(&self.roadmap_path)
    }

    /// Get lock file path
    ///
    /// PMAT-680: this used to be `<roadmap>.yaml.lock` unconditionally — a file
    /// PER CHECKOUT. Two worktrees of one repository therefore locked two
    /// different files and read two different high-water marks, so they minted
    /// the same id without ever contending. Readers and writers of every
    /// checkout now agree on the authority's single path.
    fn lock_file_path(&self) -> PathBuf {
        self.id_authority().lock_path
    }

    /// Acquire exclusive lock for writing
    ///
    /// PMAT-673: opened `read(true)` and `truncate(false)`. It used to be
    /// `truncate(true)`, which emptied the file on every acquisition, so the
    /// lock file could hold nothing but its own existence. It now carries the
    /// id high-water mark that [`RoadmapService::add_item_with_next_id`] reads
    /// and advances, and truncating on open would have destroyed that before
    /// the first read.
    ///
    /// PMAT-1385: returns the lock TOKEN, not the file. Every write under
    /// `docs/roadmaps/` is a method of [`RoadmapWriteLock`], so this is the only way
    /// to write one.
    fn acquire_write_lock(&self) -> Result<RoadmapWriteLock> {
        Self::acquire_write_lock_at(&self.lock_file_path())
    }

    /// Acquire the exclusive lock on an authority path already resolved.
    ///
    /// PMAT-680: [`RoadmapService::add_item_with_next_id`] needs the authority
    /// itself (it reads every ref through it), and resolving it twice would
    /// run `git rev-parse` twice for one mint.
    fn acquire_write_lock_at(lock_path: &Path) -> Result<RoadmapWriteLock> {
        RoadmapWriteLock::acquire(lock_path)
    }

    /// Run `f` holding this roadmap's EXCLUSIVE lock — the same repository-wide
    /// authority every `pmat work` writer takes.
    ///
    /// PMAT-1363: for writers that are not a method of this service, such as
    /// `pmat roadmap aggregate --write`. The lock is keyed on the repository (the
    /// git common dir), not on `roadmap.yaml`'s path, so it serialises writes to
    /// `entries/` and to the aggregate alike.
    ///
    /// PMAT-1385: `f` receives the lock token, which is how it writes — and the
    /// borrow cannot outlive the lock.
    ///
    /// # Errors
    ///
    /// When the lock cannot be taken.
    pub fn with_write_lock<T>(&self, f: impl FnOnce(&RoadmapWriteLock) -> T) -> Result<T> {
        let lock = self.acquire_write_lock()?;
        Ok(f(&lock))
    }

    /// Run `f` holding this roadmap's SHARED lock, so no writer is mid-write.
    ///
    /// # Errors
    ///
    /// When the lock cannot be taken.
    pub fn with_read_lock<T>(&self, f: impl FnOnce() -> T) -> Result<T> {
        let _lock = self.acquire_read_lock()?;
        Ok(f())
    }

    /// Acquire shared lock for reading
    fn acquire_read_lock(&self) -> Result<File> {
        let lock_path = self.lock_file_path();
        let lock_file = roadmap_write_lock::open_lock_file(&lock_path)?;

        #[allow(clippy::incompatible_msrv)] // lock_shared() available in Rust 1.89.0
        lock_file
            .lock_shared()
            .with_context(|| format!("Failed to acquire shared lock: {:?}", lock_path))?;

        Ok(lock_file)
    }

    /// Parse YAML with enhanced error reporting
    fn parse_roadmap_yaml(&self, contents: &str) -> Result<Roadmap> {
        serde_yaml_ng::from_str(contents).map_err(|e| {
            // Append the location only when the rendered error does not already
            // carry one. serde_yaml_ng is inconsistent about this: `unknown
            // variant` renders as "... at line 5 column 16" (so appending here
            // produced "at line 5 column 16 at line 5, column 16"), while
            // `missing field` renders bare and depends on this append.
            let rendered = e.to_string();
            let location_info = match e.location() {
                Some(location) if !rendered.contains(" at line ") => {
                    format!(" at line {}, column {}", location.line(), location.column())
                }
                _ => String::new(),
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
                 2. List every broken row in one pass: pmat work validate\n\
                 3. See the full schema: docs/roadmap-schema.md in the pmat repo\n   \
                    (https://github.com/paiml/paiml-mcp-agent-toolkit)\n\
                 4. List valid status values: pmat work list-statuses\n\
                 5. Auto-fix common issues: pmat work migrate\n\
                 \n\
                 Common issues:\n\
                 - Missing required fields ('roadmap_version'; per item: 'id', 'title', 'status')\n\
                 - 'item_type' and 'priority' are exact lowercase with no aliases\n\
                   (item_type: task, epic, bug, feature, enhancement, documentation, refactor;\n\
                    priority: low, medium, high, critical)\n\
                 - This message reports only the first error; 'pmat work validate'\n   \
                   lists every broken row at once",
                self.roadmap_path.display(),
                e,
                location_info,
                path = self.roadmap_path.display()
            );

            anyhow::anyhow!(error_msg)
        })
    }

    /// Load roadmap from file (with shared lock)
    ///
    /// PMAT-1363: in a repository that has opted in to `entries/`, what is loaded
    /// is the base with every fragment aggregated over it — so `work list`,
    /// `work status` and every `find_item` see a ticket the moment its fragment
    /// exists, not after the next post-merge aggregation.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn load(&self) -> Result<Roadmap> {
        // Acquire shared lock (allows multiple concurrent readers)
        let _lock = self.acquire_read_lock()?;

        // Return empty roadmap if file doesn't exist
        let Some(contents) = self.read_view_unlocked()? else {
            return Ok(Roadmap::default());
        };

        self.parse_roadmap_yaml(&contents)
        // Lock released automatically when _lock goes out of scope
    }

    /// Save roadmap to file (with exclusive lock)
    ///
    /// PMAT-1363: in a repository that has opted in to `entries/`, this writes one
    /// fragment per added or changed ticket and never opens `roadmap.yaml` for
    /// write. See [`RoadmapService::write_roadmap_unlocked`].
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn save(&self, roadmap: &Roadmap) -> Result<()> {
        // Acquire exclusive lock (blocks all other readers and writers)
        let lock = self.acquire_write_lock()?;

        self.write_roadmap_unlocked(&lock, roadmap)
        // Lock released automatically when _lock goes out of scope
    }

    /// PMAT-1363: `entries/` beside the roadmap when the repository has opted in.
    ///
    /// The predicate is the one paiml/.github's shared `roadmap-fragment-parity`
    /// gate uses — `[ -d docs/roadmaps/entries ]` — so pmat writes fragments in
    /// exactly the repositories where that gate refuses a pull request that edits
    /// `roadmap.yaml`, and nowhere else.
    ///
    /// # Errors
    ///
    /// When `entries/` exists but the base roadmap does not: there is nothing to
    /// aggregate over, and silently creating a base would be a write to the one
    /// file an opted-in repository forbids.
    fn fragment_dir(&self) -> Result<Option<PathBuf>> {
        let Some(entries) = crate::services::roadmap_fragments::entries_dir_for(&self.roadmap_path)
        else {
            return Ok(None);
        };
        if !self.roadmap_path.exists() {
            anyhow::bail!(
                "refusing to write: {} exists, so {} is a GENERATED aggregate, and there is no \
                 base roadmap to aggregate over. Commit a base roadmap.yaml before adding \
                 tickets (PMAT-1363).",
                entries.display(),
                self.roadmap_path.display()
            );
        }
        Ok(Some(entries))
    }

    /// What every reader and every check sees: the base with `entries/`
    /// aggregated over it in an opted-in repository, the base alone otherwise.
    /// `None` when there is no base. The caller must hold a lock.
    fn read_view_unlocked(&self) -> Result<Option<String>> {
        if !self.roadmap_path.exists() {
            return Ok(None);
        }
        let base = fs::read_to_string(&self.roadmap_path)
            .with_context(|| format!("Failed to read roadmap file: {:?}", self.roadmap_path))?;
        let Some(entries) = crate::services::roadmap_fragments::entries_dir_for(&self.roadmap_path)
        else {
            return Ok(Some(base));
        };
        let fragments = crate::services::roadmap_fragments::read_fragments(&entries)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        crate::services::roadmap_fragments::aggregate(&base, &fragments)
            .map(Some)
            .map_err(|e| anyhow::anyhow!("{}: {e}", entries.display()))
    }

    /// PMAT-1363: persist a whole model in an opted-in repository WITHOUT opening
    /// `roadmap.yaml` for write.
    ///
    /// The model is diffed against the current view (base ⊕ fragments): every
    /// added or changed ticket becomes its own fragment, rendered exactly as
    /// `work add` renders a row, and a ticket that exists only as a fragment and
    /// is gone from the model has its fragment deleted. An unchanged ticket is not
    /// touched at all, so the bytes the model does not carry survive (PMAT-679).
    ///
    /// Everything is checked before anything is written. Refused, with nothing
    /// written: a change to the header (`roadmap_version`, `github_enabled`,
    /// `github_repo`), which exists only in the generated file; removing a row the
    /// base declares, which no fragment can express; a changed ticket whose id
    /// cannot be a filename; and a model that declares one id twice.
    fn write_roadmap_as_fragments(
        &self,
        lock: &RoadmapWriteLock,
        entries: &Path,
        roadmap: &Roadmap,
    ) -> Result<()> {
        use crate::services::roadmap_fragments as fragments;

        let base = fs::read_to_string(&self.roadmap_path)
            .with_context(|| format!("Failed to read roadmap file: {:?}", self.roadmap_path))?;
        let view = fragments::aggregate(
            &base,
            &fragments::read_fragments(entries).map_err(|e| anyhow::anyhow!("{e}"))?,
        )
        .map_err(|e| anyhow::anyhow!("{}: {e}", entries.display()))?;
        let current = self.parse_roadmap_yaml(&view)?;

        self.refuse_header_change(entries, &current, roadmap)?;
        let (changed, removed) = fragment_changes(&current, roadmap)?;
        self.refuse_unwritable(&base, &changed, &removed)?;

        let indent = crate::services::roadmap_text::row_indent(&base);
        for item in changed {
            let block = crate::services::roadmap_text::render_item_block(item, indent);
            fragments::write_fragment(lock, entries, &item.id, &block)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
        }
        for id in removed {
            fragments::remove_fragment(lock, entries, id).map_err(|e| anyhow::anyhow!("{e}"))?;
        }
        Ok(())
    }

    /// The header (`roadmap_version`, `github_enabled`, `github_repo`) exists only
    /// in the generated aggregate, so no fragment can carry a change to it.
    fn refuse_header_change(&self, entries: &Path, current: &Roadmap, wanted: &Roadmap) -> Result<()> {
        if current.roadmap_version != wanted.roadmap_version
            || current.github_enabled != wanted.github_enabled
            || current.github_repo != wanted.github_repo
        {
            anyhow::bail!(
                "refusing to change the roadmap header: in a repository with {} the header \
                 exists only in {}, which is generated. Change it on the default branch, not \
                 through a ticket (PMAT-1363).",
                entries.display(),
                self.roadmap_path.display()
            );
        }
        Ok(())
    }

    /// Refuse, before anything is written, a change no fragment can express: a
    /// changed ticket whose id cannot be a filename, or removing a base row.
    fn refuse_unwritable(&self, base: &str, changed: &[&RoadmapItem], removed: &[&str]) -> Result<()> {
        use crate::services::roadmap_fragments as fragments;
        if let Some(item) = changed.iter().find(|item| !fragments::is_filename_safe(&item.id)) {
            anyhow::bail!(
                "refusing to save {:?}: an id that cannot be a filename cannot be a fragment, \
                 and {} is generated here, so this ticket has no write path (PMAT-1363)",
                item.id,
                self.roadmap_path.display()
            );
        }
        let base_ids: std::collections::BTreeSet<String> =
            fragments::split_entries(base).1.into_iter().map(|(id, _)| id).collect();
        if let Some(id) = removed.iter().find(|id| base_ids.contains(**id)) {
            anyhow::bail!(
                "refusing to remove {id}: its row is in the generated {}; a fragment can \
                 supersede a base row but cannot delete one (PMAT-1363)",
                self.roadmap_path.display()
            );
        }
        Ok(())
    }
}

/// `(changed, removed)` between the current view and the model to save: every
/// ticket that is new or differs, and every current id the model no longer
/// declares. A model that declares one id twice is refused.
fn fragment_changes<'a>(
    current: &'a Roadmap,
    wanted: &'a Roadmap,
) -> Result<(Vec<&'a RoadmapItem>, Vec<&'a str>)> {
    use std::collections::{BTreeSet, HashMap};
    let mut ids: BTreeSet<&str> = BTreeSet::new();
    if let Some(item) = wanted.roadmap.iter().find(|item| !ids.insert(item.id.as_str())) {
        anyhow::bail!("refusing to save: the roadmap declares {} twice", item.id);
    }
    let before: HashMap<&str, &RoadmapItem> =
        current.roadmap.iter().map(|item| (item.id.as_str(), item)).collect();
    let changed = wanted
        .roadmap
        .iter()
        .filter(|item| before.get(item.id.as_str()).copied() != Some(*item))
        .collect();
    let removed = current
        .roadmap
        .iter()
        .map(|item| item.id.as_str())
        .filter(|id| !ids.contains(id))
        .collect();
    Ok((changed, removed))
}
