impl AgentContextIndex {
    /// Get callees for a given function index.
    ///
    /// Uses in-memory HashMap when available (blob load), falls back to
    /// on-demand SQLite query (SQLite load — avoids loading 3.8M edges upfront).
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_calls(&self, func_idx: usize) -> Vec<&str> {
        if let Some(indices) = self.calls.get(&func_idx) {
            return indices
                .iter()
                .map(|&i| self.functions[i].function_name.as_str())
                .collect();
        }
        // On-demand SQLite query when call graph not loaded in memory
        if let Some(ref db_path) = self.db_path {
            if let Ok(conn) = super::sqlite_backend::open_db(db_path) {
                if let Ok(indices) = super::sqlite_backend::query_callees(&conn, func_idx) {
                    return indices
                        .iter()
                        .filter(|&&i| i < self.functions.len())
                        .map(|&i| self.functions[i].function_name.as_str())
                        .collect();
                }
            }
        }
        Vec::new()
    }

    /// Get callers for a given function index.
    ///
    /// Uses in-memory HashMap when available (blob load), falls back to
    /// on-demand SQLite query (SQLite load — avoids loading 3.8M edges upfront).
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_called_by(&self, func_idx: usize) -> Vec<&str> {
        if let Some(indices) = self.called_by.get(&func_idx) {
            return indices
                .iter()
                .map(|&i| self.functions[i].function_name.as_str())
                .collect();
        }
        // On-demand SQLite query when call graph not loaded in memory
        if let Some(ref db_path) = self.db_path {
            if let Ok(conn) = super::sqlite_backend::open_db(db_path) {
                if let Ok(indices) = super::sqlite_backend::query_callers(&conn, func_idx) {
                    return indices
                        .iter()
                        .filter(|&&i| i < self.functions.len())
                        .map(|&i| self.functions[i].function_name.as_str())
                        .collect();
                }
            }
        }
        Vec::new()
    }

    /// Get raw caller indices for a function (O(1) from in-memory map).
    /// Returns None if call graph is not loaded in memory (SQLite-only mode).
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn called_by_indices(&self, func_idx: usize) -> Option<&[usize]> {
        self.called_by.get(&func_idx).map(|v| v.as_slice())
    }

    /// Get raw callee indices for a function (O(1) from in-memory map).
    /// Returns None if call graph is not loaded in memory (SQLite-only mode).
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn calls_indices(&self, func_idx: usize) -> Option<&[usize]> {
        self.calls.get(&func_idx).map(|v| v.as_slice())
    }

