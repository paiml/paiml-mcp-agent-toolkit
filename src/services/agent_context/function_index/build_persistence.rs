impl AgentContextIndex {
    /// Save index to directory
    pub fn save(&self, index_path: &Path) -> Result<(), String> {
        fs::create_dir_all(index_path)
            .map_err(|e| format!("Failed to create index directory: {e}"))?;

        // Save manifest
        let manifest_json = serde_json::to_string_pretty(&self.manifest)
            .map_err(|e| format!("Failed to serialize manifest: {e}"))?;
        fs::write(index_path.join("manifest.json"), manifest_json)
            .map_err(|e| format!("Failed to write manifest: {e}"))?;

        // Phase 3 (#159): Write only SQLite + FTS5 index (no more LZ4 blob)
        // context.db lives alongside context.idx directory
        let db_path = index_path.with_extension("db");
        super::sqlite_backend::save_to_sqlite(
            &db_path,
            &self.functions,
            &self.calls,
            &self.graph_metrics,
            &self.manifest,
            &self.coverage_off_files,
        )?;

        Ok(())
    }

    /// Load index from directory.
    ///
    /// Prefers SQLite `context.db` when available (v2.0), falls back to
    /// LZ4+bincode blob `context.idx/functions.lz4` (v1.x).
    pub fn load(index_path: &Path) -> Result<Self, String> {
        // Try SQLite path first (v2.0)
        let db_candidate = index_path.with_extension("db");
        if db_candidate.exists() {
            // Validate schema before attempting full load — stale DBs from older
            // versions may lack required tables, producing confusing warnings.
            let schema_ok = super::sqlite_backend::open_db(&db_candidate)
                .map(|conn| super::sqlite_backend::has_valid_schema(&conn))
                .unwrap_or(false);

            if schema_ok {
                match Self::load_from_sqlite(&db_candidate) {
                    Ok(index) => return Ok(index),
                    Err(e) => {
                        eprintln!("  Warning: SQLite load failed, falling back to blob: {e}");
                    }
                }
            } else {
                // Delete broken/stale DB so next save() regenerates it
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
        let functions = load_functions_lightweight(&conn)?;
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

        // Load and decompress blob
        let compressed = fs::read(index_path.join("functions.lz4"))
            .map_err(|e| format!("Failed to read functions: {e}"))?;
        let decompressed = lz4_flex::decompress_size_prepended(&compressed)
            .map_err(|e| format!("Failed to decompress functions: {e}"))?;

        // Deserialize payload (v1.3.0+ has cached indices)
        let payload: IndexPayload = bincode::deserialize(&decompressed)
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
