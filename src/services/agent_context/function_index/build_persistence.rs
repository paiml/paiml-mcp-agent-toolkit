/// Enrich functions with contract metadata from SQLite (optional columns).
///
/// Handles old schema gracefully — if contract_level/contract_equation
/// columns don't exist, silently skips enrichment.
fn enrich_contract_metadata(conn: &rusqlite::Connection, functions: &mut [FunctionEntry]) {
    let query = "SELECT id, contract_level, contract_equation FROM functions WHERE contract_level IS NOT NULL ORDER BY id";
    let Ok(mut stmt) = conn.prepare(query) else {
        return; // Column doesn't exist in old schema — skip
    };
    let Ok(rows) = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)? as usize,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, Option<String>>(2)?,
        ))
    }) else {
        return;
    };
    for row in rows.flatten() {
        let (id, level, equation) = row;
        if id >= 1 && id <= functions.len() {
            functions[id - 1].quality.contract_level = level;
            functions[id - 1].quality.contract_equation = equation;
        }
    }
}

/// Serialize a manifest to `<index_path>/manifest.json` atomically.
///
/// Writes `manifest.json.tmp` beside the target and renames it over the
/// destination. The temp file is created (not just opened), so a directory
/// that cannot be written fails HERE rather than silently truncating an
/// existing manifest in place — which is how a failed incremental save used to
/// look like a successful one.
fn write_manifest_atomically(index_path: &Path, manifest: &IndexManifest) -> Result<(), String> {
    let manifest_json = serde_json::to_string_pretty(manifest)
        .map_err(|e| format!("Failed to serialize manifest: {e}"))?;
    let tmp = index_path.join("manifest.json.tmp");
    fs::write(&tmp, manifest_json).map_err(|e| format!("Failed to write manifest: {e}"))?;
    fs::rename(&tmp, index_path.join("manifest.json")).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("Failed to replace manifest: {e}")
    })?;
    Ok(())
}

/// Reject an index whose `manifest.json` is present but not parseable.
///
/// The SQLite fast path reads its metadata from `context.db` and never opens
/// `manifest.json`, so a manifest torn by a crash mid-save was invisible: the
/// index kept serving rows from a database whose companion record of what was
/// indexed no longer said anything. Half a manifest is not a manifest — the
/// pair is treated as corrupt and the caller rebuilds.
fn check_manifest_integrity(index_path: &Path) -> Result<(), String> {
    let manifest_file = index_path.join("manifest.json");
    if !manifest_file.exists() {
        return Ok(());
    }
    let raw = fs::read_to_string(&manifest_file)
        .map_err(|e| format!("manifest.json at {} is unreadable: {e}", index_path.display()))?;
    serde_json::from_str::<IndexManifest>(&raw).map(|_| ()).map_err(|e| {
        format!(
            "manifest.json at {} is torn or corrupt ({e})",
            index_path.display()
        )
    })
}

/// Refuse a pair whose `context.db` was written AFTER its `manifest.json`.
///
/// [`AgentContextIndex::save`] writes the database first and renames the
/// manifest over the old one last, so in a pair produced by one save the
/// manifest's mtime is never older than the database's. A database that is
/// newer therefore had a writer the manifest knows nothing about: a save that
/// died between the two writes, a `.db` restored from a backup beside an
/// untouched manifest, a second process. The manifest still describes the
/// previous contents, and the SQLite fast path would serve rows from the newer
/// database under it.
///
/// The two FILE mtimes are compared, not `built_at` against the db's mtime:
/// `built_at` is the moment the build STARTED, which necessarily predates
/// writing either artifact, so a clean pair would fail that comparison every
/// time. Comparing the two writes to each other is the only form of the
/// question that a clean save answers "no".
///
/// A missing artifact is not this function's problem (the blob path and the
/// scale guard report those), and a metadata error is treated as no evidence.
fn check_manifest_not_older_than_db(index_path: &Path) -> Result<(), String> {
    let manifest_file = index_path.join("manifest.json");
    let db_file = index_path.with_extension("db");
    let (Ok(manifest_meta), Ok(db_meta)) = (fs::metadata(&manifest_file), fs::metadata(&db_file))
    else {
        return Ok(());
    };
    let (Ok(manifest_mtime), Ok(db_mtime)) = (manifest_meta.modified(), db_meta.modified()) else {
        return Ok(());
    };
    if db_mtime > manifest_mtime {
        return Err(format!(
            "the index is stale: {} was written after {}, so the manifest does not describe the database",
            db_file.display(),
            manifest_file.display()
        ));
    }
    Ok(())
}