    /// Find function index by file path and name
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "non_empty_index")]
    pub fn find_function_index(&self, file_path: &str, function_name: &str) -> Option<usize> {
        self.functions
            .iter()
            .position(|f| f.file_path == file_path && f.function_name == function_name)
    }

    /// Count callers from different projects for a given function.
    ///
    /// In a workspace index, file paths are prefixed with the project name
    /// (e.g., `aprender/src/lib.rs`). A cross-project caller is one whose
    /// project prefix differs from the callee's prefix.
    ///
    /// Falls back to on-demand SQLite query when call graph is not loaded in memory.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn count_cross_project_callers(&self, func_idx: usize) -> u32 {
        if func_idx >= self.functions.len() {
            return 0;
        }
        let callee_project = project_prefix(&self.functions[func_idx].file_path);

        // In-memory path (eager-loaded call graph from blob or rebuild)
        if let Some(caller_indices) = self.called_by.get(&func_idx) {
            return caller_indices
                .iter()
                .filter(|&&i| {
                    i < self.functions.len()
                        && project_prefix(&self.functions[i].file_path) != callee_project
                })
                .count() as u32;
        }

        // SQLite fallback when call graph not eagerly loaded
        if let Some(ref db_path) = self.db_path {
            if let Ok(conn) = super::sqlite_backend::open_db(db_path) {
                if let Ok(indices) = super::sqlite_backend::query_callers(&conn, func_idx) {
                    return indices
                        .iter()
                        .filter(|&&i| {
                            i < self.functions.len()
                                && project_prefix(&self.functions[i].file_path) != callee_project
                        })
                        .count() as u32;
                }
            }
        }
        0
    }

    /// Load source code for all entries that are missing it (deferred-source mode).
    ///
    /// Fills from SQLite first (matched by `(file_path, start_line)`, not row
    /// id — incremental rebuilds reorder functions relative to the persisted
    /// DB), then falls back to reading project files. Only entries with empty
    /// source are filled: re-parsed entries already carry fresh source and
    /// must never be overwritten with stale DB rows.
    ///
    /// Used when regex/literal search mode needs full source for pattern
    /// matching, and by the incremental save path: checksum-reused entries
    /// are cloned from a lightweight SQLite load with `source == ""`, and
    /// saving them unfilled would rewrite every reused row with empty source
    /// (the source-wipe bug breaking `--include-source` / `pmat_get_function`).
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn load_all_source(&mut self) {
        if !self.functions.iter().any(|f| f.source.is_empty()) {
            return; // Nothing deferred — keep pure-read paths cheap
        }
        self.load_source_from_db();
        self.load_source_from_filesystem();
    }

    fn load_source_from_db(&mut self) {
        let db_path = match self.db_path {
            Some(ref p) => p,
            None => return,
        };
        let conn = match super::sqlite_backend::open_db(db_path) {
            Ok(c) => c,
            Err(_) => return,
        };
        let source_map = match super::sqlite_backend::load_source_map(&conn) {
            Ok(m) => m,
            Err(_) => return,
        };
        for func in &mut self.functions {
            if !func.source.is_empty() {
                continue;
            }
            if let Some(src) = source_map
                .get(&func.file_path)
                .and_then(|by_line| by_line.get(&func.start_line))
            {
                func.source = src.clone();
            }
        }
    }

    fn load_source_from_filesystem(&mut self) {
        let mut files_to_read: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, func) in self.functions.iter().enumerate() {
            if func.source.is_empty() && func.end_line > 0 {
                files_to_read.entry(func.file_path.clone()).or_default().push(idx);
            }
        }
        for (file_path, indices) in &files_to_read {
            // Prefer the project_root-anchored path (correct regardless of
            // cwd); fall back to cwd-relative for workspace-prefixed paths.
            let content = match std::fs::read_to_string(self.project_root.join(file_path))
                .or_else(|_| std::fs::read_to_string(file_path))
            {
                Ok(c) => c,
                Err(_) => continue,
            };
            let lines: Vec<&str> = content.lines().collect();
            for &idx in indices {
                let start = self.functions[idx].start_line.saturating_sub(1);
                let end = self.functions[idx].end_line.min(lines.len());
                if start < end {
                    self.functions[idx].source = lines[start..end].join("\n");
                }
            }
        }
    }

    /// Eagerly load the call graph from SQLite into memory.
    ///
    /// Used when PTX flow or cross-project ranking needs full in-memory call graph
    /// for `called_by_indices()`/`calls_indices()` access. No-op if call graph
    /// is already loaded (non-empty) or db_path is not set.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn ensure_call_graph(&mut self) {
        if !self.calls.is_empty() {
            return; // Already loaded
        }
        if let Some(ref db_path) = self.db_path {
            if let Ok(conn) = super::sqlite_backend::open_db(db_path) {
                if let Ok((calls, called_by)) = super::sqlite_backend::load_call_graph(&conn) {
                    self.calls = calls;
                    self.called_by = called_by;
                }
            }
        }
    }

    /// Get the SQLite database path (if available).
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn db_path(&self) -> Option<&Path> {
        self.db_path.as_deref()
    }

    /// Load source code for a single function by file path and start line.
    ///
    /// Returns the source from in-memory, SQLite, or filesystem (in that order).
    /// Falls back to reading the file directly when both in-memory and SQLite
    /// source are empty (e.g., lightweight-loaded index with stale DB).
    fn find_in_memory(&self, file_path: &str, start_line: usize) -> (Option<String>, usize) {
        let indices = match self.file_index.get(file_path) {
            Some(i) => i,
            None => return (None, 0),
        };
        for &idx in indices {
            if self.functions[idx].start_line != start_line {
                continue;
            }
            if !self.functions[idx].source.is_empty() {
                return (Some(self.functions[idx].source.clone()), self.functions[idx].end_line);
            }
            return (None, self.functions[idx].end_line);
        }
        (None, 0)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "non_empty_index")]
    /// Load source for.
    pub fn load_source_for(&self, file_path: &str, start_line: usize) -> String {
        let (in_memory, end_line) = self.find_in_memory(file_path, start_line);
        if let Some(src) = in_memory {
            return src;
        }
        if let Some(ref db_path) = self.db_path {
            if let Some(src) = load_source_from_sqlite(db_path, file_path, start_line) {
                return src;
            }
        }
        load_source_from_file(file_path, start_line, end_line).unwrap_or_default()
    }
}
