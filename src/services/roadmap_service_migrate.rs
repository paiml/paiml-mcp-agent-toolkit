// Roadmap service text migration: `pmat work migrate` under the repository lock.
// Included by roadmap_service.rs - shares parent module scope.

/// What a text migration read, changed and wrote.
#[derive(Debug, Default)]
pub struct TextMigration {
    /// The roadmap the transform judged: `roadmap.yaml`, or base ⊕ `entries/` in a
    /// repository that has opted in to fragments.
    pub before: String,
    /// The same view after the transform; equal to `before` when nothing changed.
    pub after: String,
    /// One line per substitution, in the transform's order, without repeats.
    pub changes: Vec<String>,
    /// Every file written, in write order. Empty on a dry run or a no-op.
    pub written: Vec<PathBuf>,
}

/// The writes a migration will make, all computed before the first one.
enum MigrationPlan {
    /// Whole-file mode: the file as read, and as migrated.
    Whole { original: String, migrated: String },
    /// Fragment mode: `(id, block)` per fragment to write, `roadmap.yaml` untouched.
    Fragments { entries: PathBuf, rows: Vec<(String, String)> },
}

impl RoadmapService {
    /// Apply a TEXT transform to the roadmap under the repository lock, keeping every
    /// byte the transform does not change.
    ///
    /// PMAT-1385 (#1385). `pmat work migrate` read `roadmap.yaml` with no lock and
    /// wrote it back with a bare `std::fs::write`, so a `work add` landing between
    /// the read and the write was silently undone; and in a repository with
    /// `entries/` it rewrote the generated aggregate and never looked at a fragment.
    /// Reading, transforming and writing now happen under ONE exclusive lock (a dry
    /// run takes the shared one), and the write goes through the lock token.
    ///
    /// It is a text rewrite, not [`Self::save`]: `save` re-serialises the model over
    /// the whole file, which drops comments, unknown keys and block scalars
    /// (PMAT-679), and a migration exists precisely for hand-written legacy files.
    ///
    /// Whole-file mode writes `roadmap.yaml.bak` (when `backup`) and `roadmap.yaml`.
    /// In a repository with `entries/`, `roadmap.yaml` is never opened for write and
    /// no backup of it is made; instead every fragment the transform changes is
    /// rewritten, and every BASE row it changes gets a fragment holding its migrated
    /// row — which supersedes the base row at the next aggregation, exactly as
    /// [`Self::replace_item_raw`] does for an edit. Every fragment is checked before
    /// any is written, so a refusal writes nothing.
    ///
    /// # Errors
    ///
    /// No roadmap; the lock cannot be taken; in fragment mode, a change to the
    /// header, a changed row whose id cannot be a filename or is declared twice, or a
    /// fragment that would break the aggregate; or the I/O error of a write.
    pub fn migrate_text(
        &self,
        transform: impl Fn(&str) -> (String, Vec<String>),
        dry_run: bool,
        backup: bool,
    ) -> Result<TextMigration> {
        if dry_run {
            let _lock = self.acquire_read_lock()?;
            return self.plan_text_migration(&transform).map(|(report, _)| report);
        }
        let lock = self.acquire_write_lock()?;
        let (mut report, plan) = self.plan_text_migration(&transform)?;
        match plan {
            MigrationPlan::Whole { original, migrated } if migrated != original => {
                if backup {
                    let backup_path = self.roadmap_path.with_extension("yaml.bak");
                    lock.write(&backup_path, &original)
                        .with_context(|| format!("Failed to write backup: {:?}", backup_path))?;
                    report.written.push(backup_path);
                }
                lock.write(&self.roadmap_path, &migrated).with_context(|| {
                    format!("Failed to write roadmap file: {:?}", self.roadmap_path)
                })?;
                report.written.push(self.roadmap_path.clone());
            }
            MigrationPlan::Whole { .. } => {}
            MigrationPlan::Fragments { entries, rows } => {
                for (id, block) in rows {
                    let path = crate::services::roadmap_fragments::write_fragment(
                        &lock, &entries, &id, &block,
                    )
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                    report.written.push(path);
                }
            }
        }
        Ok(report)
        // Lock released automatically
    }

    /// Read and transform, writing nothing. The caller holds a lock.
    fn plan_text_migration(
        &self,
        transform: &impl Fn(&str) -> (String, Vec<String>),
    ) -> Result<(TextMigration, MigrationPlan)> {
        let base = fs::read_to_string(&self.roadmap_path)
            .with_context(|| format!("Failed to read roadmap file: {:?}", self.roadmap_path))?;
        if let Some(entries) = self.fragment_dir()? {
            return self.plan_fragment_migration(entries, base, transform);
        }
        let (migrated, changes) = transform(&base);
        let report = TextMigration {
            before: base.clone(),
            after: migrated.clone(),
            changes,
            written: Vec::new(),
        };
        Ok((
            report,
            MigrationPlan::Whole {
                original: base,
                migrated,
            },
        ))
    }

    /// Fragment mode: the transform applied row by row, fragments first — a fragment
    /// supersedes its base row, so a base row with a fragment is not what anyone reads.
    fn plan_fragment_migration(
        &self,
        entries: PathBuf,
        base: String,
        transform: &impl Fn(&str) -> (String, Vec<String>),
    ) -> Result<(TextMigration, MigrationPlan)> {
        use crate::services::roadmap_fragments as fragments;

        let current = fragments::read_fragments(&entries).map_err(|e| anyhow::anyhow!("{e}"))?;
        let before = fragments::aggregate(&base, &current)
            .map_err(|e| anyhow::anyhow!("{}: {e}", entries.display()))?;
        let (preamble, base_rows) = fragments::split_entries(&base);
        if transform(&preamble).0 != preamble {
            anyhow::bail!(
                "refusing to migrate the roadmap header: in a repository with {} it exists \
                 only in {}, which is generated. Migrate it on the default branch (PMAT-1385).",
                entries.display(),
                self.roadmap_path.display()
            );
        }

        let held: BTreeSet<&str> = current.iter().map(|(id, _)| id.as_str()).collect();
        let unheld_base = base_rows.iter().filter(|(id, _)| !held.contains(id.as_str()));
        let mut changes: Vec<String> = Vec::new();
        let mut rows: Vec<(String, String)> = Vec::new();
        for (id, block) in current.iter().chain(unheld_base) {
            let (migrated, found) = transform(block);
            if migrated == *block {
                continue;
            }
            if rows.iter().any(|(seen, _)| seen == id) {
                anyhow::bail!(
                    "refusing to migrate {id}: {} declares it twice, and one fragment can \
                     carry only one row (PMAT-1385)",
                    self.roadmap_path.display()
                );
            }
            for change in found {
                if !changes.contains(&change) {
                    changes.push(change);
                }
            }
            rows.push((id.clone(), migrated));
        }

        let indent = crate::services::roadmap_text::row_indent(&base);
        for (id, block) in &rows {
            fragments::check_fragment(id, block, indent).map_err(|e| {
                anyhow::anyhow!("refusing to migrate, nothing was written: {e} (PMAT-1385)")
            })?;
        }
        let mut next: Vec<(String, String)> = current
            .into_iter()
            .filter(|(id, _)| !rows.iter().any(|(changed, _)| changed == id))
            .collect();
        next.extend(rows.iter().cloned());
        let after = fragments::aggregate(&base, &next)
            .map_err(|e| anyhow::anyhow!("{}: {e}", entries.display()))?;

        let report = TextMigration {
            before,
            after,
            changes,
            written: Vec::new(),
        };
        Ok((report, MigrationPlan::Fragments { entries, rows }))
    }
}