impl AgentContextIndex {
    /// Save index to directory
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn save(&self, index_path: &Path) -> Result<(), String> {
        fs::create_dir_all(index_path)
            .map_err(|e| format!("Failed to create index directory: {e}"))?;

        // Phase 3 (#159): Write only SQLite + FTS5 index (no more LZ4 blob)
        // context.db lives alongside context.idx directory.
        //
        // The DATABASE goes first and the manifest last, because that is what
        // makes "the db is newer than the manifest" a detectable fault rather
        // than the normal case (see `check_manifest_not_older_than_db`). A save
        // interrupted between the two leaves a manifest older than its db, and
        // the next `load()` rebuilds instead of serving one under the other.
        let db_path = index_path.with_extension("db");
        super::sqlite_backend::save_to_sqlite(
            &db_path,
            &self.functions,
            &self.calls,
            &self.graph_metrics,
            &self.manifest,
            &self.coverage_off_files,
        )?;

        // Save manifest ATOMICALLY: temp file in the same directory, then
        // rename. `fs::write` truncates first, so a crash (or a full disk)
        // between truncate and write left a half-written manifest next to a
        // complete database — the torn pair `load()` now has to detect. rename(2)
        // within a directory is atomic, so a reader sees either the old
        // manifest or the new one, never half of either.
        write_manifest_atomically(index_path, &self.manifest)?;

        Ok(())
    }

    /// Load index from directory.
    ///
    /// Prefers SQLite `context.db` when available (v2.0), falls back to
    /// LZ4+bincode blob `context.idx/functions.lz4` (v1.x).
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn load(index_path: &Path) -> Result<Self, String> {
        // R30: ONE scale check, ahead of both backends, with ONE remediation.
        //
        // A pre-v3.30.0 index has every required table and every required
        // column, so the structural check below passes it happily — while its
        // `tdg_score` column holds 0-10 lower-is-better debt numbers that
        // today's readers report as 0-100 quality scores, turning a stored 0.12
        // (the BEST legacy score) into an F. Structure valid does not mean
        // contents readable.
        //
        // The remediation is to DISCARD both artifacts, not merely to report.
        // This used to be split: the SQLite branch deleted its own stale `.db`
        // and let the blob path produce the error, so `pmat query` self-healed
        // while MCP's `IndexManager` — which propagates the error — answered
        // `-32603 … rebuild required` on every default call, forever. With the
        // whole stale index removed, the next `load()` sees no index at all and
        // every caller (CLI, MCP, comply) takes its ordinary build path.
        // A torn manifest invalidates the whole pair, whichever backend is
        // about to answer. Checked before the scale guard so the message names
        // the real problem rather than "no scale marker".
        if let Err(reason) = check_manifest_integrity(index_path) {
            eprintln!("  {reason} — discarding the index and rebuilding.");
            super::scale_guard::discard_stale_index(index_path);
            return Err(reason);
        }

        // A manifest that predates its database is the same class of fault as a
        // torn one — the two artifacts no longer describe one save — so it is
        // judged in the same place, before either backend answers.
        if let Err(reason) = check_manifest_not_older_than_db(index_path) {
            eprintln!("  {reason} — discarding the index and rebuilding.");
            super::scale_guard::discard_stale_index(index_path);
            return Err(reason);
        }

        if let Err(reason) = super::scale_guard::verify_index_scale(index_path) {
            eprintln!(
                "  Index at {} is stale: {reason} (scores are now 0-100, higher is better).",
                index_path.display()
            );
            super::scale_guard::discard_stale_index(index_path);
            return Err(reason);
        }

        // Try SQLite path first (v2.0)
        let db_candidate = index_path.with_extension("db");
        if db_candidate.exists() {
            // Validate schema before attempting full load — stale DBs from older
            // versions may lack required tables, producing confusing warnings.
            let conn = super::sqlite_backend::open_db(&db_candidate).ok();
            let schema_ok = conn
                .as_ref()
                .is_some_and(super::sqlite_backend::has_valid_schema);
            drop(conn);

            if schema_ok {
                match Self::load_from_sqlite(&db_candidate) {
                    Ok(index) => return Ok(index),
                    Err(e) => {
                        eprintln!("  Warning: SQLite load failed, falling back to blob: {e}");
                    }
                }
            } else {
                // Delete broken DB so next save() regenerates it
                let _ = std::fs::remove_file(&db_candidate);
            }
        }

        Self::load_from_blob(index_path)
    }

