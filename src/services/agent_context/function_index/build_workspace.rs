impl AgentContextIndex {
    /// Build a workspace-level index across multiple project roots.
    ///
    /// For cross-project RAG, each project is indexed and merged into a single
    /// searchable index. File paths are prefixed with the project directory name
    /// to disambiguate across projects.
    pub fn build_workspace(project_paths: &[&Path]) -> Result<Self, String> {
        debug_assert!(!project_paths.is_empty(), "project_paths must not be empty");
        if project_paths.is_empty() {
            return Err("No project paths provided".to_string());
        }

        if project_paths.len() == 1 {
            return Self::build(project_paths[0]);
        }

        // Build first project as base
        let mut merged = Self::build(project_paths[0])?;

        // Merge remaining projects
        for &path in &project_paths[1..] {
            let other = Self::build(path)?;
            merged.merge(other);
        }

        Ok(merged)
    }

    /// Load an index and prefix all file paths with a project name.
    ///
    /// Used for cross-project search: each sibling project's functions get
    /// paths like `aprender/src/lib.rs` instead of `src/lib.rs`.
    /// Only rebuilds file_index (paths changed). Corpus, name_index, and
    /// call graph are reused from the persisted payload.
    pub fn load_with_prefix(index_path: &Path, prefix: &str) -> Result<Self, String> {
        debug_assert!(index_path.exists(), "index_path must exist: {}", index_path.display());
        let mut index = Self::load(index_path)?;

        // Backfill source and call graph for workspace construction.
        // load_from_sqlite() uses lightweight loading (no source, empty call graph)
        // for fast query startup, but workspace builds need full data to persist
        // correctly into workspace.db.
        index.load_all_source();
        index.ensure_call_graph();

        // Prefix file paths in functions
        for func in &mut index.functions {
            func.file_path = format!("{prefix}/{}", func.file_path);
        }

        // Rebuild only file_index (paths changed), name_index is unchanged
        let mut file_index: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, func) in index.functions.iter().enumerate() {
            file_index
                .entry(func.file_path.clone())
                .or_default()
                .push(idx);
        }
        index.file_index = file_index;

        // Corpus still valid for search (path token changes are minor)
        // calls/called_by indices still valid (positional)
        Ok(index)
    }

    /// Read `.pmat/workspace.toml` to find configured sibling projects.
    ///
    /// Users opt in to cross-project search by creating this file:
    /// ```toml
    /// siblings = ["../aprender", "../trueno", "../realizar"]
    /// ```
    ///
    /// Returns `(index_path, project_name)` pairs for siblings that have
    /// a `.pmat/context.idx` or `.pmat/context.db`. Silently skips siblings
    /// without an index.
    pub fn discover_sibling_indexes(project_path: &Path) -> Vec<(PathBuf, String)> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let workspace_config = project_path.join(".pmat/workspace.toml");
        let config_str = match fs::read_to_string(&workspace_config) {
            Ok(s) => s,
            Err(_) => return Vec::new(), // No config = no siblings
        };

        // Minimal TOML parsing: extract siblings = ["...", "..."]
        let sibling_paths = parse_workspace_siblings(&config_str);
        let mut siblings = Vec::new();

        for rel_path in sibling_paths {
            let abs_path = project_path.join(&rel_path);
            let resolved = match abs_path.canonicalize() {
                Ok(p) => p,
                Err(_) => continue, // Path doesn't exist
            };
            let project_name = resolved
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Check for SQLite index first (v2.0), then blob directory (v1.x)
            let db_path = resolved.join(".pmat/context.db");
            let idx_path = resolved.join(".pmat/context.idx");
            if db_path.exists() || idx_path.exists() {
                // Pass the blob directory path; load() will detect .db alongside it
                siblings.push((idx_path, project_name));
            }
        }

        siblings
    }

    /// Merge sibling project indexes into this index for cross-project search.
    ///
    /// Uses a fast path that concatenates pre-built data from each sibling
    /// instead of rebuilding indices/call-graph from scratch. Each sibling's
    /// corpus, name_index, file_index, and call graph are offset-adjusted
    /// and appended in O(n) time.
    ///
    /// After all siblings are merged, rebuilds the unified call graph so that
    /// cross-project call edges are resolved and PageRank reflects cross-project
    /// importance.
    ///
    /// Each sibling's `.pmat/context.idx` is never modified.
    pub fn merge_siblings(&mut self, siblings: &[(PathBuf, String)]) {
        debug_assert!(!siblings.is_empty(), "siblings must not be empty");
        for (idx_path, project_name) in siblings {
            match Self::load_with_prefix(idx_path, project_name) {
                Ok(sibling) => {
                    let count = sibling.functions.len();
                    self.merge_fast(sibling);
                    eprintln!("  + {}: {} functions", project_name, count);
                }
                Err(e) => {
                    eprintln!("  ! {}: failed to load ({})", project_name, e);
                }
            }
        }
        // Rebuild unified call graph + PageRank across all projects
        if !siblings.is_empty() {
            self.rebuild_cross_project_graph();
        }
    }

    /// Rebuild call graph and graph metrics on the merged index.
    ///
    /// After `merge_fast()` appends per-project call graphs with offset-adjusted
    /// indices, cross-project calls are not yet resolved (e.g., a function in
    /// aprender calling a trueno function). This method rebuilds the entire call
    /// graph from scratch using the unified `name_index`, then recomputes
    /// PageRank and centrality so cross-project importance is reflected.
    pub fn rebuild_cross_project_graph(&mut self) {
        let (calls, called_by) = build_call_graph(&self.functions, &self.name_index);
        let graph_metrics = compute_graph_metrics(self.functions.len(), &calls, &called_by);
        self.calls = calls;
        self.called_by = called_by;
        self.graph_metrics = graph_metrics;
    }

    /// Fast merge: concatenate pre-built data with index offset adjustment.
    ///
    /// Unlike `merge()` which rebuilds all indices from scratch (O(n*m) for
    /// call graph), this just offsets positional indices and appends. O(n).
    pub(super) fn merge_fast(&mut self, other: Self) {
        let offset = self.functions.len();

        // Append functions and corpus
        self.functions.extend(other.functions);
        self.corpus.extend(other.corpus.iter().cloned());
        self.corpus_lower.extend(other.corpus_lower);

        // Offset and append name_index
        for (name, indices) in other.name_index {
            self.name_index
                .entry(name)
                .or_default()
                .extend(indices.iter().map(|i| i + offset));
        }

        // Offset and append file_index
        for (path, indices) in other.file_index {
            self.file_index
                .entry(path)
                .or_default()
                .extend(indices.iter().map(|i| i + offset));
        }

        // Offset and append call graph
        for (caller, callees) in other.calls {
            self.calls.insert(
                caller + offset,
                callees.iter().map(|i| i + offset).collect(),
            );
        }
        for (callee, callers) in other.called_by {
            self.called_by.insert(
                callee + offset,
                callers.iter().map(|i| i + offset).collect(),
            );
        }

        // Append graph_metrics (per-function values, no offset needed)
        self.graph_metrics.extend(other.graph_metrics);

        // Merge name_frequency (recompute — cheap, just HashMap iteration)
        self.name_frequency = compute_name_frequency(&self.name_index, self.functions.len());

        // Update manifest
        self.manifest.function_count = self.functions.len();
        self.manifest.file_count += other.manifest.file_count;
    }

    /// Merge another index into this one.
    fn merge(&mut self, other: Self) {
        for func in other.functions {
            self.functions.push(func);
        }

        // Rebuild all derived data from scratch after merge
        let indices = build_indices(&self.functions);
        let (calls, called_by) = build_call_graph(&self.functions, &indices.name_index);
        let name_frequency = compute_name_frequency(&indices.name_index, self.functions.len());

        self.name_index = indices.name_index;
        self.file_index = indices.file_index;
        self.corpus = indices.corpus;
        self.corpus_lower = self.corpus.iter().map(|d| d.to_lowercase()).collect();
        self.name_frequency = name_frequency;
        self.calls = calls;
        self.called_by = called_by;

        // Update manifest
        self.manifest.function_count = self.functions.len();
        self.manifest.file_count += other.manifest.file_count;
        for (k, v) in other.manifest.file_checksums {
            self.manifest.file_checksums.insert(k, v);
        }
    }
}