    /// Load index from SQLite database (v2.0 fast path).
    ///
    /// Reads functions (without source) and graph metrics from `context.db`.
    /// Skips corpus (FTS5 handles search), call graph (queried on-demand),
    /// and source code (loaded on-demand for display or regex/literal search).
    fn load_from_sqlite(db_path: &Path) -> Result<Self, String> {
        use super::sqlite_backend::{
            load_functions_lightweight, load_graph_metrics, load_metadata, open_db,
        };

        let conn = open_db(db_path)?;
        let manifest = load_metadata(&conn)?;
        let mut functions = load_functions_lightweight(&conn)?;
        // Enrich with contract metadata (optional columns, may not exist in old schemas)
        enrich_contract_metadata(&conn, &mut functions);
        let graph_metrics = load_graph_metrics(&conn)?;
        // Call graph loaded on-demand via get_calls()/get_called_by() SQLite fallback
        let calls = HashMap::new();
        let called_by = HashMap::new();

        // Build name_index + file_index only (no corpus — FTS5 handles search)
        let indices = build_indices_without_corpus(&functions);
        let name_frequency = compute_name_frequency(&indices.name_index, functions.len());

        let project_root = PathBuf::from(&manifest.project_root);

        // Load cached coverage_off_files from SQLite metadata
        let coverage_off_files = load_coverage_off_files(&conn);

        Ok(Self {
            functions,
            name_index: indices.name_index,
            file_index: indices.file_index,
            corpus: Vec::new(),
            corpus_lower: Vec::new(),
            name_frequency,
            calls,
            called_by,
            graph_metrics,
            project_root,
            manifest,
            db_path: Some(db_path.to_path_buf()),
            coverage_off_files,
        })
    }

    /// Load index from LZ4+bincode blob (v1.x legacy path).
    fn load_from_blob(index_path: &Path) -> Result<Self, String> {
        // Load manifest
        let manifest_str = fs::read_to_string(index_path.join("manifest.json"))
            .map_err(|e| format!("Failed to read manifest: {e}"))?;
        let manifest: IndexManifest = serde_json::from_str(&manifest_str)
            .map_err(|e| format!("Failed to parse manifest: {e}"))?;

        // R30: the manifest's scale marker is checked by `load()` via
        // `scale_guard::verify_index_scale`, which also discards the stale
        // index. A second comparison here would be a second decision point —
        // exactly the duplication that let `pmat sql` drift.

        // Load and decompress blob
        let compressed = fs::read(index_path.join("functions.lz4"))
            .map_err(|e| format!("Failed to read functions: {e}"))?;
        let decompressed = lz4_flex::decompress_size_prepended(&compressed)
            .map_err(|e| format!("Failed to decompress functions: {e}"))?;

        // Deserialize payload (v1.3.0+ has cached indices)
        let payload: IndexPayload = rmp_serde::from_slice(&decompressed)
            .map_err(|e| format!("Failed to parse payload: {e}"))?;

        let functions = payload.functions;
        let corpus = payload.corpus;
        let calls = payload.calls;
        let called_by = payload.called_by;

        // Check if we have cached indices (v1.3.0+) - avoids expensive PageRank recomputation
        let has_cached_indices = !payload.name_index.is_empty();

        let (name_index, file_index, graph_metrics, corpus_lower, name_frequency) =
            if has_cached_indices {
                let corpus_lower = if payload.corpus_lower.is_empty() {
                    corpus.iter().map(|d| d.to_lowercase()).collect()
                } else {
                    payload.corpus_lower
                };
                (
                    payload.name_index,
                    payload.file_index,
                    payload.graph_metrics,
                    corpus_lower,
                    payload.name_frequency,
                )
            } else {
                // Slow path: rebuild indices for legacy formats (v1.0-v1.2)
                let is_legacy =
                    manifest.version.starts_with("1.0") || manifest.version.starts_with("1.1");
                let indices = build_indices(&functions);

                let (calls_rebuilt, called_by_rebuilt) = if is_legacy && calls.is_empty() {
                    build_call_graph(&functions, &indices.name_index)
                } else {
                    (calls.clone(), called_by.clone())
                };

                let graph_metrics =
                    compute_graph_metrics(functions.len(), &calls_rebuilt, &called_by_rebuilt);
                let name_frequency = compute_name_frequency(&indices.name_index, functions.len());
                let corpus_lower: Vec<String> = corpus.iter().map(|d| d.to_lowercase()).collect();

                (
                    indices.name_index,
                    indices.file_index,
                    graph_metrics,
                    corpus_lower,
                    name_frequency,
                )
            };

        let project_root = PathBuf::from(&manifest.project_root);

        // Detect SQLite FTS5 database alongside blob
        let db_candidate = index_path.with_extension("db");
        let db_path = if db_candidate.exists() {
            Some(db_candidate)
        } else {
            None
        };

        Ok(Self {
            functions,
            name_index,
            file_index,
            corpus,
            corpus_lower,
            name_frequency,
            calls,
            called_by,
            graph_metrics,
            project_root,
            manifest,
            db_path,
            coverage_off_files: HashSet::new(), // Legacy blob path — no cached data
        })
    }
}
